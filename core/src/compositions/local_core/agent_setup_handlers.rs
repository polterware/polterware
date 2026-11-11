//! Port of `backend/compositions/local-core/agent_setup_handlers.go`.
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
            "/polterware.local.v1.AgentRunService/GetAgentSetup",
            get_agent_setup,
        ),
        Route::unary(
            "/polterware.local.v1.AgentRunService/ListHookSwitches",
            list_hook_switches,
        ),
        Route::unary(
            "/polterware.local.v1.AgentRunService/SetHookSwitch",
            set_hook_switch,
        ),
    ]
}

/// `polterware.local.v1.AgentRunService.GetAgentSetup`, not ported yet.
pub fn get_agent_setup(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::GetAgentSetupRequest>,
) -> Result<local_v1::GetAgentSetupResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.AgentRunService.GetAgentSetup is not implemented",
    ))
}

/// `polterware.local.v1.AgentRunService.ListHookSwitches`, not ported yet.
pub fn list_hook_switches(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListHookSwitchesRequest>,
) -> Result<local_v1::ListHookSwitchesResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.AgentRunService.ListHookSwitches is not implemented",
    ))
}

/// `polterware.local.v1.AgentRunService.SetHookSwitch`, not ported yet.
pub fn set_hook_switch(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::SetHookSwitchRequest>,
) -> Result<local_v1::SetHookSwitchResponse, ConnectError> {
    Err(ConnectError::unimplemented(
        "polterware.local.v1.AgentRunService.SetHookSwitch is not implemented",
    ))
}
