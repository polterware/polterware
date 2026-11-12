//! The canvas as its Mermaid text, edited by line.
//!
//! The canvas is Mermaid source and the core reads its graph back from that text, so the
//! editor keeps the source as the one truth: [`document`] reads the nodes, the connections,
//! and their places for the inspector, and every edit here rewrites the text in place, keeping
//! the lines, comments, and hand-written parts it does not touch. A node's Polterware part
//! rides in its `@{ ... }` block (`id`, `label`, `kind`, `ref`, `props`, `shape`), exactly as
//! the core writes it.
//!
//! Positions are not part of the text. They are the pins `layout_json` keeps, keyed by the
//! Mermaid id the drawing knows, and [`with_pin`] / [`clear_pins`] write them.

use std::collections::{HashMap, HashSet};

use polterware_mermaid::Diagram;
use polterware_ui::canvas::Viewport;
use serde_json::{Map, Value, json};

use crate::canvas::document::{Doc, Edge, Node, Props};
use crate::contracts::CanvasRef;

use super::mermaid::{blocks, node_ids, viewport};

/// A node's box on screen: left, top, width, and height.
pub type Box4 = (f32, f32, f32, f32);

/// A canvas read from its source, with the places its drawing gave each node.
pub(super) fn document(source: &str, layout_json: &str, boxes: &HashMap<String, Box4>) -> Doc {
    let positions = layout_positions(layout_json);
    let metadata = blocks(source);
    let mut nodes: Vec<Node> = Vec::new();
    let mut ids: HashMap<String, String> = HashMap::new();
    let place = |mermaid: &str| {
        boxes
            .get(mermaid)
            .copied()
            .or_else(|| positions.get(mermaid).copied())
            .unwrap_or((0.0, 0.0, 180.0, 100.0))
    };
    if let Ok(Diagram::Flowchart(ast)) = polterware_mermaid::parse(source) {
        for vertex in &ast.vertices {
            let meta = metadata.get(&vertex.id);
            let canvas = meta
                .and_then(|meta| meta.get("id").cloned())
                .filter(|id| !id.is_empty())
                .unwrap_or_else(|| vertex.id.clone());
            if ids.contains_key(&vertex.id) {
                continue;
            }
            ids.insert(vertex.id.clone(), canvas.clone());
            let label = meta
                .and_then(|meta| meta.get("label").cloned())
                .or_else(|| vertex.text.clone().filter(|text| text != &vertex.id))
                .unwrap_or_default();
            let kind = meta
                .and_then(|meta| meta.get("kind").cloned())
                .unwrap_or_else(|| shape_kind(vertex.type_name.as_deref()));
            let (x, y, width, height) = place(&vertex.id);
            nodes.push(Node {
                id: canvas,
                name: label,
                kind,
                x,
                y,
                width,
                height,
                z: nodes.len() as i32,
                props: metadata_props(meta),
                reference: metadata_ref(meta),
            });
        }
        for graph in &ast.sub_graphs {
            if ids.contains_key(&graph.id) {
                continue;
            }
            let meta = metadata.get(&graph.id);
            let canvas = meta
                .and_then(|meta| meta.get("id").cloned())
                .filter(|id| !id.is_empty())
                .unwrap_or_else(|| graph.id.clone());
            ids.insert(graph.id.clone(), canvas.clone());
            let label = meta
                .and_then(|meta| meta.get("label").cloned())
                .unwrap_or_else(|| graph.title.clone());
            let kind = meta
                .and_then(|meta| meta.get("kind").cloned())
                .unwrap_or_else(|| "frame".to_string());
            let (x, y, width, height) = place(&graph.id);
            nodes.push(Node {
                id: canvas,
                name: label,
                kind,
                x,
                y,
                width,
                height,
                z: nodes.len() as i32,
                props: metadata_props(meta),
                reference: metadata_ref(meta),
            });
        }
        let mut edges: Vec<Edge> = Vec::new();
        for (index, edge) in ast.edges.iter().enumerate() {
            let numbered = format!("l{}", index + 1);
            let written = edge.id.clone().unwrap_or_else(|| numbered.clone());
            let meta = metadata.get(&written);
            // A link with no metadata block takes the id its line position gives it.
            let link = if meta.is_some() { written } else { numbered };
            let id = meta
                .and_then(|meta| meta.get("id").cloned())
                .filter(|id| !id.is_empty())
                .unwrap_or_else(|| link.clone());
            if edges.iter().any(|existing| existing.id == id) {
                continue;
            }
            let (Some(from), Some(to)) = (ids.get(&edge.start), ids.get(&edge.end)) else {
                continue;
            };
            let kind = meta
                .and_then(|meta| meta.get("kind").cloned())
                .filter(|kind| !kind.is_empty())
                .or_else(|| Some(edge.text.clone()).filter(|text| !text.is_empty()))
                .unwrap_or_else(|| "relates".to_string());
            edges.push(Edge {
                id,
                from: from.clone(),
                to: to.clone(),
                kind,
                props: Props::new(),
            });
        }
        return Doc {
            nodes,
            edges,
            viewport: canvas_viewport(layout_json),
        };
    }
    Doc {
        nodes,
        edges: Vec::new(),
        viewport: canvas_viewport(layout_json),
    }
}

