//! The canvas graph and its layout, as stored and as sent on the wire.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use time::OffsetDateTime;

use super::Result;

/// What a component is, in palette order. The canvas uses the same list for its
/// semantic components.
pub use crate::domains::structure::COMPONENT_KINDS;

/// The one canvas type: drawing shapes plus semantic components.
pub const TYPE_CANVAS: &str = "canvas";

/// Annotations skip structural rules.
pub const KIND_NOTE: &str = "note";
pub const KIND_TEXT: &str = "text";

/// Structure kinds cite a Structure node.
pub const KIND_DOMAIN: &str = "domain";
pub const KIND_COMPONENT: &str = "component";

pub const EDGE_RELATES: &str = "relates";
pub const EDGE_FLOWS: &str = "flows";
pub const EDGE_USES: &str = "uses";
pub const EDGE_CALLS: &str = "calls";
pub const EDGE_READS: &str = "reads";
pub const EDGE_WRITES: &str = "writes";
pub const EDGE_PUBLISHES: &str = "publishes";
pub const EDGE_SUBSCRIBES: &str = "subscribes";
pub const EDGE_DEPENDS_ON: &str = "depends_on";
pub const EDGE_CONTAINS: &str = "contains";

/// Entity refs. A node ref is a Structure node.
pub const REF_CARD: &str = "card";
pub const REF_DECISION: &str = "decision";
pub const REF_FACT: &str = "fact";
pub const REF_NODE: &str = "node";

/// The change_ops entity name for a canvas write.
pub const ENTITY: &str = "canvas";

pub const OP_CREATE: &str = "create";
pub const OP_UPDATE: &str = "update";
pub const OP_REMOVE: &str = "remove";

/// The drawing shapes, in palette order. They carry no meaning and skip
/// structural rules. Names never collide with a component kind (cylinder, not
/// database).
pub const SHAPE_KINDS: [&str; 22] = [
    "rectangle",
    "rounded",
    "square",
    "ellipse",
    "circle",
    "diamond",
    "triangle",
    "hexagon",
    "parallelogram",
    "trapezoid",
    "cylinder",
    "cloud",
    "document",
    "actor",
    "callout",
    "process",
    "terminator",
    "predefined",
    "manual-input",
    "delay",
    "display",
    "frame",
];

/// Structure kinds mirror Structure levels and always ref a Structure node.
pub const STRUCTURE_KINDS: [&str; 2] = [KIND_DOMAIN, KIND_COMPONENT];

/// The connection kinds, in palette order.
pub const EDGE_KINDS: [&str; 10] = [
    EDGE_RELATES,
    EDGE_FLOWS,
    EDGE_USES,
    EDGE_CALLS,
    EDGE_READS,
    EDGE_WRITES,
    EDGE_PUBLISHES,
    EDGE_SUBSCRIBES,
    EDGE_DEPENDS_ON,
    EDGE_CONTAINS,
];

/// Points at another project entity. It is optional on a node.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ref {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub id: String,
}

/// One vertex. Props are semantic; position lives in the layout.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Node {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub label: String,
    #[serde(default, deserialize_with = "null_as_default")]
    pub props: Map<String, Value>,
    #[serde(default, rename = "ref", skip_serializing_if = "Option::is_none")]
    pub r#ref: Option<Ref>,
}

/// Connects From to To. Props are semantic.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub to: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default, deserialize_with = "null_as_default")]
    pub props: Map<String, Value>,
}

/// The semantic document. It has no positions.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Graph {
    #[serde(default, deserialize_with = "null_as_default")]
    pub nodes: Vec<Node>,
    #[serde(default, deserialize_with = "null_as_default")]
    pub edges: Vec<Edge>,
}

/// One saved surface. `source` is the canvas as Mermaid text; `layout_json` is
/// the positions people gave its nodes, which the source does not carry.
#[derive(Debug, Clone, PartialEq)]
pub struct Canvas {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub r#type: String,
    pub agent_visible: bool,
    pub graph: Graph,
    pub source: String,
    pub layout_json: String,
    pub scope_node_id: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    /// The archive stamp; `None` while the canvas is active.
    pub archived_at: Option<OffsetDateTime>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            name: String::new(),
            r#type: String::new(),
            agent_visible: false,
            graph: Graph::default(),
            source: String::new(),
            layout_json: String::new(),
            scope_node_id: String::new(),
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
            archived_at: None,
        }
    }
}

impl Canvas {
    /// Reports whether the canvas is in the archive.
    pub fn archived(&self) -> bool {
        self.archived_at.is_some()
    }
}

/// Reports whether kind is an entity a node may cite.
pub fn valid_ref_kind(kind: &str) -> bool {
    matches!(kind, REF_CARD | REF_DECISION | REF_FACT | REF_NODE)
}

