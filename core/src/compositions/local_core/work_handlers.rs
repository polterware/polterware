//! Port of `backend/compositions/local-core/work_handlers.go`.
//!
//! The Work service's reads and writes over cards, Structure, Polter text,
//! handoffs, projections, file context, and milestones, plus the helpers the
//! feature and Inbox handlers share (the apply path, the proto of a card, and
//! the diagnostics of refused Polter text).

use std::sync::Arc;

use axum::http::HeaderMap;
use time::OffsetDateTime;

use crate::compositions::wedge;
use crate::domains::agentrun;
use crate::domains::polter;
use crate::domains::structure;
use crate::domains::work;
use crate::local_v1;
use crate::platform::ids;
use crate::providers::sqlite::{self, format_rfc3339, format_rfc3339_zone};

use super::audited_write::{change_envelope_from_proto, map_audited_err};
use super::connect::{Code, ConnectError, ConnectRequest, Route};
use super::lifecycle::client_of;
use super::server::{Core, internal, invalid};

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary(
            "/polterware.local.v1.WorkService/ListWorkCards",
            list_work_cards,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/ListStructure",
            list_structure,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/ListStructureHistory",
            list_structure_history,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/ListAgentSessions",
            list_agent_sessions,
        ),
        Route::unary("/polterware.local.v1.WorkService/ApplyPolter", apply_polter),
        Route::unary("/polterware.local.v1.WorkService/LintPolter", lint_polter),
        Route::unary("/polterware.local.v1.WorkService/QueryPolter", query_polter),
        Route::unary(
            "/polterware.local.v1.WorkService/WriteHandoff",
            write_handoff,
        ),
        Route::unary("/polterware.local.v1.WorkService/EndSession", end_session),
        Route::unary(
            "/polterware.local.v1.WorkService/ExportProjections",
            export_projections,
        ),
        Route::unary("/polterware.local.v1.WorkService/FileContext", file_context),
        Route::unary(
            "/polterware.local.v1.WorkService/ListMilestones",
            list_milestones,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/CreateMilestone",
            create_milestone,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/UpdateMilestone",
            update_milestone,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/ArchiveMilestone",
            archive_milestone,
        ),
        Route::unary(
            "/polterware.local.v1.WorkService/ReorderMilestones",
            reorder_milestones,
        ),
    ]
}

/// `polterware.local.v1.WorkService.ListWorkCards`.
pub fn list_work_cards(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListWorkCardsRequest>,
) -> Result<local_v1::ListWorkCardsResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let store = core.store();
    let snapshot = store
        .load_work_snapshot(&msg.project_id)
        .map_err(internal)?;
    let refs = wedge::scope_refs(&msg.scope_node, &msg.scope_nodes);
    let resolved = wedge::load_node_scopes(&store, &msg.project_id, &refs, msg.scope_exact)
        .map_err(|err| invalid(err.to_string()))?;
    let mut cards = Vec::new();
    for card in wedge::filter_cards_by_scope(&resolved, &snapshot.cards) {
        if card.status == work::CARD_ACTIVE {
            cards.push(work_card_proto(&card));
        }
    }
    Ok(local_v1::ListWorkCardsResponse { cards })
}

/// `polterware.local.v1.WorkService.ListStructure`.
pub fn list_structure(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListStructureRequest>,
) -> Result<local_v1::ListStructureResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let store = core.store();
    let point = msg.as_of.trim();
    if !point.is_empty() {
        let rows = wedge::structure_at(&store, &msg.project_id, point)
            .map_err(|err| invalid(err.to_string()))?;
        let mut nodes = Vec::with_capacity(rows.len());
        for row in &rows {
            let mut node = structure_node_proto(row);
            node.status = structure::STATUS_PROPOSED.to_string();
            nodes.push(node);
        }
        return Ok(local_v1::ListStructureResponse {
            nodes,
            historical: true,
        });
    }
    let snapshot = store
        .load_work_snapshot(&msg.project_id)
        .map_err(internal)?;
    let rows = wedge::structure_rows(&store, &snapshot);
    let nodes: Vec<local_v1::StructureNode> = rows.iter().map(structure_node_proto).collect();
    Ok(local_v1::ListStructureResponse {
        nodes,
        historical: false,
    })
}

/// `polterware.local.v1.WorkService.ListStructureHistory`.
pub fn list_structure_history(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListStructureHistoryRequest>,
) -> Result<local_v1::ListStructureHistoryResponse, ConnectError> {
    if request.message.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let store = core.store();
    let entries =
        wedge::structure_history(&store, &request.message.project_id).map_err(internal)?;
    let mut out = Vec::with_capacity(entries.len());
    for entry in &entries {
        out.push(local_v1::StructureHistoryEntry {
            batch_id: entry.batch_id.clone(),
            at: format_rfc3339(entry.at),
            agent_run_id: entry.agent_run_id.clone(),
            node_ops: entry.node_ops as i32,
            summary: entry.summary.clone(),
        });
    }
    Ok(local_v1::ListStructureHistoryResponse { entries: out })
}