/// The diagram id a canvas node id stands for.
pub(super) fn mermaid_id(source: &str, canvas_id: &str) -> Option<String> {
    let ids = node_ids(source);
    ids.iter()
        .find_map(|(mermaid, id)| (id == canvas_id).then(|| mermaid.clone()))
        .or_else(|| node_line(source, canvas_id).map(|_| canvas_id.to_string()))
}

/// The diagram id a connection's canvas id stands for.
pub(super) fn edge_mermaid_id(source: &str, canvas_id: &str) -> Option<String> {
    let named = blocks(source).into_iter().find_map(|(id, meta)| {
        let names_it = meta.get("id").is_some_and(|value| value == canvas_id)
            || (id == canvas_id && !meta.contains_key("id"));
        (names_it && source.contains(&format!(" {id}@-->"))).then_some(id)
    });
    named.or_else(|| {
        let index = canvas_id.strip_prefix('l')?.parse::<usize>().ok()?;
        let edges = source
            .split_inclusive('\n')
            .filter(|line| edge_of(line).is_some())
            .count();
        (index > 0 && index <= edges).then(|| canvas_id.to_string())
    })
}

/// `layout_json` with node `id` pinned at `x` and `y`.
pub(super) fn with_pin(layout_json: &str, id: &str, x: f32, y: f32) -> String {
    let mut layout = object(layout_json);
    let nodes = layout
        .entry("nodes")
        .or_insert_with(|| Value::Object(Map::new()));
    if !nodes.is_object() {
        *nodes = Value::Object(Map::new());
    }
    let Some(nodes) = nodes.as_object_mut() else {
        return layout_json.to_string();
    };
    let mut pin = nodes
        .get(id)
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    pin.insert("x".into(), json!(f64::from(x)));
    pin.insert("y".into(), json!(f64::from(y)));
    pin.insert("pinned".into(), json!(true));
    nodes.insert(id.to_string(), Value::Object(pin));
    layout.insert("version".into(), json!(1));
    Value::Object(layout).to_string()
}

/// `layout_json` with node `id`'s box size, keeping its place and pin.
pub(super) fn with_size(layout_json: &str, id: &str, width: f32, height: f32) -> String {
    let mut layout = object(layout_json);
    let nodes = layout
        .entry("nodes")
        .or_insert_with(|| Value::Object(Map::new()));
    if !nodes.is_object() {
        *nodes = Value::Object(Map::new());
    }
    let Some(nodes) = nodes.as_object_mut() else {
        return layout_json.to_string();
    };
    let mut box_ = nodes
        .get(id)
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    box_.insert("width".into(), json!(f64::from(width)));
    box_.insert("height".into(), json!(f64::from(height)));
    nodes.insert(id.to_string(), Value::Object(box_));
    layout.insert("version".into(), json!(1));
    Value::Object(layout).to_string()
}

/// `layout_json` without the pins, so the automatic layout places every node again.
pub(super) fn clear_pins(layout_json: &str) -> String {
    let mut layout = object(layout_json);
    if let Some(nodes) = layout.get_mut("nodes").and_then(Value::as_object_mut) {
        for position in nodes.values_mut() {
            if let Some(fields) = position.as_object_mut() {
                fields.remove("pinned");
            }
        }
    }
    Value::Object(layout).to_string()
}

/// `layout_json` without node `id`'s entry.
pub(super) fn without_pin(layout_json: &str, id: &str) -> String {
    let mut layout = object(layout_json);
    if let Some(nodes) = layout.get_mut("nodes").and_then(Value::as_object_mut) {
        nodes.remove(id);
    }
    Value::Object(layout).to_string()
}

