//! The `canvas` command.

use super::{
    core::{dial, resolve_project},
    flags::{FlagError, FlagSet},
    output::{CliError, CliResult},
};
use crate::platform::coreclient::CALL_TIMEOUT;
use crate::{domains::canvas as domain, local_v1};
use std::io::Read;
use std::time::Duration;

const USAGE: &str = "usage: polterware canvas ls [--agents] [--scope <node>[,<node>]] [--exact] [--project <id>]\n       polterware canvas show <id|name> [--format mermaid|json|dot] [--project <id>]\n       polterware canvas save <id|name> --source <file|-> [--project <id>]\n       polterware canvas share <id|name> on|off [--project <id>]";
const LIST: &str = "/polterware.local.v1.CanvasService/ListCanvases";
const UPDATE: &str = "/polterware.local.v1.CanvasService/UpdateCanvas";

#[derive(Debug, PartialEq, Eq)]
struct CanvasListArgs {
    project: String,
    agents: bool,
    scope: Vec<String>,
    exact: bool,
}

#[derive(Debug, PartialEq, Eq)]
struct CanvasShowArgs {
    project: String,
    reference: String,
    format: String,
}

#[derive(Debug, PartialEq, Eq)]
struct CanvasSaveArgs {
    project: String,
    reference: String,
    source: String,
}

#[derive(Debug, PartialEq, Eq)]
struct CanvasShareArgs {
    project: String,
    reference: String,
    on: bool,
}

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
        "save" => save(db, &args[1..]),
        "share" => share(db, &args[1..]),
        _ => Err(CliError::message(USAGE)),
    }
}

fn parse_canvas_list(args: &[String]) -> Result<CanvasListArgs, CliError> {
    let mut f = flags("canvas ls");
    f.bool_flag("agents", false, "list only canvases agents can read");
    f.list_flag("scope", "Structure nodes", true);
    f.bool_flag("exact", false, "match exact scope");
    f.parse_interspersed(args).map_err(parse_err)?;
    if !f.rest().is_empty() {
        return Err(CliError::message(USAGE));
    }
    Ok(CanvasListArgs {
        project: f.string_value("project").to_owned(),
        agents: f.bool_value("agents"),
        scope: f.list_value("scope").to_vec(),
        exact: f.bool_value("exact"),
    })
}

fn parse_canvas_show(args: &[String]) -> Result<CanvasShowArgs, CliError> {
    let mut f = flags("canvas show");
    f.string_flag("format", "mermaid", "mermaid, json, or dot");
    f.parse_interspersed(args).map_err(parse_err)?;
    if f.rest().len() != 1 {
        return Err(CliError::message(USAGE));
    }
    let format = f.string_value("format").trim().to_lowercase();
    if !["json", "mermaid", "dot"].contains(&format.as_str()) {
        return Err(CliError::message(format!(
            "unknown format {:?}: use mermaid, json, or dot",
            f.string_value("format")
        )));
    }
    Ok(CanvasShowArgs {
        project: f.string_value("project").to_owned(),
        reference: f.rest()[0].clone(),
        format,
    })
}

fn parse_canvas_save(args: &[String]) -> Result<CanvasSaveArgs, CliError> {
    let mut f = flags("canvas save");
    f.string_flag("source", "", "file holding the Mermaid source, or - for stdin");
    f.parse_interspersed(args).map_err(parse_err)?;
    if f.rest().len() != 1 || f.string_value("source").is_empty() {
        return Err(CliError::message(USAGE));
    }
    Ok(CanvasSaveArgs {
        project: f.string_value("project").to_owned(),
        reference: f.rest()[0].clone(),
        source: f.string_value("source").to_owned(),
    })
}

fn parse_canvas_share(args: &[String]) -> Result<CanvasShareArgs, CliError> {
    let mut f = flags("canvas share");
    f.parse_interspersed(args).map_err(parse_err)?;
    if f.rest().len() != 2 || !["on", "off"].contains(&f.rest()[1].as_str()) {
        return Err(CliError::message(USAGE));
    }
    Ok(CanvasShareArgs {
        project: f.string_value("project").to_owned(),
        reference: f.rest()[0].clone(),
        on: f.rest()[1] == "on",
    })
}

fn ls(db: &str, args: &[String]) -> CliResult {
    let parsed = parse_canvas_list(args)?;
    let c = dial(db)?;
    let p = resolve_project(&c, &parsed.project)?
        .ok_or_else(|| CliError::message("no project found"))?;
    let scopes = if parsed.scope.is_empty() {
        vec![String::new()]
    } else {
        parsed.scope
    };
    let mut all = Vec::new();
    for scope in scopes {
        all.extend(get_canvases(&c, &p.id, &scope, parsed.exact)?);
    }
    all.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    all.dedup_by(|a, b| a.id == b.id);
    all = select_canvases(all, parsed.agents);
    print!("{}", format_canvas_list(&all, parsed.agents));
    Ok(())
}
fn resolve_canvas(
    c: &crate::platform::coreclient::Clients,
    p: &str,
    refname: &str,
) -> Result<local_v1::Canvas, CliError> {
    let items = get_canvases(c, p, "", false)?;
    find_canvas(&items, refname)
}

