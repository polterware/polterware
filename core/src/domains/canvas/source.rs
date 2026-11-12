//! A canvas read back from its Mermaid source.
//!
//! The canvas is Mermaid text. [`crate::domains::canvas::export_mermaid`] writes
//! it and this module reads it: the flowchart's vertices and links become the
//! semantic graph, and the Polterware metadata rides in Mermaid's `@{ ... }`
//! node and link blocks (`id`, `shape`, `label`, `kind`, `ref`, `props`) so the
//! text stays valid Mermaid. A flowchart written by hand without that metadata
//! still becomes a canvas: its shapes pick the node kind, a link's label is its
//! edge kind, and a subgraph is a frame.
//!
//! Layout is not part of the source. Pins stay in `layout_json`; a node with no
//! pin takes the automatic layout.

use std::collections::HashMap;

use serde_json::{Map, Value};

use super::{EDGE_RELATES, Edge, Error, Graph, KIND_NOTE, Node, Ref, Result, valid_ref_kind};

/// The `@{ ... }` metadata blocks of the source, by the id before them.
type Blocks = HashMap<String, Map<String, Value>>;

/// Reads a canvas's semantic graph from its Mermaid source. The text must be a
/// flowchart; a parse failure carries the line and column the parser stopped at.
pub fn graph_from_source(source: &str) -> Result<Graph> {
    let diagram = polterware_mermaid::parse(source).map_err(source_error)?;
    let polterware_mermaid::Diagram::Flowchart(ast) = diagram else {
        return Err(Error::Source {
            line: 1,
            column: 1,
            message: "a canvas is a flowchart: start it with \"flowchart LR\"".to_string(),
        });
    };
    let blocks = metadata_blocks(source);

    // Every vertex keeps the id the text gives it unless its metadata renames it.
    let mut node_ids: HashMap<String, String> = HashMap::new();
    let mut nodes: Vec<Node> = Vec::new();
    for vertex in &ast.vertices {
        let metadata = blocks.get(&vertex.id);
        let id = metadata_string(metadata, "id").unwrap_or_else(|| vertex.id.clone());
        if node_ids.contains_key(&vertex.id) {
            continue;
        }
        node_ids.insert(vertex.id.clone(), id.clone());
        let label = metadata_string(metadata, "label").unwrap_or_else(|| {
            vertex
                .text
                .clone()
                .filter(|text| text != &vertex.id)
                .unwrap_or_default()
        });
        let kind = metadata_string(metadata, "kind")
            .unwrap_or_else(|| shape_kind(vertex.type_name.as_deref()));
        nodes.push(Node {
            id,
            kind,
            label,
            props: metadata_props(metadata),
            r#ref: metadata_ref(metadata),
        });
    }
    // A subgraph is a frame; its metadata carries the Polterware node when the
    // export wrote one.
    for sub_graph in &ast.sub_graphs {
        if node_ids.contains_key(&sub_graph.id) {
            continue;
        }
        let metadata = blocks.get(&sub_graph.id);
        let id = metadata_string(metadata, "id").unwrap_or_else(|| sub_graph.id.clone());
        node_ids.insert(sub_graph.id.clone(), id.clone());
        let label = metadata_string(metadata, "label").unwrap_or_else(|| sub_graph.title.clone());
        let kind = metadata_string(metadata, "kind").unwrap_or_else(|| "frame".to_string());
        nodes.push(Node {
            id,
            kind,
            label,
            props: metadata_props(metadata),
            r#ref: metadata_ref(metadata),
        });
    }

    let mut edges: Vec<Edge> = Vec::new();
    for (index, edge) in ast.edges.iter().enumerate() {
        let link = edge
            .id
            .clone()
            .unwrap_or_else(|| format!("l{}", index + 1));
        let metadata = blocks.get(&link);
        let id = metadata_string(metadata, "id").unwrap_or_else(|| link.clone());
        if edges.iter().any(|existing| existing.id == id) {
            continue;
        }
        let Some(from) = node_ids.get(&edge.start) else {
            continue;
        };
        let Some(to) = node_ids.get(&edge.end) else {
            continue;
        };
        let kind = metadata_string(metadata, "kind")
            .filter(|kind| !kind.is_empty())
            .or_else(|| Some(edge.text.clone()).filter(|text| !text.is_empty()))
            .unwrap_or_else(|| EDGE_RELATES.to_string());
        edges.push(Edge {
            id,
            from: from.clone(),
            to: to.clone(),
            kind,
            props: Map::new(),
        });
    }
    let graph = Graph { nodes, edges };
    Ok(graph)
}