/// Appends a node of `kind` to the source, returning the text, its Mermaid id, and its canvas id.
pub(super) fn insert_node(
    source: &str,
    kind: &str,
    label: &str,
    reference: Option<&CanvasRef>,
    props: &Props,
) -> (String, String, String) {
    let id = fresh_id(source);
    let canvas_id = format!("{id}-{}", kind.replace(['.', '_'], "-"));
    let mut fields = vec![
        format!("id: {}", text(&canvas_id)),
        format!("label: {}", text(label)),
        format!("kind: {}", scalar(kind)),
    ];
    if let Some(reference) = reference {
        fields.push(format!(
            "ref: {}",
            scalar(&format!("{}/{}", reference.kind, reference.id))
        ));
    }
    if let Some(shape) = mermaid_shape(kind) {
        fields.push(format!("shape: {shape}"));
    }
    if !props.is_empty() {
        fields.push(format!(
            "props: {}",
            text(&Value::Object(props.clone()).to_string())
        ));
    }
    let line = format!("  {id}@{{ {} }}", fields.join(", "));
    (append(source, &[line]), id, canvas_id)
}

/// Sets a node's label, in the bracket the diagram draws and in its metadata.
pub(super) fn set_label(source: &str, id: &str, label: &str) -> String {
    let source = bracket_label(source, id, label);
    set_field(&source, id, "label", Some(&text(label)))
}

/// Rewrites a node's bracket label, which is the text Mermaid draws.
fn bracket_label(source: &str, id: &str, label: &str) -> String {
    let Some((start, end)) = node_line(source, id) else {
        return source.to_string();
    };
    let line = &source[start..end];
    let trimmed = line.len() - line.trim_start().len();
    let head = trimmed + leading_id(line).map_or(0, str::len);
    let after = &line[head..];
    let quoted = text(label);
    let rewritten = match bracket_span(after) {
        Some((open, close)) => {
            let mut rewritten = String::with_capacity(after.len() + quoted.len());
            rewritten.push_str(&after[..open + 1]);
            rewritten.push_str(&quoted);
            rewritten.push_str(&after[close..]);
            rewritten
        }
        None => format!("[{quoted}]{after}"),
    };
    let mut out = String::with_capacity(source.len() + rewritten.len());
    out.push_str(&source[..start]);
    out.push_str(&line[..head]);
    out.push_str(&rewritten);
    out.push_str(&source[end..]);
    out
}

/// The byte range of the `[` and `]` a node's label sits between, when it has one.
fn bracket_span(text: &str) -> Option<(usize, usize)> {
    let open = text.find('[')?;
    let bytes = text.as_bytes();
    let mut quote = false;
    let mut index = open + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' if quote => index += 1,
            b'"' => quote = !quote,
            b']' if !quote => return Some((open, index)),
            _ => {}
        }
        index += 1;
    }
    None
}

/// Sets a node's `style` line, replacing the one it already has.
pub(super) fn set_style_line(source: &str, id: &str, style: &str) -> String {
    let mut out = String::with_capacity(source.len() + style.len() + 16);
    for line in source.split_inclusive('\n') {
        let is_style = line
            .trim_start()
            .strip_prefix("style ")
            .is_some_and(|rest| rest.split_whitespace().next() == Some(id));
        if !is_style {
            out.push_str(line);
        }
    }
    if style.trim().is_empty() {
        return out;
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&format!("  style {id} {style}\n"));
    out
}

/// Sets a node's kind, and the Mermaid shape the drawing uses for it.
pub(super) fn set_kind(source: &str, id: &str, kind: &str) -> String {
    let source = set_field(source, id, "kind", Some(&scalar(kind)));
    match mermaid_shape(kind) {
        Some(shape) => set_field(&source, id, "shape", Some(shape)),
        None => set_field(&source, id, "shape", None),
    }
}

/// Sets or clears a node's project reference.
pub(super) fn set_ref(source: &str, id: &str, reference: Option<&CanvasRef>) -> String {
    match reference {
        Some(reference) => set_field(
            source,
            id,
            "ref",
            Some(&scalar(&format!("{}/{}", reference.kind, reference.id))),
        ),
        None => set_field(source, id, "ref", None),
    }
}

/// Writes a node's Polterware properties.
pub(super) fn set_props(source: &str, id: &str, props: &Props) -> String {
    if props.is_empty() {
        return set_field(source, id, "props", None);
    }
    let json = Value::Object(props.clone()).to_string();
    set_field(source, id, "props", Some(&text(&json)))
}

/// Removes a node's line, and every connection that names it.
pub(super) fn remove_node(source: &str, id: &str) -> String {
    let mut removed: HashSet<String> = HashSet::new();
    for line in source.split_inclusive('\n') {
        if let Some((from, to, link)) = edge_of(line)
            && (from == id || to == id)
        {
            removed.insert(link);
        }
    }
    let mut out = String::with_capacity(source.len());
    for line in source.split_inclusive('\n') {
        if let Some((_, _, link)) = edge_of(line)
            && (removed.contains(&link) || link == id)
        {
            continue;
        }
        if metadata_id(line).is_some_and(|link| removed.contains(&link)) {
            continue;
        }
        if leading_id(line).is_some_and(|leading| leading == id) && !is_edge_line(line) {
            continue;
        }
        out.push_str(line);
    }
    out
}