fn find_canvas(items: &[local_v1::Canvas], refname: &str) -> Result<local_v1::Canvas, CliError> {
    let needle = refname.trim();
    if let Some(x) = items.iter().find(|x| x.id == needle) {
        return Ok(x.clone());
    }
    let mut named = items.iter().filter(|x| x.name == needle);
    match (named.next(), named.next()) {
        (Some(item), None) => Ok(item.clone()),
        (None, _) => Err(CliError::message(format!(
            "unknown canvas {needle:?}: list them with polterware canvas ls"
        ))),
        (Some(_), Some(_)) => {
            let count = items.iter().filter(|x| x.name == needle).count();
            Err(CliError::message(format!(
                "canvas name {needle:?} matches {count} canvases: use an id"
            )))
        }
    }
}

fn select_canvases(items: Vec<local_v1::Canvas>, agents_only: bool) -> Vec<local_v1::Canvas> {
    if agents_only {
        items
            .into_iter()
            .filter(|item| item.agent_visible)
            .collect()
    } else {
        items
    }
}

fn format_canvas_list(items: &[local_v1::Canvas], agents_only: bool) -> String {
    if items.is_empty() {
        return if agents_only {
            "no agent-visible canvases\n".to_owned()
        } else {
            "no canvases\n".to_owned()
        };
    }
    let mut widths = [2usize, 4, 4, 7];
    for item in items {
        widths[0] = widths[0].max(item.id.len());
        widths[1] = widths[1].max(item.name.len());
        widths[2] = widths[2].max(item.r#type.len());
        widths[3] = widths[3].max(if item.agent_visible { 3 } else { 2 });
    }
    let mut text = format!(
        "{:<w0$}  {:<w1$}  {:<w2$}  {:<w3$}  scope\n",
        "id",
        "name",
        "type",
        "visible",
        w0 = widths[0],
        w1 = widths[1],
        w2 = widths[2],
        w3 = widths[3]
    );
    for item in items {
        text.push_str(&format!(
            "{:<w0$}  {:<w1$}  {:<w2$}  {:<w3$}  {}\n",
            item.id,
            item.name,
            item.r#type,
            if item.agent_visible { "yes" } else { "no" },
            item.scope_node_id,
            w0 = widths[0],
            w1 = widths[1],
            w2 = widths[2],
            w3 = widths[3]
        ));
    }
    text
}
fn show(db: &str, args: &[String]) -> CliResult {
    let parsed = parse_canvas_show(args)?;
    let c = dial(db)?;
    let p = resolve_project(&c, &parsed.project)?
        .ok_or_else(|| CliError::message("no project found"))?;
    let item = resolve_canvas(&c, &p.id, &parsed.reference)?;
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
    let text = domain::export_format(&g, &item.layout_json, &parsed.format)
        .map_err(|e| CliError::message(e.to_string()))?;
    print!("{}{}", text, if text.ends_with('\n') { "" } else { "\n" });
    Ok(())
}
fn save(db: &str, args: &[String]) -> CliResult {
    let parsed = parse_canvas_save(args)?;
    let source = read_source(&parsed.source)?;
    let c = dial(db)?;
    let p = resolve_project(&c, &parsed.project)?
        .ok_or_else(|| CliError::message("no project found"))?;
    let item = resolve_canvas(&c, &p.id, &parsed.reference)?;
    let change = local_v1::ChangeEnvelope {
        summary: "Save canvas source".to_string(),
        rationale: "Mermaid source written from the CLI".to_string(),
        ..Default::default()
    };
    let response: local_v1::UpdateCanvasResponse = c.unary(
        UPDATE,
        &local_v1::UpdateCanvasRequest {
            project_id: p.id,
            id: item.id,
            source: Some(source),
            change: Some(change),
            ..Default::default()
        },
        Duration::from_secs(10),
    )?;
    let updated = response
        .canvas
        .ok_or_else(|| CliError::message("canvas update returned no canvas"))?;
    println!("{}: source saved", updated.name);
    Ok(())
}

/// Reads the Mermaid source from a file, or from stdin when the path is `-`.
fn read_source(path: &str) -> Result<String, CliError> {
    if path == "-" {
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text)?;
        return Ok(text);
    }
    Ok(std::fs::read_to_string(path)?)
}