/// Marks every position in the layout as pinned, so a canvas carried over to
/// the source keeps the places people gave its nodes.
pub fn pin_layout(layout_json: &str) -> String {
    let Ok(mut layout) = serde_json::from_str::<Value>(layout_json) else {
        return layout_json.to_string();
    };
    if let Value::Object(fields) = &mut layout
        && let Some(Value::Object(nodes)) = fields.get_mut("nodes")
    {
        for position in nodes.values_mut() {
            match position {
                Value::Object(map) => {
                    map.insert("pinned".to_string(), Value::Bool(true));
                }
                Value::Null => {
                    *position = serde_json::json!({"pinned": true});
                }
                _ => {}
            }
        }
    }
    serde_json::to_string(&layout).unwrap_or_else(|_| layout_json.to_string())
}

fn source_error(err: polterware_mermaid::ParseError) -> Error {
    let (line, column) = err
        .span
        .map(|span| (span.line, span.column))
        .unwrap_or((1, 1));
    Error::Source {
        line,
        column,
        message: err.message,
    }
}

fn metadata_string(metadata: Option<&Map<String, Value>>, key: &str) -> Option<String> {
    match metadata?.get(key) {
        Some(Value::String(text)) => Some(text.clone()),
        Some(Value::Number(number)) => Some(number.to_string()),
        _ => None,
    }
}

fn metadata_ref(metadata: Option<&Map<String, Value>>) -> Option<Ref> {
    let raw = metadata_string(metadata, "ref")?;
    let (kind, id) = raw.split_once('/')?;
    let kind = kind.trim();
    let id = id.trim();
    if !valid_ref_kind(kind) || id.is_empty() {
        return None;
    }
    Some(Ref {
        kind: kind.to_string(),
        id: id.to_string(),
    })
}

fn metadata_props(metadata: Option<&Map<String, Value>>) -> Map<String, Value> {
    let Some(text) = metadata_string(metadata, "props") else {
        return Map::new();
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(fields)) => fields,
        _ => Map::new(),
    }
}

/// The canvas kind a shape drawn without metadata reads as.
fn shape_kind(type_name: Option<&str>) -> String {
    let kind = match type_name {
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
        Some("note") => KIND_NOTE,
        _ => "rectangle",
    };
    kind.to_string()
}

/// Reads every `@{ ... }` block in the source, keyed by the id in front of it.
/// The parser has already validated the text; a block this reader cannot split
/// is skipped rather than failing the write.
fn metadata_blocks(source: &str) -> Blocks {
    let bytes = source.as_bytes();
    let mut blocks: Blocks = HashMap::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'{' || index == 0 || bytes[index - 1] != b'@' {
            index += 1;
            continue;
        }
        let Some(close) = matching_brace(source, index) else {
            break;
        };
        let id = id_before(source, index - 1);
        if !id.is_empty() {
            blocks.insert(id, split_metadata(&source[index + 1..close]));
        }
        index = close + 1;
    }
    blocks
}