/// Removes a connection's line and its metadata block.
pub(super) fn remove_edge(source: &str, id: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut index = 0;
    for line in source.split_inclusive('\n') {
        if let Some((_, _, link)) = edge_of(line) {
            index += 1;
            if edge_id(link, index) == id {
                continue;
            }
        }
        if metadata_id(line).is_some_and(|link| link == id) {
            continue;
        }
        out.push_str(line);
    }
    out
}

/// Appends a connection from one node to another, returning the text and the edge's canvas id.
pub(super) fn connect(source: &str, from: &str, to: &str, kind: &str, edge_id: &str) -> String {
    let link = fresh_edge_id(source);
    let kind = kind.trim();
    let labelled = !kind.is_empty() && kind != "relates";
    let line = if labelled {
        format!("  {from} {link}@-->|{kind}| {to}")
    } else {
        format!("  {from} {link}@--> {to}")
    };
    let mut fields = vec![format!("id: {}", text(edge_id))];
    if !labelled && !kind.is_empty() {
        fields.push(format!("kind: {}", scalar(kind)));
    }
    let metadata = format!("  {link}@{{ {} }}", fields.join(", "));
    append(source, &[line, metadata])
}

/// Sets a connection's kind, in the link's text.
pub(super) fn set_edge_kind(source: &str, id: &str, kind: &str) -> String {
    let kind = kind.trim();
    let mut out = String::with_capacity(source.len());
    let mut index = 0;
    for line in source.split_inclusive('\n') {
        let Some((_, _, link)) = edge_of(line) else {
            out.push_str(line);
            continue;
        };
        index += 1;
        if edge_id(link, index) != id {
            out.push_str(line);
            continue;
        }
        let Some((left, operator, right)) = split_edge(line) else {
            out.push_str(line);
            continue;
        };
        let rest = match right.strip_prefix('|') {
            Some(rest) => rest.split_once('|').map_or(right, |(_, rest)| rest),
            None => right,
        };
        let text = if kind.is_empty() || kind == "relates" {
            String::new()
        } else {
            format!("|{kind}|")
        };
        let newline = line.ends_with('\n');
        let mut rewritten = format!("{left}{operator}{text}{rest}");
        if newline {
            rewritten.push('\n');
        }
        out.push_str(&rewritten);
    }
    set_field(&out, id, "kind", None)
}

/// A connection line cut at its operator: the text before it, the operator, and the text after.
fn split_edge(line: &str) -> Option<(&str, &str, &str)> {
    ["-->", "---", "-.->", "==>", "--x", "--o"]
        .iter()
        .find_map(|operator| {
            line.split_once(operator)
                .map(|(left, right)| (left, *operator, right))
        })
}

