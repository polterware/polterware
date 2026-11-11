//! Port of `backend/compositions/local-core/inbox_handlers.go`.
//!
//! The derived Inbox: open items (blocking first), deferred ones, the items a
//! person routed to agents, and the one action that resolves an item.

use std::sync::Arc;

use crate::compositions::wedge;
use crate::domains::agentrun;
use crate::domains::canon;
use crate::domains::inbox;
use crate::local_v1;
use crate::providers::sqlite::{self, SCOPE_ITEM, Store};

use super::connect::{Code, ConnectError, ConnectRequest, Route};
use super::lifecycle::client_of;
use super::server::{Core, internal, invalid};
use super::work_handlers::{
    diagnostics_proto, invalid_polter, map_apply_err, map_audited_err_ref, plan_lines,
};

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary("/polterware.local.v1.WorkService/ListInbox", list_inbox),
        Route::unary(
            "/polterware.local.v1.WorkService/ResolveInboxItem",
            resolve_inbox_item,
        ),
    ]
}

/// `polterware.local.v1.WorkService.ListInbox`.
pub fn list_inbox(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListInboxRequest>,
) -> Result<local_v1::ListInboxResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let store = core.store();
    let refs = wedge::scope_refs(&msg.scope_node, &msg.scope_nodes);
    let scope = wedge::resolve_scope_refs(&store, &msg.project_id, &refs, msg.scope_exact)
        .map_err(|err| invalid(err.to_string()))?;
    let result = wedge::list_inbox_scoped(&store, &msg.project_id, &scope).map_err(internal)?;
    let mut out = local_v1::ListInboxResponse {
        open_count: result.open() as i32,
        blocking_count: result.blocking() as i32,
        ..local_v1::ListInboxResponse::default()
    };
    let assigned = store
        .scope_map(&msg.project_id, SCOPE_ITEM)
        .map_err(internal)?;
    let with_scope = |item: &inbox::Item| {
        let mut message = inbox_item_proto(item);
        message.scope_node_ids = assigned.get(&item.key).cloned().unwrap_or_default();
        message
    };
    for item in &result.items {
        out.items.push(with_scope(item));
    }
    for routed in &result.routed {
        out.routed.push(local_v1::RoutedInboxItem {
            item: Some(with_scope(&routed.item)),
            action: routed.action.clone(),
        });
    }
    Ok(out)
}

/// `polterware.local.v1.WorkService.ResolveInboxItem`.
pub fn resolve_inbox_item(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ResolveInboxItemRequest>,
) -> Result<local_v1::ResolveInboxItemResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let action = inbox::parse_action(&msg.action).map_err(|err| invalid(err.to_string()))?;
    let mut in_ = wedge::ResolveInboxInput {
        project_id: msg.project_id.clone(),
        item_id: msg.item_id.clone(),
        action,
        answer: msg.answer.clone(),
        ..wedge::ResolveInboxInput::default()
    };
    let client = client_of(&request.headers);
    if client == "desktop" {
        in_.actor = "desktop".to_string();
        in_.apply.actor_kind = agentrun::ACTOR_KIND_HUMAN.to_string();
        in_.apply.tool = "desktop.inbox.resolve".to_string();
    } else {
        in_.actor = msg.session_label.trim().to_string();
        if in_.actor.is_empty() {
            in_.actor = client.clone();
        }
        in_.apply.agent = msg.session_label.trim().to_string();
        in_.apply.auto_start_label = client;
        in_.apply.tool = "polter.inbox".to_string();
    }
    let store = core.store();
    let result = match wedge::resolve_inbox(&store, &in_) {
        Ok(result) => result,
        Err(err) => return Err(resolve_inbox_error(&store, &in_, err)),
    };
    let mut out = local_v1::ResolveInboxItemResponse {
        item: Some(inbox_item_proto(&result.item)),
        action: result.action.clone(),
        ..local_v1::ResolveInboxItemResponse::default()
    };
    if let Some(applied) = &result.apply {
        if !applied.ops.is_empty() {
            out.ops = plan_lines(&applied.ops);
        }
        out.diagnostics = diagnostics_proto(&applied.diagnostics);
    }
    Ok(out)
}

