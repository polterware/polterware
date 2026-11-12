//! Installing the coding agent tooling: MCP, hooks, the skill, and setup.
//!
//! Installs Polterware into agents: the embedded Claude Code plugin (through
//! the `claude` CLI, never by editing Claude Code's files) and the skill file
//! for Codex and other agents. The plugin version is the binary's.

mod claude;
mod codex;
mod command;
mod hooks;
mod mcp;
mod plugin;

pub use claude::{
    PluginState, Report, doctor, install_claude, set_claude_enabled, uninstall_claude,
};
pub use codex::{AgentsState, inspect_agents, install_agents, skill_dir, uninstall_agents};
pub use hooks::{HOOKS, Hook, switch_for, valid_hook};
pub use mcp::{
    MCP_SERVER_NAME, McpClient, McpStatus, configure_mcp, find_mcp_client, mcp_args, mcp_clients,
};
pub use plugin::{
    MARKETPLACE_NAME, PLUGIN_ID, PLUGIN_NAME, embedded_version, marketplace_dir, skill_file,
    stale_notices, write_marketplace,
};

/// A setup failure, printed as the message it carries.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct Error(pub String);

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
pub(crate) mod test_support;
