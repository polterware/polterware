//! Port of `backend/runtimes/cli/mcpsetup.go`.
use super::{
    flags::{FlagError, FlagSet},
    output::{CliError, CliResult},
};
use crate::providers::agentsetup;
const USAGE: &str = "usage: polterware setup mcp [list] [--json] | add <claude|codex|cursor|all> | remove <claude|codex|cursor|all>";
fn p(e: FlagError) -> CliError {
    CliError::message(e.message())
}
pub fn setup_mcp(db: &str, args: &[String]) -> CliResult {
    let mut action = "list";
    let mut rest = args;
    if let Some(first) = args.first() {
        if first != "--json" {
            action = first;
            rest = &args[1..]
        }
    }
    let exe = crate::platform::coreclient::executable()?
        .display()
        .to_string();
    match action {
        "list" => {
            let mut f = FlagSet::new("setup mcp list");
            f.bool_flag("json", false, "print JSON");
            f.parse(rest).map_err(p)?;
            let mut rows = Vec::new();
            for c in agentsetup::mcp_clients() {
                let (status, error) = match c.status(&exe, db) {
                    Ok(x) => (x.as_str().to_string(), None),
                    Err(e) => ("unknown".to_string(), Some(e.to_string())),
                };
                rows.push(
                    serde_json::json!({"id":c.id,"name":c.name,"status":status,"error":error}),
                );
            }
            if f.bool_value("json") {
                println!(
                    "{}",
                    serde_json::to_string(
                        &serde_json::json!({"command":exe,"dbPath":db,"clients":rows})
                    )
                    .unwrap_or_default()
                )
            } else {
                for row in rows {
                    let name = row["name"].as_str().unwrap_or("");
                    let label = match row["status"].as_str().unwrap_or("") {
                        "not_installed" => "not installed",
                        "not_configured" => "not set up",
                        "configured" => "set up",
                        "configured_other" => "set up for another binary or database",
                        _ => "unknown",
                    };
                    let mut line = format!("{name:<12} {label}");
                    if let Some(e) = row["error"].as_str() {
                        line.push_str(&format!(" ({e})"))
                    }
                    println!("{line}")
                }
            }
            Ok(())
        }
        "add" | "remove" => {
            if rest.len() != 1 {
                return Err(CliError::message(USAGE));
            }
            let clients = if rest[0] == "all" {
                agentsetup::mcp_clients()
                    .into_iter()
                    .filter(|c| {
                        c.status(&exe, "")
                            .is_ok_and(|s| s != agentsetup::McpStatus::NotInstalled)
                    })
                    .collect()
            } else {
                vec![agentsetup::find_mcp_client(&rest[0]).ok_or_else(|| {
                    CliError::message(format!(
                        "unknown MCP client {:?} (claude, codex, cursor, or all)",
                        rest[0]
                    ))
                })?]
            };
            for c in clients {
                if action == "add" {
                    agentsetup::configure_mcp(&c, &exe, db)
                        .map_err(|e| CliError::message(e.to_string()))?;
                    println!("{} will launch Polterware MCP in new sessions", c.name)
                } else {
                    c.remove().map_err(|e| CliError::message(e.to_string()))?;
                    println!("Polterware removed from {}", c.name)
                }
            }
            Ok(())
        }
        _ => Err(CliError::message(USAGE)),
    }
}
