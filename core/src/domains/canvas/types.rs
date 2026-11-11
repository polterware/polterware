//! Port of `backend/domains/canvas/types.go`.

use super::{
    COMPONENT_KINDS, EDGE_KINDS, KIND_NOTE, KIND_TEXT, SHAPE_KINDS, STRUCTURE_KINDS, TYPE_CANVAS,
};

/// The canvas palette and the rules [`validate`](super::validate) applies.
/// `node_kinds` lists components, structure kinds, drawing shapes, note, and
/// text so a client can build the palette from the definition alone.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Definition {
    pub r#type: String,
    pub label: String,
    pub node_kinds: Vec<String>,
    pub edge_kinds: Vec<String>,
    pub rules: Vec<String>,
    pub unrestricted: bool,
}

fn node_kinds() -> Vec<String> {
    let mut out: Vec<String> = COMPONENT_KINDS
        .iter()
        .map(|kind| (*kind).to_string())
        .collect();
    out.extend(STRUCTURE_KINDS.iter().map(|kind| (*kind).to_string()));
    out.extend(SHAPE_KINDS.iter().map(|kind| (*kind).to_string()));
    out.push(KIND_NOTE.to_string());
    out.push(KIND_TEXT.to_string());
    out
}

/// The registry. There is one generalist canvas type.
fn definitions() -> Vec<Definition> {
    vec![Definition {
        r#type: TYPE_CANVAS.to_string(),
        label: "Canvas".to_string(),
        node_kinds: node_kinds(),
        edge_kinds: EDGE_KINDS.iter().map(|kind| (*kind).to_string()).collect(),
        rules: vec![
            "Components may carry a technology and an optional ref to a structure node."
                .to_string(),
            "Domain and component nodes ref a structure node.".to_string(),
            "Drawing shapes, notes, and text carry no meaning and skip structural rules."
                .to_string(),
        ],
        unrestricted: false,
    }]
}

/// The registry in palette order.
pub fn types() -> Vec<Definition> {
    definitions()
}

/// The definition for a canvas type.
pub fn lookup(typ: &str) -> Option<Definition> {
    definitions().into_iter().find(|def| def.r#type == typ)
}
