//! The AgentSetupService handlers: the hook switches and the setup doctor.
//!
//! The hook switches and the setup doctor. Installing is done by
//! `polterware setup`; the core only reads.

use std::sync::Arc;

use crate::compositions::wedge;
use crate::local_v1;
use crate::providers::agentsetup;
use crate::providers::sqlite;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::{Core, internal, invalid};

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

fn hook_switches(core: &Core) -> sqlite::Result<Vec<local_v1::HookSwitch>> {
    let stored = core.store().hook_switches()?;
    Ok(agentsetup::HOOKS
        .iter()
        .map(|hook| local_v1::HookSwitch {
            name: hook.name.to_string(),
            label: hook.label.to_string(),
            note: hook.note.to_string(),
            enabled: stored.get(hook.name).copied().unwrap_or(true),
        })
        .collect())
}

/// Reports the hook switches and the setup doctor.
pub fn get_agent_setup(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::GetAgentSetupRequest>,
) -> Result<local_v1::GetAgentSetupResponse, ConnectError> {
    let hooks = hook_switches(&core).map_err(internal)?;
    let report = agentsetup::doctor(&core.db_dir());
    let mut root = String::new();
    if !request.message.project_id.is_empty() {
        root = wedge::first_checkout(&core.store(), &request.message.project_id);
    }
    let agents = agentsetup::inspect_agents(&root);
    Ok(local_v1::GetAgentSetupResponse {
        setup: Some(local_v1::AgentSetup {
            hooks,
            polterware_path: report.polterware_path,
            claude_path: report.claude_path,
            plugin_installed: report.plugin.installed,
            plugin_enabled: report.plugin.enabled,
            plugin_version: report.plugin.version,
            embedded_version: report.embedded_ver,
            marketplace_dir: report.marketplace,
            problem: report.problem,
            skill_path: agents.skill_path,
            skill_ready: agents.skill_ready,
            agents_line_ready: agents.line_ready,
        }),
    })
}

pub fn set_hook_switch(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::SetHookSwitchRequest>,
) -> Result<local_v1::SetHookSwitchResponse, ConnectError> {
    if !agentsetup::valid_hook(&request.message.name) {
        return Err(invalid(format!("unknown hook {}", request.message.name)));
    }
    core.store()
        .set_hook_switch(&request.message.name, request.message.enabled)
        .map_err(internal)?;
    let hooks = hook_switches(&core).map_err(internal)?;
    Ok(local_v1::SetHookSwitchResponse { hooks })
}

pub fn list_hook_switches(
    core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListHookSwitchesRequest>,
) -> Result<local_v1::ListHookSwitchesResponse, ConnectError> {
    let hooks = hook_switches(&core).map_err(internal)?;
    Ok(local_v1::ListHookSwitchesResponse { hooks })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositions::local_core::test_support::Harness;
    use axum::http::HeaderMap;

    fn request<T>(message: T) -> ConnectRequest<T> {
        ConnectRequest {
            headers: HeaderMap::new(),
            message,
        }
    }

    #[test]
    fn hook_switches_default_on_and_persist() {
        let harness = Harness::start();
        let listed = list_hook_switches(
            Arc::clone(&harness.core),
            request(local_v1::ListHookSwitchesRequest::default()),
        )
        .expect("list");
        assert_eq!(listed.hooks.len(), agentsetup::HOOKS.len());
        for hook in &listed.hooks {
            assert!(hook.enabled, "{} should default to on", hook.name);
        }

        set_hook_switch(
            Arc::clone(&harness.core),
            request(local_v1::SetHookSwitchRequest {
                name: "stop".to_string(),
                enabled: false,
            }),
        )
        .expect("switch off");
        let error = set_hook_switch(
            Arc::clone(&harness.core),
            request(local_v1::SetHookSwitchRequest {
                name: "nope".to_string(),
                enabled: false,
            }),
        )
        .expect_err("unknown hook accepted");
        assert_eq!(error.message, "unknown hook nope");

        let after = list_hook_switches(
            Arc::clone(&harness.core),
            request(local_v1::ListHookSwitchesRequest::default()),
        )
        .expect("list");
        for hook in &after.hooks {
            assert_eq!(
                hook.enabled,
                hook.name != "stop",
                "{} enabled={}",
                hook.name,
                hook.enabled
            );
        }
    }
}
