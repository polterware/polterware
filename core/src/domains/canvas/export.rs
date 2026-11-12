//! A canvas as a Mermaid flowchart for agents.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::model::{go_json_escape, normalize, normalize_numbers_in, null_as_default};
use super::{
    COMPONENT_KINDS, Error, Graph, Node, REF_NODE, Ref, Result, STRUCTURE_KINDS,
};
use crate::domains::structure;

/// Marks a node that cites a store entity.
pub const TRUST_REFERENCED: &str = "referenced";
/// Marks a node with no entity ref.
pub const TRUST_UNVERIFIED: &str = "unverified";

/// One exported node: the graph node plus its trust.
#[derive(Debug, Serialize)]
struct ExportNode {
    id: String,
    kind: String,
    label: String,
    props: Map<String, Value>,
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    r#ref: Option<Ref>,
    trust: &'static str,
}

#[derive(Debug, Serialize)]
struct ExportLink {
    id: String,
    source: String,
    target: String,
    kind: String,
    props: Map<String, Value>,
}

#[derive(Debug, Serialize)]
struct ExportDoc {
    directed: bool,
    nodes: Vec<ExportNode>,
    links: Vec<ExportLink>,
}

/// Gives each node that references a Structure node that node's title and
/// technology, and a semantic component its kind, the way the Desktop draws it.
/// Refs to missing nodes and other entities are left as written.
pub fn with_structure(graph: &Graph, nodes: &[structure::Node]) -> Graph {
    let by_id: HashMap<&str, &structure::Node> =
        nodes.iter().map(|node| (node.id.as_str(), node)).collect();
    let mut out = graph.clone();
    for (i, node) in graph.nodes.iter().enumerate() {
        let Some(linked) = node
            .r#ref
            .as_ref()
            .filter(|reference| reference.kind == REF_NODE)
            .and_then(|reference| by_id.get(reference.id.as_str()).copied())
        else {
            continue;
        };
        if linked.status == structure::NODE_REMOVED {
            continue;
        }
        let next = &mut out.nodes[i];
        next.label = linked.title.clone();
        if !linked.tech.is_empty() {
            let mut props = node.props.clone();
            props.insert("technology".to_string(), Value::String(linked.tech.clone()));
            next.props = props;
        }
        // A component with several kinds draws as its first.
        let kind = linked.kind.split(',').next().unwrap_or("");
        if !kind.is_empty() && COMPONENT_KINDS.contains(&node.kind.as_str()) {
            next.kind = kind.trim().to_string();
        }
    }
    out
}

/// Writes the graph as node-link JSON (nodes + links). Layout and the visual
/// "style" prop are omitted. A node with a ref is "referenced"; a node without
/// one is "unverified".
pub fn export(graph: &Graph) -> Result<Vec<u8>> {
    let mut graph = graph.clone();
    normalize(&mut graph);
    let doc = ExportDoc {
        directed: true,
        nodes: graph
            .nodes
            .iter()
            .map(|node| ExportNode {
                id: node.id.clone(),
                kind: node.kind.clone(),
                label: node.label.clone(),
                props: without_style(&node.props),
                r#ref: node.r#ref.clone(),
                trust: trust_of(node),
            })
            .collect(),
        links: graph
            .edges
            .iter()
            .map(|edge| ExportLink {
                id: edge.id.clone(),
                source: edge.from.clone(),
                target: edge.to.clone(),
                kind: edge.kind.clone(),
                props: without_style(&edge.props),
            })
            .collect(),
    };
    Ok(go_json_escape(&serde_json::to_string(&doc)?).into_bytes())
}

