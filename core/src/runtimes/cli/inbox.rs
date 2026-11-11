//! Derived Inbox items.
use super::{
    core::{dial, resolve_project},
    output::{CliError, CliResult},
    scope::{add_scope_flags, project_flags, query},
};
use crate::{domains::inbox, local_v1, platform::coreclient::CALL_TIMEOUT};
const USAGE: &str = "usage: polterware inbox [--scope <node>[,<node>]] [--exact] [--project <id>]\n       polterware inbox <id> decide \"<answer>\" | delegate | defer | ignore [--agent name] [--project <id>]";
pub fn run_inbox(db: &str, args: &[String]) -> CliResult {
    let mut f = project_flags("inbox");
    f.string_flag(
        "agent",
        "",
        "agent label acting on the item (default: $POLTERWARE_AGENT_LABEL)",
    );
    add_scope_flags(&mut f);
    f.parse_interspersed(args)
        .map_err(|e| CliError::message(e.message()))?;
    let p = f.rest();
    if p.len() == 1 || p.len() > 3 {
        return Err(CliError::message(USAGE));
    }
    if p.is_empty() {
        return query(
            db,
            f.string_value("project"),
            "inbox".into(),
            f.list_value("scope").to_vec(),
            f.bool_value("exact"),
        );
    }
    let action = inbox::parse_action(&p[1]).map_err(|e| CliError::message(e.to_string()))?;
    if (action == "decide") != (p.len() == 3) {
        return Err(CliError::message(USAGE));
    }
    let c = dial(db)?;
    let project = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let env = std::env::var("POLTERWARE_AGENT_LABEL").unwrap_or_default();
    let label = f.string_value("agent").trim();
    let label = if label.is_empty() { env.trim() } else { label };
    let r: local_v1::ResolveInboxItemResponse = c.unary(
        "/polterware.local.v1.WorkService/ResolveInboxItem",
        &local_v1::ResolveInboxItemRequest {
            project_id: project.id,
            item_id: p[0].clone(),
            action,
            answer: p.get(2).cloned().unwrap_or_default(),
            session_label: label.into(),
        },
        CALL_TIMEOUT,
    )?;
    let item = r.item.unwrap_or_default();
    println!(
        "{} {}: {}",
        inbox::action_label(&r.action),
        item.id,
        item.title
    );
    if !r.ops.is_empty() {
        println!("{}", r.ops.join("\n"));
    }
    for d in r.diagnostics {
        if d.severity == "hint" {
            println!("hint: {}", d.message);
        }
    }
    Ok(())
}