/// `polterware.local.v1.WorkService.ListAgentSessions`.
pub fn list_agent_sessions(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListAgentSessionsRequest>,
) -> Result<local_v1::ListAgentSessionsResponse, ConnectError> {
    if request.message.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let store = core.store();
    let (sessions, handoffs, branch) =
        wedge::agent_sessions(&store, &request.message.project_id).map_err(internal)?;
    let mut response = local_v1::ListAgentSessionsResponse {
        branch,
        ..local_v1::ListAgentSessionsResponse::default()
    };
    for session in &sessions {
        let mut out = local_v1::AgentSession {
            run_id: session.run_id.clone(),
            agent: session.agent.clone(),
            task: session.task.clone(),
            started_at: String::new(),
            claims: Vec::new(),
        };
        if session.started_at != OffsetDateTime::UNIX_EPOCH {
            out.started_at = format_rfc3339(session.started_at);
        }
        for claim in &session.claims {
            out.claims.push(local_v1::SessionClaim {
                slug: claim.slug.clone(),
                number: claim.number,
                title: claim.title.clone(),
                scope: claim.scope.clone(),
            });
        }
        response.sessions.push(out);
    }
    for handoff in &handoffs {
        response.handoffs.push(local_v1::SessionHandoff {
            slug: handoff.slug.clone(),
            branch: handoff.branch.clone(),
            goal: handoff.goal.clone(),
            done: handoff.done.clone(),
            in_progress: handoff.in_progress.clone(),
            next: handoff.next.clone(),
            agent: handoff.agent.clone(),
            at: format_rfc3339(handoff.at),
        });
    }
    Ok(response)
}

/// `polterware.local.v1.WorkService.ApplyPolter`.
pub fn apply_polter(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ApplyPolterRequest>,
) -> Result<local_v1::ApplyPolterResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let change = msg.change.as_ref();
    let in_ = wedge::ApplyInput {
        project_id: msg.project_id.clone(),
        text: msg.text.clone(),
        dry_run: msg.dry_run,
        summary: change
            .map(|change| change.summary.clone())
            .unwrap_or_default(),
        rationale: change
            .map(|change| change.rationale.clone())
            .unwrap_or_default(),
        agent_run_id: change
            .map(|change| change.agent_run_id.clone())
            .unwrap_or_default(),
        ..wedge::ApplyInput::default()
    };
    let client = client_of(&request.headers);
    let result = apply_as(
        &core,
        &client,
        &msg.session_id,
        &msg.session_label,
        "desktop.work.apply",
        in_,
    )?;
    Ok(apply_response(&result))
}

/// Applies Polter text for a client: the Desktop writes as a human under
/// `desktop_tool` and never claims; agent clients (the CLI) write as an agent,
/// join their session run when `session_id` is set, and otherwise open a run
/// labelled with the client. Errors are already Connect errors.
pub(super) fn apply_as(
    core: &Core,
    client: &str,
    session_id: &str,
    session_label: &str,
    desktop_tool: &str,
    mut in_: wedge::ApplyInput,
) -> Result<wedge::ApplyResult, ConnectError> {
    if client == "desktop" {
        // The Desktop never claims: coordination lines have no agent.
        in_.actor_kind = agentrun::ACTOR_KIND_HUMAN.to_string();
        in_.tool = desktop_tool.to_string();
    } else {
        in_.agent = session_label.trim().to_string();
        in_.auto_start_label = client.to_string();
    }
    let store = core.store();
    let outcome = if !session_id.is_empty() && client != "desktop" {
        let mut applied: Option<std::result::Result<wedge::ApplyResult, wedge::ApplyFailure>> =
            None;
        core.sessions
            .apply(session_id, session_label, |session| {
                in_.auto_start_label.clear();
                applied = Some(wedge::apply_in_session(&store, session, in_.clone()));
                Ok(())
            })
            .map_err(internal)?;
        applied.expect("the session apply runs once")
    } else {
        wedge::apply_polter(&store, in_.clone())
    };
    match outcome {
        Ok(result) => Ok(result),
        Err(failure) => {
            if failure.is_polter_invalid() {
                Err(invalid_polter(&failure.result.diagnostics))
            } else {
                Err(map_apply_err(&failure.error))
            }
        }
    }
}

/// `polterware.local.v1.WorkService.LintPolter`.
pub fn lint_polter(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::LintPolterRequest>,
) -> Result<local_v1::LintPolterResponse, ConnectError> {
    // Without a project only syntax is checked.
    let store = core.store();
    let refs = if request.message.project_id.is_empty() {
        None
    } else {
        Some(wedge::polter_references(&store, &request.message.project_id).map_err(internal)?)
    };
    let (_, diags) = polter::lint(&request.message.text, refs.as_ref());
    Ok(local_v1::LintPolterResponse {
        diagnostics: diagnostics_proto(&diags),
    })
}

/// `polterware.local.v1.WorkService.QueryPolter`.
pub fn query_polter(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::QueryPolterRequest>,
) -> Result<local_v1::QueryPolterResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let store = core.store();
    let refs = wedge::scope_refs(&msg.scope_node, &msg.scope_nodes);
    let scope = wedge::resolve_scope_refs(&store, &msg.project_id, &refs, msg.scope_exact)
        .map_err(|err| invalid(err.to_string()))?;
    let text = wedge::query_polter_scoped(&store, &msg.project_id, &msg.query, &scope)
        .map_err(|err| invalid(err.to_string()))?;
    Ok(local_v1::QueryPolterResponse { text })
}