/// Renders the graph as mermaid (with the layout's containment), json, or dot.
/// Positions themselves are never part of the output.
pub fn export_format(graph: &Graph, layout_json: &str, format: &str) -> Result<String> {
    match format.trim().to_lowercase().as_str() {
        "" | "mermaid" => Ok(export_mermaid(graph, layout_json)),
        "json" => String::from_utf8(export(graph)?).map_err(|err| Error::Json(err.to_string())),
        "dot" => Ok(export_dot(graph)),
        _ => Err(Error::UnknownFormat(format.to_string())),
    }
}

/// Writes the canvas as a left-to-right flowchart: the canvas source. Every
/// node keeps its id, kind, label, ref, and props in a Mermaid `@{ ... }` block,
/// and every edge keeps its id and kind there too, so
/// [`graph_from_source`](super::graph_from_source) reads the same graph back. A
/// node drawn inside another (its center within the other's rectangle in the
/// layout) sits in that node's subgraph, so frames become subgraphs. The
/// closing class line lists the nodes with no ref, which are unverified.
pub fn export_mermaid(graph: &Graph, layout_json: &str) -> String {
    let mut graph = graph.clone();
    normalize(&mut graph);
    let Graph { nodes, edges } = graph;
    let rects = layout_rects(layout_json);
    let mut taken: HashSet<String> = HashSet::new();
    let mut ids: HashMap<String, String> = HashMap::new();
    for (index, node) in nodes.iter().enumerate() {
        let id = if super::is_mermaid_id(&node.id) && !taken.contains(&node.id) {
            node.id.clone()
        } else {
            let mut next = index + 1;
            while taken.contains(&format!("u{next}")) {
                next += 1;
            }
            format!("u{next}")
        };
        taken.insert(id.clone());
        ids.insert(node.id.clone(), id);
    }
    let parent = containers(&nodes, &rects);
    let mut children: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, node) in nodes.iter().enumerate() {
        let home = parent.get(&node.id).cloned().unwrap_or_default();
        children.entry(home).or_default().push(index);
    }

    let mut body = String::from("flowchart LR\n");
    write_tree(&mut body, &nodes, &ids, &children, "", 1);
    for (index, edge) in edges.iter().enumerate() {
        let (Some(from), Some(to)) = (ids.get(&edge.from), ids.get(&edge.to)) else {
            continue;
        };
        let link = if super::is_mermaid_id(&edge.id) && !taken.contains(&edge.id) {
            edge.id.clone()
        } else {
            let mut next = index + 1;
            while taken.contains(&format!("l{next}")) {
                next += 1;
            }
            format!("l{next}")
        };
        taken.insert(link.clone());
        let labelled = !edge.kind.is_empty()
            && edge.kind != super::EDGE_RELATES
            && mermaid_text(&edge.kind) == edge.kind;
        if labelled {
            body.push_str(&format!("  {from} {link}@-->|{}| {to}\n", edge.kind));
        } else {
            body.push_str(&format!("  {from} {link}@--> {to}\n"));
        }
        let mut fields: Vec<String> = Vec::new();
        // A link with no label already reads as relates.
        if !labelled && !edge.kind.is_empty() && edge.kind != super::EDGE_RELATES {
            fields.push(format!("kind: {}", metadata_text(&edge.kind)));
        }
        if edge.id != link {
            fields.push(format!("id: {}", metadata_text(&edge.id)));
        }
        if !fields.is_empty() {
            body.push_str(&format!("  {link}@{{ {} }}\n", fields.join(", ")));
        }
    }
    let mut unverified: Vec<String> = Vec::new();
    for node in &nodes {
        if node.r#ref.is_some() || !is_semantic(&node.kind) {
            continue;
        }
        if let Some(id) = ids.get(&node.id) {
            unverified.push(id.clone());
        }
    }
    if !unverified.is_empty() {
        body.push_str(&format!(
            "  class {} {}\n",
            unverified.join(","),
            TRUST_UNVERIFIED
        ));
    }
    body
}

