//! A canvas as the editor holds it (`model.ts`, `style.ts`, `types/shared.ts`): the core's
//! property graph with each node's position and size from the layout, its properties as a
//! JSON object, and the viewport. Saving writes the graph back with the layout, keeping every
//! layout field the editor does not know.

use polterware_ui::canvas::{Box2, Viewport, clamp_zoom};
use serde_json::{Map, Value, json};

use super::catalog::{edge_palette, node_palette};
use crate::contracts::{
    Canvas, CanvasEdge, CanvasGraph, CanvasNode, CanvasRef, CanvasTypeDefinition,
};

pub type Props = Map<String, Value>;

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub z: i32,
    pub props: Props,
    pub reference: Option<CanvasRef>,
}

impl Node {
    pub fn rect(&self) -> Box2 {
        Box2::new(self.x, self.y, self.width, self.height)
    }

    /// A property as text (`propertyText`): strings and numbers, else empty.
    pub fn text(&self, key: &str) -> String {
        property_text(&self.props, key)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Edge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub kind: String,
    pub props: Props,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Doc {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub viewport: Viewport,
}

impl Doc {
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn edge(&self, id: &str) -> Option<&Edge> {
        self.edges.iter().find(|edge| edge.id == id)
    }
}

pub fn property_text(props: &Props, key: &str) -> String {
    match props.get(key) {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Number(number)) => number.to_string(),
        _ => String::new(),
    }
}

fn object(text: &str) -> Props {
    match serde_json::from_str::<Value>(if text.is_empty() { "{}" } else { text }) {
        Ok(Value::Object(map)) => map,
        _ => Props::new(),
    }
}

fn finite(value: Option<&Value>, fallback: f32) -> f32 {
    value
        .and_then(Value::as_f64)
        .map(|value| value as f32)
        .filter(|value| value.is_finite())
        .unwrap_or(fallback)
}

/// The service's graph read as the editor's document (`toDocument`): a node the layout has no
/// place for lands in rows of four, 220 by 150 apart.
pub fn to_document(canvas: &Canvas) -> Doc {
    let layout = object(&canvas.layout_json);
    let positions = layout.get("nodes").and_then(Value::as_object);
    let viewport = layout.get("viewport").and_then(Value::as_object);
    let view = |key: &str, fallback: f32| finite(viewport.and_then(|view| view.get(key)), fallback);
    let graph = canvas.graph.clone().unwrap_or_default();
    let nodes = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let at = positions
                .and_then(|nodes| nodes.get(&node.id))
                .and_then(Value::as_object);
            let get = |key: &str, fallback: f32| finite(at.and_then(|at| at.get(key)), fallback);
            Node {
                id: node.id.clone(),
                name: node.label.clone(),
                kind: node.kind.clone(),
                x: get("x", (index % 4) as f32 * 220.0),
                y: get("y", (index / 4) as f32 * 150.0),
                width: get("width", 180.0).max(40.0),
                height: get("height", 100.0).max(32.0),
                z: index as i32,
                props: object(&node.props_json),
                reference: node.r#ref.clone(),
            }
        })
        .collect();
    let edges = graph
        .edges
        .iter()
        .map(|edge| Edge {
            id: edge.id.clone(),
            from: edge.from.clone(),
            to: edge.to.clone(),
            kind: edge.kind.clone(),
            props: object(&edge.props_json),
        })
        .collect();
    Doc {
        nodes,
        edges,
        viewport: Viewport {
            x: view("x", 40.0),
            y: view("y", 40.0),
            zoom: clamp_zoom(view("zoom", 1.0)),
        },
    }
}

fn number(value: f32) -> Value {
    json!(f64::from(value))
}

/// The document as the service stores it (`serializeCanvas`): the graph, and the layout with
/// every field it had besides the viewport and each node's box.
pub fn serialize(doc: &Doc, original_layout: &str) -> (CanvasGraph, String) {
    let mut layout = object(original_layout);
    let previous = layout
        .get("nodes")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let graph = CanvasGraph {
        nodes: doc
            .nodes
            .iter()
            .map(|node| CanvasNode {
                id: node.id.clone(),
                kind: node.kind.clone(),
                label: node.name.clone(),
                props_json: Value::Object(node.props.clone()).to_string(),
                r#ref: node.reference.clone(),
            })
            .collect(),
        edges: doc
            .edges
            .iter()
            .map(|edge| CanvasEdge {
                id: edge.id.clone(),
                from: edge.from.clone(),
                to: edge.to.clone(),
                kind: edge.kind.clone(),
                props_json: Value::Object(edge.props.clone()).to_string(),
            })
            .collect(),
    };
    let nodes: Props = doc
        .nodes
        .iter()
        .map(|node| {
            let mut rect = previous
                .get(&node.id)
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            rect.insert("x".into(), number(node.x));
            rect.insert("y".into(), number(node.y));
            rect.insert("width".into(), number(node.width));
            rect.insert("height".into(), number(node.height));
            (node.id.clone(), Value::Object(rect))
        })
        .collect();
    layout.insert("version".into(), json!(1));
    layout.insert(
        "viewport".into(),
        json!({
            "x": f64::from(doc.viewport.x),
            "y": f64::from(doc.viewport.y),
            "zoom": f64::from(doc.viewport.zoom),
        }),
    );
    layout.insert("nodes".into(), Value::Object(nodes));
    (graph, Value::Object(layout).to_string())
}

