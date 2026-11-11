//! Port of `backend/runtimes/cli/repo.go`: the project's repositories.

use std::path::Path;

use crate::local_v1;
use crate::platform::coreclient::CALL_TIMEOUT;

use super::core::{dial, resolve_project};
use super::flags::{FlagError, FlagSet};
use super::output::{CliError, CliResult, write_proto_json};

const ROUTE: &str = "/polterware.local.v1.ProjectService/ListProjectRepos";
const USAGE: &str = "usage: polterware repo list [--project <id>] [--json]\n(a Structure component links a repository with dir: <folder> in polterware apply)";

pub fn run_repo(db_path: &str, args: &[String]) -> CliResult {
    if args.first().map(String::as_str) != Some("list") {
        return Err(CliError::message(USAGE));
    }
    let mut flags = FlagSet::new("repo list");
    flags.string_flag(
        "project",
        "",
        "project id (default: project containing the cwd)",
    );
    flags.bool_flag("json", false, "print JSON");
    flags.parse(&args[1..]).map_err(flag_error)?;
    let clients = dial(db_path)?;
    let project = resolve_project(&clients, flags.string_value("project"))?
        .ok_or_else(|| CliError::message("project not found"))?;
    let response: local_v1::ListProjectReposResponse = clients.unary(
        ROUTE,
        &local_v1::ListProjectReposRequest {
            project_id: project.id,
        },
        CALL_TIMEOUT,
    )?;
    if flags.bool_value("json") {
        return write_proto_json(&response);
    }
    if response.repos.is_empty() {
        println!("no repositories: link a component to a clone with dir: <folder>");
        return Ok(());
    }
    for repo in response.repos {
        println!(
            "{}  {}  {}",
            repo.name,
            dash(&repo.remote),
            dash(&repo.local_path)
        );
    }
    Ok(())
}

fn flag_error(err: FlagError) -> CliError {
    match err {
        FlagError::Help => CliError::message("flag: help requested"),
        FlagError::Invalid(message) => CliError::message(message),
    }
}
fn dash(value: &str) -> &str {
    if value.is_empty() { "-" } else { value }
}

/// Returns the clone containing cwd, or the only clone in the project.
pub fn cwd_repository(clients: &crate::platform::coreclient::Clients, project_id: &str) -> String {
    let response: local_v1::ListProjectReposResponse = match clients.unary(
        ROUTE,
        &local_v1::ListProjectReposRequest {
            project_id: project_id.to_string(),
        },
        CALL_TIMEOUT,
    ) {
        Ok(value) => value,
        Err(_) => return String::new(),
    };
    let cwd = match std::env::current_dir() {
        Ok(value) => value,
        Err(_) => return String::new(),
    };
    let mut clones = Vec::new();
    let mut best = String::new();
    for repo in response.repos {
        let root = expand_home(&repo.local_path);
        if root.is_empty() {
            continue;
        }
        let root_path = Path::new(&root);
        clones.push(root.clone());
        if cwd.strip_prefix(root_path).is_ok() && root.len() > best.len() {
            best = root;
        }
    }
    if best.is_empty() && clones.len() == 1 {
        best = clones.remove(0);
    }
    best
}

fn expand_home(path: &str) -> String {
    if path == "~" {
        return std::env::var("HOME").unwrap_or_default();
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return format!("{}/{}", std::env::var("HOME").unwrap_or_default(), rest);
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    #[test]
    fn empty_values_render_as_dash() {
        assert_eq!(dash(""), "-");
        assert_eq!(dash("origin"), "origin");
    }
}
