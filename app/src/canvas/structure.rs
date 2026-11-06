//! Structure drawn as a canvas (`structure-canvas.ts`): the project's loose components and
//! then its domains in rows of three, each domain a group holding its components in rows of
//! three. Every drawn node cites its Structure node, so its title, kind, and technology follow
//! the tree. Redrawing keeps the connections between nodes still drawn and every item someone
//! added by hand; nodes that left the tree leave the drawing.

use std::collections::HashMap;

use serde_json::{Map, Value, json};

use crate::contracts::{Canvas, CanvasGraph, CanvasNode, CanvasRef, StructureNode};

/// Marks a canvas drawn from the Structure tree, in its layout.
pub const SOURCE: &str = "structure";
pub const NAME: &str = "Structure";

const CARD: (f32, f32) = (220.0, 64.0);
const EMPTY_GROUP: (f32, f32) = (260.0, 120.0);
/// Room for a group's own icon tile and title above its children.
const PAD_TOP: f32 = 64.0;
const PAD_SIDE: f32 = 20.0;
const PAD_BOTTOM: f32 = 20.0;
const GAP: f32 = 16.0;
const COLUMNS: usize = 3;

struct Measured<'a> {
    node: &'a StructureNode,
    width: f32,
    height: f32,
    children: Vec<(Measured<'a>, f32, f32)>,
}

fn layout_of(raw: &str) -> Map<String, Value> {
    match serde_json::from_str::<Value>(if raw.is_empty() { "{}" } else { raw }) {
        Ok(Value::Object(map)) => map,
        _ => Map::new(),
    }
}

/// Whether the canvas was drawn from the Structure tree.
pub fn is_structure_canvas(canvas: &Canvas) -> bool {
    layout_of(&canvas.layout_json)
        .get("source")
        .and_then(Value::as_str)
        == Some(SOURCE)
}

fn measure<'a>(
    node: &'a StructureNode,
    children: &HashMap<&str, Vec<&'a StructureNode>>,
) -> Measured<'a> {
    let kids = children.get(node.id.as_str()).cloned().unwrap_or_default();
    if node.level == "component" {
        return Measured {
            node,
            width: CARD.0,
            height: CARD.1,
            children: Vec::new(),
        };
    }
    if kids.is_empty() {
        return Measured {
            node,
            width: EMPTY_GROUP.0,
            height: EMPTY_GROUP.1,
            children: Vec::new(),
        };
    }
    // Loose components first, then domains, each in the store's order.
    let ordered: Vec<&StructureNode> = kids
        .iter()
        .filter(|kid| kid.level == "component")
        .chain(kids.iter().filter(|kid| kid.level != "component"))
        .copied()
        .collect();
    let boxes: Vec<Measured> = ordered
        .into_iter()
        .map(|kid| measure(kid, children))
        .collect();
    let mut placed = Vec::new();
    let mut y = PAD_TOP;
    let mut width: f32 = 0.0;
    let mut boxes = boxes.into_iter().peekable();
    while boxes.peek().is_some() {
        let row: Vec<Measured> = boxes.by_ref().take(COLUMNS).collect();
        let tallest = row.iter().map(|item| item.height).fold(f32::MIN, f32::max);
        let mut x = PAD_SIDE;
        for item in row {
            let item_width = item.width;
            placed.push((item, x, y));
            x += item_width + GAP;
        }
        width = width.max(x - GAP + PAD_SIDE);
        y += tallest + GAP;
    }
    Measured {
        node,
        width,
        height: y - GAP + PAD_BOTTOM,
        children: placed,
    }
}

/// A drawn node and its box (`x, y, width, height`).
type Placed<'a> = (&'a StructureNode, (f32, f32, f32, f32));

fn flatten<'a>(item: &Measured<'a>, x: f32, y: f32, out: &mut Vec<Placed<'a>>) {
    out.push((item.node, (x, y, item.width, item.height)));
    for (child, cx, cy) in &item.children {
        flatten(child, x + cx, y + cy, out);
    }
}

fn canvas_id(id: &str) -> String {
    format!("structure-{id}")
}