fn share(db: &str, args: &[String]) -> CliResult {
    let parsed = parse_canvas_share(args)?;
    let c = dial(db)?;
    let p = resolve_project(&c, &parsed.project)?
        .ok_or_else(|| CliError::message("no project found"))?;
    let item = resolve_canvas(&c, &p.id, &parsed.reference)?;
    let on = parsed.on;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn canvas_usage() {
        assert_eq!(run_canvas("", &[]), Err(CliError::message(USAGE)));
        assert_eq!(
            run_canvas("", &args(&["nope"])),
            Err(CliError::message(USAGE))
        );
        assert!(matches!(
            run_canvas("", &args(&["ls", "--all"])),
            Err(CliError::Message(_))
        ));
    }

    #[test]
    fn parses_canvas_list() {
        let got = parse_canvas_list(&args(&[
            "--agents",
            "--scope",
            "auth, billing",
            "--scope",
            "login",
            "--exact",
            "--project",
            "p1",
        ]))
        .unwrap();
        assert!(got.agents && got.exact);
        assert_eq!(got.project, "p1");
        assert_eq!(got.scope.join("|"), "auth|billing|login");
        assert!(parse_canvas_list(&args(&["extra"])).is_err());
        let plain = parse_canvas_list(&[]).unwrap();
        assert!(!plain.agents && !plain.exact && plain.scope.is_empty());
    }

    #[test]
    fn parses_canvas_show() {
        let got = parse_canvas_show(&args(&[
            "Platform map",
            "--format",
            "dot",
            "--project",
            "p1",
        ]))
        .unwrap();
        assert_eq!(got.reference, "Platform map");
        assert_eq!(got.format, "dot");
        assert_eq!(got.project, "p1");
        let got = parse_canvas_show(&args(&["--format", "Mermaid", "id-1"])).unwrap();
        assert_eq!(got.format, "mermaid");
        assert_eq!(got.reference, "id-1");
        assert_eq!(
            parse_canvas_show(&args(&["id-1"])).unwrap().format,
            "mermaid"
        );
        assert!(parse_canvas_show(&[]).is_err());
        assert!(parse_canvas_show(&args(&["id-1", "--format", "yaml"])).is_err());
    }

    #[test]
    fn parses_canvas_share() {
        let on = parse_canvas_share(&args(&["Platform", "on", "--project", "p1"])).unwrap();
        assert!(on.on);
        assert_eq!(on.reference, "Platform");
        assert_eq!(on.project, "p1");
        let off = parse_canvas_share(&args(&["--project", "p1", "id-1", "off"])).unwrap();
        assert!(!off.on);
        assert_eq!(off.reference, "id-1");
        assert!(parse_canvas_share(&args(&["id-1", "maybe"])).is_err());
        assert!(parse_canvas_share(&args(&["id-1"])).is_err());
    }

    #[test]
    fn format_canvas_list_shows_visible_column() {
        let items = vec![
            local_v1::Canvas {
                id: "c1".into(),
                name: "Map".into(),
                r#type: "canvas".into(),
                agent_visible: true,
                scope_node_id: "n1".into(),
                ..Default::default()
            },
            local_v1::Canvas {
                id: "c2".into(),
                name: "Draft".into(),
                r#type: "canvas".into(),
                ..Default::default()
            },
        ];
        let text = format_canvas_list(&items, false);
        for expected in ["visible", "c1", "yes", "no", "n1"] {
            assert!(
                text.contains(expected),
                "list did not contain {expected:?}:\n{text}"
            );
        }
        let only = select_canvases(items.clone(), true);
        assert_eq!(only.len(), 1);
        assert_eq!(only[0].id, "c1");
        assert_eq!(format_canvas_list(&[], true), "no agent-visible canvases\n");
    }

    #[test]
    fn find_canvas_by_id_or_name() {
        let items = vec![
            local_v1::Canvas {
                id: "id-1".into(),
                name: "Map".into(),
                ..Default::default()
            },
            local_v1::Canvas {
                id: "id-2".into(),
                name: "Map".into(),
                ..Default::default()
            },
            local_v1::Canvas {
                id: "id-3".into(),
                name: "Draft".into(),
                ..Default::default()
            },
        ];
        assert_eq!(find_canvas(&items, "id-2").unwrap().id, "id-2");
        assert_eq!(find_canvas(&items, "Draft").unwrap().id, "id-3");
        assert!(matches!(
            find_canvas(&items, "Map"),
            Err(CliError::Message(message)) if message.contains("matches 2")
        ));
        assert!(matches!(
            find_canvas(&items, "missing"),
            Err(CliError::Message(message)) if message.contains("unknown canvas")
        ));
    }
}
