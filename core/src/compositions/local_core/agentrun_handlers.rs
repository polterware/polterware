//! The AgentRunService handlers: run history and the Unified Change History.
//!
//! The agent run history: a project's runs, one run with its steps, and the
//! Unified Change History (`ListChanges`) with its actor and scope filters.

use std::sync::Arc;

use crate::compositions::wedge;
use crate::domains::agentrun;
use crate::local_v1;
use crate::providers::sqlite::{self, change_limit, format_rfc3339};

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::{Core, internal, invalid};

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary(
            "/polterware.local.v1.AgentRunService/ListAgentRuns",
            list_agent_runs,
        ),
        Route::unary(
            "/polterware.local.v1.AgentRunService/GetAgentRun",
            get_agent_run,
        ),
        Route::unary(
            "/polterware.local.v1.AgentRunService/ListChanges",
            list_changes,
        ),
    ]
}

pub fn list_agent_runs(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListAgentRunsRequest>,
) -> Result<local_v1::ListAgentRunsResponse, ConnectError> {
    if request.message.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let runs = core
        .store()
        .list_agent_runs(&request.message.project_id, request.message.limit as i64)
        .map_err(internal)?;
    Ok(local_v1::ListAgentRunsResponse {
        runs: runs.iter().map(agent_run_proto).collect(),
    })
}

pub fn get_agent_run(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::GetAgentRunRequest>,
) -> Result<local_v1::GetAgentRunResponse, ConnectError> {
    if request.message.id.is_empty() {
        return Err(invalid("agent run id is required"));
    }
    let record = match core.store().get_agent_run_record(&request.message.id) {
        Ok(record) => record,
        Err(err) if is_unknown_run(&err) => {
            return Err(ConnectError::not_found(err.to_string()));
        }
        Err(err) => return Err(internal(err)),
    };
    Ok(local_v1::GetAgentRunResponse {
        run: Some(agent_run_proto(&record.run)),
        steps: record.steps.iter().map(agent_run_step_proto).collect(),
    })
}

pub fn list_changes(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListChangesRequest>,
) -> Result<local_v1::ListChangesResponse, ConnectError> {
    let message = request.message;
    if message.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let mut actor_kind = message.actor_kind.clone();
    if !actor_kind.is_empty() {
        actor_kind = agentrun::normalize_actor_kind(&actor_kind)
            .map_err(|err| invalid(err.to_string()))?
            .to_string();
    }
    let refs = wedge::scope_refs(&message.scope_node, &message.scope_nodes);
    let resolved = {
        let store = core.store();
        wedge::load_node_scopes(&store, &message.project_id, &refs, message.scope_exact)
            .map_err(|err| invalid(err.to_string()))?
    };
    let subject = message.subject_id.trim();
    let rows = if !subject.is_empty() {
        core.store()
            .list_subject_changes(
                &message.project_id,
                &actor_kind,
                subject,
                message.limit as i64,
            )
            .map_err(internal)?
    } else if resolved.set.is_nil() {
        core.store()
            .list_changes(&message.project_id, &actor_kind, message.limit as i64)
            .map_err(internal)?
    } else {
        let store = core.store();
        let rows = store
            .list_all_changes(&message.project_id, &actor_kind)
            .map_err(internal)?;
        let index = wedge::entity_scope_index(&store, &message.project_id)
            .map_err(|err| internal(err.to_string()))?;
        let mut rows = wedge::filter_changes_by_scope(&resolved, &index, &rows);
        let limit = change_limit(message.limit as i64) as usize;
        if rows.len() > limit {
            rows.truncate(limit);
        }
        rows
    };
    Ok(local_v1::ListChangesResponse {
        changes: rows.iter().map(change_proto).collect(),
    })
}

/// Reports whether the store said the run does not exist.
fn is_unknown_run(err: &sqlite::Error) -> bool {
    matches!(err, sqlite::Error::AgentRun(agentrun::Error::UnknownRun))
}

pub fn agent_run_proto(run: &agentrun::Run) -> local_v1::AgentRun {
    let mut actor_kind = run.actor_kind.clone();
    if actor_kind.is_empty() {
        actor_kind = agentrun::ACTOR_KIND_AGENT.to_string();
    }
    local_v1::AgentRun {
        id: run.id.clone(),
        project_id: run.project_id.clone(),
        task: run.task.clone(),
        agent_label: run.agent_label.clone(),
        status: run.status.clone(),
        actor_kind,
        created_at: format_rfc3339(run.created_at),
        completed_at: run.completed_at.map(format_rfc3339).unwrap_or_default(),
    }
}

