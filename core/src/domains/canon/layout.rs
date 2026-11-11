//! Port of `backend/domains/canon/layout.go`.

use serde_json::{Map, Value};

use super::{LayoutNode, PRED_CALLS, PRED_DEPENDS_ON, PRED_RELATED_TO};

/// Extracts component refs from an architecture layout `body_json`.
/// Supports formatVersion 1 (`name`) and legacy seed (`label`).
pub fn parse_layout_nodes(body_json: &str) -> Vec<LayoutNode> {
    if body_json.trim().is_empty() {
        return Vec::new();
    }
    let Ok(doc) = serde_json::from_str::<Value>(body_json) else {
        return Vec::new();
    };
    let Some(items) = object_array(&doc, "nodes") else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(items.len());
    for node in items {
        let id = string_field(node, "id");
        if id.is_empty() {
            continue;
        }
        let mut title = string_field(node, "name");
        if title.is_empty() {
            title = string_field(node, "label");
        }
        if title.is_empty() {
            title = id.clone();
        }
        out.push(LayoutNode { id, title });
    }
    out
}

/// A canvas relationship projected into `canon_relationships`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LayoutEdge {
    pub source_node_id: String,
    pub target_node_id: String,
    pub predicate: String,
    pub evidence: bool,
}

/// Extracts edges from an architecture layout `body_json`.
/// Supports formatVersion 1 `relationships[]` and legacy `edges[]`.
pub fn parse_layout_edges(body_json: &str) -> Vec<LayoutEdge> {
    if body_json.trim().is_empty() {
        return Vec::new();
    }
    let Ok(doc) = serde_json::from_str::<Value>(body_json) else {
        return Vec::new();
    };
    let Some(relationships) = object_array(&doc, "relationships") else {
        return Vec::new();
    };
    let Some(edges) = object_array(&doc, "edges") else {
        return Vec::new();
    };

    let mut out = Vec::with_capacity(relationships.len() + edges.len());
    for rel in relationships {
        let source = string_field(rel, "sourceNodeId");
        let target = string_field(rel, "targetNodeId");
        if source.is_empty() || target.is_empty() {
            continue;
        }
        out.push(LayoutEdge {
            source_node_id: source,
            target_node_id: target,
            predicate: predicate_for_canvas_type(&string_field(rel, "type")).to_string(),
            evidence: evidence_field(rel),
        });
    }
    for edge in edges {
        let source = string_field(edge, "from");
        let target = string_field(edge, "to");
        if source.is_empty() || target.is_empty() {
            continue;
        }
        let mut kind = string_field(edge, "kind");
        if kind.is_empty() {
            kind = string_field(edge, "type");
        }
        out.push(LayoutEdge {
            source_node_id: source,
            target_node_id: target,
            predicate: predicate_for_canvas_type(&kind).to_string(),
            evidence: evidence_field(edge),
        });
    }
    out
}

/// Maps canvas relationship types to Canon predicates.
pub fn predicate_for_canvas_type(canvas_type: &str) -> &'static str {
    match canvas_type.trim().to_lowercase().as_str() {
        "depends_on" | "hosted_on" | "deployed_on" | "stores_in" | "deploys_to" | "reads_from"
        | "writes_to" | "consumes_from" | "publishes_to" => PRED_DEPENDS_ON,
        "calls" | "routes_to" | "connects_to" | "authenticates_with" => PRED_CALLS,
        _ => PRED_RELATED_TO,
    }
}

/// The object elements of `key` in `doc`, or `None` when the document does not
/// hold a list of objects there (Go's `json.Unmarshal` then fails).
fn object_array<'a>(doc: &'a Value, key: &str) -> Option<Vec<&'a Map<String, Value>>> {
    match doc.get(key) {
        None | Some(Value::Null) => Some(Vec::new()),
        Some(Value::Array(items)) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(item.as_object()?);
            }
            Some(out)
        }
        Some(_) => None,
    }
}

fn string_field(node: &Map<String, Value>, key: &str) -> String {
    match node.get(key) {
        Some(Value::String(value)) => value.trim().to_string(),
        _ => String::new(),
    }
}

/// Defaults to true when the key is omitted (an intentional declared edge).
fn evidence_field(node: &Map<String, Value>) -> bool {
    match node.get("evidence") {
        None | Some(Value::Null) => true,
        Some(Value::Bool(value)) => *value,
        Some(Value::Number(value)) => value.as_f64().is_none_or(|number| number != 0.0),
        Some(Value::String(value)) => {
            !matches!(value.trim().to_lowercase().as_str(), "false" | "0" | "no")
        }
        _ => true,
    }
}
