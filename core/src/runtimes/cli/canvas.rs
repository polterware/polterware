//! Port of `backend/runtimes/cli/canvas.go`.

use super::{
    core::{dial, resolve_project},
    flags::{FlagError, FlagSet},
    output::{CliError, CliResult},
};
use crate::platform::coreclient::CALL_TIMEOUT;
use crate::{domains::canvas as domain, local_v1};
use std::time::Duration;

const USAGE: &str = "usage: polterware canvas ls [--agents] [--scope <node>[,<node>]] [--exact] [--project <id>]\n       polterware canvas show <id|name> [--format mermaid|json|dot] [--project <id>]\n       polterware canvas share <id|name> on|off [--project <id>]";
const LIST: &str = "/polterware.local.v1.CanvasService/ListCanvases";
const UPDATE: &str = "/polterware.local.v1.CanvasService/UpdateCanvas";

fn parse_err(err: FlagError) -> CliError {
    CliError::message(err.message())
}
fn flags(name: &'static str) -> FlagSet {
    let mut f = FlagSet::new(name);
    f.string_flag(
        "project",
        "",
        "project id (default: project containing the cwd)",
    );
    f
}
fn get_canvases(
    c: &crate::platform::coreclient::Clients,
    project: &str,
    node: &str,
    exact: bool,
) -> Result<Vec<local_v1::Canvas>, CliError> {
    let r: local_v1::ListCanvasesResponse = c.unary(
        LIST,
        &local_v1::ListCanvasesRequest {
            project_id: project.into(),
            scope_node: node.into(),
            scope_exact: exact,
            ..Default::default()
        },
        CALL_TIMEOUT,
    )?;
    Ok(r.canvases)
}
pub fn run_canvas(db: &str, args: &[String]) -> CliResult {
    if args.is_empty() {
        return Err(CliError::message(USAGE));
    }
    match args[0].as_str() {
        "ls" => ls(db, &args[1..]),
        "show" => show(db, &args[1..]),
        "share" => share(db, &args[1..]),
        _ => Err(CliError::message(USAGE)),
    }
}
fn ls(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("canvas ls");
    f.bool_flag("agents", false, "list only canvases agents can read");
    f.list_flag("scope", "Structure nodes", true);
    f.bool_flag("exact", false, "match exact scope");
    f.parse_interspersed(args).map_err(parse_err)?;
    if !f.rest().is_empty() {
        return Err(CliError::message(USAGE));
    }
    let c = dial(db)?;
    let p = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let scopes = f.list_value("scope");
    let scopes = if scopes.is_empty() {
        vec![String::new()]
    } else {
        scopes.to_vec()
    };
    let mut all = Vec::new();
    for scope in scopes {
        all.extend(get_canvases(&c, &p.id, &scope, f.bool_value("exact"))?);
    }
    all.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    all.dedup_by(|a, b| a.id == b.id);
    if f.bool_value("agents") {
        all.retain(|x| x.agent_visible)
    }
    if all.is_empty() {
        print!(
            "{}\n",
            if f.bool_value("agents") {
                "no agent-visible canvases"
            } else {
                "no canvases"
            }
        );
        return Ok(());
    }
    let mut w = [2usize, 4, 4, 7];
    for x in &all {
        w[0] = w[0].max(x.id.len());
        w[1] = w[1].max(x.name.len());
        w[2] = w[2].max(x.r#type.len());
        w[3] = w[3].max(if x.agent_visible { 3 } else { 2 });
    }
    println!(
        "{:<w0$}  {:<w1$}  {:<w2$}  {:<w3$}  scope",
        "id",
        "name",
        "type",
        "visible",
        w0 = w[0],
        w1 = w[1],
        w2 = w[2],
        w3 = w[3]
    );
    for x in all {
        println!(
            "{:<w0$}  {:<w1$}  {:<w2$}  {:<w3$}  {}",
            x.id,
            x.name,
            x.r#type,
            if x.agent_visible { "yes" } else { "no" },
            x.scope_node_id,
            w0 = w[0],
            w1 = w[1],
            w2 = w[2],
            w3 = w[3]
        );
    }
    Ok(())
}
fn resolve_canvas(
    c: &crate::platform::coreclient::Clients,
    p: &str,
    refname: &str,
) -> Result<local_v1::Canvas, CliError> {
    let mut items = get_canvases(c, p, "", false)?;
    let needle = refname.trim();
    if let Some(x) = items.iter().find(|x| x.id == needle) {
        return Ok(x.clone());
    }
    items.retain(|x| x.name == needle);
    match items.len() {
        1 => Ok(items.remove(0)),
        0 => Err(CliError::message(format!(
            "unknown canvas {needle:?}: list them with polterware canvas ls"
        ))),
        n => Err(CliError::message(format!(
            "canvas name {needle:?} matches {n} canvases: use an id"
        ))),
    }
}
fn show(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("canvas show");
    f.string_flag("format", "mermaid", "mermaid, json, or dot");
    f.parse_interspersed(args).map_err(parse_err)?;
    if f.rest().len() != 1 {
        return Err(CliError::message(USAGE));
    }
    let fmt = f.string_value("format").trim().to_lowercase();
    if !["json", "mermaid", "dot"].contains(&fmt.as_str()) {
        return Err(CliError::message(format!(
            "unknown format {:?}: use mermaid, json, or dot",
            f.string_value("format")
        )));
    }
    let c = dial(db)?;
    let p = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let item = resolve_canvas(&c, &p.id, &f.rest()[0])?;
    let mut g = domain::Graph::default();
    if let Some(raw) = item.graph {
        for n in raw.nodes {
            let props = serde_json::from_str(&n.props_json).unwrap_or_default();
            g.nodes.push(domain::Node {
                id: n.id,
                kind: n.kind,
                label: n.label,
                props,
                r#ref: n.r#ref.map(|r| domain::Ref {
                    kind: r.kind,
                    id: r.id,
                }),
            });
        }
        for e in raw.edges {
            let props = serde_json::from_str(&e.props_json).unwrap_or_default();
            g.edges.push(domain::Edge {
                id: e.id,
                from: e.from,
                to: e.to,
                kind: e.kind,
                props,
            });
        }
    }
    let text = domain::export_format(&g, &item.layout_json, &fmt)
        .map_err(|e| CliError::message(e.to_string()))?;
    print!("{}{}", text, if text.ends_with('\n') { "" } else { "\n" });
    Ok(())
}
fn share(db: &str, args: &[String]) -> CliResult {
    let mut f = flags("canvas share");
    f.parse_interspersed(args).map_err(parse_err)?;
    if f.rest().len() != 2 || !["on", "off"].contains(&f.rest()[1].as_str()) {
        return Err(CliError::message(USAGE));
    }
    let c = dial(db)?;
    let p = resolve_project(&c, f.string_value("project"))?
        .ok_or_else(|| CliError::message("no project found"))?;
    let item = resolve_canvas(&c, &p.id, &f.rest()[0])?;
    let on = f.rest()[1] == "on";
    let change = local_v1::ChangeEnvelope {
        summary: if on {
            "Share canvas with agents"
        } else {
            "Hide canvas from agents"
        }
        .into(),
        rationale: if on {
            "agent_visible turned on from the CLI"
        } else {
            "agent_visible turned off from the CLI"
        }
        .into(),
        ..Default::default()
    };
    let response: local_v1::UpdateCanvasResponse = c.unary(
        UPDATE,
        &local_v1::UpdateCanvasRequest {
            project_id: p.id,
            id: item.id,
            agent_visible: Some(on),
            change: Some(change),
            ..Default::default()
        },
        Duration::from_secs(10),
    )?;
    let updated = response
        .canvas
        .ok_or_else(|| CliError::message("canvas update returned no canvas"))?;
    println!(
        "{}: agent_visible {}",
        updated.name,
        if updated.agent_visible { "on" } else { "off" }
    );
    Ok(())
}
