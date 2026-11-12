//! Port of `backend/compositions/local-core/audited_write.go`.
//!
//! The change envelope every semantic Desktop write carries, and how its
//! failures become Connect errors.

use crate::compositions::wedge;
use crate::domains::agentrun;
use crate::domains::canvas;
use crate::domains::knowledge;
use crate::local_v1;
use crate::providers::sqlite;

use super::connect::{Code, ConnectError};
use super::server::invalid;

/// Reads the envelope of a Desktop write. Only the summary is checked here: the
/// audited write checks the rationale for its actor, since a person may leave
/// the reason out.
pub fn change_envelope_from_proto(
    message: Option<&local_v1::ChangeEnvelope>,
) -> Result<wedge::ChangeEnvelope, sqlite::Error> {
    let Some(message) = message else {
        return Err(agentrun::Error::ChangeIncomplete.into());
    };
    let env = wedge::ChangeEnvelope {
        summary: message.summary.clone(),
        rationale: message.rationale.clone(),
        agent_run_id: message.agent_run_id.clone(),
        ..wedge::ChangeEnvelope::default()
    };
    agentrun::validate_change_as(agentrun::ACTOR_KIND_HUMAN, &env.summary, &env.rationale)?;
    Ok(env)
}

/// Maps an audited write's failure to its Connect code: a refused envelope is
/// `invalid_argument`, an unknown run `not_found`, anything else `internal`.
pub fn map_audited_err(err: sqlite::Error) -> ConnectError {
    let sqlite::Error::AgentRun(inner) = &err else {
        return super::server::internal(err);
    };
    match inner {
        agentrun::Error::ChangeIncomplete
        | agentrun::Error::MissingAgentRunId
        | agentrun::Error::EmptyProject
        | agentrun::Error::InvalidActorKind(_)
        | agentrun::Error::ProjectMismatch
        | agentrun::Error::RunNotOpen
        | agentrun::Error::InvalidKind(_) => invalid(err.to_string()),
        agentrun::Error::UnknownRun => ConnectError::new(Code::NotFound, err.to_string()),
        _ => super::server::internal(err),
    }
}

/// Maps the refusal of an archive or restore: a knowledge item or canvas in the
/// wrong state is `failed_precondition`, and anything else goes to
/// [`map_audited_err`], as Go's `mapArchiveErr` (in `archive_handlers.go`) does.
pub fn map_archive_err(err: impl Into<sqlite::Error>) -> ConnectError {
    let err = err.into();
    match &err {
        sqlite::Error::Knowledge(
            knowledge::KnowledgeError::NotArchived | knowledge::KnowledgeError::Archived,
        )
        | sqlite::Error::Canvas(canvas::Error::NotArchived | canvas::Error::Archived) => {
            ConnectError::failed_precondition(err.to_string())
        }
        _ => map_audited_err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_envelope_is_incomplete() {
        let err = change_envelope_from_proto(None).unwrap_err();
        assert_eq!(
            err.to_string(),
            "change step requires summary and rationale"
        );
        assert_eq!(map_audited_err(err).code, Code::InvalidArgument);
    }

    #[test]
    fn an_envelope_needs_a_summary_but_not_a_reason_for_a_person() {
        let request = local_v1::ChangeEnvelope {
            summary: String::new(),
            ..local_v1::ChangeEnvelope::default()
        };
        let err = change_envelope_from_proto(Some(&request)).unwrap_err();
        assert_eq!(map_audited_err(err).code, Code::InvalidArgument);
        let request = local_v1::ChangeEnvelope {
            summary: "Create doc: plan".to_string(),
            ..local_v1::ChangeEnvelope::default()
        };
        let env =
            change_envelope_from_proto(Some(&request)).expect("a person may leave the reason out");
        assert_eq!(env.summary, "Create doc: plan");
        assert_eq!(env.rationale, "");
    }

    #[test]
    fn archive_refusals_are_failed_preconditions() {
        let err = map_archive_err(knowledge::KnowledgeError::Archived);
        assert_eq!(err.code, Code::FailedPrecondition);
        assert_eq!(err.message, "restore the item before editing it");
        let err = map_archive_err(canvas::Error::NotArchived);
        assert_eq!(err.code, Code::FailedPrecondition);
        let err = map_archive_err(sqlite::Error::message("boom"));
        assert_eq!(err.code, Code::Internal);
        assert_eq!(err.message, "boom");
    }

    #[test]
    fn an_unknown_run_is_not_found() {
        let err = map_audited_err(agentrun::Error::UnknownRun.into());
        assert_eq!(err.code, Code::NotFound);
        assert_eq!(err.message, "unknown agent run");
    }
}