/// `polterware.local.v1.WorkService.WriteHandoff`.
pub fn write_handoff(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::WriteHandoffRequest>,
) -> Result<local_v1::WriteHandoffResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let change = msg.change.clone();
    let mut agent = msg.agent.trim().to_string();
    if agent.is_empty() {
        agent = msg.session_label.trim().to_string();
    }
    let mut in_ = wedge::HandoffInput {
        project_id: msg.project_id.clone(),
        card_slug: msg.card_slug.clone(),
        note: msg.note.clone(),
        agent,
        summary: change
            .as_ref()
            .map(|change| change.summary.clone())
            .unwrap_or_default(),
        rationale: change
            .as_ref()
            .map(|change| change.rationale.clone())
            .unwrap_or_default(),
        apply: wedge::ApplyInput::default(),
    };
    in_.apply.agent_run_id = change
        .as_ref()
        .map(|change| change.agent_run_id.clone())
        .unwrap_or_default();
    let client = client_of(&request.headers);
    if client == "desktop" {
        in_.apply.actor_kind = agentrun::ACTOR_KIND_HUMAN.to_string();
        in_.apply.tool = "desktop.work.handoff".to_string();
    } else {
        in_.apply.auto_start_label = client.clone();
        in_.apply.tool = "polter.handoff".to_string();
    }
    let store = core.store();
    let (apply_in, hints, err) = wedge::compose_handoff(&store, &in_);
    if let Some(err) = err {
        return Err(invalid(err.to_string()));
    }
    let outcome = if !msg.session_id.is_empty() && client != "desktop" {
        let mut applied: Option<std::result::Result<wedge::ApplyResult, wedge::ApplyFailure>> =
            None;
        core.sessions
            .apply(&msg.session_id, &msg.session_label, |session| {
                let mut apply_in = apply_in.clone();
                apply_in.auto_start_label.clear();
                applied = Some(wedge::apply_in_session(&store, session, apply_in));
                Ok(())
            })
            .map_err(internal)?;
        applied.expect("the session apply runs once")
    } else {
        wedge::apply_polter(&store, apply_in)
    };
    let (mut result, error) = match outcome {
        Ok(result) => (result, None),
        Err(failure) => (
            failure.result.as_ref().clone(),
            Some((failure.is_polter_invalid(), failure.error)),
        ),
    };
    let mut diagnostics = hints;
    diagnostics.append(&mut result.diagnostics);
    result.diagnostics = diagnostics;
    if let Some((polter_invalid, _)) = &error
        && *polter_invalid
    {
        return Err(invalid_polter(&result.diagnostics));
    }
    if let Some((_, error)) = error {
        return Err(map_apply_err(&error));
    }
    let applied = apply_response(&result);
    Ok(local_v1::WriteHandoffResponse {
        applied: applied.applied,
        ops: applied.ops,
        summary: applied.summary,
        diagnostics: applied.diagnostics,
        agent_run_id: applied.agent_run_id,
        batch_id: applied.batch_id,
    })
}

/// `polterware.local.v1.WorkService.EndSession`.
pub fn end_session(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::EndSessionRequest>,
) -> Result<local_v1::EndSessionResponse, ConnectError> {
    let Some(mut session) = core.sessions.end(&request.message.session_id) else {
        return Ok(local_v1::EndSessionResponse::default());
    };
    let count = session.runs.len() as i32;
    let store = core.store();
    session.close(&store);
    Ok(local_v1::EndSessionResponse {
        completed_runs: count,
    })
}

/// `polterware.local.v1.WorkService.ExportProjections`.
pub fn export_projections(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ExportProjectionsRequest>,
) -> Result<local_v1::ExportProjectionsResponse, ConnectError> {
    if request.message.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let store = core.store();
    let files =
        wedge::export_projections(&store, &request.message.project_id, request.message.dry_run)
            .map_err(internal)?;
    let mut proto = Vec::with_capacity(files.len());
    for file in &files {
        proto.push(local_v1::ProjectionFile {
            repo: file.repo.clone(),
            path: file.path.clone(),
            action: file.action.clone(),
            reason: file.reason.clone(),
        });
    }
    Ok(local_v1::ExportProjectionsResponse { files: proto })
}

/// Serves the context hook: the brief of the node that holds a file, once per
/// session.
pub fn file_context(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::FileContextRequest>,
) -> Result<local_v1::FileContextResponse, ConnectError> {
    if request.message.project_id.is_empty() || request.message.file.trim().is_empty() {
        return Err(invalid("project id and file are required"));
    }
    let store = core.store();
    let (text, node_path) = wedge::file_context(
        &store,
        &request.message.project_id,
        &request.message.session_id,
        &request.message.file,
    )
    .map_err(internal)?;
    Ok(local_v1::FileContextResponse { text, node_path })
}

/// `polterware.local.v1.WorkService.ListMilestones`.
pub fn list_milestones(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListMilestonesRequest>,
) -> Result<local_v1::ListMilestonesResponse, ConnectError> {
    if request.message.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let milestones = core
        .store()
        .list_milestones(&request.message.project_id)
        .map_err(internal)?;
    Ok(local_v1::ListMilestonesResponse {
        milestones: milestones_proto(&milestones),
    })
}

/// `polterware.local.v1.WorkService.CreateMilestone`.
pub fn create_milestone(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::CreateMilestoneRequest>,
) -> Result<local_v1::CreateMilestoneResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let draft = work::Milestone {
        name: msg.name.clone(),
        goal: msg.goal.clone(),
        exit_criteria: msg.exit_criteria.clone(),
        target_date: msg.target_date.clone(),
        state: msg.state.clone(),
        ..work::Milestone::default()
    };
    {
        let store = core.store();
        let existing = store.list_milestones(&msg.project_id).map_err(internal)?;
        work::prepare_milestone_create(draft.clone(), &existing)
            .map_err(map_milestone_domain_err)?;
    }
    let env = change_envelope_from_proto(msg.change.as_ref()).map_err(map_audited_err)?;
    let saved = write_milestone(
        &core,
        &request.headers,
        &msg.project_id,
        "create",
        env,
        |store, run, batch_id| {
            let created =
                store.create_milestone(&msg.project_id, &run.id, batch_id, draft.clone())?;
            Ok((vec![created.id.clone()], created))
        },
    )?;
    Ok(local_v1::CreateMilestoneResponse {
        milestone: Some(milestone_proto(&saved)),
    })
}