fn write_tree(
    body: &mut String,
    nodes: &[Node],
    ids: &HashMap<String, String>,
    children: &HashMap<String, Vec<usize>>,
    container: &str,
    depth: usize,
) {
    let indent = "  ".repeat(depth);
    let Some(kids) = children.get(container) else {
        return;
    };
    for index in kids {
        let node = &nodes[*index];
        let Some(id) = ids.get(&node.id) else {
            continue;
        };
        if children.get(&node.id).is_none_or(|kids| kids.is_empty()) {
            let bracket = match bracket_label(node, id) {
                Some(label) => format!("[\"{label}\"]"),
                None => String::new(),
            };
            body.push_str(&format!(
                "{indent}{id}{bracket}{}\n",
                node_metadata(node, id, !bracket.is_empty())
            ));
            continue;
        }
        // A frame is a node and a subgraph: its properties sit on a node line
        // before the block, because Mermaid reads a subgraph's own `@{ }` block
        // only after it exists, and the block must stay readable on its own.
        body.push_str(&format!(
            "{indent}{id}{}\n",
            node_metadata(node, id, false)
        ));
        let title = bracket_label(node, id)
            .unwrap_or_else(|| mermaid_text(&label_or_kind(node)));
        body.push_str(&format!("{indent}subgraph {id}[\"{title}\"]\n"));
        write_tree(body, nodes, ids, children, &node.id, depth + 1);
        body.push_str(&format!("{indent}end\n"));
    }
}

/// The node's label as bracket text, when the text reaches the reader unchanged
/// and is not just the node id (which Mermaid substitutes for a missing label).
fn bracket_label(node: &Node, mermaid_id: &str) -> Option<String> {
    let label = label(node);
    if label.is_empty() || label == mermaid_id || mermaid_text(&label) != label {
        return None;
    }
    Some(label)
}

/// The Polterware part of a node: everything the text does not say by itself.
/// `bracketed` is true when the line carries the label in brackets.
fn node_metadata(node: &Node, mermaid_id: &str, bracketed: bool) -> String {
    let mut fields: Vec<String> = Vec::new();
    if node.id != mermaid_id {
        fields.push(format!("id: {}", metadata_text(&node.id)));
    }
    if !bracketed && !node.label.is_empty() {
        fields.push(format!("label: {}", metadata_text(&node.label)));
    }
    if !node.kind.is_empty() {
        fields.push(format!("kind: {}", metadata_scalar(&node.kind)));
    }
    if let Some(reference) = &node.r#ref {
        fields.push(format!(
            "ref: {}",
            metadata_scalar(&format!("{}/{}", reference.kind, reference.id))
        ));
    }
    let shape = super::mermaid_shape(&node.kind).filter(|shape| *shape != "rect");
    if let Some(shape) = shape {
        fields.push(format!("shape: {shape}"));
    }
    let props = without_style(&node.props);
    if !props.is_empty() {
        let json = serde_json::to_string(&Value::Object(props)).unwrap_or_default();
        fields.push(format!("props: {}", metadata_text(&json)));
    }
    if fields.is_empty() {
        return String::new();
    }
    format!("@{{ {} }}", fields.join(", "))
}

/// A metadata scalar: a JSON string, which is also a YAML double-quoted scalar.
fn metadata_text(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string())
}

