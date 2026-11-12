//! The `DispatchHostService` handlers Polterware Dispatch reads from this core.
//!
//! `DispatchHostService` is what Polterware Dispatch reads from this core: the
//! projects, where a run works and what it works on, the open cards, and the
//! AgentRun a run writes under. Dispatch owns its runs; Polterware keeps none.

use std::sync::Arc;

use crate::compositions::wedge;
use crate::domains::agentrun;
use crate::domains::work;
use crate::local_v1;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::{Core, internal, invalid};

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary(
            "/polterware.local.v1.DispatchHostService/ListHostProjects",
            list_host_projects,
        ),
        Route::unary(
            "/polterware.local.v1.DispatchHostService/ResolveHostProject",
            resolve_host_project,
        ),
        Route::unary(
            "/polterware.local.v1.DispatchHostService/ResolveRun",
            resolve_run,
        ),
        Route::unary(
            "/polterware.local.v1.DispatchHostService/ListHostCards",
            list_host_cards,
        ),
        Route::unary(
            "/polterware.local.v1.DispatchHostService/StartHostAgentRun",
            start_host_agent_run,
        ),
        Route::unary(
            "/polterware.local.v1.DispatchHostService/CompleteHostAgentRun",
            complete_host_agent_run,
        ),
    ]
}

pub fn list_host_projects(
    core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListHostProjectsRequest>,
) -> Result<local_v1::ListHostProjectsResponse, ConnectError> {
    let items = core.store().list_projects(false).map_err(internal)?;
    Ok(local_v1::ListHostProjectsResponse {
        projects: items
            .iter()
            .map(|project| local_v1::HostProject {
                id: project.id.clone(),
                name: project.name.clone(),
            })
            .collect(),
    })
}

/// The project with id, or the one whose repository is cloned at dir.
pub fn resolve_host_project(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ResolveHostProjectRequest>,
) -> Result<local_v1::ResolveHostProjectResponse, ConnectError> {
    let project = wedge::resolve_project(&core.store(), &request.message.id, &request.message.dir)
        .map_err(|err| ConnectError::not_found(err.to_string()))?;
    Ok(local_v1::ResolveHostProjectResponse {
        project: Some(local_v1::HostProject {
            id: project.id,
            name: project.name,
        }),
    })
}

/// Settles a run of the project: its directory (the request's, else the
/// repository's clone, else the card's component repository, else the only
/// one), the component's kinds and technologies, and the card's text.
pub fn resolve_run(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ResolveRunRequest>,
) -> Result<local_v1::ResolveRunResponse, ConnectError> {
    let message = request.message;
    let project_id = message.project_id.trim().to_string();
    if core.store().get_project(&project_id).is_err() {
        return Err(invalid(format!("unknown project {project_id}")));
    }
    let card_slug = message.card_slug.trim().to_string();
    let mut dir = wedge::expand_path(&message.dir);
    if dir.is_empty() {
        dir = {
            let store = core.store();
            wedge::run_dir(&store, &project_id, &message.repo, &card_slug)
                .map_err(|err| invalid(err.to_string()))?
        };
    }
    dir = wedge::absolute_path(&dir);
    let mut out = local_v1::ResolveRunResponse {
        dir,
        card_text: {
            let store = core.store();
            wedge::run_card_text(&store, &project_id, &card_slug)
        },
        ..local_v1::ResolveRunResponse::default()
    };
    let node = {
        let store = core.store();
        wedge::run_node(&store, &project_id, &card_slug, &out.dir)
    };
    if let Ok((node, true)) = node {
        out.kinds = split_list(&node.kind);
        out.tech = split_list(&node.tech);
    }
    Ok(out)
}