/// `polterware.local.v1.WorkService.UpdateMilestone`.
pub fn update_milestone(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::UpdateMilestoneRequest>,
) -> Result<local_v1::UpdateMilestoneResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() || msg.id.is_empty() {
        return Err(invalid("project id and milestone id are required"));
    }
    let draft = work::Milestone {
        id: msg.id.clone(),
        name: msg.name.clone(),
        goal: msg.goal.clone(),
        exit_criteria: msg.exit_criteria.clone(),
        target_date: msg.target_date.clone(),
        state: msg.state.clone(),
        ..work::Milestone::default()
    };
    {
        let store = core.store();
        let existing = store.list_milestones(&msg.project_id).map_err(internal)?;
        work::prepare_milestone_update(draft.clone(), &existing)
            .map_err(map_milestone_domain_err)?;
    }
    let env = change_envelope_from_proto(msg.change.as_ref()).map_err(map_audited_err)?;
    let saved = write_milestone(
        &core,
        &request.headers,
        &msg.project_id,
        "update",
        env,
        |store, run, batch_id| {
            let updated =
                store.update_milestone(&msg.project_id, &run.id, batch_id, draft.clone())?;
            Ok((vec![updated.id.clone()], updated))
        },
    )?;
    Ok(local_v1::UpdateMilestoneResponse {
        milestone: Some(milestone_proto(&saved)),
    })
}

/// `polterware.local.v1.WorkService.ArchiveMilestone`.
pub fn archive_milestone(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ArchiveMilestoneRequest>,
) -> Result<local_v1::ArchiveMilestoneResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() || msg.id.is_empty() {
        return Err(invalid("project id and milestone id are required"));
    }
    let env = change_envelope_from_proto(msg.change.as_ref()).map_err(map_audited_err)?;
    let action = if msg.archived { "archive" } else { "restore" };
    let saved = write_milestone(
        &core,
        &request.headers,
        &msg.project_id,
        action,
        env,
        |store, run, batch_id| {
            let item = store.set_milestone_archived(
                &msg.project_id,
                &run.id,
                batch_id,
                &msg.id,
                msg.archived,
            )?;
            Ok((vec![item.id.clone()], item))
        },
    )?;
    Ok(local_v1::ArchiveMilestoneResponse {
        milestone: Some(milestone_proto(&saved)),
    })
}

/// `polterware.local.v1.WorkService.ReorderMilestones`.
pub fn reorder_milestones(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ReorderMilestonesRequest>,
) -> Result<local_v1::ReorderMilestonesResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let env = change_envelope_from_proto(msg.change.as_ref()).map_err(map_audited_err)?;
    let milestones = audit_milestone(
        &core,
        &request.headers,
        &msg.project_id,
        "reorder",
        env,
        |store, run, batch_id| {
            let milestones =
                store.reorder_milestones(&msg.project_id, &run.id, batch_id, &msg.milestone_ids)?;
            Ok((msg.milestone_ids.clone(), milestones))
        },
    )?;
    Ok(local_v1::ReorderMilestonesResponse {
        milestones: milestones_proto(&milestones),
    })
}

/// Runs one milestone write and returns its milestone.
fn write_milestone(
    core: &Arc<Core>,
    headers: &HeaderMap,
    project_id: &str,
    action: &str,
    env: wedge::ChangeEnvelope,
    mutate: impl FnOnce(
        &sqlite::Store,
        &agentrun::Run,
        &str,
    ) -> sqlite::Result<(Vec<String>, work::Milestone)>,
) -> Result<work::Milestone, ConnectError> {
    let milestone = audit_milestone(core, headers, project_id, action, env, mutate)?;
    if milestone.id.is_empty() {
        return Err(internal("milestone write returned work.Milestone"));
    }
    Ok(milestone)
}

/// Runs one milestone write as an audited change. Milestones are created in the
/// Desktop; another client writes as an agent.
fn audit_milestone<T>(
    core: &Arc<Core>,
    headers: &HeaderMap,
    project_id: &str,
    action: &str,
    mut env: wedge::ChangeEnvelope,
    mutate: impl FnOnce(&sqlite::Store, &agentrun::Run, &str) -> sqlite::Result<(Vec<String>, T)>,
) -> Result<T, ConnectError> {
    let mut kind = agentrun::ACTOR_KIND_HUMAN;
    let mut tool = format!("desktop.work.milestone.{action}");
    let client = client_of(headers);
    if client != "desktop" {
        kind = agentrun::ACTOR_KIND_AGENT;
        tool = format!("polter.milestone.{action}");
        if env.agent_run_id.trim().is_empty() {
            env.auto_start_label = client;
        }
    }
    env.project_id = project_id.to_string();
    env.batch_id = ids::new_ulid();
    let store = core.store();
    let batch_id = env.batch_id.clone();
    let written = wedge::execute_audited_write(&store, env, kind, &tool, |run| {
        mutate(&store, run, &batch_id)
    })
    .map_err(map_milestone_err)?;
    Ok(written.result)
}