/// Control characters out, at most 200 characters (`sanitizeText`).
pub fn sanitize(text: &str) -> String {
    text.chars()
        .filter(|ch| !matches!(*ch as u32, 0..=8 | 11 | 12 | 14..=31))
        .take(200)
        .collect()
}

/// A new node of `kind` at `at` (`insertNode`), refused when the canvas does not allow it.
pub fn insert_node(
    doc: &Doc,
    definition: &CanvasTypeDefinition,
    kind: &str,
    at: (f32, f32),
    id: String,
) -> Result<Doc, String> {
    if !node_palette(definition).iter().any(|item| item == kind) {
        return Err(format!("Shape {kind} is not allowed on this canvas."));
    }
    let mut next = doc.clone();
    next.nodes.push(Node {
        id,
        name: if kind == "note" {
            "New note".into()
        } else {
            format!("New {kind}")
        },
        kind: kind.to_string(),
        x: at.0,
        y: at.1,
        width: 180.0,
        height: 100.0,
        z: doc.nodes.len() as i32,
        props: Props::new(),
        reference: None,
    });
    Ok(next)
}

/// A connection of `kind` from one node to another (`connectNodes`).
pub fn connect_nodes(
    doc: &Doc,
    definition: &CanvasTypeDefinition,
    from: &str,
    to: &str,
    kind: &str,
    id: String,
) -> Result<Doc, String> {
    let kind = kind.trim();
    if kind.is_empty()
        || (!definition.unrestricted && !definition.edge_kinds.iter().any(|item| item == kind))
    {
        return Err("Choose an allowed edge type.".into());
    }
    if doc.node(from).is_none() || doc.node(to).is_none() {
        return Err("Choose two existing nodes.".into());
    }
    let mut next = doc.clone();
    next.edges.push(Edge {
        id,
        from: from.to_string(),
        to: to.to_string(),
        kind: kind.to_string(),
        props: Props::new(),
    });
    Ok(next)
}

/// The connection kinds this canvas offers, the first one new connections take.
pub fn edge_kinds(definition: &CanvasTypeDefinition) -> Vec<String> {
    edge_palette(definition)
}

/// A resize handle: a corner or a side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    Nw,
    N,
    Ne,
    E,
    Se,
    S,
    Sw,
    W,
}

pub const HANDLES: [Handle; 8] = [
    Handle::Nw,
    Handle::N,
    Handle::Ne,
    Handle::E,
    Handle::Se,
    Handle::S,
    Handle::Sw,
    Handle::W,
];

impl Handle {
    pub fn west(self) -> bool {
        matches!(self, Handle::Nw | Handle::Sw | Handle::W)
    }
    pub fn east(self) -> bool {
        matches!(self, Handle::Ne | Handle::Se | Handle::E)
    }
    pub fn north(self) -> bool {
        matches!(self, Handle::Nw | Handle::Ne | Handle::N)
    }
    pub fn south(self) -> bool {
        matches!(self, Handle::Sw | Handle::Se | Handle::S)
    }
    pub fn label(self) -> &'static str {
        match self {
            Handle::Nw => "nw",
            Handle::N => "n",
            Handle::Ne => "ne",
            Handle::E => "e",
            Handle::Se => "se",
            Handle::S => "s",
            Handle::Sw => "sw",
            Handle::W => "w",
        }
    }
}

/// Resizes `node` from a handle by (dx, dy) of its pre-gesture box; west and north handles
/// keep the opposite edge fixed (`resizeFrom`), never under 40 by 32.
pub fn resize_from(doc: &Doc, node: &Node, handle: Handle, dx: f32, dy: f32) -> Doc {
    let (min_width, min_height) = (40.0, 32.0);
    let (mut x, mut y, mut width, mut height) = (node.x, node.y, node.width, node.height);
    if handle.east() {
        width = (width + dx).max(min_width);
    }
    if handle.south() {
        height = (height + dy).max(min_height);
    }
    if handle.west() {
        let next = (width - dx).max(min_width);
        x += width - next;
        width = next;
    }
    if handle.north() {
        let next = (height - dy).max(min_height);
        y += height - next;
        height = next;
    }
    update_node(doc, &node.id, |node| {
        node.x = x;
        node.y = y;
        node.width = width;
        node.height = height;
    })
}

