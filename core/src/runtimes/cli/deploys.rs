//! The `deploys` command: deploy links and their latest check.
//! state.

use crate::local_v1;
use crate::platform::coreclient::{self, CALL_TIMEOUT};

use super::core::{dial, resolve_project};
use super::flags::{FlagError, FlagSet};
use super::output::{CliError, CliResult};

const USAGE: &str = "usage: polterware deploys [<path>]                  list deploy links with their latest check\n       polterware deploys check [<path>] [--force]    ask each platform's CLI for the latest deploy\n       polterware deploys targets <provider> [--node <path>]   list what a deploy can link to";
const TARGETS: &str = "/polterware.local.v1.IntegrationService/ListDeployTargets";
const CHECK: &str = "/polterware.local.v1.IntegrationService/CheckDeploys";
const QUERY: &str = "/polterware.local.v1.WorkService/QueryPolter";

pub fn run_deploys(db_path: &str, args: &[String]) -> CliResult {
    let mut flags = FlagSet::new("deploys");
    flags.string_flag(
        "project",
        "",
        "project id (default: project containing the cwd)",
    );
    flags.bool_flag("force", false, "check links checked in the last minute too");
    flags.string_flag(
        "node",
        "",
        "Structure path whose checkout is read for targets",
    );
    flags.parse_interspersed(args).map_err(|err| match err {
        FlagError::Help => CliError::message("flag: help requested"),
        FlagError::Invalid(message) => CliError::message(message),
    })?;
    let clients = dial(db_path)?;
    let project = resolve_project(&clients, flags.string_value("project"))?
        .ok_or_else(|| CliError::message("project not found"))?;
    let positional = flags.rest();
    let sub = positional.first().map(String::as_str).unwrap_or("");
    match sub {
        "help" | "-h" | "--help" => {
            println!("{USAGE}");
            Ok(())
        }
        "check" => {
            let path = positional.get(1..).unwrap_or_default().join("");
            let response: local_v1::CheckDeploysResponse = clients.unary(
                CHECK,
                &local_v1::CheckDeploysRequest {
                    project_id: project.id.clone(),
                    node_id: path.clone(),
                    force: flags.bool_value("force"),
                },
                CALL_TIMEOUT,
            )?;
            println!("checked {}", response.checked);
            print_deploys(&clients, &project.id, &path)
        }
        "targets" => {
            if positional.len() != 2 {
                return Err(CliError::message(USAGE));
            }
            let response: local_v1::ListDeployTargetsResponse = clients.unary(
                TARGETS,
                &local_v1::ListDeployTargetsRequest {
                    project_id: project.id,
                    provider: positional[1].clone(),
                    node_id: flags.string_value("node").to_string(),
                },
                CALL_TIMEOUT,
            )?;
            for target in response.targets {
                println!(
                    "{} {}@{}  {}  {}",
                    if target.detected { "*" } else { " " },
                    target.name,
                    dash(&target.id),
                    target.kind,
                    dash(&target.url)
                );
            }
            if !response.message.is_empty() {
                eprintln!("{}", response.message);
            }
            Ok(())
        }
        _ => {
            if positional.len() > 1 {
                return Err(CliError::message(USAGE));
            }
            print_deploys(&clients, &project.id, &positional.join(""))
        }
    }
}

fn print_deploys(clients: &coreclient::Clients, project_id: &str, path: &str) -> CliResult {
    let query = format!("deploys {path}").trim().to_string();
    let response: local_v1::QueryPolterResponse = clients.unary(
        QUERY,
        &local_v1::QueryPolterRequest {
            project_id: project_id.to_string(),
            query,
            ..local_v1::QueryPolterRequest::default()
        },
        CALL_TIMEOUT,
    )?;
    print!("{}", response.text);
    Ok(())
}

fn dash(value: &str) -> &str {
    if value.is_empty() { "-" } else { value }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn dash_matches_go_helper() {
        assert_eq!(dash(""), "-");
        assert_eq!(dash("main"), "main");
    }
}