/// A metadata scalar written plainly when YAML reads it back unchanged.
fn metadata_scalar(text: &str) -> String {
    let plain = !text.is_empty()
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b'/'))
        && !matches!(text, "true" | "false" | "null" | "yes" | "no" | "on" | "off")
        && text.parse::<f64>().is_err();
    if plain {
        text.to_string()
    } else {
        metadata_text(text)
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Rect {
    fn area(&self) -> f64 {
        self.w * self.h
    }

    fn holds(&self, x: f64, y: f64) -> bool {
        x > self.x && x < self.x + self.w && y > self.y && y < self.y + self.h
    }
}

#[derive(Debug, Default, Deserialize)]
struct Layout {
    #[serde(default, deserialize_with = "null_as_default")]
    nodes: HashMap<String, Option<LayoutPosition>>,
}

#[derive(Debug, Default, Deserialize)]
struct LayoutPosition {
    #[serde(default, alias = "X")]
    x: f64,
    #[serde(default, alias = "Y")]
    y: f64,
    #[serde(default, alias = "Width")]
    width: f64,
    #[serde(default, alias = "Height")]
    height: f64,
}

/// Reads node rectangles from the Desktop layout, with the editor's default
/// size when one is missing.
fn layout_rects(layout_json: &str) -> HashMap<String, Rect> {
    let Ok(layout) = serde_json::from_str::<Layout>(layout_json) else {
        return HashMap::new();
    };
    let mut out: HashMap<String, Rect> = HashMap::new();
    for (id, position) in layout.nodes {
        let position = position.unwrap_or_default();
        let mut w = position.width;
        let mut h = position.height;
        if w <= 0.0 {
            w = 180.0;
        }
        if h <= 0.0 {
            h = 100.0;
        }
        out.insert(
            id,
            Rect {
                x: position.x,
                y: position.y,
                w,
                h,
            },
        );
    }
    out
}


/// Maps each node to the smallest larger node whose rectangle holds its center.
/// Nodes without a layout position sit at the top level.
fn containers(nodes: &[Node], rects: &HashMap<String, Rect>) -> HashMap<String, String> {
    let mut out: HashMap<String, String> = HashMap::new();
    for node in nodes {
        if let Some(rect) = rects.get(&node.id) {
            let inner = *rect;
            let id = node.id.as_str();
            let best = smallest_containing(nodes, rects, inner, &|candidate: &str| {
                let outer_area = rects.get(candidate).map_or(0.0, Rect::area);
                candidate != id && outer_area > inner.area()
            });
            out.insert(node.id.clone(), best);
        }
    }
    out
}

fn smallest_containing(
    nodes: &[Node],
    rects: &HashMap<String, Rect>,
    inner: Rect,
    allowed: &dyn Fn(&str) -> bool,
) -> String {
    let mut best = String::new();
    let mut best_area = 0.0;
    let (cx, cy) = (inner.x + inner.w / 2.0, inner.y + inner.h / 2.0);
    for node in nodes {
        let Some(outer) = rects.get(&node.id) else {
            continue;
        };
        if !allowed(&node.id) || !outer.holds(cx, cy) {
            continue;
        }
        if best.is_empty() || outer.area() < best_area {
            best = node.id.clone();
            best_area = outer.area();
        }
    }
    best
}


/// What the person wrote; the editor's "New <kind>" placeholder counts as no
/// label.
fn label(node: &Node) -> String {
    let text = node.label.trim();
    if text == format!("New {}", node.kind) {
        return String::new();
    }
    text.to_string()
}

fn label_or_kind(node: &Node) -> String {
    let text = label(node);
    if text.is_empty() {
        node.kind.clone()
    } else {
        text
    }
}

fn is_semantic(kind: &str) -> bool {
    COMPONENT_KINDS.contains(&kind) || STRUCTURE_KINDS.contains(&kind)
}



/// Writes a Graphviz digraph. Edge kinds are link labels, and each node carries
/// a trust attribute. Layout is omitted.
pub fn export_dot(graph: &Graph) -> String {
    let mut graph = graph.clone();
    normalize(&mut graph);
    let mut body = String::from("digraph canvas {\n  rankdir=LR;\n");
    for node in &graph.nodes {
        body.push_str(&format!(
            "  \"{}\" [label=\"{}\" kind=\"{}\" trust=\"{}\"];\n",
            dot_escape(&node.id),
            dot_escape(&node.label),
            dot_escape(&node.kind),
            trust_of(node)
        ));
    }
    for edge in &graph.edges {
        body.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
            dot_escape(&edge.from),
            dot_escape(&edge.to),
            dot_escape(&edge.kind)
        ));
    }
    body.push_str("}\n");
    body
}