/// Maps a milestone write's failure to its Connect code: a refused field is
/// `invalid_argument`, a missing milestone `not_found`, anything else as the
/// audited write maps it.
fn map_milestone_err(err: sqlite::Error) -> ConnectError {
    match &err {
        sqlite::Error::Work(inner) => map_milestone_domain_err(*inner),
        _ if err.is_no_rows() => ConnectError::new(Code::NotFound, err.to_string()),
        _ => map_audited_err(err),
    }
}

/// Maps a milestone domain error, as `mapMilestoneErr` maps the sentinels.
fn map_milestone_domain_err(err: work::Error) -> ConnectError {
    match err {
        work::Error::NameRequired
        | work::Error::NameLength
        | work::Error::NameTaken
        | work::Error::GoalLength
        | work::Error::State
        | work::Error::TargetDate
        | work::Error::Order => invalid(err.to_string()),
        work::Error::NotFound => ConnectError::new(Code::NotFound, err.to_string()),
    }
}

/// Maps an apply failure's error, as `applyAs` does after the invalid-text case.
pub(super) fn map_apply_err(err: &wedge::Error) -> ConnectError {
    match err {
        // Go wraps `agentrun.ErrChangeIncomplete`, which `mapAuditedErr` turns
        // into InvalidArgument with the hint's text.
        wedge::Error::ChangeIncompleteHint | wedge::Error::ChangeIncompleteFrontmatter => {
            invalid(err.to_string())
        }
        wedge::Error::AgentRun(inner) => map_agentrun_err(inner),
        wedge::Error::Sqlite(inner) => map_audited_err_ref(inner),
        other => internal(other.to_string()),
    }
}

/// [`super::audited_write::map_audited_err`] over a borrowed error.
pub(super) fn map_audited_err_ref(err: &sqlite::Error) -> ConnectError {
    let sqlite::Error::AgentRun(inner) = err else {
        return internal(err.to_string());
    };
    map_agentrun_err(inner)
}

/// [`super::audited_write::map_audited_err`] over a borrowed agent-run error.
fn map_agentrun_err(err: &agentrun::Error) -> ConnectError {
    match err {
        agentrun::Error::ChangeIncomplete
        | agentrun::Error::MissingAgentRunId
        | agentrun::Error::EmptyProject
        | agentrun::Error::InvalidActorKind(_)
        | agentrun::Error::ProjectMismatch
        | agentrun::Error::RunNotOpen
        | agentrun::Error::InvalidKind(_) => invalid(err.to_string()),
        agentrun::Error::UnknownRun => ConnectError::new(Code::NotFound, err.to_string()),
        other => internal(other.to_string()),
    }
}

/// The port of `invalidPolter`: InvalidArgument with the line-numbered
/// diagnostics message. The Go core also attaches the diagnostics as an error
/// detail; the port's [`ConnectError`] carries no details (see the report).
pub(super) fn invalid_polter(diags: &[polter::Diagnostic]) -> ConnectError {
    invalid(diagnostics_message(diags))
}

pub(super) fn diagnostics_proto(diags: &[polter::Diagnostic]) -> Vec<local_v1::PolterDiagnostic> {
    let mut out = Vec::with_capacity(diags.len());
    for diag in diags {
        out.push(local_v1::PolterDiagnostic {
            line: diag.line,
            severity: diag.severity.clone(),
            message: diag.message.clone(),
        });
    }
    out
}

/// Joins error diagnostics into one line-numbered message.
pub(super) fn diagnostics_message(diags: &[polter::Diagnostic]) -> String {
    let mut lines: Vec<String> = Vec::new();
    for diag in diags {
        if diag.severity == polter::SEVERITY_ERROR {
            lines.push(format!("line {}: {}", diag.line, diag.message));
        }
    }
    if lines.is_empty() {
        return "polter text has errors".to_string();
    }
    lines.join("; ")
}

pub(super) fn apply_response(result: &wedge::ApplyResult) -> local_v1::ApplyPolterResponse {
    let mut response = local_v1::ApplyPolterResponse {
        applied: result.applied,
        agent_run_id: result.agent_run_id.clone(),
        batch_id: result.batch_id.clone(),
        ..local_v1::ApplyPolterResponse::default()
    };
    if !result.ops.is_empty() {
        response.ops = plan_lines(&result.ops);
        response.summary = polter::summarize(&result.ops);
    }
    response.diagnostics = diagnostics_proto(&result.diagnostics);
    response
}

/// The plan as one terraform-style line per operation, without a trailing
/// empty line.
pub(super) fn plan_lines(ops: &[work::Op]) -> Vec<String> {
    let plan = polter::format_plan(ops);
    let plan = plan.strip_suffix('\n').unwrap_or(&plan);
    plan.split('\n').map(str::to_string).collect()
}