pub fn agent_run_step_proto(step: &agentrun::Step) -> local_v1::AgentRunStep {
    local_v1::AgentRunStep {
        id: step.id.clone(),
        agent_run_id: step.agent_run_id.clone(),
        kind: step.kind.clone(),
        summary: step.summary.clone(),
        payload_json: step.payload_json.clone(),
        created_at: format_rfc3339(step.created_at),
    }
}

pub fn change_proto(row: &sqlite::ChangeRow) -> local_v1::ProjectChange {
    local_v1::ProjectChange {
        step_id: row.step_id.clone(),
        agent_run_id: row.agent_run_id.clone(),
        project_id: row.project_id.clone(),
        actor_kind: row.actor_kind.clone(),
        agent_label: row.agent_label.clone(),
        run_status: row.run_status.clone(),
        run_task: row.run_task.clone(),
        summary: row.summary.clone(),
        payload_json: row.payload_json.clone(),
        created_at: format_rfc3339(row.created_at),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositions::local_core::test_support::{Harness, TEST_TOKEN};

    #[test]
    fn agent_runs_and_changes_answer_over_connect() {
        let harness = Harness::start();
        let project_id = {
            let store = harness.core.store();
            let project = crate::compositions::wedge::init_project(
                &store,
                crate::compositions::wedge::InitInput {
                    name: "Runs".to_string(),
                    ..crate::compositions::wedge::InitInput::default()
                },
            )
            .expect("project");
            project.id
        };

        let run = {
            let store = harness.core.store();
            let run = agentrun::start(&project_id, "Work", "cli", None).expect("start");
            store.create_agent_run(run.clone()).expect("save");
            run
        };
        let listed = core_call::<local_v1::ListAgentRunsResponse>(
            &harness,
            "/polterware.local.v1.AgentRunService/ListAgentRuns",
            &format!("{{\"projectId\":\"{project_id}\"}}"),
        );
        assert_eq!(listed.runs.len(), 1);
        assert_eq!(listed.runs[0].actor_kind, "agent");

        let got = core_call::<local_v1::GetAgentRunResponse>(
            &harness,
            "/polterware.local.v1.AgentRunService/GetAgentRun",
            &format!("{{\"id\":\"{}\"}}", run.id),
        );
        assert_eq!(
            got.run.as_ref().map(|run| run.id.as_str()),
            Some(run.id.as_str())
        );

        let changes = core_call::<local_v1::ListChangesResponse>(
            &harness,
            "/polterware.local.v1.AgentRunService/ListChanges",
            &format!("{{\"projectId\":\"{project_id}\"}}"),
        );
        assert!(changes.changes.is_empty());
    }

    fn core_call<Res: serde::de::DeserializeOwned>(
        harness: &Harness,
        path: &str,
        body: &str,
    ) -> Res {
        let reply = harness.unary(path, Some(TEST_TOKEN), body);
        assert_eq!(reply.status, 200, "{}", reply.text());
        serde_json::from_slice(&reply.body).expect("response body")
    }

    #[test]
    fn list_changes_validates_project_and_actor() {
        let reply = {
            let harness = Harness::start();
            harness.unary(
                "/polterware.local.v1.AgentRunService/ListChanges",
                Some(TEST_TOKEN),
                "{}",
            )
        };
        assert_eq!(reply.status, 400);
        assert_eq!(reply.error_code(), "invalid_argument");
        assert_eq!(
            reply.json()["message"],
            serde_json::Value::String("project id is required".to_string())
        );

        let harness = Harness::start();
        let reply = harness.unary(
            "/polterware.local.v1.AgentRunService/ListChanges",
            Some(TEST_TOKEN),
            "{\"projectId\":\"p\",\"actorKind\":\"robot\"}",
        );
        assert_eq!(reply.status, 400);
        assert_eq!(
            reply.json()["message"],
            serde_json::Value::String("invalid actor kind: \"robot\"".to_string())
        );
    }

    #[test]
    fn get_agent_run_requires_an_id_and_reports_unknown_runs() {
        let harness = Harness::start();
        let missing = harness.unary(
            "/polterware.local.v1.AgentRunService/GetAgentRun",
            Some(TEST_TOKEN),
            "{}",
        );
        assert_eq!(missing.status, 400);
        assert_eq!(missing.error_code(), "invalid_argument");

        let unknown = harness.unary(
            "/polterware.local.v1.AgentRunService/GetAgentRun",
            Some(TEST_TOKEN),
            "{\"id\":\"nope\"}",
        );
        assert_eq!(unknown.status, 404);
        assert_eq!(unknown.error_code(), "not_found");
        assert_eq!(
            unknown.json()["message"],
            serde_json::Value::String("unknown agent run".to_string())
        );
    }
}
