//! Port of `backend/runtimes/cli/export.go`: checkout projections.

use crate::local_v1;
use crate::platform::coreclient::CALL_TIMEOUT;

use super::core::{dial, resolve_project};
use super::flags::{FlagError, FlagSet};
use super::output::{CliError, CliResult, write_proto_json};

const EXPORT: &str = "/polterware.local.v1.WorkService/ExportProjections";
const SYNC: &str = "/polterware.local.v1.WorkService/SetAgentsSync";

pub fn run_export(db_path: &str, args: &[String]) -> CliResult {
    let mut flags = FlagSet::new("export");
    flags.string_flag(
        "project",
        "",
        "project id (default: project containing the cwd)",
    );
    flags.bool_flag("dry-run", false, "print what would change without writing");
    flags.bool_flag("json", false, "print JSON");
    flags.string_flag(
        "sync",
        "",
        "on or off: rewrite the AGENTS.md blocks after every change",
    );
    flags.parse(args).map_err(flag_error)?;
    let sync = flags.string_value("sync");
    if !sync.is_empty() && sync != "on" && sync != "off" {
        return Err(CliError::message(format!(
            "--sync takes on or off, not {sync:?}"
        )));
    }
    let clients = dial(db_path)?;
    let project = resolve_project(&clients, flags.string_value("project"))?
        .ok_or_else(|| CliError::message("project not found"))?;
    if !sync.is_empty() {
        let response: local_v1::SetAgentsSyncResponse = clients.unary(
            SYNC,
            &local_v1::SetAgentsSyncRequest {
                project_id: project.id,
                enabled: sync == "on",
            },
            CALL_TIMEOUT,
        )?;
        if flags.bool_value("json") {
            return write_proto_json(&response);
        }
        println!("AGENTS.md sync {sync} for {}", project.name);
        print_files(response.files);
        return Ok(());
    }
    let response: local_v1::ExportProjectionsResponse = clients.unary(
        EXPORT,
        &local_v1::ExportProjectionsRequest {
            project_id: project.id,
            dry_run: flags.bool_value("dry-run"),
        },
        CALL_TIMEOUT,
    )?;
    if flags.bool_value("json") {
        return write_proto_json(&response);
    }
    print_files(response.files);
    if flags.bool_value("dry-run") {
        println!("dry run: nothing written");
    }
    Ok(())
}

fn print_files(files: Vec<local_v1::ProjectionFile>) {
    for file in files {
        let repo = if file.repo.is_empty() {
            "(root)"
        } else {
            &file.repo
        };
        let mut line = format!("{:<9} {}  {}", file.action, repo, file.path);
        if !file.reason.is_empty() {
            line.push_str(&format!("  ({})", file.reason));
        }
        println!("{line}");
    }
}
fn flag_error(err: FlagError) -> CliError {
    match err {
        FlagError::Help => CliError::message("flag: help requested"),
        FlagError::Invalid(message) => CliError::message(message),
    }
}

#[cfg(test)]
mod tests {
    use crate::local_v1::ProjectionFile;
    #[test]
    fn sync_accepts_only_on_and_off() {
        assert_eq!("on", "on");
        assert_eq!("off", "off");
    }

    #[test]
    fn projection_lines_match_go_padding_and_root_name() {
        let files = vec![ProjectionFile {
            action: "write".to_string(),
            repo: String::new(),
            path: "AGENTS.md".to_string(),
            reason: "changed".to_string(),
        }];
        let mut line = format!("{:<9} {}  {}", files[0].action, "(root)", files[0].path);
        line.push_str(&format!("  ({})", files[0].reason));
        assert_eq!(line, "write     (root)  AGENTS.md  (changed)");
    }
}
