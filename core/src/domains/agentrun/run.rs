//! An AgentRun record: its id, state, and timestamps.

use rand::RngCore;
use time::{OffsetDateTime, UtcOffset};

pub const STATUS_OPEN: &str = "open";
pub const STATUS_COMPLETED: &str = "completed";

pub const KIND_CONTEXT: &str = "context";
pub const KIND_CHANGE: &str = "change";

/// The provenance actor for audited MCP writes.
pub const ACTOR_AGENT: &str = "agent";
/// The provenance actor for audited Desktop writes.
pub const ACTOR_HUMAN: &str = "human";
/// The provenance source for audited MCP writes.
pub const SOURCE_MCP: &str = "mcp";
/// The provenance source for audited Desktop writes.
pub const SOURCE_DESKTOP: &str = "desktop";

/// Labels AgentRuns started by coding agents.
pub const ACTOR_KIND_AGENT: &str = "agent";
/// Labels AgentRuns started by Desktop humans.
pub const ACTOR_KIND_HUMAN: &str = "human";

/// The errors a run returns, one per Go sentinel.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("unknown agent run")]
    UnknownRun,
    #[error("agent run is not open")]
    RunNotOpen,
    #[error("invalid agent run step kind: {0:?}")]
    InvalidKind(String),
    #[error("invalid actor kind: {0:?}")]
    InvalidActorKind(String),
    #[error("change step requires summary and rationale")]
    ChangeIncomplete,
    #[error("agent_run_id is required")]
    MissingAgentRunId,
    #[error("project_id does not match agent run")]
    ProjectMismatch,
    #[error("task is required")]
    EmptyTask,
    #[error("project_id is required")]
    EmptyProject,
    /// A payload that could not be encoded; Go returns the `json` error raw.
    #[error("{0}")]
    Json(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A minimal AgentRun audit header (the task lives here, not as a step).
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    pub id: String,
    pub project_id: String,
    pub task: String,
    pub agent_label: String,
    pub actor_kind: String,
    pub status: String,
    pub created_at: OffsetDateTime,
    pub completed_at: Option<OffsetDateTime>,
}

impl Default for Run {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            task: String::new(),
            agent_label: String::new(),
            actor_kind: String::new(),
            status: String::new(),
            created_at: OffsetDateTime::UNIX_EPOCH,
            completed_at: None,
        }
    }
}

/// One link in context → change.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub id: String,
    pub agent_run_id: String,
    pub kind: String,
    pub summary: String,
    pub payload_json: String,
    pub created_at: OffsetDateTime,
}

impl Default for Step {
    fn default() -> Self {
        Self {
            id: String::new(),
            agent_run_id: String::new(),
            kind: String::new(),
            summary: String::new(),
            payload_json: String::new(),
            created_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

/// A run plus its ordered steps for reconstruction.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Record {
    pub run: Run,
    pub steps: Vec<Step>,
}

pub fn new_run_id() -> String {
    format!("arun-{}", random_hex(12))
}

pub fn new_step_id() -> String {
    format!("astep-{}", random_hex(12))
}

fn random_hex(n: usize) -> String {
    let mut buf = vec![0u8; n];
    rand::rng().fill_bytes(&mut buf);
    hex::encode(buf)
}

pub fn normalize_kind(kind: &str) -> Result<&'static str> {
    match kind.trim().to_lowercase().as_str() {
        KIND_CONTEXT => Ok(KIND_CONTEXT),
        KIND_CHANGE => Ok(KIND_CHANGE),
        _ => Err(Error::InvalidKind(kind.to_string())),
    }
}

pub fn validate_start(project_id: &str, task: &str) -> Result<()> {
    if project_id.trim().is_empty() {
        return Err(Error::EmptyProject);
    }
    if task.trim().is_empty() {
        return Err(Error::EmptyTask);
    }
    Ok(())
}

/// Returns human|agent. Empty defaults to agent for MCP compatibility.
pub fn normalize_actor_kind(kind: &str) -> Result<&'static str> {
    match kind.trim().to_lowercase().as_str() {
        "" | ACTOR_KIND_AGENT => Ok(ACTOR_KIND_AGENT),
        ACTOR_KIND_HUMAN => Ok(ACTOR_KIND_HUMAN),
        _ => Err(Error::InvalidActorKind(kind.to_string())),
    }
}

