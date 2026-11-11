//! Port of `backend/runtimes/cli/mcpsetup.go`: `setup mcp`, which registers the
//! MCP server with Claude Code, Codex, and Cursor.
//!
//! It is reached through `setup`, and stubbed until this file is ported.

use super::output::{CliResult, not_ported};

pub fn setup_mcp(_db_path: &str, _args: &[String]) -> CliResult {
    not_ported("setup mcp")
}