fn structure_node_proto(row: &structure::Row) -> local_v1::StructureNode {
    let node = &row.node;
    let mut out = local_v1::StructureNode {
        id: node.id.clone(),
        parent_id: node.parent_id.clone(),
        level: node.level.clone(),
        slug: node.slug.clone(),
        path: node.path.clone(),
        title: node.title.clone(),
        intent: node.intent.clone(),
        decisions: node.decisions.clone(),
        status: row.derived.clone(),
        child_count: row.child_count as i32,
        feature_count: row.feature_count as i32,
        open_questions: row.open_questions,
        repo: node.repo.clone(),
        folder: node.folder.clone(),
        kind: node.kind.clone(),
        tech: node.tech.clone(),
        migration_from: node.migration_from.clone(),
        migration_to: node.migration_to.clone(),
        deploys: Vec::new(),
    };
    for deploy in &node.deploys {
        let mut item = local_v1::NodeDeployment {
            id: deploy.id.clone(),
            environment_id: deploy.environment_id.clone(),
            environment: deploy.environment.clone(),
            provider: deploy.provider.clone(),
            resource: deploy.resource.clone(),
            target_id: deploy.target_id.clone(),
            url: deploy.url.clone(),
            ..local_v1::NodeDeployment::default()
        };
        if let Some(check) = &deploy.check {
            item.state = check.state.clone();
            item.state_summary = check.summary.clone();
            item.state_url = check.url.clone();
            item.checked_at = check.checked_at.clone();
        }
        out.deploys.push(item);
    }
    out
}

pub(super) fn work_card_proto(card: &work::Card) -> local_v1::WorkCard {
    let mut out = local_v1::WorkCard {
        id: card.id.clone(),
        project_id: card.project_id.clone(),
        slug: card.slug.clone(),
        title: card.title.clone(),
        intent: card.intent.clone(),
        stage: card.stage.clone(),
        owner: card.owner.clone(),
        under: card.under.clone(),
        kind: card.kind.clone(),
        decisions: card.decisions.clone(),
        milestone_id: card.milestone_id.clone(),
        created_at: format_rfc3339_zone(card.created_at),
        updated_at: format_rfc3339_zone(card.updated_at),
        tasks: Vec::new(),
        questions: Vec::new(),
        invariants: Vec::new(),
    };
    for task in &card.tasks {
        // The Board shows the scope with its checkout, as a claim stores it.
        let mut row = local_v1::WorkTask {
            id: task.id.clone(),
            number: task.number,
            title: task.title.clone(),
            state: task.state.clone(),
            scope: work::qualify_scope(&task.repo, &task.scope),
            claim_holder: String::new(),
        };
        if let Some(claim) = &task.claim {
            row.claim_holder = claim.holder.clone();
        }
        out.tasks.push(row);
    }
    for question in &card.questions {
        if question.state == work::QUESTION_REMOVED {
            continue;
        }
        out.questions.push(local_v1::WorkQuestion {
            id: question.id.clone(),
            text: question.text.clone(),
            blocking: question.blocking,
            options: question.options.clone(),
            state: question.state.clone(),
            answer: question.answer.clone(),
        });
    }
    for invariant in &card.invariants {
        if invariant.state != work::INVARIANT_ACTIVE {
            continue;
        }
        out.invariants.push(local_v1::WorkInvariant {
            id: invariant.id.clone(),
            text: invariant.text.clone(),
        });
    }
    out
}

pub(super) fn milestones_proto(milestones: &[work::Milestone]) -> Vec<local_v1::Milestone> {
    milestones.iter().map(milestone_proto).collect()
}

