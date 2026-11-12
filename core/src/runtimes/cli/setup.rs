//! The `setup` and `hooks` commands: installing the agent tooling and
//! switching its hooks.
use super::{
    core::dial,
    flags::{FlagError, FlagSet},
    output::{CliError, CliResult},
};
use super::{core::resolve_project, repo::cwd_repository};
use crate::platform::coreclient::CALL_TIMEOUT;
use crate::{local_v1, providers::agentsetup};
use std::path::Path;
use std::time::Duration;
const USAGE: &str = "usage: polterware setup [doctor] | claude [--uninstall | --enable | --disable | --skill-only] | codex [--uninstall] [--project <id>] | mcp [list] [--json] | add <client|all> | remove <client|all>";
const SWITCHES: &str = "/polterware.local.v1.AgentRunService/SetHookSwitch";
const LIST: &str = "/polterware.local.v1.AgentRunService/ListHookSwitches";
fn p(e: FlagError) -> CliError {
    CliError::message(e.message())
}
pub fn run_setup(db: &str, args: &[String]) -> CliResult {
    if args.is_empty() || args[0] == "doctor" {
        return setup_doctor(db);
    }
    match args[0].as_str() {
        "mcp" => super::mcpsetup::setup_mcp(db, &args[1..]),
        "claude" => claude(db, &args[1..]),
        "codex" => codex(db, &args[1..]),
        _ => Err(CliError::message(USAGE)),
    }
}
fn claude(db: &str, args: &[String]) -> CliResult {
    let mut f = FlagSet::new("setup claude");
    f.bool_flag(
        "uninstall",
        false,
        "remove the plugin and the local marketplace",
    );
    f.bool_flag("enable", false, "enable the installed plugin");
    f.bool_flag("disable", false, "disable the installed plugin");
    f.bool_flag(
        "skill-only",
        false,
        "install the plugin with every hook switched off",
    );
    f.parse(args).map_err(p)?;
    let n = ["uninstall", "enable", "disable", "skill-only"]
        .iter()
        .filter(|x| f.bool_value(x))
        .count();
    if n > 1 {
        return Err(CliError::message(
            "choose one of --uninstall, --enable, --disable, --skill-only",
        ));
    }
    if f.bool_value("uninstall") {
        agentsetup::uninstall_claude().map_err(|e| CliError::message(e.to_string()))?;
        println!("polterware plugin removed from Claude Code");
        return Ok(());
    }
    if f.bool_value("enable") || f.bool_value("disable") {
        let on = f.bool_value("enable");
        agentsetup::set_claude_enabled(on).map_err(|e| CliError::message(e.to_string()))?;
        println!(
            "polterware plugin {}",
            if on { "enabled" } else { "disabled" }
        );
        return Ok(());
    }
    if f.bool_value("skill-only") {
        set_all_hooks(db, false)?;
    }
    agentsetup::install_claude(
        &Path::new(db)
            .parent()
            .unwrap_or(Path::new("."))
            .display()
            .to_string(),
    )
    .map_err(|e| CliError::message(e.to_string()))?;
    println!(
        "polterware plugin {} installed in Claude Code; restart Claude Code to load it",
        agentsetup::embedded_version()
    );
    if f.bool_value("skill-only") {
        println!(
            "all hooks are off: the skill only. Turn one on with: polterware hooks enable <name>"
        );
    }
    Ok(())
}
fn codex(db: &str, args: &[String]) -> CliResult {
    let mut f = FlagSet::new("setup codex");
    f.bool_flag(
        "uninstall",
        false,
        "remove the skill file and the AGENTS.md line",
    );
    f.string_flag(
        "project",
        "",
        "project id (defaults to the project of the current directory)",
    );
    f.parse(args).map_err(p)?;
    let root = if let Ok(c) = dial(db) {
        resolve_project(&c, f.string_value("project"))
            .ok()
            .flatten()
            .map(|project| cwd_repository(&c, &project.id))
            .unwrap_or_default()
    } else {
        String::new()
    };
    if f.bool_value("uninstall") {
        agentsetup::uninstall_agents(&root).map_err(|e| CliError::message(e.to_string()))?;
        println!("polterware skill and AGENTS.md line removed");
        return Ok(());
    }
    agentsetup::install_agents(&root).map_err(|e| CliError::message(e.to_string()))?;
    println!("polter skill written for Codex and other agents");
    if root.is_empty() {
        println!("no project found here: AGENTS.md line skipped");
    } else {
        println!("AGENTS.md line added in {root}");
    }
    Ok(())
}
pub fn run_hooks(db: &str, args: &[String]) -> CliResult {
    if args.is_empty() || args[0] == "list" {
        return print_hooks(db);
    }
    if args.len() != 2 || !["enable", "disable"].contains(&args[0].as_str()) {
        return Err(CliError::message(
            "usage: polterware hooks [list] | enable|disable <name|all>",
        ));
    }
    let enabled = args[0] == "enable";
    let name = &args[1];
    if name == "dispatch" {
        return super::moved::run_moved(if enabled { "on" } else { "off" }, &[]);
    }
    let names: Vec<String> = if name == "all" {
        agentsetup::HOOKS
            .iter()
            .map(|h| h.name.to_string())
            .collect()
    } else if agentsetup::valid_hook(name) {
        vec![name.clone()]
    } else {
        return Err(CliError::message(format!(
            "unknown hook {name:?}: use {}",
            agentsetup::HOOKS
                .iter()
                .map(|h| h.name)
                .collect::<Vec<_>>()
                .join(", ")
        )));
    };
    let c = dial(db)?;
    for n in names {
        let _: local_v1::SetHookSwitchResponse = c.unary(
            SWITCHES,
            &local_v1::SetHookSwitchRequest {
                name: n.clone(),
                enabled,
            },
            CALL_TIMEOUT,
        )?;
        if name != "all" {
            println!("hook {} {}", n, if enabled { "on" } else { "off" })
        }
    }
    Ok(())
}
fn print_hooks(db: &str) -> CliResult {
    let c = dial(db)?;
    let r: local_v1::ListHookSwitchesResponse = c.unary(
        LIST,
        &local_v1::ListHookSwitchesRequest {},
        Duration::from_secs(10),
    )?;
    for h in r.hooks {
        println!(
            "{:<3} {:<8} {}",
            if h.enabled { "on" } else { "off" },
            h.name,
            h.label
        )
    }
    Ok(())
}

