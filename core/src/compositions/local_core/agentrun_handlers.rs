//! Port of `backend/compositions/local-core/agentrun_handlers.go`.
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

/// `polterware.local.v1.AgentRunService.ListAgentRuns`, not ported yet.
pub fn list_agent_runs(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListAgentRunsRequest>,
) -> Result<local_v1::ListAgentRunsResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.AgentRunService.ListAgentRuns is not implemented",
    ))
}

/// `polterware.local.v1.AgentRunService.GetAgentRun`, not ported yet.
pub fn get_agent_run(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::GetAgentRunRequest>,
) -> Result<local_v1::GetAgentRunResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.AgentRunService.GetAgentRun is not implemented",
    ))
}

/// `polterware.local.v1.AgentRunService.ListChanges`, not ported yet.
pub fn list_changes(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListChangesRequest>,
) -> Result<local_v1::ListChangesResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.AgentRunService.ListChanges is not implemented",
    ))
}