/// The node or link id in front of a `@{ ... }` block: the last id on the line
/// once quoted text, bracket labels, and the block itself are ignored.
fn id_before(source: &str, at: usize) -> String {
    let line_start = source[..at].rfind('\n').map_or(0, |index| index + 1);
    let head = &source[line_start..at];
    let bytes = head.as_bytes();
    let mut kept = String::with_capacity(head.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            quote @ (b'"' | b'\'') => {
                index = skip_quoted(bytes, index, quote);
                kept.push(' ');
            }
            open @ (b'[' | b'(' | b'{') => {
                index = skip_group(bytes, index, open);
                kept.push(' ');
            }
            byte if byte.is_ascii() => {
                kept.push(byte as char);
                index += 1;
            }
            _ => {
                kept.push(' ');
                index += 1;
            }
        }
    }
    let kept = kept.trim_end();
    let start = kept.len() - kept.bytes().rev().take_while(|byte| is_id_byte(*byte)).count();
    kept[start..].to_string()
}

fn skip_quoted(bytes: &[u8], start: usize, quote: u8) -> usize {
    let mut index = start + 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' && quote == b'"' {
            index += 2;
            continue;
        }
        if bytes[index] == quote {
            return index + 1;
        }
        index += 1;
    }
    bytes.len()
}

fn skip_group(bytes: &[u8], start: usize, open: u8) -> usize {
    let close = match open {
        b'[' => b']',
        b'(' => b')',
        _ => b'}',
    };
    let mut depth = 0usize;
    let mut index = start;
    while index < bytes.len() {
        match bytes[index] {
            byte if byte == open => depth += 1,
            byte if byte == close => {
                depth -= 1;
                if depth == 0 {
                    return index + 1;
                }
            }
            b'"' => {
                index = skip_quoted(bytes, index, b'"');
                continue;
            }
            _ => {}
        }
        index += 1;
    }
    bytes.len()
}

fn is_id_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')
}

/// The index of the `}` that closes the block opened at `start`, skipping
/// quoted text.
fn matching_brace(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut quote: Option<u8> = None;
    let mut index = start;
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
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(index);
                    }
                }
                _ => {}
            },
        }
        index += 1;
    }
    None
}

/// Splits a `@{ ... }` body into its keys. Values are kept as text (a quoted
/// value is unescaped) because the reader only needs the keys Polterware writes.
fn split_metadata(text: &str) -> Map<String, Value> {
    let mut fields = Map::new();
    for part in split_top_level(text) {
        let Some((key, value)) = split_key_value(part) else {
            continue;
        };
        let key = key.trim().to_string();
        if key.is_empty() {
            continue;
        }
        fields.insert(key, metadata_value(value.trim()));
    }
    fields
}

fn split_top_level(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
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
                    parts.push(&text[start..index]);
                    start = index + 1;
                }
                _ => {}
            },
        }
        index += 1;
    }
    if start < text.len() {
        parts.push(&text[start..]);
    }
    parts
}

fn split_key_value(part: &str) -> Option<(&str, &str)> {
    let bytes = part.as_bytes();
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
                b':' => return Some((&part[..index], &part[index + 1..])),
                _ => {}
            },
        }
        index += 1;
    }
    None
}

fn metadata_value(raw: &str) -> Value {
    let unquoted = match raw.strip_prefix('"').and_then(|text| text.strip_suffix('"')) {
        Some(inner) => unescape(inner),
        None => match raw.strip_prefix('\'').and_then(|text| text.strip_suffix('\'')) {
            Some(inner) => inner.replace("''", "'"),
            None => raw.to_string(),
        },
    };
    Value::String(unquoted)
}

fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('b') => out.push('\u{0008}'),
            Some('f') => out.push('\u{000c}'),
            Some('0') => out.push('\0'),
            Some('u') => {
                let hex: String = chars.by_ref().take(4).collect();
                match u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                    Some(character) => out.push(character),
                    None => out.push_str(&hex),
                }
            }
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