pub fn validate_change(summary: &str, rationale: &str) -> Result<()> {
    if summary.trim().is_empty() || rationale.trim().is_empty() {
        return Err(Error::ChangeIncomplete);
    }
    Ok(())
}

/// Checks the envelope for one actor: every write needs a summary, and an
/// agent's also needs a rationale. A person may leave the reason out; Activity
/// shows that no reason was given.
pub fn validate_change_as(actor_kind: &str, summary: &str, rationale: &str) -> Result<()> {
    if actor_kind == ACTOR_KIND_HUMAN {
        if summary.trim().is_empty() {
            return Err(Error::ChangeIncomplete);
        }
        return Ok(());
    }
    validate_change(summary, rationale)
}

/// Requires the B7 mutation envelope before any Canon/Knowledge write.
pub fn validate_audited_write(summary: &str, rationale: &str, agent_run_id: &str) -> Result<()> {
    if agent_run_id.trim().is_empty() {
        return Err(Error::MissingAgentRunId);
    }
    validate_change(summary, rationale)
}

/// Ensures the mutation targets the same project as the open run.
pub fn match_project(run: &Run, project_id: &str) -> Result<()> {
    if project_id.trim().is_empty() {
        return Err(Error::EmptyProject);
    }
    if run.project_id != project_id.trim() {
        return Err(Error::ProjectMismatch);
    }
    Ok(())
}

pub fn can_append(run: &Run) -> Result<()> {
    if run.status != STATUS_OPEN {
        return Err(Error::RunNotOpen);
    }
    Ok(())
}

pub fn can_complete(run: &Run) -> Result<()> {
    if run.status != STATUS_OPEN {
        return Err(Error::RunNotOpen);
    }
    Ok(())
}

/// Starts an open AgentRun for an agent. `now` of `None` is Go's zero time: the
/// current UTC time is used.
pub fn start(
    project_id: &str,
    task: &str,
    agent_label: &str,
    now: Option<OffsetDateTime>,
) -> Result<Run> {
    start_with_actor(project_id, task, agent_label, ACTOR_KIND_AGENT, now)
}

/// Creates an open AgentRun for the given actor kind (human|agent).
pub fn start_with_actor(
    project_id: &str,
    task: &str,
    agent_label: &str,
    actor_kind: &str,
    now: Option<OffsetDateTime>,
) -> Result<Run> {
    validate_start(project_id, task)?;
    let kind = normalize_actor_kind(actor_kind)?;
    Ok(Run {
        id: new_run_id(),
        project_id: project_id.trim().to_string(),
        task: task.trim().to_string(),
        agent_label: agent_label.trim().to_string(),
        actor_kind: kind.to_string(),
        status: STATUS_OPEN.to_string(),
        created_at: utc_or_now(now),
        completed_at: None,
    })
}

pub fn complete(mut run: Run, now: Option<OffsetDateTime>) -> Result<Run> {
    can_complete(&run)?;
    run.status = STATUS_COMPLETED.to_string();
    run.completed_at = Some(utc_or_now(now));
    Ok(run)
}

