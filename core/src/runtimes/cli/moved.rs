//! Port of `backend/runtimes/cli/moved.go`: the delegation commands Dispatch
//! took over.
//!
//! `run`/`runs`/`wait`/... now belong to Polterware Dispatch. When the
//! `dispatch` binary is on PATH the bridge replaces this process with it
//! (`dispatch <command> ...`), keeping the same stdin, stdout, and process
//! identity a person expects from the old command; otherwise it names where the
//! command went.

use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::flags::go_quote;
use super::output::{CliError, CliResult};

/// The delegation commands Polterware Dispatch took over; the value is
/// Dispatch's name for each.
const MOVED_COMMANDS: &[(&str, &str)] = &[
    ("run", "run"),
    ("runs", "runs"),
    ("wait", "wait"),
    ("result", "result"),
    ("cancel", "cancel"),
    ("route", "route"),
    ("classify", "classify"),
    ("models", "models"),
    ("merge", "merge"),
    ("clean", "clean"),
    ("recover", "recover"),
    ("subagents", "subagents"),
    ("tiers", "tiers"),
    ("usage", "usage"),
    ("agents", "agents"),
    ("doctor", "doctor"),
    ("config", "config"),
    ("skill", "skill"),
    ("update", "update"),
];

/// Maps `polterware dispatch <sub>` to Dispatch's command.
const LEGACY_DISPATCH: &[(&str, &str)] = &[("ls", "runs"), ("stats", "usage")];

/// The Dispatch command a moved CLI command runs, or `None` for a command
/// Dispatch does not own.
pub fn moved_command(command: &str) -> Option<&'static str> {
    MOVED_COMMANDS
        .iter()
        .find(|(name, _)| *name == command)
        .map(|(_, dispatch)| *dispatch)
}

/// Hands a delegation command to `dispatch`, the Polterware Dispatch binary,
/// when it is on PATH, and otherwise says where the command went.
pub fn run_moved(command: &str, args: &[String]) -> CliResult {
    let Some(bin) = look_path("dispatch") else {
        return Err(missing_dispatch(command));
    };
    let mut invocation = Command::new(&bin);
    invocation.arg("dispatch").arg(command).args(args);
    let err = invocation.exec();
    Err(CliError::message(format!(
        "exec {}: {err}",
        go_quote(&bin.to_string_lossy())
    )))
}

/// The error naming where a command went when Dispatch is not installed.
fn missing_dispatch(command: &str) -> CliError {
    CliError::message(format!(
        "moved to Dispatch: dispatch {command} (install Dispatch to get it)"
    ))
}

/// Handles `polterware dispatch <sub>`. The SessionStart hook Polterware
/// registered (`dispatch hook`) stays silent without Dispatch, so a stale hook
/// is harmless.
pub fn run_legacy_dispatch(args: &[String]) -> CliResult {
    let Some(first) = args.first() else {
        return run_moved("help", &[]);
    };
    let mut sub = first.as_str();
    if let Some((_, mapped)) = LEGACY_DISPATCH.iter().find(|(name, _)| *name == sub) {
        sub = mapped;
    }
    if sub == "hook" && look_path("dispatch").is_none() {
        return Ok(());
    }
    run_moved(sub, &args[1..])
}

/// Go's `exec.LookPath`: the first executable file named `name` on PATH.
pub fn look_path(name: &str) -> Option<PathBuf> {
    find_in_path(name, std::env::var_os("PATH").as_deref())
}

/// The PATH lookup over an explicit search path.
fn find_in_path(name: &str, path: Option<&OsStr>) -> Option<PathBuf> {
    if name.contains('/') {
        let candidate = Path::new(name);
        return is_executable(candidate).then(|| candidate.to_path_buf());
    }
    let path = path?;
    for dir in std::env::split_paths(path) {
        let dir = if dir.as_os_str().is_empty() {
            PathBuf::from(".")
        } else {
            dir
        };
        let candidate = dir.join(name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    match std::fs::metadata(path) {
        Ok(metadata) => metadata.is_file() && metadata.permissions().mode() & 0o111 != 0,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs;

    #[test]
    fn the_moved_table_matches_the_go_map() {
        assert_eq!(moved_command("run"), Some("run"));
        assert_eq!(moved_command("update"), Some("update"));
        assert_eq!(moved_command("serve"), None);
        assert_eq!(MOVED_COMMANDS.len(), 19);
        assert_eq!(LEGACY_DISPATCH, [("ls", "runs"), ("stats", "usage")]);
    }

    #[test]
    fn a_missing_dispatch_names_where_the_command_went() {
        assert_eq!(
            missing_dispatch("tiers"),
            CliError::Message(
                "moved to Dispatch: dispatch tiers (install Dispatch to get it)".to_string()
            )
        );
    }

    #[test]
    fn the_path_lookup_wants_an_executable_file() {
        let dir = tempfile::tempdir().expect("temp dir");
        let tool = dir.path().join("dispatch");
        fs::write(&tool, b"#!/bin/sh\n").expect("write");
        let path = dir.path().as_os_str();
        assert_eq!(find_in_path("dispatch", Some(path)), None);
        fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("chmod");
        assert_eq!(find_in_path("dispatch", Some(path)), Some(tool));
        assert_eq!(find_in_path("missing", Some(path)), None);
    }
}