/// Maps a failed resolve, as `ResolveInboxItem` maps it: an unknown item is
/// NotFound, a caller mistake or a refused canon write InvalidArgument, refused
/// Polter text InvalidArgument with its diagnostics, and the rest as the
/// audited write maps them.
fn resolve_inbox_error(
    store: &Store,
    in_: &wedge::ResolveInboxInput,
    err: anyhow::Error,
) -> ConnectError {
    if err.downcast_ref::<wedge::UnknownInboxItem>().is_some() {
        return ConnectError::new(Code::NotFound, err.to_string());
    }
    if let Some(input) = err.downcast_ref::<wedge::InboxInputError>() {
        return invalid(input.to_string());
    }
    if let Some(failure) = err.downcast_ref::<wedge::Error>() {
        match failure {
            wedge::Error::PolterInvalid => {
                // Go reports the refused decide's diagnostics; this wedge port
                // returns only the sentinel, so the refused write is rebuilt to
                // read them back (invalid text writes nothing).
                return match refused_decide(store, in_) {
                    Some(result) => invalid_polter(&result.diagnostics),
                    None => invalid_polter(&[]),
                };
            }
            wedge::Error::Sqlite(inner) => return map_sqlite_err(inner),
            other => return map_apply_err(other),
        }
    }
    if let Some(inner) = err.downcast_ref::<sqlite::Error>() {
        return map_sqlite_err(inner);
    }
    internal(err.to_string())
}

fn map_sqlite_err(err: &sqlite::Error) -> ConnectError {
    if is_canon_input_refusal(err) {
        return invalid(err.to_string());
    }
    map_audited_err_ref(err)
}

fn is_canon_input_refusal(err: &sqlite::Error) -> bool {
    matches!(
        err,
        sqlite::Error::Canon(canon::Error::VerifiedRequiresActor | canon::Error::UnknownFact)
    )
}

/// Rebuilds the decide a refused resolve attempted, to read the diagnostics the
/// apply result carried in Go.
fn refused_decide(store: &Store, in_: &wedge::ResolveInboxInput) -> Option<wedge::ApplyResult> {
    if in_.action != inbox::ACTION_DECIDE {
        return None;
    }
    let current = wedge::list_inbox(store, &in_.project_id).ok()?;
    let item = current.find(&in_.item_id)?;
    let text = wedge::decide_text(&item, &in_.answer).ok()?;
    let mut apply = in_.apply.clone();
    apply.project_id = in_.project_id.clone();
    apply.text = text;
    match wedge::apply_polter(store, apply) {
        Ok(result) => Some(result),
        Err(failure) => Some(*failure.result),
    }
}

fn inbox_item_proto(item: &inbox::Item) -> local_v1::InboxItem {
    local_v1::InboxItem {
        id: item.id.clone(),
        kind: item.kind.clone(),
        blocking: item.blocking,
        title: item.title.clone(),
        detail: item.detail.clone(),
        options: item.options.clone(),
        r#where: item.where_.clone(),
        source: item.source.clone(),
        state: item.state.clone(),
        actions: item.actions.clone(),
        details: item.details.clone(),
        ..local_v1::InboxItem::default()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::http::HeaderMap;

    use super::super::connect::Code;
    use super::*;

    fn request<T>(message: T) -> ConnectRequest<T> {
        ConnectRequest {
            headers: HeaderMap::new(),
            message,
        }
    }

    #[test]
    fn an_unknown_item_is_not_found_and_an_unknown_action_is_refused() {
        let harness = crate::compositions::local_core::test_support::Harness::start();
        let core = Arc::clone(&harness.core);
        let project = {
            let store = core.store();
            wedge::init_project(
                &store,
                wedge::InitInput {
                    name: "Demo".to_string(),
                    ..wedge::InitInput::default()
                },
            )
            .expect("init project")
        };
        assert_eq!(
            list_inbox(
                Arc::clone(&core),
                request(local_v1::ListInboxRequest::default()),
            )
            .unwrap_err()
            .code,
            Code::InvalidArgument
        );
        let listed = list_inbox(
            Arc::clone(&core),
            request(local_v1::ListInboxRequest {
                project_id: project.id.clone(),
                ..local_v1::ListInboxRequest::default()
            }),
        )
        .expect("list inbox");
        assert_eq!(
            (listed.open_count, listed.blocking_count, listed.items.len()),
            (0, 0, 0),
            "a project with no items has an empty inbox: {listed:?}"
        );

        let refused = resolve_inbox_item(
            Arc::clone(&core),
            request(local_v1::ResolveInboxItemRequest {
                project_id: project.id.clone(),
                item_id: "i00000000".to_string(),
                action: "defer".to_string(),
                ..local_v1::ResolveInboxItemRequest::default()
            }),
        )
        .unwrap_err();
        assert_eq!(refused.code, Code::NotFound);
        assert_eq!(
            refused.message,
            "unknown inbox item: list them with polterware inbox (i00000000)"
        );
        assert_eq!(
            resolve_inbox_item(
                Arc::clone(&core),
                request(local_v1::ResolveInboxItemRequest {
                    project_id: project.id.clone(),
                    item_id: "i00000000".to_string(),
                    action: "nope".to_string(),
                    ..local_v1::ResolveInboxItemRequest::default()
                }),
            )
            .unwrap_err()
            .code,
            Code::InvalidArgument
        );
    }
}
