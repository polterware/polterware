//! Port of the setup and hook switch commands.
use super::{
    core::dial,
    flags::{FlagError, FlagSet},
    output::{CliError, CliResult},
};
use crate::platform::coreclient::CALL_TIMEOUT;
use crate::{local_v1, providers::agentsetup};
use std::time::Duration;
const USAGE: &str = "usage: polterware setup [doctor] | claude [--uninstall | --enable | --disable | --skill-only] | codex [--uninstall] [--project <id>] | mcp [list] [--json] | add <client|all> | remove <client|all>";
const SWITCHES: &str = "/polterware.local.v1.AgentRunService/SetHookSwitch";
const LIST: &str = "/polterware.local.v1.AgentRunService/ListHookSwitches";
fn p(e: FlagError) -> CliError {
    CliError::message(e.message())
}
pub fn run_setup(db: &str, args: &[String]) -> CliResult {
    if args.is_empty() || args[0] == "doctor" {
        return print_hooks(db);
    }
    match args[0].as_str() {
        "mcp" => super::mcpsetup::setup_mcp(db, &args[1..]),
        "claude" => claude(db, &args[1..]),
        "codex" => codex(db, &args[1..]),
        _ => Err(CliError::message(USAGE)),
    }
}
fn claude(_db: &str, args: &[String]) -> CliResult {
    let mut f = FlagSet::new("setup claude");
    f.bool_flag("uninstall", false, "remove plugin");
    f.bool_flag("enable", false, "enable plugin");
    f.bool_flag("disable", false, "disable plugin");
    f.bool_flag("skill-only", false, "install skill only");
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
    Err(CliError::message(
        "Claude installation is not supported by the Rust runtime",
    ))
}
fn codex(_db: &str, args: &[String]) -> CliResult {
    let mut f = FlagSet::new("setup codex");
    f.bool_flag("uninstall", false, "remove skill");
    f.string_flag("project", "", "project id");
    f.parse(args).map_err(p)?;
    Err(CliError::message(
        "Codex setup requires the Go-compatible checkout resolver",
    ))
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
        return Err(CliError::message("dispatch on|off"));
    }
    let c = dial(db)?;
    let names: Vec<String> = if name == "all" {
        agentsetup::HOOKS
            .iter()
            .map(|h| h.name.to_string())
            .collect()
    } else if agentsetup::valid_hook(name) {
        vec![name.clone()]
    } else {
        return Err(CliError::message(format!(
            "unknown hook {name:?}: use status, stop, lint, context, session"
        )));
    };
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
