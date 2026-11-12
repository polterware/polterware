//! The justified mutation envelope shared by MCP and the Desktop.

#[allow(unused_imports)]
use super::*;

use time::OffsetDateTime;

use crate::domains::agentrun;
use crate::providers::sqlite::{self, Store};

/// The justified mutation envelope shared by MCP and Desktop.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChangeEnvelope {
    pub summary: String,
    pub rationale: String,
    pub agent_run_id: String,
    pub project_id: String,
    pub paths: Vec<String>,
    /// Lets an agent write without an open run: the core opens and completes a
    /// run with this label (the CLI uses "cli"). MCP leaves it empty, so MCP
    /// writes still require an explicit `agent_run_id`.
    pub auto_start_label: String,
    /// Links the change step to its `change_ops` rows.
    pub batch_id: String,
}

/// Returned after a successful audited mutation.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AuditedWriteResult<T> {
    pub agent_run_id: String,
    pub subject_ids: Vec<String>,
    pub result: T,
}

/// Validates the open run (or starts a human run when `agent_run_id` is empty
/// and `actor_kind` is human), runs the mutation, appends a change step, and
/// completes auto-started runs.
pub fn execute_audited_write<T>(
    store: &Store,
    env: ChangeEnvelope,
    actor_kind: &str,
    tool: &str,
    mutate: impl FnOnce(&agentrun::Run) -> sqlite::Result<(Vec<String>, T)>,
) -> sqlite::Result<AuditedWriteResult<T>> {
    let kind = agentrun::normalize_actor_kind(actor_kind)?;
    agentrun::validate_change_as(kind, &env.summary, &env.rationale)?;
    if env.project_id.is_empty() {
        return Err(agentrun::Error::EmptyProject.into());
    }

    let now = OffsetDateTime::now_utc();
    let mut run_id = env.agent_run_id.clone();
    let mut auto_complete = false;

    if run_id.is_empty() {
        let auto_label = env.auto_start_label.trim().to_string();
        let label = if !auto_label.is_empty() {
            auto_label
        } else if kind != agentrun::ACTOR_KIND_HUMAN {
            return Err(agentrun::Error::MissingAgentRunId.into());
        } else {
            "Desktop".to_string()
        };
        let run =
            agentrun::start_with_actor(&env.project_id, &env.summary, &label, kind, Some(now))?;
        store.create_agent_run(run.clone())?;
        run_id = run.id.clone();
        auto_complete = true;
    }

    let run = store.get_agent_run(&run_id)?;
    agentrun::can_append(&run)?;
    agentrun::match_project(&run, &env.project_id)?;
    if !run.actor_kind.is_empty() && run.actor_kind != kind {
        return Err(sqlite::Error::message(format!(
            "invalid actor kind: run is {}, write is {kind}",
            run.actor_kind
        )));
    }

    let (subject_ids, result) = mutate(&run)?;

    store
        .append_agent_run_step_if_open(&run_id, |open: &agentrun::Run| {
            agentrun::match_project(open, &env.project_id)?;
            agentrun::new_batch_change_step(
                &open.id,
                &env.summary,
                &env.rationale,
                tool,
                &env.batch_id,
                &subject_ids,
                &env.paths,
                Some(now),
            )
        })
        .map_err(|err| {
            sqlite::Error::message(format!("mutation saved but change step failed: {err}"))
        })?;

    if auto_complete {
        let done = agentrun::complete(run.clone(), Some(now)).map_err(|err| {
            sqlite::Error::message(format!("mutation saved but complete failed: {err}"))
        })?;
        store.update_agent_run(done).map_err(|err| {
            sqlite::Error::message(format!("mutation saved but complete failed: {err}"))
        })?;
    }

    Ok(AuditedWriteResult {
        agent_run_id: run_id,
        subject_ids,
        result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::{knowledge, projects};
    use crate::platform::ids;
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    /// A project row; the Go test also links a repository, which no assertion
    /// of this test reads.
    fn project(store: &Store) -> projects::Project {
        let project = projects::new_project(&ids::new_ulid(), "Demo").expect("project");
        store.save_project(project.clone()).expect("save project");
        project
    }

    #[test]
    fn human_write_auto_starts_and_completes_a_run() {
        let TempStore { store, .. } = TempStore::new();
        let project = project(&store);

        let written = execute_audited_write(
            &store,
            ChangeEnvelope {
                summary: "Create doc".to_string(),
                rationale: "Project needs its docs recorded".to_string(),
                project_id: project.id.clone(),
                ..ChangeEnvelope::default()
            },
            agentrun::ACTOR_KIND_HUMAN,
            "desktop.knowledge.save",
            |_run| {
                let mut item = knowledge::new_item(
                    "kn-1",
                    &project.id,
                    "Local-first truth",
                    "# Rules",
                    knowledge::KIND_DOC,
                    Vec::new(),
                )
                .map_err(|err| sqlite::Error::message(err.to_string()))?;
                item.owner = agentrun::ACTOR_HUMAN.to_string();
                store.save_knowledge(item.clone())?;
                Ok((vec![item.id.clone()], item))
            },
        )
        .expect("audited write");

        assert!(
            !written.agent_run_id.is_empty(),
            "expected auto-created agent run id"
        );
        let record = store
            .get_agent_run_record(&written.agent_run_id)
            .expect("run record");
        assert_eq!(record.run.actor_kind, agentrun::ACTOR_KIND_HUMAN);
        assert_eq!(record.run.status, agentrun::STATUS_COMPLETED);
        assert_eq!(record.steps.len(), 1, "steps: {:?}", record.steps);
        assert_eq!(record.steps[0].kind, agentrun::KIND_CHANGE);

        let changes = store
            .list_changes(&project.id, agentrun::ACTOR_KIND_HUMAN, 10)
            .expect("changes");
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].summary, "Create doc");

        // One entity's history: the change that named it, and nothing for
        // another id.
        let subject = store
            .list_subject_changes(&project.id, "", "kn-1", 10)
            .expect("subject changes");
        assert_eq!(subject.len(), 1);
        assert_eq!(subject[0].summary, "Create doc");
        let other = store
            .list_subject_changes(&project.id, "", "kn-2", 10)
            .expect("other subject");
        assert!(other.is_empty(), "other subject: {other:?}");
    }

    #[test]
    fn list_changes_hides_workspace_layout_writes() {
        let TempStore { store, .. } = TempStore::new();
        save_project(&store, "p", OffsetDateTime::UNIX_EPOCH);
        for tool in [
            "desktop.canvas.update",
            "desktop.work.milestone.archive",
            "desktop.knowledge.save",
        ] {
            execute_audited_write(
                &store,
                ChangeEnvelope {
                    summary: tool.to_string(),
                    rationale: "test".to_string(),
                    project_id: "p".to_string(),
                    ..ChangeEnvelope::default()
                },
                agentrun::ACTOR_KIND_HUMAN,
                tool,
                |_run| Ok((Vec::new(), ())),
            )
            .expect("audited write");
        }

        let changes = store.list_all_changes("p", "").expect("changes");
        // Milestone changes move cards, so they stay in the history.
        assert_eq!(changes.len(), 2, "canvas writes are hidden: {:?}", changes);
        for row in &changes {
            assert_ne!(row.summary, "desktop.canvas.update");
        }
    }

    #[test]
    fn an_agent_write_needs_an_open_run() {
        let TempStore { store, .. } = TempStore::new();
        let err = execute_audited_write(
            &store,
            ChangeEnvelope {
                summary: "x".to_string(),
                rationale: "y".to_string(),
                project_id: "p".to_string(),
                ..ChangeEnvelope::default()
            },
            agentrun::ACTOR_KIND_AGENT,
            "fact.upsert",
            |_run| Ok((Vec::new(), ())),
        )
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            agentrun::Error::MissingAgentRunId.to_string()
        );
    }
}