/// The Mermaid shape a canvas kind draws as, when the shape registry names it.
/// A kind without one keeps its metadata only.
pub fn mermaid_shape(kind: &str) -> Option<&'static str> {
    match kind {
        "rectangle" => Some("rect"),
        "rounded" => Some("rounded"),
        "circle" => Some("circle"),
        "diamond" => Some("diamond"),
        "triangle" => Some("triangle"),
        "hexagon" => Some("hexagon"),
        "trapezoid" => Some("trapezoid"),
        "cylinder" => Some("cylinder"),
        "cloud" => Some("cloud"),
        "document" => Some("document"),
        "process" => Some("process"),
        "manual-input" => Some("manual-input"),
        "delay" => Some("delay"),
        "display" => Some("display"),
        _ => None,
    }
}

/// Whether a canvas node id can be a Mermaid id as written.
pub fn is_mermaid_id(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('-')
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
        && !is_reserved(id)
}

fn is_reserved(id: &str) -> bool {
    matches!(
        id,
        "end"
            | "subgraph"
            | "graph"
            | "flowchart"
            | "class"
            | "classdef"
            | "click"
            | "style"
            | "linkstyle"
            | "default"
    )
}

#[cfg(test)]
mod tests {
    use crate::domains::canvas::{KIND_NOTE, REF_NODE};

    use super::*;

    #[test]
    fn reads_metadata_keys_and_values() {
        let blocks = metadata_blocks(
            "flowchart LR\n  n1@{ shape: rect, label: \"Say \\\"hi\\\"\", kind: service, ref: node/n1, props: \"{\\\"technology\\\":\\\"Go\\\"}\" }\n",
        );
        let block = blocks.get("n1").expect("n1 metadata");
        assert_eq!(metadata_string(Some(block), "label"), Some("Say \"hi\"".to_string()));
        assert_eq!(metadata_string(Some(block), "kind"), Some("service".to_string()));
        assert_eq!(
            metadata_ref(Some(block)),
            Some(Ref {
                kind: REF_NODE.to_string(),
                id: "n1".to_string()
            })
        );
        assert_eq!(
            metadata_props(Some(block)).get("technology"),
            Some(&Value::String("Go".to_string()))
        );
    }

    #[test]
    fn a_label_with_a_brace_or_comma_does_not_split_the_block() {
        let blocks = metadata_blocks("flowchart LR\n  n1@{ label: \"a, b { c }\", kind: note }\n");
        let block = blocks.get("n1").expect("n1 metadata");
        assert_eq!(metadata_string(Some(block), "label"), Some("a, b { c }".to_string()));
        assert_eq!(metadata_string(Some(block), "kind"), Some(KIND_NOTE.to_string()));
    }

    fn sorted(mut graph: Graph) -> Graph {
        graph.nodes.sort_by(|a, b| a.id.cmp(&b.id));
        graph.edges.sort_by(|a, b| a.id.cmp(&b.id));
        for node in &mut graph.nodes {
            node.props = node
                .props
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();
        }
        graph
    }