/// Moves a node's line to the front or the back of the diagram, which is its stacking order.
pub(super) fn move_node_line(source: &str, id: &str, front: bool) -> String {
    let Some((start, end)) = node_line(source, id) else {
        return source.to_string();
    };
    let line = &source[start..end];
    let mut rest = String::with_capacity(source.len());
    rest.push_str(&source[..start]);
    rest.push_str(&source[end..]);
    let lines: Vec<&str> = rest.split_inclusive('\n').collect();
    let at = if front {
        lines
            .iter()
            .position(|line| {
                let line = line.trim_start();
                line.starts_with("flowchart") || line.starts_with("graph")
            })
            .map_or(0, |index| index + 1)
    } else {
        lines
            .iter()
            .rposition(|line| !line.trim().is_empty())
            .map_or(lines.len(), |index| index + 1)
    };
    let mut out = String::with_capacity(rest.len() + line.len() + 1);
    let mut placed = false;
    for (index, item) in lines.iter().enumerate() {
        if index == at {
            out.push_str(line);
            if !line.ends_with('\n') {
                out.push('\n');
            }
            placed = true;
        }
        out.push_str(item);
    }
    if !placed {
        out.push_str(line);
        if !line.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

// Reading the lines.

/// The byte range of the line that defines node `id`, without its newline.
fn node_line(source: &str, id: &str) -> Option<(usize, usize)> {
    let mut start = 0;
    for line in source.split_inclusive('\n') {
        let end = start + line.trim_end_matches('\n').len();
        if !is_edge_line(line)
            && !is_keyword_line(line)
            && leading_id(line).is_some_and(|leading| leading == id)
        {
            return Some((start, end));
        }
        start += line.len();
    }
    None
}

/// Sets `key` in the node's `@{ ... }` block, adding the block when the line has none.
fn set_field(source: &str, id: &str, key: &str, value: Option<&str>) -> String {
    let Some((start, end)) = node_line(source, id) else {
        if let Some(value) = value {
            return append(source, &[format!("  {id}@{{ {key}: {value} }}")]);
        }
        return source.to_string();
    };
    let line = &source[start..end];
    let rewritten = match block_range(line) {
        Some((open, close)) => {
            let inner = rewrite_block(&line[open + 1..close], key, value);
            let mut rewritten = String::with_capacity(line.len() + inner.len());
            rewritten.push_str(&line[..=open]);
            rewritten.push(' ');
            rewritten.push_str(&inner);
            if !inner.is_empty() {
                rewritten.push(' ');
            }
            rewritten.push('}');
            rewritten.push_str(&line[close + 1..]);
            rewritten
        }
        None => {
            let Some(value) = value else {
                return source.to_string();
            };
            let trimmed = line.trim_end();
            format!("{trimmed}@{{ {key}: {value} }}{}", &line[trimmed.len()..])
        }
    };
    let mut out = String::with_capacity(source.len());
    out.push_str(&source[..start]);
    out.push_str(&rewritten);
    out.push_str(&source[end..]);
    out
}

/// A `@{ ... }` block rewritten with `key` set to `value`, or without it when `None`.
fn rewrite_block(inner: &str, key: &str, value: Option<&str>) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut found = false;
    for part in split_fields(inner) {
        let Some((name, _)) = part.split_once(':') else {
            let part = part.trim();
            if !part.is_empty() {
                parts.push(part.to_string());
            }
            continue;
        };
        if name.trim() != key {
            parts.push(part.trim().to_string());
            continue;
        }
        found = true;
        if let Some(value) = value {
            parts.push(format!("{key}: {value}"));
        }
    }
    if !found && let Some(value) = value {
        parts.push(format!("{key}: {value}"));
    }
    parts.join(", ")
}

/// The byte range of the `{` and `}` of the line's `@{ ... }` block.
fn block_range(line: &str) -> Option<(usize, usize)> {
    let at = line.find("@{")?;
    let open = at + 1;
    let bytes = line.as_bytes();
    let mut depth = 0usize;
    let mut quote: Option<u8> = None;
    let mut index = open;
    while index < bytes.len() {
        let byte = bytes[index];
        match quote {
            Some(open_quote) => {
                if byte == b'\\' && open_quote == b'"' {
                    index += 2;
                    continue;
                }
                if byte == open_quote {
                    quote = None;
                }
            }
            None => match byte {
                b'"' | b'\'' => quote = Some(byte),
                b'{' => depth += 1,
                b'}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Some((open, index));
                    }
                }
                _ => {}
            },
        }
        index += 1;
    }
    None
}

/// The parts of a metadata block between its top-level commas.
fn split_fields(inner: &str) -> Vec<&str> {
    let bytes = inner.as_bytes();
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut quote: Option<u8> = None;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        match quote {
            Some(open) => {
                if byte == b'\\' && open == b'"' {
                    index += 2;
                    continue;
                }
                if byte == open {
                    quote = None;
                }
            }
            None => match byte {
                b'"' | b'\'' => quote = Some(byte),
                b'{' | b'[' => depth += 1,
                b'}' | b']' => depth = depth.saturating_sub(1),
                b',' if depth == 0 => {
                    parts.push(&inner[start..index]);
                    start = index + 1;
                }
                _ => {}
            },
        }
        index += 1;
    }
    parts.push(&inner[start..]);
    parts
}

/// The id a connection line's link carries, or the one a link without an id takes.
fn edge_id(link: String, index: usize) -> String {
    if link.is_empty() {
        format!("l{index}")
    } else {
        link
    }
}

/// The id a line starts with.
fn leading_id(line: &str) -> Option<&str> {
    let line = line.trim_start();
    let end = line.bytes().take_while(|byte| is_id_byte(*byte)).count();
    (end > 0).then(|| &line[..end])
}

/// The id in front of a line's `@{ ... }` block.
fn metadata_id(line: &str) -> Option<String> {
    let at = line.find("@{")?;
    leading_id(&line[..at]).map(str::to_string)
}

fn is_edge_line(line: &str) -> bool {
    ["-->", "---", "-.->", "==>", "--x", "--o"]
        .iter()
        .any(|operator| line.contains(operator))
}

fn is_keyword_line(line: &str) -> bool {
    let line = line.trim_start();
    [
        "end",
        "subgraph",
        "class",
        "classDef",
        "style",
        "linkStyle",
        "click",
        "flowchart",
        "graph",
        "%%",
        "direction",
    ]
    .iter()
    .any(|keyword| {
        line == *keyword
            || line.starts_with(&format!("{keyword} "))
            || line.starts_with(&format!("{keyword}\t"))
    })
}

