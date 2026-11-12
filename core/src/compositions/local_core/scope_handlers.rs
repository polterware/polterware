//! The handlers that set the Structure contexts of a Rule, a doc, or an Inbox item.
//!
//! Sets the Structure contexts of a Rule or a doc, or of a derived Inbox item;
//! none clears them, which means the whole project.

use std::sync::Arc;

use crate::compositions::wedge;
use crate::domains::polter;
use crate::local_v1;
use crate::providers::sqlite::{SCOPE_ITEM, SCOPE_KNOWLEDGE, SCOPE_RULE};

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::{Core, internal, invalid};

pub fn routes() -> Vec<Route> {
    vec![Route::unary(
        "/polterware.local.v1.WorkService/SetScope",
        set_scope,
    )]
}

/// `polterware.local.v1.WorkService.SetScope`.
pub fn set_scope(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::SetScopeRequest>,
) -> Result<local_v1::SetScopeResponse, ConnectError> {
    let msg = request.message;
    if msg.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    if msg.id.trim().is_empty() {
        return Err(invalid("id is required"));
    }
    match msg.kind.as_str() {
        SCOPE_KNOWLEDGE | SCOPE_RULE | SCOPE_ITEM => {}
        _ => {
            return Err(invalid(format!(
                "unknown scope kind {}: use knowledge, rule, or item",
                polter::go_quote(&msg.kind)
            )));
        }
    }
    let store = core.store();
    // The nodes replace the entity's contexts; none clears them (the whole
    // project).
    let scope = wedge::resolve_scope_refs(&store, &msg.project_id, &msg.nodes, false)
        .map_err(|err| invalid(err.to_string()))?;
    let mut id = msg.id.clone();
    if msg.kind == SCOPE_RULE {
        // A Rule is named by its slug as often as by its id.
        let rules = store.list_rules(&msg.project_id).map_err(internal)?;
        for rule in &rules {
            if !rule.slug.is_empty() && rule.slug == id {
                id = rule.id.clone();
            }
        }
    }
    store
        .set_scope(&msg.project_id, &msg.kind, &id, &scope.node_ids)
        .map_err(|err| invalid(err.to_string()))?;
    Ok(local_v1::SetScopeResponse::default())
}
