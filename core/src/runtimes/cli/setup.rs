//! Port of `backend/runtimes/cli/setup.go`: agent setup and the hook switches.
//!
//! The commands are stubbed until this file is ported.

use super::output::{CliResult, not_ported};

pub fn run_setup(_db_path: &str, _args: &[String]) -> CliResult {
    not_ported("setup")
}

pub fn run_hooks(_db_path: &str, _args: &[String]) -> CliResult {
    not_ported("hooks")
}
