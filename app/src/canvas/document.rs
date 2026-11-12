//! A canvas as the editor holds it (`model.ts`, `style.ts`, `types/shared.ts`): each node's
//! label, kind, properties, and reference, its connections, and its place.
//!
//! The source is the canvas's truth and the pins are its places, so the editor reads this
//! document from the Mermaid text (see `source_edit`) and writes every change back into it;
//! nothing here is stored as such.

use polterware_ui::canvas::{Box2, Viewport};
use serde_json::{Map, Value, json};

use crate::contracts::CanvasRef;

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

/// Control characters out, at most 200 characters (`sanitizeText`).
pub fn sanitize(text: &str) -> String {
    text.chars()
        .filter(|ch| !matches!(*ch as u32, 0..=8 | 11 | 12 | 14..=31))
        .take(200)
        .collect()
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
    fn a_selection_names_what_it_holds() {
        assert!(Selection::default().is_empty());
        assert_eq!(Selection::node("a").nodes, vec!["a".to_string()]);
        assert_eq!(Selection::edge("e").edges, vec!["e".to_string()]);
    }
}