/// Lists the project's open cards (not removed, not Done) and its
/// repositories, for a run's Card and Repository pills.
pub fn list_host_cards(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListHostCardsRequest>,
) -> Result<local_v1::ListHostCardsResponse, ConnectError> {
    let project_id = request.message.project_id.trim().to_string();
    let snapshot = core
        .store()
        .load_work_snapshot(&project_id)
        .map_err(internal)?;
    let mut out = local_v1::ListHostCardsResponse::default();
    for card in &snapshot.cards {
        if card.status != work::CARD_ACTIVE || card.stage == work::STAGE_DELIVERED {
            continue;
        }
        out.cards.push(local_v1::HostCard {
            slug: card.slug.clone(),
            title: card.title.clone(),
            milestone: card.milestone.clone(),
            stage: card.stage.clone(),
        });
    }
    let repos = core
        .store()
        .list_repositories(&project_id)
        .map_err(internal)?;
    for repository in &repos {
        out.repositories.push(local_v1::HostRepository {
            name: repository.name.clone(),
            remote: repository.remote.clone(),
            local_path: repository.local_path.clone(),
        });
    }
    Ok(out)
}

/// Opens the audited AgentRun a Dispatch run's agent writes under, so Activity
/// shows one history for both.
pub fn start_host_agent_run(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::StartHostAgentRunRequest>,
) -> Result<local_v1::StartHostAgentRunResponse, ConnectError> {
    let project_id = request.message.project_id.trim().to_string();
    if core.store().get_project(&project_id).is_err() {
        return Err(invalid(format!("unknown project {project_id}")));
    }
    let run = agentrun::start(
        &project_id,
        &host_run_task(&request.message.task),
        "dispatch",
        Some(time::OffsetDateTime::now_utc()),
    )
    .map_err(|err| invalid(err.to_string()))?;
    let id = run.id.clone();
    core.store().create_agent_run(run).map_err(internal)?;
    Ok(local_v1::StartHostAgentRunResponse { agent_run_id: id })
}

/// Closes an AgentRun StartHostAgentRun opened; a closed one stays as it is.
pub fn complete_host_agent_run(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::CompleteHostAgentRunRequest>,
) -> Result<local_v1::CompleteHostAgentRunResponse, ConnectError> {
    let run = core
        .store()
        .get_agent_run(request.message.agent_run_id.trim())
        .map_err(|_| ConnectError::not_found("unknown agent run"))?;
    if run.status == agentrun::STATUS_OPEN {
        let done = agentrun::complete(run, Some(time::OffsetDateTime::now_utc()))
            .map_err(|err| invalid(err.to_string()))?;
        core.store().update_agent_run(done).map_err(internal)?;
    }
    Ok(local_v1::CompleteHostAgentRunResponse {})
}

