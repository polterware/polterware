//! Port of `backend/compositions/local-core/inbox_handlers.go`.
//!
//! The handlers are not ported yet: each answers Connect `unimplemented`, worded as
//! the generated `Unimplemented…Handler` of `localv1connect` words it.

use std::sync::Arc;

use crate::local_v1;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::Core;

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary("/polterware.local.v1.WorkService/ListInbox", list_inbox),
        Route::unary(
            "/polterware.local.v1.WorkService/ResolveInboxItem",
            resolve_inbox_item,
        ),
    ]
}

/// `polterware.local.v1.WorkService.ListInbox`, not ported yet.
pub fn list_inbox(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListInboxRequest>,
) -> Result<local_v1::ListInboxResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.ListInbox is not implemented",
    ))
}

/// `polterware.local.v1.WorkService.ResolveInboxItem`, not ported yet.
pub fn resolve_inbox_item(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ResolveInboxItemRequest>,
) -> Result<local_v1::ResolveInboxItemResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.ResolveInboxItem is not implemented",
    ))
}
