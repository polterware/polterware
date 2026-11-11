//! Port of `backend/runtimes/cli/context.go`: the brief and search reads.
//!
//! The commands are stubbed until this file is ported.

use super::output::{CliResult, not_ported};

pub fn run_brief(_db_path: &str, _args: &[String]) -> CliResult {
    not_ported("brief")
}

pub fn run_search(_db_path: &str, _args: &[String]) -> CliResult {
    not_ported("search")
}
