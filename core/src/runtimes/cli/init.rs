//! The `init` command: registering a project.

use std::path::Path;

use crate::local_v1;
use crate::platform::coreclient::CALL_TIMEOUT;

use super::core::dial;
use super::flags::{FlagError, FlagSet};
use super::output::{CliError, CliResult, write_json};

const ROUTE: &str = "/polterware.local.v1.ProjectService/InitProject";

pub fn run_init(db_path: &str, args: &[String]) -> CliResult {
    let mut flags = FlagSet::new("init");
    flags.string_flag("name", "", "project name (default: the folder's name)");
    flags.string_flag(
        "dir",
        "",
        "local clone the first component links (default: cwd)",
    );
    flags.string_flag("path", "", "alias of --dir");
    flags.bool_flag("no-dir", false, "start with no repository");
    flags.string_flag("id", "", "optional project id");
    flags.bool_flag("json", false, "print JSON");
    flags.parse(args).map_err(flag_error)?;
    let mut dir = String::new();
    if !flags.bool_value("no-dir") {
        let source = if !flags.string_value("path").is_empty() {
            flags.string_value("path")
        } else {
            flags.string_value("dir")
        };
        let source = if source.is_empty() {
            std::env::current_dir()?.to_string_lossy().into_owned()
        } else {
            expand_path(source)
        };
        dir = super::db::abs(&source)?;
    }
    let name = if flags.string_value("name").is_empty() && !dir.is_empty() {
        Path::new(&dir)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    } else {
        flags.string_value("name").to_string()
    };
    let clients = dial(db_path)?;
    let response: local_v1::InitProjectResponse = clients.unary(
        ROUTE,
        &local_v1::InitProjectRequest {
            id: flags.string_value("id").to_string(),
            name,
            dir: dir.clone(),
        },
        CALL_TIMEOUT,
    )?;
    let project = response
        .project
        .ok_or_else(|| CliError::message("init response has no project"))?;
    if flags.bool_value("json") {
        return write_json(
            &serde_json::json!({"id": project.id, "name": project.name, "dir": dir, "status": project.status}),
        );
    }
    if dir.is_empty() {
        println!("Initialized project {} ({})", project.name, project.id);
    } else {
        println!(
            "Initialized project {} ({}); its first component links {}",
            project.name, project.id, dir
        );
    }
    Ok(())
}

fn expand_path(path: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    if path == "~" {
        home
    } else if let Some(rest) = path.strip_prefix("~/") {
        format!("{home}/{rest}")
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn the_path_alias_wins_over_dir_like_flag_stringvar() {
        let mut flags = FlagSet::new("init");
        flags.string_flag("dir", "", "dir");
        flags.string_flag("path", "", "path");
        flags
            .parse(&[
                "--dir".into(),
                "first".into(),
                "--path".into(),
                "second".into(),
            ])
            .expect("flags");
        assert_eq!(flags.string_value("path"), "second");
    }

    #[test]
    fn json_init_shape_has_go_field_names_and_order() {
        let value = serde_json::json!({"id":"p1", "name":"Demo", "dir":"", "status":"active"});
        let json = super::super::output::json_string(&value).expect("json");
        assert_eq!(
            json,
            "{\n  \"dir\": \"\",\n  \"id\": \"p1\",\n  \"name\": \"Demo\",\n  \"status\": \"active\"\n}\n"
        );
    }
    #[test]
    fn tilde_expansion_is_limited_to_home_prefix() {
        assert_eq!(
            expand_path("~/repo"),
            format!("{}/repo", std::env::var("HOME").unwrap_or_default())
        );
        assert_eq!(expand_path("~other/repo"), "~other/repo");
    }
}

fn flag_error(err: FlagError) -> CliError {
    match err {
        FlagError::Help => CliError::message("flag: help requested"),
        FlagError::Invalid(message) => CliError::message(message),
    }
}