    fn sample() -> Graph {
        Graph {
            nodes: vec![
                Node {
                    id: "01HORDERS".to_string(),
                    kind: "service".to_string(),
                    label: "Orders".to_string(),
                    props: serde_json::from_str(r#"{"technology":"Go"}"#).expect("props"),
                    r#ref: Some(Ref {
                        kind: REF_NODE.to_string(),
                        id: "01HORDER".to_string(),
                    }),
                },
                Node {
                    id: "web".to_string(),
                    kind: "component".to_string(),
                    label: "Web".to_string(),
                    props: Map::new(),
                    r#ref: Some(Ref {
                        kind: REF_NODE.to_string(),
                        id: "01HWEB".to_string(),
                    }),
                },
                Node {
                    id: "frame-1".to_string(),
                    kind: "frame".to_string(),
                    label: "Backend".to_string(),
                    props: Map::new(),
                    r#ref: None,
                },
                Node {
                    id: "ask".to_string(),
                    kind: KIND_NOTE.to_string(),
                    label: "ask design".to_string(),
                    props: Map::new(),
                    r#ref: None,
                },
            ],
            edges: vec![
                super::super::Edge {
                    id: "e1".to_string(),
                    from: "01HORDERS".to_string(),
                    to: "web".to_string(),
                    kind: super::super::EDGE_READS.to_string(),
                    props: Map::new(),
                },
                super::super::Edge {
                    id: "e2".to_string(),
                    from: "frame-1".to_string(),
                    to: "01HORDERS".to_string(),
                    kind: EDGE_RELATES.to_string(),
                    props: Map::new(),
                },
            ],
        }
    }

    #[test]
    fn a_canvas_reads_back_from_its_source() {
        let graph = sample();
        let layout = r#"{"nodes":{"01HORDERS":{"x":10,"y":20},"web":{"x":260,"y":20},"frame-1":{"x":0,"y":0,"width":600,"height":400}},"viewport":{"x":1,"y":2,"zoom":1}}"#;
        let pinned = pin_layout(layout);
        let source = super::super::export_mermaid(&graph, &pinned);
        let back = graph_from_source(&source).unwrap_or_else(|err| panic!("parse: {err}\n{source}"));
        assert_eq!(sorted(back), sorted(graph.clone()), "source:\n{source}");
        let layout: Value = serde_json::from_str(&pinned).expect("pinned layout");
        for (id, _) in graph.nodes.iter().map(|node| (node.id.as_str(), node)) {
            if id == "ask" {
                continue;
            }
            assert_eq!(
                layout["nodes"][id]["pinned"],
                Value::Bool(true),
                "position of {id} is not pinned"
            );
        }
    }

    #[test]
    fn a_parse_failure_carries_the_line_and_column() {
        let err = graph_from_source("flowchart LR\n  A@{ shape: nonsense }\n").expect_err("bad shape");
        assert_eq!(err.to_string(), "line 2, column 4: No such shape: nonsense.");
        let err = graph_from_source("stateDiagram-v2\n  A --> B\n").expect_err("not a flowchart");
        assert_eq!(
            err.to_string(),
            "line 1, column 1: a canvas is a flowchart: start it with \"flowchart LR\""
        );
    }

    #[test]
    fn hand_written_mermaid_becomes_a_canvas() {
        let graph = graph_from_source(
            "flowchart LR\n  api[Orders]\n  db[(Postgres)]\n  api -->|reads| db\n  api --> db\n  subgraph lanes[Backend]\n    api\n  end\n",
        )
        .expect("parse");
        let kind = |id: &str| {
            graph
                .nodes
                .iter()
                .find(|node| node.id == id)
                .map(|node| node.kind.clone())
        };
        assert_eq!(kind("api"), Some("rectangle".to_string()));
        assert_eq!(kind("db"), Some("cylinder".to_string()));
        assert_eq!(kind("lanes"), Some("frame".to_string()));
        let kinds: Vec<&str> = graph.edges.iter().map(|edge| edge.kind.as_str()).collect();
        assert_eq!(kinds, vec!["reads", EDGE_RELATES]);
        assert_eq!(graph.edges[0].from, "api");
        assert_eq!(graph.edges[0].to, "db");
    }

    #[test]
    fn pinning_keeps_every_position_and_the_viewport() {        let pinned = pin_layout(r#"{"nodes":{"a":{"x":1,"y":2}},"viewport":{"x":4,"y":5,"zoom":2}}"#);
        let value: Value = serde_json::from_str(&pinned).expect("layout");
        assert_eq!(value["nodes"]["a"]["pinned"], Value::Bool(true));
        assert_eq!(value["nodes"]["a"]["x"], serde_json::json!(1));
        assert_eq!(value["viewport"]["zoom"], serde_json::json!(2));
        assert_eq!(pin_layout("not json"), "not json");
    }
}