pub(super) fn milestone_proto(item: &work::Milestone) -> local_v1::Milestone {
    local_v1::Milestone {
        id: item.id.clone(),
        project_id: item.project_id.clone(),
        slug: item.slug.clone(),
        name: item.name.clone(),
        goal: item.goal.clone(),
        exit_criteria: item.exit_criteria.clone(),
        target_date: item.target_date.clone(),
        state: item.state.clone(),
        position: item.position,
        archived_at: item.archived_at.clone(),
        created_at: format_rfc3339(item.created_at),
        updated_at: format_rfc3339(item.updated_at),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositions::local_core::test_support::Harness;
    use crate::domains::projects;

    /// A request without a client header: the Desktop, as the Go tests call.
    fn request<T>(message: T) -> ConnectRequest<T> {
        ConnectRequest {
            headers: HeaderMap::new(),
            message,
        }
    }

    fn init_project(core: &Core, name: &str) -> projects::Project {
        let store = core.store();
        wedge::init_project(
            &store,
            wedge::InitInput {
                name: name.to_string(),
                ..wedge::InitInput::default()
            },
        )
        .expect("init project")
    }

    const CHANGE_SUMMARY: &str = "Plan import";
    const CHANGE_RATIONALE: &str = "Board needs cards";

    fn change() -> Option<local_v1::ChangeEnvelope> {
        Some(local_v1::ChangeEnvelope {
            summary: CHANGE_SUMMARY.to_string(),
            rationale: CHANGE_RATIONALE.to_string(),
            ..local_v1::ChangeEnvelope::default()
        })
    }

    #[test]
    fn work_service_apply_and_list() {
        let harness = Harness::start();
        let core = Arc::clone(&harness.core);
        let project = init_project(&core, "Demo");
        let apply = |text: &str| {
            apply_polter(
                Arc::clone(&core),
                request(local_v1::ApplyPolterRequest {
                    project_id: project.id.clone(),
                    text: text.to_string(),
                    change: change(),
                    ..local_v1::ApplyPolterRequest::default()
                }),
            )
        };

        let created = apply(
            "---\npolter: 1\n---\n## git-import: Git import\nstage: plan\n- [ ] Commit evidence\n? Merge commits? [blocking]\n! No network\n",
        )
        .expect("apply");
        assert!(created.applied, "apply: {created:?}");
        assert!(!created.agent_run_id.is_empty(), "apply: {created:?}");
        assert_eq!(created.ops.len(), 4, "apply: {created:?}");
        {
            let store = core.store();
            let record = store
                .get_agent_run_record(&created.agent_run_id)
                .expect("run record");
            assert_eq!(
                record.run.actor_kind,
                agentrun::ACTOR_KIND_HUMAN,
                "desktop apply should record a human run"
            );
        }

        apply("---\npolter: 1\n---\n## git-import\nstage: build\n").expect("stage move");
        let listed = list_work_cards(
            Arc::clone(&core),
            request(local_v1::ListWorkCardsRequest {
                project_id: project.id.clone(),
                ..local_v1::ListWorkCardsRequest::default()
            }),
        )
        .expect("list");
        assert_eq!(listed.cards.len(), 1, "list: {listed:?}");
        let card = &listed.cards[0];
        assert_eq!(card.stage, "build");
        assert_eq!(card.tasks.len(), 1, "card = {card:?}");
        assert_eq!(card.questions.len(), 1, "card = {card:?}");
        assert_eq!(card.invariants.len(), 1, "card = {card:?}");

        // The wire answers the same card.
        let reply = harness.unary(
            "/polterware.local.v1.WorkService/ListWorkCards",
            Some(crate::compositions::local_core::test_support::TEST_TOKEN),
            &format!(r#"{{"projectId":"{}"}}"#, project.id),
        );
        assert_eq!(reply.status, 200, "{}", reply.text());
        assert_eq!(
            reply.json()["cards"][0]["stage"],
            "build",
            "{}",
            reply.text()
        );

        let handoff = apply(
            "---\npolter: 1\n---\n## git-import\nhandoff:\nbranch: feat/git-import\nagent: codex\ndone: parser\nin_progress: status\nnext: docs\ntouched: docs/spec.md\n",
        )
        .expect("handoff apply");
        assert!(handoff.applied, "handoff apply: {handoff:?}");
        {
            let store = core.store();
            let entity: String = store
                .db
                .query_row(
                    "SELECT entity FROM change_ops WHERE batch_id=?1 AND entity='handoff'",
                    [&handoff.batch_id],
                    |row| row.get(0),
                )
                .expect("change_ops handoff");
            assert_eq!(entity, "handoff");
        }
        let queried = query_polter(
            Arc::clone(&core),
            request(local_v1::QueryPolterRequest {
                project_id: project.id.clone(),
                query: "handoff git-import".to_string(),
                ..local_v1::QueryPolterRequest::default()
            }),
        )
        .expect("query handoff");
        assert!(
            queried.text.contains("branch: feat/git-import"),
            "query handoff = {}",
            queried.text
        );
        assert!(
            queried.text.contains("Merge commits? [blocking]"),
            "query handoff = {}",
            queried.text
        );
        let again = apply(
            "---\npolter: 1\n---\n## git-import\nhandoff:\nbranch: feat/git-import\nagent: codex\ndone: parser\nin_progress: status\nnext: docs\ntouched: docs/spec.md\n",
        )
        .expect("re-apply handoff");
        assert!(!again.applied, "re-apply handoff should be a no-op");
        assert_eq!(again.ops.len(), 0, "re-apply handoff should be a no-op");

        apply("---\npolter: 1\n---\n## git-import\nstatus: removed\n").expect("remove");
        let listed = list_work_cards(
            Arc::clone(&core),
            request(local_v1::ListWorkCardsRequest {
                project_id: project.id.clone(),
                ..local_v1::ListWorkCardsRequest::default()
            }),
        )
        .expect("list");
        assert!(
            listed.cards.is_empty(),
            "removed cards must not be listed: {listed:?}"
        );

        let refused =
            apply("---\npolter: 1\n---\n## missing\nstage: plan\n").expect_err("invalid text");
        assert_eq!(refused.code, Code::InvalidArgument);
        assert!(
            refused.message.contains("line 4"),
            "invalid text should name its line: {}",
            refused.message
        );
    }

    #[test]
    fn milestone_handlers_audit_writes() {
        let harness = Harness::start();
        let core = Arc::clone(&harness.core);
        let project = init_project(&core, "Demo");
        let change = || {
            Some(local_v1::ChangeEnvelope {
                summary: "Plan the beta".to_string(),
                rationale: "Group the cards that ship first".to_string(),
                ..local_v1::ChangeEnvelope::default()
            })
        };
        let create = |name: &str, target_date: &str, change: Option<local_v1::ChangeEnvelope>| {
            create_milestone(
                Arc::clone(&core),
                request(local_v1::CreateMilestoneRequest {
                    project_id: project.id.clone(),
                    name: name.to_string(),
                    target_date: target_date.to_string(),
                    change,
                    ..local_v1::CreateMilestoneRequest::default()
                }),
            )
        };

        let refused = create("Beta", "", None).expect_err("create without a change envelope");
        assert_eq!(refused.code, Code::InvalidArgument, "{refused:?}");
        let refused = create("Beta", "soon", change()).expect_err("create with a bad date");
        assert_eq!(refused.code, Code::InvalidArgument, "{refused:?}");

        let created = create_milestone(
            Arc::clone(&core),
            request(local_v1::CreateMilestoneRequest {
                project_id: project.id.clone(),
                name: " Public beta ".to_string(),
                goal: "First outside users".to_string(),
                exit_criteria: vec!["Signup works".to_string(), " ".to_string()],
                target_date: "2026-12-01".to_string(),
                state: "active".to_string(),
                change: change(),
            }),
        )
        .expect("create milestone");
        let beta = created.milestone.expect("created milestone");
        assert_eq!(beta.name, "Public beta", "create milestone = {beta:?}");
        assert_eq!(beta.slug, "public-beta", "create milestone = {beta:?}");
        assert_eq!(beta.state, "active", "create milestone = {beta:?}");
        assert_eq!(
            beta.exit_criteria,
            vec!["Signup works"],
            "create = {beta:?}"
        );
        assert_eq!(beta.position, 0, "create milestone = {beta:?}");

        let second = create_milestone(
            Arc::clone(&core),
            request(local_v1::CreateMilestoneRequest {
                project_id: project.id.clone(),
                name: "GA".to_string(),
                change: change(),
                ..local_v1::CreateMilestoneRequest::default()
            }),
        )
        .expect("second milestone");
        let second = second.milestone.expect("second milestone");
        assert_eq!(second.state, "planned", "second milestone = {second:?}");
        assert_eq!(second.position, 1, "second milestone = {second:?}");

        let renamed = update_milestone(
            Arc::clone(&core),
            request(local_v1::UpdateMilestoneRequest {
                project_id: project.id.clone(),
                id: beta.id.clone(),
                name: "Open beta".to_string(),
                goal: beta.goal.clone(),
                exit_criteria: beta.exit_criteria.clone(),
                target_date: beta.target_date.clone(),
                state: "active".to_string(),
                change: change(),
            }),
        )
        .expect("rename milestone");
        let renamed = renamed.milestone.expect("renamed milestone");
        assert_eq!(renamed.name, "Open beta", "rename = {renamed:?}");
        assert_eq!(renamed.slug, "public-beta", "a rename keeps the slug");

        let reordered = reorder_milestones(
            Arc::clone(&core),
            request(local_v1::ReorderMilestonesRequest {
                project_id: project.id.clone(),
                milestone_ids: vec![second.id.clone(), beta.id.clone()],
                change: change(),
            }),
        )
        .expect("reorder milestones");
        assert_eq!(
            reordered.milestones[0].id, second.id,
            "reorder milestones = {:?}",
            reordered.milestones
        );

        // Polter puts cards in a milestone by slug, and the queries read them
        // back.
        {
            let store = core.store();
            let apply = |text: &str| {
                wedge::apply_polter(
                    &store,
                    wedge::ApplyInput {
                        project_id: project.id.clone(),
                        text: text.to_string(),
                        auto_start_label: "cli".to_string(),
                        ..wedge::ApplyInput::default()
                    },
                )
            };
            apply(
                "---\npolter: 1\nsummary: Plan signup\nrationale: It ships in the beta\n---\n\n## signup: Signup\nmilestone: public-beta\n\n## docs: Docs\n",
            )
            .expect("apply cards");
            let refused = apply(
                "---\npolter: 1\nsummary: Plan\nrationale: Typo\n---\n\n## docs: Docs\nmilestone: nope\n",
            )
            .expect_err("unknown milestone");
            assert!(!refused.result.applied, "unknown milestone: {refused:?}");
            assert_eq!(
                refused.result.diagnostics.len(),
                1,
                "unknown milestone: {:?}",
                refused.result.diagnostics
            );
            assert!(
                refused.result.diagnostics[0]
                    .message
                    .contains("unknown milestone \"nope\""),
                "unknown milestone: {:?}",
                refused.result.diagnostics
            );
            let roadmap = wedge::query_polter(&store, &project.id, "roadmap").expect("roadmap");
            assert!(
                roadmap.contains("## public-beta: Open beta"),
                "roadmap = {roadmap}"
            );
            assert!(
                roadmap.contains("goal: First outside users"),
                "roadmap = {roadmap}"
            );
            assert!(roadmap.contains("- signup  shape"), "roadmap = {roadmap}");
            assert!(roadmap.contains("Backlog: 1 cards"), "roadmap = {roadmap}");
            assert!(!roadmap.contains("## ga:"), "roadmap = {roadmap}");
            let card = wedge::query_polter(&store, &project.id, "card signup").expect("card");
            assert!(card.contains("milestone: public-beta"), "card = {card}");
        }

        let archived = archive_milestone(
            Arc::clone(&core),
            request(local_v1::ArchiveMilestoneRequest {
                project_id: project.id.clone(),
                id: second.id.clone(),
                archived: true,
                change: change(),
            }),
        )
        .expect("archive milestone");
        let archived = archived.milestone.expect("archived milestone");
        assert!(!archived.archived_at.is_empty(), "archive = {archived:?}");

        {
            let store = core.store();
            let listed =
                wedge::query_polter(&store, &project.id, "milestones").expect("milestones query");
            assert!(!listed.contains("ga "), "milestones = {listed}");
            assert!(
                listed.contains("public-beta  active  0/1 delivered  target 2026-12-01"),
                "milestones = {listed}"
            );
            let creates: i64 = store
                .db
                .query_row(
                    "SELECT COUNT(1) FROM change_ops WHERE project_id=?1 AND entity='milestone' AND op='create'",
                    [&project.id],
                    |row| row.get(0),
                )
                .expect("audited creates");
            assert_eq!(creates, 2, "audited creates = {creates}");
            let run_id: String = store
                .db
                .query_row(
                    "SELECT agent_run_id FROM change_ops WHERE project_id=?1 AND entity='milestone' LIMIT 1",
                    [&project.id],
                    |row| row.get(0),
                )
                .expect("milestone change_ops run");
            assert!(!run_id.is_empty(), "milestone change_ops run = {run_id:?}");
        }
    }
}
