//! Port of `backend/compositions/local-core/deploy_checks.go`.
//!
//! The handlers are not ported yet: each answers Connect `unimplemented`, worded as
//! the generated `Unimplemented…Handler` of `localv1connect` words it.

use std::sync::Arc;

use crate::local_v1;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::Core;

/// `watchDeploys`, the background deploy check: a first pass shortly after the
/// core starts, then one every ten minutes while it runs. It lands with the
/// deploy port; `serve_connect` starts it where the Go core does.
pub fn watch_deploys(_core: &Arc<Core>) {}

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary(
            "/polterware.local.v1.IntegrationService/ListDeployTargets",
            list_deploy_targets,
        ),
        Route::unary(
            "/polterware.local.v1.IntegrationService/CheckDeploys",
            check_deploys,
        ),
    ]
}

/// `polterware.local.v1.IntegrationService.ListDeployTargets`, not ported yet.
pub fn list_deploy_targets(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListDeployTargetsRequest>,
) -> Result<local_v1::ListDeployTargetsResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.IntegrationService.ListDeployTargets is not implemented",
    ))
}

/// `polterware.local.v1.IntegrationService.CheckDeploys`, not ported yet.
pub fn check_deploys(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::CheckDeploysRequest>,
) -> Result<local_v1::CheckDeploysResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.IntegrationService.CheckDeploys is not implemented",
    ))
}