/// A connection line's ends and its link id.
fn edge_of(line: &str) -> Option<(String, String, String)> {
    if !is_edge_line(line) {
        return None;
    }
    let (left, _, right) = split_edge(line)?;
    let from = leading_id(left)?.to_string();
    let link = left
        .split_whitespace()
        .find(|token| token.contains('@'))
        .map(|token| token.trim_end_matches('@').to_string())
        .unwrap_or_default();
    let right = right.trim_start();
    let right = match right.strip_prefix('|') {
        Some(rest) => rest
            .split_once('|')
            .map_or(right, |(_, rest)| rest.trim_start()),
        None => right,
    };
    let to = leading_id(right)?.to_string();
    Some((from, to, link))
}

fn is_id_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')
}

fn object(layout_json: &str) -> Map<String, Value> {
    serde_json::from_str::<Value>(layout_json)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default()
}

/// Each node's box the layout remembers.
fn layout_positions(layout_json: &str) -> HashMap<String, Box4> {
    let layout = object(layout_json);
    let Some(nodes) = layout.get("nodes").and_then(Value::as_object) else {
        return HashMap::new();
    };
    nodes
        .iter()
        .filter_map(|(id, value)| {
            let fields = value.as_object()?;
            let number = |key: &str, fallback: f32| {
                fields
                    .get(key)
                    .and_then(Value::as_f64)
                    .map(|value| value as f32)
                    .unwrap_or(fallback)
            };
            Some((
                id.clone(),
                (
                    number("x", 0.0),
                    number("y", 0.0),
                    number("width", 180.0),
                    number("height", 100.0),
                ),
            ))
        })
        .collect()
}

/// The pan and zoom the layout keeps, as the document's viewport.
fn canvas_viewport(layout_json: &str) -> Viewport {
    match viewport(layout_json) {
        Some(view) => Viewport {
            x: view.x,
            y: view.y,
            zoom: view.zoom,
        },
        None => Viewport {
            x: 40.0,
            y: 40.0,
            zoom: 1.0,
        },
    }
}

fn metadata_props(meta: Option<&HashMap<String, String>>) -> Props {
    meta.and_then(|meta| meta.get("props"))
        .and_then(|props| serde_json::from_str::<Value>(props).ok())
        .and_then(|props| props.as_object().cloned())
        .unwrap_or_default()
}

fn metadata_ref(meta: Option<&HashMap<String, String>>) -> Option<CanvasRef> {
    let (kind, id) = meta?.get("ref")?.split_once('/')?;
    (!kind.trim().is_empty() && !id.trim().is_empty()).then(|| CanvasRef {
        kind: kind.trim().to_string(),
        id: id.trim().to_string(),
    })
}

/// The Mermaid id a new node takes, unique in the text.
fn fresh_id(source: &str) -> String {
    let taken = taken_ids(source);
    let mut index = taken.len() + 1;
    loop {
        let id = format!("n{index}");
        if !taken.contains(&id) {
            return id;
        }
        index += 1;
    }
}

/// The link id a new connection takes, unique in the text.
fn fresh_edge_id(source: &str) -> String {
    let mut index = 1;
    loop {
        let id = format!("l{index}");
        if !source.contains(&format!("{id}@")) {
            return id;
        }
        index += 1;
    }
}

/// Every id a line of the text starts with.
fn taken_ids(source: &str) -> HashSet<String> {
    source
        .split_inclusive('\n')
        .filter(|line| !is_keyword_line(line))
        .filter_map(leading_id)
        .map(str::to_string)
        .collect()
}

fn append(source: &str, lines: &[String]) -> String {
    let mut out = source.trim_end().to_string();
    if out.is_empty() {
        out.push_str("flowchart LR");
    }
    for line in lines {
        out.push('\n');
        out.push_str(line);
    }
    out.push('\n');
    out
}

/// A metadata value as a JSON string, which is also a YAML double-quoted scalar.
fn text(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

/// A metadata scalar written plainly when YAML reads it back unchanged.
fn scalar(value: &str) -> String {
    let plain = !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b'/'))
        && !matches!(
            value,
            "true" | "false" | "null" | "yes" | "no" | "on" | "off"
        )
        && value.parse::<f64>().is_err();
    if plain {
        value.to_string()
    } else {
        text(value)
    }
}

/// The Mermaid shape a canvas kind draws as, as the core writes it.
pub(super) fn mermaid_shape(kind: &str) -> Option<&'static str> {
    super::mermaid::shape_for_kind(kind)
}

