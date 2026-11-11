//! Port of `backend/compositions/local-core/agents_sync.go`.
//!
//! The handlers are not ported yet: each answers Connect `unimplemented`, worded as
//! the generated `Unimplemented…Handler` of `localv1connect` words it.

use std::sync::Arc;

use crate::local_v1;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::Core;

pub fn routes() -> Vec<Route> {
    vec![Route::unary(
        "/polterware.local.v1.WorkService/SetAgentsSync",
        set_agents_sync,
    )]
}

/// `polterware.local.v1.WorkService.SetAgentsSync`, not ported yet.
pub fn set_agents_sync(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::SetAgentsSyncRequest>,
) -> Result<local_v1::SetAgentsSyncResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.WorkService.SetAgentsSync is not implemented",
    ))
}