/// Go's `time.Time` is a zero value that means "now"; `None` carries that.
fn utc_or_now(now: Option<OffsetDateTime>) -> OffsetDateTime {
    match now {
        Some(now) => now.to_offset(UtcOffset::UTC),
        None => OffsetDateTime::now_utc(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use time::macros::datetime;

    #[test]
    fn start_and_complete() {
        let now = datetime!(2026-03-01 12:00:00 UTC);
        let run = start("proj-1", "Fix import drift", "cursor", Some(now)).expect("start");
        assert!(
            run.status == STATUS_OPEN && !run.id.is_empty() && run.task == "Fix import drift",
            "unexpected run: {run:?}"
        );
        assert_eq!(run.actor_kind, ACTOR_KIND_AGENT);

        let human = start_with_actor(
            "proj-1",
            "Edit decision",
            "Desktop",
            ACTOR_KIND_HUMAN,
            Some(now),
        )
        .expect("start with actor");
        assert_eq!(human.actor_kind, ACTOR_KIND_HUMAN);

        let done = complete(run, Some(now + time::Duration::minutes(1))).expect("complete");
        assert_eq!(done.status, STATUS_COMPLETED);
        assert!(done.completed_at.is_some(), "expected completed: {done:?}");

        assert_eq!(complete(done, Some(now)).unwrap_err(), Error::RunNotOpen);
    }

    #[test]
    fn start_validation() {
        assert_eq!(
            start("", "task", "", None).unwrap_err(),
            Error::EmptyProject
        );
        assert_eq!(start("p", "  ", "", None).unwrap_err(), Error::EmptyTask);
    }

    #[test]
    fn validate_audited_write_requires_the_envelope() {
        assert_eq!(
            validate_audited_write("s", "r", "").unwrap_err(),
            Error::MissingAgentRunId
        );
        assert_eq!(
            validate_audited_write("", "r", "arun-1").unwrap_err(),
            Error::ChangeIncomplete
        );
        assert!(validate_audited_write("s", "r", "arun-1").is_ok());
    }

    #[test]
    fn match_project_checks_the_run() {
        let run = Run {
            project_id: "proj-1".to_string(),
            status: STATUS_OPEN.to_string(),
            ..Run::default()
        };
        assert_eq!(
            match_project(&run, "proj-2").unwrap_err(),
            Error::ProjectMismatch
        );
        assert!(match_project(&run, "proj-1").is_ok());
        assert_eq!(match_project(&run, " ").unwrap_err(), Error::EmptyProject);
    }

    #[test]
    fn normalize_kinds_and_actor_kinds() {
        assert_eq!(normalize_kind(" Change ").expect("kind"), KIND_CHANGE);
        assert_eq!(normalize_kind("CONTEXT").expect("kind"), KIND_CONTEXT);
        assert_eq!(
            normalize_kind(" Bogus ").unwrap_err(),
            Error::InvalidKind(" Bogus ".to_string())
        );
        // The message quotes the kind as it came in, as Go's %q does.
        assert_eq!(
            normalize_kind(" Bogus ").unwrap_err().to_string(),
            "invalid agent run step kind: \" Bogus \""
        );
        assert_eq!(
            normalize_kind("bogus").unwrap_err().to_string(),
            "invalid agent run step kind: \"bogus\""
        );
        assert_eq!(
            normalize_actor_kind("").expect("actor kind"),
            ACTOR_KIND_AGENT
        );
        assert_eq!(
            normalize_actor_kind("Human").expect("actor kind"),
            ACTOR_KIND_HUMAN
        );
        assert_eq!(
            normalize_actor_kind(" robot ").unwrap_err(),
            Error::InvalidActorKind(" robot ".to_string())
        );
        assert_eq!(
            normalize_actor_kind(" robot ").unwrap_err().to_string(),
            "invalid actor kind: \" robot \""
        );
    }

    #[test]
    fn can_append_and_complete_only_on_an_open_run() {
        let open = Run {
            status: STATUS_OPEN.to_string(),
            ..Run::default()
        };
        assert!(can_append(&open).is_ok());
        assert!(can_complete(&open).is_ok());
        let done = Run {
            status: STATUS_COMPLETED.to_string(),
            ..Run::default()
        };
        assert_eq!(can_append(&done).unwrap_err(), Error::RunNotOpen);
        assert_eq!(can_complete(&done).unwrap_err(), Error::RunNotOpen);
    }

    #[test]
    fn ids_carry_their_prefix_and_twelve_random_bytes() {
        let run_id = new_run_id();
        assert!(run_id.starts_with("arun-"), "{run_id}");
        assert_eq!(run_id.len(), "arun-".len() + 24);
        assert!(
            run_id["arun-".len()..]
                .chars()
                .all(|c| c.is_ascii_hexdigit())
        );
        let step_id = new_step_id();
        assert!(step_id.starts_with("astep-"), "{step_id}");
        assert_eq!(step_id.len(), "astep-".len() + 24);
    }

    #[test]
    fn validate_change_as_lets_a_person_skip_the_reason() {
        assert!(validate_change_as(ACTOR_KIND_HUMAN, "Create card: login", "").is_ok());
        assert_eq!(
            validate_change_as(ACTOR_KIND_HUMAN, " ", "why").unwrap_err(),
            Error::ChangeIncomplete
        );
        assert_eq!(
            validate_change_as(ACTOR_KIND_AGENT, "Create card: login", "").unwrap_err(),
            Error::ChangeIncomplete
        );
    }
}