/// The canvas kind a shape drawn without metadata reads as, as the core reads it.
fn shape_kind(type_name: Option<&str>) -> String {
    match type_name {
        Some("square" | "rect") => "rectangle",
        Some("round" | "stadium") => "rounded",
        Some("diamond") => "diamond",
        Some("hexagon") => "hexagon",
        Some("circle" | "doublecircle") => "circle",
        Some("ellipse") => "ellipse",
        Some("cylinder") => "cylinder",
        Some("trapezoid" | "lean_right" | "lean_left" | "inv_trapezoid") => "trapezoid",
        Some("subroutine") => "process",
        Some("odd") => "parallelogram",
        Some("note") => "note",
        _ => "rectangle",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "flowchart LR\n  %% kept by hand\n  a[\"Orders\"]@{ id: \"01HORDERS\", kind: service }\n  b@{ label: \"Web\", kind: component }\n  a l1@-->|reads| b\n  l1@{ id: \"e1\" }\n";

    #[test]
    fn the_document_reads_nodes_and_connections() {
        let doc = document(SOURCE, "{}", &HashMap::new());
        assert_eq!(doc.nodes.len(), 2);
        assert_eq!(doc.nodes[0].id, "01HORDERS");
        assert_eq!(doc.nodes[0].kind, "service");
        assert_eq!(doc.nodes[0].name, "Orders");
        assert_eq!(doc.nodes[1].name, "Web");
        assert_eq!(doc.edges.len(), 1);
        assert_eq!(doc.edges[0].id, "e1");
        assert_eq!(doc.edges[0].kind, "reads");
        assert_eq!(doc.edges[0].from, "01HORDERS");
    }

    #[test]
    fn a_document_id_follows_its_metadata() {
        assert_eq!(mermaid_id(SOURCE, "01HORDERS").as_deref(), Some("a"));
        assert_eq!(edge_mermaid_id(SOURCE, "e1").as_deref(), Some("l1"));
        assert_eq!(mermaid_id(SOURCE, "b").as_deref(), Some("b"));
    }

    #[test]
    fn inserting_a_node_appends_one_line_and_keeps_the_rest() {
        let (source, id, canvas) = insert_node(SOURCE, "rectangle", "New box", None, &Props::new());
        assert!(source.contains("%% kept by hand"));
        let line = source
            .lines()
            .find(|line| line.starts_with(&format!("  {id}@{{")))
            .expect("the new node line");
        assert!(line.contains("label: \"New box\""));
        assert!(line.contains("kind: rectangle"));
        assert!(line.contains("shape: rect"));
        let doc = document(&source, "{}", &HashMap::new());
        assert!(doc.nodes.iter().any(|node| node.name == "New box"));
        assert!(doc.node(&canvas).is_some());
    }

    #[test]
    fn a_label_edit_keeps_the_other_fields() {
        let source = set_label(SOURCE, "a", "Checkout");
        assert!(source.contains("%% kept by hand"));
        assert!(source.contains("label: \"Checkout\""));
        assert!(source.contains("id: \"01HORDERS\""));
        assert!(source.contains("kind: service"));
        let doc = document(&source, "{}", &HashMap::new());
        assert_eq!(doc.node("01HORDERS").expect("the node").name, "Checkout");
    }

    #[test]
    fn a_kind_edit_writes_the_shape_too() {
        let source = set_kind(SOURCE, "b", "cylinder");
        assert!(source.contains("kind: cylinder"));
        assert!(source.contains("shape: cyl"));
        let doc = document(&source, "{}", &HashMap::new());
        assert_eq!(doc.node("b").expect("the node").kind, "cylinder");
        let source = set_kind(&source, "b", "database");
        assert!(source.contains("shape: cyl"));
        let source = set_kind(&source, "b", "component");
        assert!(source.contains("shape: rounded"));
        assert!(source.contains("kind: component"));
    }

    #[test]
    fn a_label_edit_rewrites_the_bracket_the_diagram_draws() {
        let source = set_label(SOURCE, "a", "Checkout");
        assert!(source.contains("a[\"Checkout\"]@{"), "{source}");
        let doc = document(&source, "{}", &HashMap::new());
        assert_eq!(doc.node("01HORDERS").expect("the node").name, "Checkout");
        let source = set_label(SOURCE, "b", "Store");
        assert!(source.contains("b[\"Store\"]@{"), "{source}");
    }

    #[test]
    fn a_style_line_replaces_the_one_the_node_had() {
        let source = "flowchart LR\n  a[\"A\"]@{ kind: database }\n  style a fill:#000000\n";
        let styled = set_style_line(
            source,
            "a",
            "fill:#ff0000,stroke:#00ff00,stroke-width:2px,stroke-dasharray:4 4",
        );
        assert_eq!(styled.matches("style a ").count(), 1, "{styled}");
        assert!(!styled.contains("fill:#000000"));
        assert!(styled.contains("stroke-dasharray:4 4"));
        let cleared = set_style_line(&styled, "a", "");
        assert!(!cleared.contains("style a "));
    }

    #[test]
    fn a_reference_is_written_and_cleared() {
        let reference = CanvasRef {
            kind: "node".into(),
            id: "01HWEB".into(),
        };
        let source = set_ref(SOURCE, "b", Some(&reference));
        assert!(source.contains("ref: node/01HWEB"));
        let doc = document(&source, "{}", &HashMap::new());
        assert_eq!(
            doc.node("b").expect("the node").reference,
            Some(reference.clone())
        );
        let source = set_ref(&source, "b", None);
        assert!(!source.contains("ref:"));
    }

    #[test]
    fn deleting_a_node_takes_its_connections_and_their_blocks() {
        let source = remove_node(SOURCE, "a");
        assert!(!source.contains("01HORDERS"));
        assert!(!source.contains("l1@"));
        assert!(!source.contains("-->"));
        assert!(source.contains("%% kept by hand"));
        assert!(source.contains("b@{"));
        let doc = document(&source, "{}", &HashMap::new());
        assert_eq!(doc.nodes.len(), 1);
        assert!(doc.edges.is_empty());
    }

    #[test]
    fn deleting_an_edge_keeps_its_nodes() {
        let source = remove_edge(SOURCE, "l1");
        assert!(!source.contains("-->"));
        assert!(!source.contains("l1@"));
        assert!(source.contains("01HORDERS"));
        assert!(source.contains("Web"));
    }

    #[test]
    fn connecting_two_nodes_appends_a_labelled_link() {
        let source = connect(SOURCE, "b", "a", "depends_on", "e2");
        let doc = document(&source, "{}", &HashMap::new());
        let edge = doc
            .edges
            .iter()
            .find(|edge| edge.id == "e2")
            .expect("the new edge");
        assert_eq!(edge.kind, "depends_on");
        assert_eq!((edge.from.as_str(), edge.to.as_str()), ("b", "01HORDERS"));
        assert!(source.contains("%% kept by hand"));
    }

    #[test]
    fn an_edge_kind_edit_rewrites_the_link_text() {
        let source = set_edge_kind(SOURCE, "l1", "calls");
        assert!(source.contains("|calls|"));
        assert!(!source.contains("|reads|"));
        let doc = document(&source, "{}", &HashMap::new());
        assert_eq!(doc.edges[0].kind, "calls");
    }

    #[test]
    fn a_hand_written_edge_takes_its_position_id() {
        let source = "flowchart LR\n  a[One]\n  b[Two]\n  a --> b\n";
        let doc = document(source, "{}", &HashMap::new());
        assert_eq!(doc.edges[0].id, "l1");
        assert_eq!(edge_mermaid_id(source, "l1").as_deref(), Some("l1"));
        let edited = set_edge_kind(source, "l1", "calls");
        assert!(edited.contains("-->|calls|"));
        assert!(document(&edited, "{}", &HashMap::new()).edges[0].kind == "calls");
        let removed = remove_edge(source, "l1");
        assert!(!removed.contains("-->"));
        assert!(removed.contains("a[One]"));
    }

    #[test]
    fn a_node_line_moves_to_the_front_or_the_back() {
        let front = move_node_line(SOURCE, "b", true);
        let body: Vec<&str> = front
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with("flowchart"))
            .collect();
        assert!(body[0].contains("label: \"Web\""));
        let back = move_node_line(SOURCE, "a", false);
        let body: Vec<&str> = back
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with("flowchart"))
            .collect();
        assert!(body.last().is_some_and(|line| line.contains("01HORDERS")));
    }

    #[test]
    fn pins_round_trip_and_reset() {
        let layout =
            r#"{"viewport":{"x":1,"y":2,"zoom":1},"nodes":{"a":{"x":5,"y":6,"pinned":true}}}"#;
        let pinned = with_pin(layout, "b", 10.0, 20.0);
        let value: Value = serde_json::from_str(&pinned).expect("json");
        assert_eq!(value["nodes"]["b"]["pinned"], json!(true));
        assert_eq!(value["nodes"]["a"]["pinned"], json!(true));
        let cleared = clear_pins(&pinned);
        let value: Value = serde_json::from_str(&cleared).expect("json");
        assert_eq!(value["nodes"]["a"].get("pinned"), None);
        assert_eq!(value["viewport"]["zoom"], json!(1));
    }

    #[test]
    fn a_pinned_place_is_the_documents_place() {
        let layout = r#"{"nodes":{"a":{"x":40,"y":30,"pinned":true}}}"#;
        let doc = document(SOURCE, layout, &HashMap::new());
        let node = doc.node("01HORDERS").expect("the node");
        assert_eq!((node.x, node.y), (40.0, 30.0));
    }
}