/// Drops the presentation-only "style" prop the Desktop editor stores; it is
/// not part of the graph's meaning.
fn without_style(props: &Map<String, Value>) -> Map<String, Value> {
    let props = normalize_numbers_in(props);
    if !props.contains_key("style") {
        return props;
    }
    props
        .into_iter()
        .filter(|(key, _)| key != "style")
        .collect()
}

fn trust_of(node: &Node) -> &'static str {
    if node.r#ref.is_some() {
        TRUST_REFERENCED
    } else {
        TRUST_UNVERIFIED
    }
}

fn mermaid_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for r in text.chars() {
        match r {
            '"' => out.push_str("#quot;"),
            '#' => out.push_str("#35;"),
            '\\' => out.push_str("#92;"),
            '|' => out.push_str("#124;"),
            '\n' | '\r' => out.push(' '),
            _ => out.push(r),
        }
    }
    out
}

fn dot_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for r in text.chars() {
        match r {
            '\\' | '"' => {
                out.push('\\');
                out.push(r);
            }
            '\n' | '\r' => out.push_str("\\n"),
            _ => out.push(r),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value};

    use super::*;
    use crate::domains::canvas::{
        EDGE_READS, EDGE_RELATES, EDGE_USES, Edge, KIND_COMPONENT, KIND_DOMAIN, KIND_NOTE, Ref,
        marshal_graph, unmarshal_graph,
    };
    use crate::domains::structure::{NODE_ACTIVE, Node as StructureNode};

    fn props(pairs: &[(&str, Value)]) -> Map<String, Value> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), value.clone()))
            .collect()
    }

    fn text(value: &str) -> Value {
        Value::String(value.to_string())
    }

    fn node(id: &str, kind: &str, label: &str) -> Node {
        Node {
            id: id.to_string(),
            kind: kind.to_string(),
            label: label.to_string(),
            ..Node::default()
        }
    }

    fn edge(id: &str, from: &str, to: &str, kind: &str) -> Edge {
        Edge {
            id: id.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            kind: kind.to_string(),
            ..Edge::default()
        }
    }

    fn structure_ref(id: &str) -> Option<Ref> {
        Some(Ref {
            kind: REF_NODE.to_string(),
            id: id.to_string(),
        })
    }

    #[test]
    fn test_export_node_link_shape() {
        let body = export(&Graph {
            nodes: vec![
                Node {
                    props: props(&[("tier", text("edge"))]),
                    r#ref: structure_ref("structure-1"),
                    ..node("s", KIND_DOMAIN, "API")
                },
                node("n", KIND_NOTE, "later"),
            ],
            edges: vec![Edge {
                props: props(&[("sync", Value::Bool(true))]),
                ..edge("e", "s", "n", EDGE_USES)
            }],
        })
        .expect("export");
        let body_text = String::from_utf8(body.clone()).expect("utf-8 JSON");
        assert!(
            !body_text.contains("layout")
                && !body_text.contains("\"from\"")
                && !body_text.contains("\"edges\""),
            "export mixed layout or storage shape into node-link JSON: {body_text}"
        );
        let doc: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(doc["directed"], Value::Bool(true));
        assert!(doc.get("layout").is_none(), "layout leaked into export");
        let nodes = doc["nodes"].as_array().expect("nodes");
        let links = doc["links"].as_array().expect("links");
        assert_eq!((nodes.len(), links.len()), (2, 1), "body = {body_text}");
        let first = &nodes[0];
        assert_eq!(first["id"], "s");
        assert_eq!(first["kind"], KIND_DOMAIN);
        assert_eq!(first["label"], "API");
        assert_eq!(first["props"]["tier"], "edge");
        assert_eq!(first["ref"]["kind"], REF_NODE);
        assert_eq!(first["ref"]["id"], "structure-1");
        assert_eq!(first["trust"], TRUST_REFERENCED);
        let second = &nodes[1];
        assert!(second.get("ref").is_none(), "note ref = {}", second["ref"]);
        assert_eq!(second["trust"], TRUST_UNVERIFIED);
        let link = &links[0];
        assert_eq!(link["id"], "e");
        assert_eq!(link["source"], "s");
        assert_eq!(link["target"], "n");
        assert_eq!(link["kind"], EDGE_USES);
        assert_eq!(link["props"]["sync"], Value::Bool(true));
    }

    #[test]
    fn test_export_omits_style() {
        let body = export(&Graph {
            nodes: vec![Node {
                props: props(&[
                    ("style", Value::Object(props(&[("fill", text("blue"))]))),
                    ("technology", text("Postgres")),
                ]),
                ..node("r", "database", "")
            }],
            edges: vec![],
        })
        .expect("export");
        let body_text = String::from_utf8(body).expect("utf-8 JSON");
        assert!(
            !body_text.contains("style") && body_text.contains("Postgres"),
            "export = {body_text}"
        );
    }

    #[test]
    fn test_export_empty_graph() {
        let body = export(&Graph::default()).expect("export");
        let body_text = String::from_utf8(body.clone()).expect("utf-8 JSON");
        assert_eq!(
            body_text, r#"{"directed":true,"nodes":[],"links":[]}"#,
            "empty export = {body_text}"
        );
        let doc: Value = serde_json::from_slice(&body).expect("json");
        let nodes = doc["nodes"].as_array().expect("nodes");
        let links = doc["links"].as_array().expect("links");
        assert_eq!(
            (nodes.len(), links.len()),
            (0, 0),
            "empty export = {body_text}"
        );
    }

    #[test]
    fn export_escapes_html_the_way_go_json_does() {
        let body = export(&Graph {
            nodes: vec![node("a", KIND_NOTE, "a<b&c")],
            edges: vec![],
        })
        .expect("export");
        let body_text = String::from_utf8(body).expect("utf-8 JSON");
        assert!(
            body_text.contains(r#""label":"a\u003cb\u0026c""#),
            "export = {body_text}"
        );
    }

    #[test]
    fn test_export_trust_clears_empty_ref() {
        let body = export(&Graph {
            nodes: vec![
                Node {
                    r#ref: Some(Ref::default()),
                    ..node("a", KIND_NOTE, "n")
                },
                Node {
                    r#ref: structure_ref("f1"),
                    ..node("b", KIND_NOTE, "y")
                },
            ],
            edges: vec![],
        })
        .expect("export");
        let doc: Value = serde_json::from_slice(&body).expect("json");
        let nodes = doc["nodes"].as_array().expect("nodes");
        assert_eq!(nodes[0]["trust"], TRUST_UNVERIFIED);
        assert!(nodes[0].get("ref").is_none(), "empty ref = {}", nodes[0]);
        assert_eq!(nodes[1]["trust"], TRUST_REFERENCED);
        assert_eq!(nodes[1]["ref"]["id"], "f1");
    }

    fn mermaid_graph() -> Graph {
        Graph {
            nodes: vec![
                node("frame", "frame", "Backend"),
                Node {
                    props: props(&[
                        ("technology", text("Go")),
                        ("style", Value::Object(props(&[("fill", text("red"))]))),
                    ]),
                    r#ref: structure_ref("c"),
                    ..node("api", "service", "API")
                },
                node("inner", "frame", "Workers"),
                node("job", "worker", "Mailer"),
                node("db", "database", "Postgres"),
                node("note-frame", KIND_NOTE, "Writes are\nidempotent"),
                node("note-db", KIND_NOTE, "Backups nightly"),
                node("note-linked", KIND_NOTE, "Hot path"),
                node("blank", "rectangle", ""),
                node("placeholder", "ellipse", "New ellipse"),
                node("decision", "diamond", ""),
                node("loose", "cache", "Redis"),
            ],
            edges: vec![
                edge("e1", "api", "db", EDGE_READS),
                edge("e2", "job", "decision", EDGE_RELATES),
                edge("e3", "note-linked", "api", EDGE_RELATES),
                edge("e4", "api", "inner", "uses|sync"),
            ],
        }
    }

    #[test]
    fn test_export_mermaid_nests_by_layout_and_keeps_every_node() {
        let graph = mermaid_graph();
        let layout = r#"{"nodes":{"frame":{"x":0,"y":0,"width":1000,"height":600},"api":{"x":40,"y":40},"inner":{"x":400,"y":200,"width":500,"height":300},"job":{"x":450,"y":250},"note-frame":{"x":40,"y":400,"width":200,"height":100},"db":{"x":1200,"y":40},"note-db":{"x":1210,"y":50,"width":100,"height":40},"blank":{"x":1500,"y":0},"placeholder":{"x":1500,"y":300},"decision":{"x":700,"y":250,"width":80,"height":80}}}"#;
        let want = concat!(
            "flowchart LR\n",
            "  frame@{ label: \"Backend\", kind: frame }\n",
            "  subgraph frame[\"Backend\"]\n",
            "    api[\"API\"]@{ kind: service, ref: node/c, props: \"{\\\"technology\\\":\\\"Go\\\"}\" }\n",
            "    inner@{ label: \"Workers\", kind: frame }\n",
            "    subgraph inner[\"Workers\"]\n",
            "      job[\"Mailer\"]@{ kind: worker }\n",
            "      decision@{ kind: diamond, shape: diamond }\n",
            "    end\n",
            "    note-frame@{ label: \"Writes are\\nidempotent\", kind: note }\n",
            "  end\n",
            "  db@{ label: \"Postgres\", kind: database }\n",
            "  subgraph db[\"Postgres\"]\n",
            "    note-db[\"Backups nightly\"]@{ kind: note }\n",
            "  end\n",
            "  note-linked[\"Hot path\"]@{ kind: note }\n",
            "  blank@{ kind: rectangle }\n",
            "  placeholder@{ label: \"New ellipse\", kind: ellipse }\n",
            "  loose[\"Redis\"]@{ kind: cache }\n",
            "  api e1@-->|reads| db\n",
            "  job e2@--> decision\n",
            "  note-linked e3@--> api\n",
            "  api e4@--> inner\n",
            "  e4@{ kind: \"uses|sync\" }\n",
            "  class job,db,loose unverified\n",
        );
        let got = export_mermaid(&graph, layout);
        assert_eq!(got, want, "mermaid");
        assert!(
            !got.contains("fill") && !got.contains("1200"),
            "style or layout leaked into the export"
        );
        assert!(
            got.contains("blank@{ kind: rectangle }"),
            "a shape with no label and no links is still a node:\n{got}"
        );
        let flat = export_mermaid(&graph, "not json");
        assert!(
            !flat.contains("subgraph") && flat.contains("  frame@{"),
            "without a layout every node is top level:\n{flat}"
        );
    }

    #[test]
    fn test_export_format_defaults_to_mermaid() {
        let graph = Graph {
            nodes: vec![
                Node {
                    r#ref: structure_ref("structure-1"),
                    ..node("s", KIND_DOMAIN, "API")
                },
                node("n", KIND_NOTE, "later"),
            ],
            edges: vec![edge("e", "s", "n", EDGE_USES)],
        };
        let text = export_format(&graph, "", "").expect("default");
        assert_eq!(
            text,
            "flowchart LR\n  s[\"API\"]@{ kind: domain, ref: node/structure-1 }\n  n[\"later\"]@{ kind: note }\n  s e@-->|uses| n\n"
        );
        let dot = export_format(&graph, "", "dot").expect("dot");
        let want_dot = "digraph canvas {\n  rankdir=LR;\n  \"s\" [label=\"API\" kind=\"domain\" trust=\"referenced\"];\n  \"n\" [label=\"later\" kind=\"note\" trust=\"unverified\"];\n  \"s\" -> \"n\" [label=\"uses\"];\n}\n";
        assert_eq!(dot, want_dot, "dot");
        assert!(
            export_format(&graph, "", "yaml").is_err(),
            "expected unknown format"
        );
        let json = export_format(&graph, r#"{"nodes":{}}"#, "json").expect("json");
        assert!(
            json.contains("\"trust\":\"referenced\"") && !json.contains("nodes\":{}"),
            "json format = {json}"
        );
    }

    #[test]
    fn test_export_mermaid_escapes_labels() {
        let text = export_mermaid(
            &Graph {
                nodes: vec![node("a-b", "service", "say \"hi\" #1")],
                edges: vec![],
            },
            "",
        );
        assert!(
            text.contains(r#"a-b@{ label: "say \"hi\" #1", kind: service }"#),
            "escape:\n{text}"
        );
        let dot = export_dot(&Graph {
            nodes: vec![node("a\"b", KIND_NOTE, "line\n2")],
            edges: vec![],
        });
        assert!(
            dot.contains("label=\"line\\n2\"")
                && dot.contains("a\\\"b")
                && dot.contains("trust=\"unverified\""),
            "dot escape =\n{dot}"
        );
    }

    #[test]
    fn test_marshal_graph_keeps_storage_shape() {
        let raw = marshal_graph(&Graph {
            nodes: vec![node("a", "idea", "One")],
            edges: vec![edge("e", "a", "a", EDGE_RELATES)],
        })
        .expect("marshal");
        assert!(
            !raw.contains("\"source\"") && !raw.contains("layout"),
            "stored graph = {raw}"
        );
        let graph = unmarshal_graph(&raw).expect("round trip");
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(
            (graph.edges[0].from.as_str(), graph.edges[0].to.as_str()),
            ("a", "a")
        );
    }

    #[test]
    fn test_with_structure_lets_the_linked_node_win() {
        let graph = Graph {
            nodes: vec![
                Node {
                    props: props(&[("technology", text("Node.js"))]),
                    r#ref: structure_ref("db"),
                    ..node("a", "service", "Old")
                },
                Node {
                    r#ref: structure_ref("db"),
                    ..node("b", KIND_COMPONENT, "Box")
                },
                Node {
                    props: props(&[("technology", text("Redis"))]),
                    ..node("c", "cache", "Free")
                },
                Node {
                    r#ref: structure_ref("missing"),
                    ..node("d", "queue", "Gone")
                },
            ],
            edges: vec![],
        };
        let nodes = vec![StructureNode {
            id: "db".to_string(),
            title: "Database".to_string(),
            kind: "database".to_string(),
            tech: "Postgres".to_string(),
            status: NODE_ACTIVE.to_string(),
            ..StructureNode::default()
        }];
        let out = with_structure(&graph, &nodes);
        let tech = |node: &Node| node.props.get("technology").cloned();
        assert_eq!(out.nodes[0].kind, "database");
        assert_eq!(out.nodes[0].label, "Database");
        assert_eq!(tech(&out.nodes[0]), Some(text("Postgres")));
        assert_eq!(out.nodes[1].kind, KIND_COMPONENT);
        assert_eq!(out.nodes[1].label, "Database");
        assert_eq!(tech(&out.nodes[1]), Some(text("Postgres")));
        assert_eq!(tech(&out.nodes[2]), Some(text("Redis")));
        assert_eq!(out.nodes[3].label, "Gone");
        assert_eq!(out.nodes[3].kind, "queue");
        assert_eq!(tech(&graph.nodes[0]), Some(text("Node.js")));
        assert_eq!(graph.nodes[0].label, "Old");
    }
}