/// The graph and layout of the Structure drawing, from the tree and the drawing before.
pub fn draw_structure(nodes: &[StructureNode], previous: Option<&Canvas>) -> (CanvasGraph, String) {
    let live: Vec<&StructureNode> = nodes
        .iter()
        .filter(|node| node.status != "removed")
        .collect();
    let known: std::collections::HashSet<&str> = live.iter().map(|node| node.id.as_str()).collect();
    let mut children: HashMap<&str, Vec<&StructureNode>> = HashMap::new();
    for node in &live {
        if !node.parent_id.is_empty() && known.contains(node.parent_id.as_str()) {
            children
                .entry(node.parent_id.as_str())
                .or_default()
                .push(node);
        } else if node.parent_id.is_empty() {
            children.entry("").or_default().push(node);
        }
    }
    // The project is not drawn: its children sit where a group's would, moved to the origin.
    let project = StructureNode {
        level: "project".into(),
        ..StructureNode::default()
    };
    let measured = measure(&project, &children);
    let mut drawn = Vec::new();
    for (child, x, y) in &measured.children {
        flatten(child, x - PAD_SIDE, y - PAD_TOP, &mut drawn);
    }

    let drawn_ids: std::collections::HashSet<String> =
        drawn.iter().map(|(node, _)| canvas_id(&node.id)).collect();
    let mut layout = previous
        .map(|canvas| layout_of(&canvas.layout_json))
        .unwrap_or_default();
    let previous_rects = layout
        .get("nodes")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let previous_graph = previous
        .and_then(|canvas| canvas.graph.clone())
        .unwrap_or_default();
    // Hand-drawn items stay where they are; generated nodes of a removed Structure node go.
    let kept: Vec<CanvasNode> = previous_graph
        .nodes
        .iter()
        .filter(|node| !node.id.starts_with("structure-"))
        .cloned()
        .collect();
    let ids: std::collections::HashSet<String> = drawn_ids
        .iter()
        .cloned()
        .chain(kept.iter().map(|node| node.id.clone()))
        .collect();
    let edges = previous_graph
        .edges
        .iter()
        .filter(|edge| ids.contains(&edge.from) && ids.contains(&edge.to))
        .cloned()
        .collect();
    // Parents come first, so a group is painted under what it holds.
    let graph = CanvasGraph {
        nodes: drawn
            .iter()
            .map(|(node, _)| CanvasNode {
                id: canvas_id(&node.id),
                kind: node.level.clone(),
                label: node.title.clone(),
                props_json: "{}".into(),
                r#ref: Some(CanvasRef {
                    kind: "node".into(),
                    id: node.id.clone(),
                }),
            })
            .chain(kept.iter().cloned())
            .collect(),
        edges,
    };
    let width = drawn
        .iter()
        .map(|(_, rect)| rect.0 + rect.2)
        .fold(0.0_f32, f32::max);
    let height = drawn
        .iter()
        .map(|(_, rect)| rect.1 + rect.3)
        .fold(0.0_f32, f32::max);
    let zoom = if drawn.is_empty() {
        1.0
    } else {
        (1200.0 / width).min(800.0 / height).clamp(0.25, 1.0)
    };
    let mut rects = Map::new();
    for node in &kept {
        if let Some(rect) = previous_rects.get(&node.id) {
            rects.insert(node.id.clone(), rect.clone());
        }
    }
    for (node, (x, y, w, h)) in &drawn {
        rects.insert(
            canvas_id(&node.id),
            json!({ "x": x, "y": y, "width": w, "height": h }),
        );
    }
    layout.insert("version".into(), json!(1));
    layout.insert("source".into(), json!(SOURCE));
    layout.insert(
        "viewport".into(),
        json!({ "x": 40, "y": 40, "zoom": (zoom * 100.0).round() / 100.0 }),
    );
    layout.insert("nodes".into(), Value::Object(rects));
    (graph, Value::Object(layout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, parent: &str, level: &str) -> StructureNode {
        StructureNode {
            id: id.into(),
            parent_id: parent.into(),
            level: level.into(),
            title: id.to_uppercase(),
            ..StructureNode::default()
        }
    }

    #[test]
    fn domains_hold_their_components_in_rows_of_three() {
        let nodes = vec![
            node("billing", "", "domain"),
            node("api", "billing", "component"),
            node("db", "billing", "component"),
            node("cli", "", "component"),
        ];
        let (graph, layout) = draw_structure(&nodes, None);
        let ids: Vec<_> = graph.nodes.iter().map(|node| node.id.as_str()).collect();
        // The loose component first, then the domain and what it holds.
        assert_eq!(
            ids,
            [
                "structure-cli",
                "structure-billing",
                "structure-api",
                "structure-db"
            ]
        );
        let layout: Value = serde_json::from_str(&layout).unwrap();
        assert_eq!(layout["source"], "structure");
        let billing = &layout["nodes"]["structure-billing"];
        assert_eq!(billing["x"], 236.0);
        assert_eq!(billing["width"], 496.0);
        assert_eq!(billing["height"], 148.0);
        assert_eq!(layout["nodes"]["structure-api"]["y"], 64.0);
    }

    #[test]
    fn a_redraw_keeps_hand_drawn_items_and_their_connections() {
        let nodes = vec![node("billing", "", "domain")];
        let (graph, layout) = draw_structure(&nodes, None);
        let mut graph = graph;
        graph.nodes.push(CanvasNode {
            id: "note-1".into(),
            kind: "note".into(),
            ..CanvasNode::default()
        });
        graph.edges.push(crate::contracts::CanvasEdge {
            id: "e".into(),
            from: "note-1".into(),
            to: "structure-billing".into(),
            ..Default::default()
        });
        let previous = Canvas {
            graph: Some(graph),
            layout_json: layout,
            ..Canvas::default()
        };
        let (redrawn, _) = draw_structure(&[], Some(&previous));
        assert_eq!(redrawn.nodes.len(), 1);
        assert_eq!(redrawn.nodes[0].id, "note-1");
        assert!(redrawn.edges.is_empty());
    }
}
