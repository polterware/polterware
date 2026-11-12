//! Applying a write inside a session's AgentRun.

#[allow(unused_imports)]
use super::*;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::domains::agentrun;
use crate::providers::sqlite::{self, Store};

/// Keeps one open agent run per project for an agent session (an MCP process or
/// a Claude Code session), so writes carry only summary and rationale. Runs
/// open lazily on the first write and close with the session.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRuns {
    pub label: String,
    pub runs: HashMap<String, String>,
}

impl SessionRuns {
    /// Returns the session's open run for `project_id`, starting one when the
    /// session has none or its run was completed elsewhere.
    pub fn run_for(
        &mut self,
        store: &Store,
        project_id: &str,
        task: &str,
    ) -> sqlite::Result<String> {
        if let Some(id) = self.runs.get(project_id)
            && let Ok(run) = store.get_agent_run(id)
            && agentrun::can_append(&run).is_ok()
        {
            return Ok(id.clone());
        }
        let label = self.label.trim();
        let label = if label.is_empty() {
            "agent session"
        } else {
            label
        };
        let run = agentrun::start_with_actor(
            project_id,
            task,
            label,
            agentrun::ACTOR_KIND_AGENT,
            Some(OffsetDateTime::now_utc()),
        )?;
        store.create_agent_run(run.clone())?;
        self.runs.insert(project_id.to_string(), run.id.clone());
        Ok(run.id)
    }

    /// Completes every run the session still holds open.
    pub fn close(&mut self, store: &Store) {
        let now = OffsetDateTime::now_utc();
        for id in self.runs.values() {
            if let Ok(run) = store.get_agent_run(id)
                && agentrun::can_complete(&run).is_ok()
                && let Ok(done) = agentrun::complete(run, Some(now))
            {
                let _ = store.update_agent_run(done);
            }
        }
        self.runs.clear();
    }
}

/// [`apply_polter`] for an agent session: without `agent_run_id` a real write
/// joins the session's run for the project. Invalid, no-op, or unjustified text
/// is rejected before any run opens.
pub fn apply_in_session(
    store: &Store,
    session: &mut SessionRuns,
    mut in_: ApplyInput,
) -> std::result::Result<ApplyResult, ApplyFailure> {
    if !in_.agent_run_id.is_empty() || in_.dry_run {
        return apply_polter(store, in_);
    }
    let mut preview = in_.clone();
    preview.dry_run = true;
    let planned = apply_polter(store, preview)?;
    if planned.ops.is_empty() {
        return Ok(planned);
    }
    if agentrun::validate_change(&planned.summary, &planned.rationale).is_err() {
        return Err(ApplyFailure {
            result: Box::new(planned),
            error: Error::ChangeIncompleteFrontmatter,
        });
    }
    let run_id = match session.run_for(store, &in_.project_id, &planned.summary) {
        Ok(run_id) => run_id,
        Err(error) => {
            return Err(ApplyFailure {
                result: Box::new(planned),
                error: error.into(),
            });
        }
    };
    in_.agent_run_id = run_id;
    apply_polter(store, in_)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositions::wedge::work::test_support::{ROADMAP, init_test_project};

    #[test]
    fn a_session_reuses_one_run() {
        let crate::providers::sqlite::test_support::TempStore { store, .. } =
            crate::providers::sqlite::test_support::TempStore::new();
        let project = init_test_project(&store, "Demo");
        let mut session = SessionRuns {
            label: "claude-code".to_string(),
            ..SessionRuns::default()
        };

        let unjustified = "---\npolter: 1\n---\n\n## draft: Draft\n";
        apply_in_session(
            &store,
            &mut session,
            ApplyInput {
                project_id: project.id.clone(),
                text: unjustified.to_string(),
                ..ApplyInput::default()
            },
        )
        .expect_err("expected missing summary error");
        assert!(
            session.runs.is_empty(),
            "unjustified write opened a run: {:?}",
            session.runs
        );

        let first = apply_in_session(
            &store,
            &mut session,
            ApplyInput {
                project_id: project.id.clone(),
                text: ROADMAP.to_string(),
                ..ApplyInput::default()
            },
        )
        .expect("first apply");
        assert!(first.applied, "first apply: {first:?}");
        let second = apply_in_session(
            &store,
            &mut session,
            ApplyInput {
                project_id: project.id.clone(),
                text: "---\npolter: 1\nsummary: Start build\nrationale: Plan approved\n---\n\n## git-metadata-import\nstage: build\n".to_string(),
                ..ApplyInput::default()
            },
        )
        .expect("second apply");
        assert_eq!(
            second.agent_run_id, first.agent_run_id,
            "second apply should reuse the run"
        );

        session.close(&store);
        let run = store
            .get_agent_run(&first.agent_run_id)
            .expect("session run after close");
        assert_eq!(run.status, agentrun::STATUS_COMPLETED);
        assert_eq!(run.agent_label, "claude-code");

        let third = apply_in_session(
            &store,
            &mut session,
            ApplyInput {
                project_id: project.id.clone(),
                text: format!("{ROADMAP}- [ ] Backfill old commits\n"),
                ..ApplyInput::default()
            },
        )
        .expect("apply after close");
        assert_ne!(
            third.agent_run_id, first.agent_run_id,
            "apply after close should open a new run"
        );
    }
}
