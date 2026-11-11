//! Port of `backend/compositions/local-core/dispatch_host_handlers.go`.
//!
//! The handlers are not ported yet: each answers Connect `unimplemented`, worded as
//! the generated `Unimplemented…Handler` of `localv1connect` words it.

use std::sync::Arc;

use crate::local_v1;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::Core;

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

/// `polterware.local.v1.DispatchHostService.ListHostProjects`, not ported yet.
pub fn list_host_projects(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListHostProjectsRequest>,
) -> Result<local_v1::ListHostProjectsResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.DispatchHostService.ListHostProjects is not implemented",
    ))
}

/// `polterware.local.v1.DispatchHostService.ResolveHostProject`, not ported yet.
pub fn resolve_host_project(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ResolveHostProjectRequest>,
) -> Result<local_v1::ResolveHostProjectResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.DispatchHostService.ResolveHostProject is not implemented",
    ))
}

/// `polterware.local.v1.DispatchHostService.ResolveRun`, not ported yet.
pub fn resolve_run(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ResolveRunRequest>,
) -> Result<local_v1::ResolveRunResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.DispatchHostService.ResolveRun is not implemented",
    ))
}

/// `polterware.local.v1.DispatchHostService.ListHostCards`, not ported yet.
pub fn list_host_cards(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListHostCardsRequest>,
) -> Result<local_v1::ListHostCardsResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.DispatchHostService.ListHostCards is not implemented",
    ))
}

/// `polterware.local.v1.DispatchHostService.StartHostAgentRun`, not ported yet.
pub fn start_host_agent_run(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::StartHostAgentRunRequest>,
) -> Result<local_v1::StartHostAgentRunResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.DispatchHostService.StartHostAgentRun is not implemented",
    ))
}

/// `polterware.local.v1.DispatchHostService.CompleteHostAgentRun`, not ported yet.
pub fn complete_host_agent_run(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::CompleteHostAgentRunRequest>,
) -> Result<local_v1::CompleteHostAgentRunResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.DispatchHostService.CompleteHostAgentRun is not implemented",
    ))
}