/// Encodes the semantic graph. Layout is not included.
pub fn marshal_graph(graph: &Graph) -> Result<String> {
    let mut graph = graph.clone();
    normalize(&mut graph);
    for node in &mut graph.nodes {
        node.props = normalize_numbers_in(&node.props);
    }
    for edge in &mut graph.edges {
        edge.props = normalize_numbers_in(&edge.props);
    }
    Ok(go_json_escape(&serde_json::to_string(&graph)?))
}

/// Reads a semantic graph. Empty input is an empty graph.
pub fn unmarshal_graph(raw: &str) -> Result<Graph> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(Graph::default());
    }
    let mut graph: Graph = serde_json::from_str(raw)?;
    normalize(&mut graph);
    Ok(graph)
}

/// Trims identity fields and drops an empty ref, the way the store keeps them.
pub(super) fn normalize(graph: &mut Graph) {
    for node in &mut graph.nodes {
        node.id = node.id.trim().to_string();
        node.kind = node.kind.trim().to_string();
        node.label = node.label.trim().to_string();
        if let Some(reference) = &mut node.r#ref {
            reference.kind = reference.kind.trim().to_string();
            reference.id = reference.id.trim().to_string();
            if reference.kind.is_empty() && reference.id.is_empty() {
                node.r#ref = None;
            }
        }
    }
    for edge in &mut graph.edges {
        edge.id = edge.id.trim().to_string();
        edge.from = edge.from.trim().to_string();
        edge.to = edge.to.trim().to_string();
        edge.kind = edge.kind.trim().to_string();
    }
}

/// A missing or null field reads as the type's default, as Go's `encoding/json`
/// does for a nil map, slice, or struct pointer.
pub(super) fn null_as_default<'de, D, T>(deserializer: D) -> std::result::Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

/// Go reads JSON numbers as float64 and writes an integral one without a
/// fraction (`1.0` becomes `1`).
pub(super) fn normalize_numbers(value: &Value) -> Value {
    match value {
        Value::Number(number) => match number.as_f64() {
            Some(float) if float.fract() == 0.0 && float.abs() < 9.0e15 => match number.as_i64() {
                Some(_) => value.clone(),
                None => Value::from(float as i64),
            },
            _ => value.clone(),
        },
        Value::Array(items) => Value::Array(items.iter().map(normalize_numbers).collect()),
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(key, item)| (key.clone(), normalize_numbers(item)))
                .collect(),
        ),
        _ => value.clone(),
    }
}

pub(super) fn normalize_numbers_in(props: &Map<String, Value>) -> Map<String, Value> {
    props
        .iter()
        .map(|(key, value)| (key.clone(), normalize_numbers(value)))
        .collect()
}

/// Go's `json.Marshal` escapes `<`, `>`, `&`, and the two line separators, and
/// sorts map keys; serde_json sorts them too but leaves those five characters
/// alone. They only ever appear inside JSON strings, so the rewrite is safe.
pub(super) fn go_json_escape(text: &str) -> String {
    text.replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_an_empty_graph() {
        let graph = unmarshal_graph("  ").expect("empty input");
        assert!(graph.nodes.is_empty() && graph.edges.is_empty());
    }

    #[test]
    fn unmarshal_trims_identity_and_clears_an_empty_ref() {
        let graph = unmarshal_graph(
            r#"{"nodes":[{"id":" a ","kind":" note ","label":" x ","props":null,"ref":{"kind":" ","id":""}}],"edges":null}"#,
        )
        .expect("graph");
        assert_eq!(graph.nodes[0].id, "a");
        assert_eq!(graph.nodes[0].kind, KIND_NOTE);
        assert_eq!(graph.nodes[0].label, "x");
        assert!(graph.nodes[0].props.is_empty());
        assert_eq!(graph.nodes[0].r#ref, None);
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn marshal_keeps_the_storage_shape() {
        let raw = marshal_graph(&Graph {
            nodes: vec![Node {
                id: "a".to_string(),
                kind: "idea".to_string(),
                label: "One".to_string(),
                ..Node::default()
            }],
            edges: vec![Edge {
                id: "e".to_string(),
                from: "a".to_string(),
                to: "a".to_string(),
                kind: EDGE_RELATES.to_string(),
                ..Edge::default()
            }],
        })
        .expect("marshal");
        assert_eq!(
            raw,
            r#"{"nodes":[{"id":"a","kind":"idea","label":"One","props":{}}],"edges":[{"id":"e","from":"a","to":"a","kind":"relates","props":{}}]}"#
        );
    }

    #[test]
    fn ref_kinds_are_the_entities_a_node_may_cite() {
        for kind in [REF_CARD, REF_DECISION, REF_FACT, REF_NODE] {
            assert!(valid_ref_kind(kind), "ref kind {kind}");
        }
        assert!(!valid_ref_kind("widget"));
        assert!(!valid_ref_kind(""));
    }
}
