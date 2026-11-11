//! Port of `backend/compositions/local-core/integration_sync.go`.
//!
//! The handlers are not ported yet: each answers Connect `unimplemented`, worded as
//! the generated `Unimplemented…Handler` of `localv1connect` words it.

use std::sync::Arc;

use crate::local_v1;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::Core;

pub fn routes() -> Vec<Route> {
    vec![Route::unary(
        "/polterware.local.v1.IntegrationService/SyncIntegrationConnection",
        sync_integration_connection,
    )]
}

/// `polterware.local.v1.IntegrationService.SyncIntegrationConnection`, not ported yet.
pub fn sync_integration_connection(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::SyncIntegrationConnectionRequest>,
) -> Result<local_v1::SyncIntegrationConnectionResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.IntegrationService.SyncIntegrationConnection is not implemented",
    ))
}