/// The document with `id`'s node changed by `change`.
pub fn update_node(doc: &Doc, id: &str, change: impl FnOnce(&mut Node)) -> Doc {
    let mut next = doc.clone();
    if let Some(node) = next.nodes.iter_mut().find(|node| node.id == id) {
        change(node);
        node.name = sanitize(&node.name);
    }
    next
}

/// The document with `id`'s connection changed by `change`.
pub fn update_edge(doc: &Doc, id: &str, change: impl FnOnce(&mut Edge)) -> Doc {
    let mut next = doc.clone();
    if let Some(edge) = next.edges.iter_mut().find(|edge| edge.id == id) {
        change(edge);
    }
    next
}

/// Moves each listed node by its delta.
pub fn move_nodes(doc: &Doc, ids: &[String], dx: f32, dy: f32) -> Doc {
    let mut next = doc.clone();
    for node in next.nodes.iter_mut().filter(|node| ids.contains(&node.id)) {
        node.x += dx;
        node.y += dy;
    }
    next
}

/// What is selected: nodes, or connections.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    pub nodes: Vec<String>,
    pub edges: Vec<String>,
}

impl Selection {
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }

    pub fn node(id: &str) -> Self {
        Self {
            nodes: vec![id.to_string()],
            edges: Vec::new(),
        }
    }

    pub fn edge(id: &str) -> Self {
        Self {
            nodes: Vec::new(),
            edges: vec![id.to_string()],
        }
    }

    /// A click on a node (`selectClick`): it alone, or toggled in the selection with Shift.
    pub fn click_node(&self, id: &str, shift: bool) -> Self {
        if !shift {
            return Self::node(id);
        }
        let mut next = self.clone();
        match next.nodes.iter().position(|item| item == id) {
            Some(index) => {
                next.nodes.remove(index);
            }
            None => next.nodes.push(id.to_string()),
        }
        next
    }
}

/// The document without the selection, and without the connections of removed nodes.
pub fn remove_selection(doc: &Doc, selection: &Selection) -> Doc {
    let mut next = doc.clone();
    next.nodes
        .retain(|node| !selection.nodes.contains(&node.id));
    next.edges.retain(|edge| {
        !selection.edges.contains(&edge.id)
            && !selection.nodes.contains(&edge.from)
            && !selection.nodes.contains(&edge.to)
    });
    next
}

/// Sets or clears a node's reference (`setReference`).
pub fn set_reference(doc: &Doc, id: &str, reference: Option<CanvasRef>) -> Doc {
    update_node(doc, id, |node| node.reference = reference)
}

/// A semantic color tag, never a user-supplied color.
pub const COLOR_TAGS: [&str; 5] = ["neutral", "yellow", "blue", "green", "red"];

/// A shape's presentation, under `props.style` (`ShapeStyle`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapeStyle {
    pub fill: String,
    pub stroke: String,
    pub stroke_width: u8,
    pub dashed: bool,
    pub text_size: TextSize,
    pub text_align: TextAlign,
    pub bold: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextSize {
    Sm,
    Md,
    Lg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl TextSize {
    pub fn wire(self) -> &'static str {
        match self {
            TextSize::Sm => "sm",
            TextSize::Md => "md",
            TextSize::Lg => "lg",
        }
    }
    /// `text-xs`, `text-sm`, `text-base`.
    pub fn px(self) -> f32 {
        match self {
            TextSize::Sm => 12.0,
            TextSize::Md => 14.0,
            TextSize::Lg => 16.0,
        }
    }
}

impl TextAlign {
    pub fn wire(self) -> &'static str {
        match self {
            TextAlign::Left => "left",
            TextAlign::Center => "center",
            TextAlign::Right => "right",
        }
    }
}

