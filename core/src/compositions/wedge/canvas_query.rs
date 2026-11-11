#[allow(unused_imports)] // The package-wide glob; every name this file uses is explicit.
use super::*;

use std::collections::HashMap;

use crate::domains::canvas;
use crate::domains::polter;
use crate::domains::structure;
use crate::providers::sqlite as storepkg;

/// Lists agent-visible canvases, or returns one of them as a Mermaid flowchart.
/// A canvas with `agent_visible` false is omitted from the list and answers as
/// unknown, including when the caller names its id.
/// Called by the work.go query router, ported separately; unexported in Go, so
/// public here only for the package's glob re-export.
pub fn query_canvas(
    store: &storepkg::Store,
    project_id: &str,
    reference: &str,
    scope: &structure::MultiScope,
) -> anyhow::Result<String> {
    let items = store.list_canvases(project_id)?;
    let resolved = store.load_scopes(project_id, scope)?;
    let visible: Vec<canvas::Canvas> = items
        .into_iter()
        .filter(|item| item.agent_visible && resolved.matches(&item.scope_node_id))
        .collect();
    let nodes = store.structure_nodes(project_id)?;
    let reference = reference.trim();
    if reference.is_empty() {
        return Ok(format_agent_canvases(&visible, &nodes));
    }
    let item = find_agent_canvas(&visible, reference)?;
    Ok(canvas::export_mermaid(
        &canvas::with_structure(&item.graph, &nodes),
        &item.layout_json,
    ))
}

fn format_agent_canvases(items: &[canvas::Canvas], nodes: &[structure::Node]) -> String {
    if items.is_empty() {
        return "no canvases\n".to_string();
    }
    let mut paths: HashMap<&str, &str> = HashMap::with_capacity(nodes.len());
    for node in nodes {
        paths.insert(node.id.as_str(), node.path.as_str());
    }
    let mut out = String::from("id  scope  name\n");
    for item in items {
        let mut scope = "-";
        if !item.scope_node_id.is_empty() {
            scope = paths
                .get(item.scope_node_id.as_str())
                .copied()
                .unwrap_or(item.scope_node_id.as_str());
        }
        out.push_str(&format!("{}  {}  {}\n", item.id, scope, item.name));
    }
    out
}

fn find_agent_canvas(items: &[canvas::Canvas], reference: &str) -> anyhow::Result<canvas::Canvas> {
    let mut by_id: Option<&canvas::Canvas> = None;
    let mut by_name: Vec<&canvas::Canvas> = Vec::new();
    for item in items {
        if item.id == reference {
            by_id = Some(item);
        }
        if item.name == reference {
            by_name.push(item);
        }
    }
    if let Some(item) = by_id {
        return Ok(item.clone());
    }
    match by_name.len() {
        1 => Ok(by_name[0].clone()),
        0 => Err(anyhow::anyhow!(
            "unknown canvas {}: list canvases with polterware query canvas",
            polter::go_quote(reference)
        )),
        count => Err(anyhow::anyhow!(
            "canvas name {} matches {} canvases: use an id",
            polter::go_quote(reference),
            count
        )),
    }
}
