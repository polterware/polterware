//! Port of `backend/runtimes/cli/polter.go`: Polter text in and out.
//!
//! The commands are stubbed until this file is ported.

use super::output::{CliResult, not_ported};

pub fn run_apply(_db_path: &str, _args: &[String]) -> CliResult {
    not_ported("apply")
}

pub fn run_lint(_db_path: &str, _args: &[String]) -> CliResult {
    not_ported("lint")
}

pub fn run_format(_args: &[String]) -> CliResult {
    not_ported("format")
}

pub fn run_query(_db_path: &str, _args: &[String]) -> CliResult {
    not_ported("query")
}

pub fn run_status(_db_path: &str, _args: &[String]) -> CliResult {
    not_ported("status")
}

pub fn run_handoff(_db_path: &str, _args: &[String]) -> CliResult {
    not_ported("handoff")
}

pub fn run_task(_db_path: &str, _args: &[String]) -> CliResult {
    not_ported("task")
}