/// Reads `props.style`, each field falling back to the default when absent or unknown.
pub fn read_style(props: &Props) -> ShapeStyle {
    let style = props.get("style").and_then(Value::as_object);
    let text = |key: &str| {
        style
            .and_then(|style| style.get(key))
            .and_then(Value::as_str)
    };
    let tag = |key: &str| {
        text(key)
            .filter(|value| COLOR_TAGS.contains(value) || *value == "none")
            .unwrap_or("neutral")
            .to_string()
    };
    let width = style
        .and_then(|style| style.get("strokeWidth"))
        .and_then(Value::as_u64)
        .filter(|width| (1..=3).contains(width))
        .unwrap_or(1) as u8;
    let flag = |key: &str| {
        style
            .and_then(|style| style.get(key))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    };
    ShapeStyle {
        fill: tag("fill"),
        stroke: tag("stroke"),
        stroke_width: width,
        dashed: flag("dashed"),
        text_size: match text("textSize") {
            Some("sm") => TextSize::Sm,
            Some("lg") => TextSize::Lg,
            _ => TextSize::Md,
        },
        text_align: match text("textAlign") {
            Some("left") => TextAlign::Left,
            Some("right") => TextAlign::Right,
            _ => TextAlign::Center,
        },
        bold: flag("bold"),
    }
}

/// Writes `style` under `props.style`, every field spelled out (`withStyle`).
pub fn write_style(props: &mut Props, style: &ShapeStyle) {
    props.insert(
        "style".into(),
        json!({
            "fill": style.fill,
            "stroke": style.stroke,
            "strokeWidth": style.stroke_width,
            "dashed": style.dashed,
            "textSize": style.text_size.wire(),
            "textAlign": style.text_align.wire(),
            "bold": style.bold,
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canvas(layout: &str) -> Canvas {
        Canvas {
            id: "c1".into(),
            project_id: "p".into(),
            name: "Map".into(),
            r#type: "canvas".into(),
            graph: Some(CanvasGraph {
                nodes: (0..5)
                    .map(|index| CanvasNode {
                        id: format!("n{index}"),
                        kind: "rectangle".into(),
                        label: format!("Box {index}"),
                        props_json: "{}".into(),
                        r#ref: None,
                    })
                    .collect(),
                edges: Vec::new(),
            }),
            layout_json: layout.into(),
            ..Canvas::default()
        }
    }

    #[test]
    fn unplaced_nodes_land_in_rows_of_four() {
        let doc = to_document(&canvas("{}"));
        assert_eq!((doc.nodes[3].x, doc.nodes[3].y), (660.0, 0.0));
        assert_eq!((doc.nodes[4].x, doc.nodes[4].y), (0.0, 150.0));
        assert_eq!(
            doc.viewport,
            Viewport {
                x: 40.0,
                y: 40.0,
                zoom: 1.0
            }
        );
    }

    #[test]
    fn saving_keeps_the_layout_fields_it_does_not_know() {
        let layout = r#"{"source":"structure","nodes":{"n0":{"x":10,"y":20,"width":200,"height":80,"pinned":true}},"viewport":{"x":0,"y":0,"zoom":9}}"#;
        let doc = to_document(&canvas(layout));
        assert_eq!(doc.viewport.zoom, 2.5);
        assert_eq!((doc.nodes[0].x, doc.nodes[0].width), (10.0, 200.0));
        let (graph, saved) = serialize(&doc, layout);
        let saved: Value = serde_json::from_str(&saved).unwrap();
        assert_eq!(saved["source"], "structure");
        assert_eq!(saved["nodes"]["n0"]["pinned"], true);
        assert_eq!(saved["version"], 1);
        assert_eq!(graph.nodes.len(), 5);
    }

    #[test]
    fn a_west_resize_keeps_the_east_edge() {
        let doc = to_document(&canvas("{}"));
        let node = doc.nodes[1].clone();
        let resized = resize_from(&doc, &node, Handle::W, 300.0, 0.0);
        let after = resized.node("n1").unwrap();
        assert_eq!(after.width, 40.0);
        assert_eq!(after.x + after.width, node.x + node.width);
    }

    #[test]
    fn styles_read_their_defaults_and_round_trip() {
        let mut props = Props::new();
        let style = read_style(&props);
        assert_eq!((style.fill.as_str(), style.stroke_width), ("neutral", 1));
        let style = ShapeStyle {
            fill: "blue".into(),
            dashed: true,
            text_align: TextAlign::Left,
            ..style
        };
        write_style(&mut props, &style);
        assert_eq!(read_style(&props), style);
    }

    #[test]
    fn removing_a_node_drops_its_connections() {
        let doc = to_document(&canvas("{}"));
        let definition = CanvasTypeDefinition {
            unrestricted: true,
            ..CanvasTypeDefinition::default()
        };
        let doc = connect_nodes(&doc, &definition, "n0", "n1", "relates", "e1".into()).unwrap();
        let doc = remove_selection(&doc, &Selection::node("n1"));
        assert!(doc.edges.is_empty());
        assert_eq!(doc.nodes.len(), 4);
    }
}