fn setup_doctor(db: &str) -> CliResult {
    let c = dial(db)?;
    let r: local_v1::ListHookSwitchesResponse = c.unary(
        LIST,
        &local_v1::ListHookSwitchesRequest {},
        Duration::from_secs(10),
    )?;
    print!(
        "{}",
        setup_doctor_text(
            &std::path::Path::new(db)
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .display()
                .to_string(),
            &r.hooks
        )
    );
    Ok(())
}

fn setup_doctor_text(db_dir: &str, hooks: &[local_v1::HookSwitch]) -> String {
    let report = agentsetup::doctor(db_dir);
    let mut lines = Vec::new();
    let mut line = |ok: bool, text: String| {
        lines.push(format!("{}  {text}\n", if ok { "ok" } else { "!!" }));
    };
    line(
        !report.polterware_path.is_empty(),
        format!(
            "polterware on PATH{}",
            suffix(&report.polterware_path, "hooks skip themselves without it")
        ),
    );
    line(
        !report.claude_path.is_empty(),
        format!(
            "claude detected{}",
            suffix(&report.claude_path, "install Claude Code to use the plugin")
        ),
    );
    if !report.claude_path.is_empty() {
        if !report.problem.is_empty() {
            line(
                false,
                format!("plugin state unreadable: {}", report.problem),
            );
        } else if !report.plugin.installed {
            line(
                false,
                "plugin not installed: polterware setup claude".to_string(),
            );
        } else if !report.current() {
            line(
                false,
                format!(
                    "plugin {} is behind this binary ({}): polterware setup claude",
                    report.plugin.version, report.embedded_ver
                ),
            );
        } else {
            let state = if report.plugin.enabled {
                "enabled"
            } else {
                "disabled"
            };
            line(
                report.plugin.enabled,
                format!("plugin {} {state}", report.plugin.version),
            );
        }
    }
    let agents = agentsetup::inspect_agents("");
    line(
        agents.skill_ready,
        format!("Codex skill file{}", suffix(&agents.skill_path, "")),
    );
    for hook in hooks {
        lines.push(format!(
            "{:<3} {:<8} {}\n",
            if hook.enabled { "on" } else { "off" },
            hook.name,
            hook.label
        ));
    }
    lines.concat()
}

fn suffix(value: &str, hint: &str) -> String {
    if !value.is_empty() {
        format!(" ({value})")
    } else if !hint.is_empty() {
        format!(": {hint}")
    } else {
        String::new()
    }
}

fn set_all_hooks(db: &str, enabled: bool) -> CliResult {
    let c = dial(db)?;
    for h in agentsetup::HOOKS {
        let _: local_v1::SetHookSwitchResponse = c.unary(
            SWITCHES,
            &local_v1::SetHookSwitchRequest {
                name: h.name.to_string(),
                enabled,
            },
            CALL_TIMEOUT,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::agentsetup::test_support::{EnvGuard, lock_env};
    use std::fs;

    #[cfg(unix)]
    fn executable(path: &std::path::Path) {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("chmod fake client");
    }

    #[test]
    fn doctor_text_reports_path_clients_plugin_skill_and_hooks() {
        let _lock = lock_env();
        let home = tempfile::tempdir().expect("fake home");
        let bin = tempfile::tempdir().expect("fake PATH");
        let claude = bin.path().join("claude");
        fs::write(&claude, "#!/bin/sh\nprintf '[]'\n").expect("fake claude");
        #[cfg(unix)]
        executable(&claude);
        let polterware = bin.path().join("polterware");
        fs::write(&polterware, "#!/bin/sh\nexit 0\n").expect("fake polterware");
        #[cfg(unix)]
        executable(&polterware);
        let _home = EnvGuard::set("HOME", home.path());
        let _path = EnvGuard::set("PATH", bin.path());
        let hooks = vec![local_v1::HookSwitch {
            name: "status".to_string(),
            label: "Session-start status".to_string(),
            enabled: true,
            note: String::new(),
        }];
        let text = setup_doctor_text("/tmp/data", &hooks);
        assert_eq!(
            text,
            format!(
                "ok  polterware on PATH ({})\nok  claude detected ({})\n!!  plugin not installed: polterware setup claude\n!!  Codex skill file ({}/.agents/skills/polter/SKILL.md)\non  status   Session-start status\n",
                polterware.display(),
                claude.display(),
                home.path().display()
            )
        );
    }
}