/// The one-line task an AgentRun shows for a delegation.
fn host_run_task(task: &str) -> String {
    let trimmed = task.trim();
    let first = trimmed.split_once('\n').map_or(trimmed, |(line, _)| line);
    let mut line = first.trim().to_string();
    if line.len() > 160 {
        line = String::from_utf8_lossy(&line.as_bytes()[..160]).into_owned();
    }
    if line.is_empty() {
        line = "delegated run".to_string();
    }
    line
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositions::local_core::test_support::{Harness, TEST_TOKEN};

    fn call<Res: serde::de::DeserializeOwned>(harness: &Harness, path: &str, body: &str) -> Res {
        let reply = harness.unary(path, Some(TEST_TOKEN), body);
        assert_eq!(reply.status, 200, "{}", reply.text());
        serde_json::from_slice(&reply.body).expect("response body")
    }

    #[test]
    fn host_run_task_keeps_one_line() {
        assert_eq!(host_run_task("  pay with cards\nmore  "), "pay with cards");
        assert_eq!(host_run_task("   "), "delegated run");
        let long = "x".repeat(200);
        assert_eq!(host_run_task(&long).len(), 160);
    }

    #[test]
    fn split_list_trims_and_drops_empty_parts() {
        assert_eq!(
            split_list(" web app, service ,, database "),
            vec!["web app", "service", "database"]
        );
        assert!(split_list("").is_empty());
    }

    #[test]
    fn dispatch_host_service_resolves_runs_and_opens_their_agent_run() {
        let harness = Harness::start();
        let root = tempfile::tempdir().expect("temp dir");
        let root_path = root.path().to_string_lossy().into_owned();
        if std::process::Command::new("git")
            .args(["-C", &root_path, "init", "-q"])
            .output()
            .is_err()
        {
            return;
        }
        let project = {
            let store = harness.core.store();
            crate::compositions::local_core::test_support::init_with_repo(
                &store, "Shop", &root_path,
            )
            .expect("project with a repository")
        };
        {
            let store = harness.core.store();
            crate::compositions::wedge::apply_polter(
                &store,
                crate::compositions::wedge::ApplyInput {
                    project_id: project.id.clone(),
                    text: "---\npolter: 1\n---\n## checkout: Checkout\nstage: build\n- [ ] Pay\n## old: Old\nstage: delivered\n".to_string(),
                    summary: "Plan".to_string(),
                    rationale: "Cards for Dispatch".to_string(),
                    auto_start_label: "cli".to_string(),
                    ..crate::compositions::wedge::ApplyInput::default()
                },
            )
            .expect("apply");
        }

        let listed = call::<local_v1::ListHostProjectsResponse>(
            &harness,
            "/polterware.local.v1.DispatchHostService/ListHostProjects",
            "{}",
        );
        assert!(
            listed
                .projects
                .iter()
                .any(|item| item.id == project.id && item.name == "Shop")
        );

        let found = call::<local_v1::ResolveHostProjectResponse>(
            &harness,
            "/polterware.local.v1.DispatchHostService/ResolveHostProject",
            &format!("{{\"dir\":{}}}", serde_json::to_string(&root_path).unwrap()),
        );
        assert_eq!(found.project.map(|item| item.id), Some(project.id.clone()));

        let run = call::<local_v1::ResolveRunResponse>(
            &harness,
            "/polterware.local.v1.DispatchHostService/ResolveRun",
            &format!(
                "{{\"projectId\":{},\"cardSlug\":\"checkout\"}}",
                serde_json::to_string(&project.id).unwrap()
            ),
        );
        assert!(!run.dir.is_empty(), "dir = {:?}", run.dir);
        assert!(
            !run.card_text.is_empty(),
            "the card's text is what the classifier reads"
        );

        let cards = call::<local_v1::ListHostCardsResponse>(
            &harness,
            "/polterware.local.v1.DispatchHostService/ListHostCards",
            &format!(
                "{{\"projectId\":{}}}",
                serde_json::to_string(&project.id).unwrap()
            ),
        );
        assert_eq!(cards.cards.len(), 1, "Done cards are left out");
        assert_eq!(cards.cards[0].slug, "checkout");
        assert_eq!(cards.repositories.len(), 1);

        let started = call::<local_v1::StartHostAgentRunResponse>(
            &harness,
            "/polterware.local.v1.DispatchHostService/StartHostAgentRun",
            &format!(
                "{{\"projectId\":{},\"task\":\"pay with cards\\nmore\",\"cardSlug\":\"checkout\"}}",
                serde_json::to_string(&project.id).unwrap()
            ),
        );
        assert!(!started.agent_run_id.is_empty());
        let _ = call::<local_v1::CompleteHostAgentRunResponse>(
            &harness,
            "/polterware.local.v1.DispatchHostService/CompleteHostAgentRun",
            &format!(
                "{{\"agentRunId\":{}}}",
                serde_json::to_string(&started.agent_run_id).unwrap()
            ),
        );
        let run = harness
            .core
            .store()
            .get_agent_run(&started.agent_run_id)
            .expect("agent run");
        assert_ne!(run.status, agentrun::STATUS_OPEN);
        assert_eq!(run.task, "pay with cards");

        let refused = harness.unary(
            "/polterware.local.v1.DispatchHostService/ResolveRun",
            Some(TEST_TOKEN),
            "{\"projectId\":\"nope\"}",
        );
        assert_eq!(refused.status, 400);
        assert_eq!(refused.error_code(), "invalid_argument");
    }
}
