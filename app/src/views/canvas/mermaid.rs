//! A canvas's Mermaid source, drawn for the Desktop.
//!
//! The canvas is Mermaid text, so the editor and the list's previews render that text instead
//! of reading the stored graph: [`render`] preprocesses the source, applies its configuration,
//! and lays the diagram out with the same text measurement the painter uses. Pins are not part
//! of the text: `layout_json` keeps the places people gave its nodes, and [`apply_pins`] moves
//! the laid-out nodes (and the edge ends that touch them) onto those places afterwards.
//!
//! What a source cannot become is a scene, and both answers stay on screen: a parse error
//! reads as `line L, column C: message` through [`message`], and a diagram type the renderer
//! does not draw yet reports [`RenderError::Unsupported`], which the editor shows as the text
//! itself with a note.

use std::collections::{HashMap, HashSet};

use gpui::Rgba;
use polterware_mermaid::DiagramKind;
use polterware_mermaid::config::Config;
use polterware_mermaid::diagrams::flowchart;
use polterware_mermaid::error::{ParseError, RenderError};
use polterware_mermaid::measure::{Font, TextMeasure, plain_text_block};
use polterware_mermaid::render::edges::{EdgeData, insert_edge_label};
use polterware_mermaid::render::markers::marker_placement;
use polterware_mermaid::scene::{
    Color, Group, Item, MarkerDef, MarkerUnits, NORMAL_WEIGHT, Path, PathCommand, PathShape,
    PathStyle, Point, Rect, RectShape, Scene, SourceKind, SourceRef, TextBaseline, TextRun,
    Transform,
};
use polterware_mermaid::theme::Theme as MermaidTheme;
use polterware_ui::mermaid::SceneViewport;
use polterware_ui::theme::Theme;

use crate::canvas::catalog::{component_of, kind_label, silhouette, structure_kind};
use crate::canvas::document::{Node, Props};
use crate::canvas::references::{StoreEntity, linked_facts, reference_state};
use crate::contracts::CanvasRef;

/// How near an edge end has to be to a node's box to be read as that node's.
const END_TOLERANCE: f64 = 0.75;
/// The room kept around a selected node when the diagram is too large to fit the view.
pub(super) const REVEAL_MARGIN: f32 = 24.0;
/// The smallest zoom a reveal zooms out to so the whole drawing fits beside the panels.
pub(super) const REVEAL_MIN_ZOOM: f32 = 0.5;
/// How far past an edge's end its arrowhead reaches: Mermaid stops the line one head short of
/// the node, so the end is that much further from the box than the tip it paints.
const ARROW_REACH: f64 = 4.0;
/// The room a scene keeps around its content when pins move nodes past the fitted view box.
const REFIT_PADDING: f64 = 24.0;

/// The fill Mermaid's default theme paints a node with.
const MERMAID_FILL: &str = "#ececff";
/// The stroke Mermaid's default theme draws a node with.
const MERMAID_STROKE: &str = "#9370db";
/// The second fill (clusters, notes) of Mermaid's default theme.
const MERMAID_SECOND_FILL: &str = "#ffffde";
/// The cluster border of Mermaid's default theme.
const MERMAID_CLUSTER_STROKE: &str = "#aaaa33";
/// The line color Mermaid's default theme draws edges with.
const MERMAID_LINE: &str = "#333333";
/// The label background Mermaid's default theme paints edge labels with.
const MERMAID_LABEL_BACKGROUND: &str = "rgba(232,232,232, 0.8)";
/// The dash a kind drawn as an external thing wears.
const DASHED_STROKE: &str = "4 4";
/// The width Mermaid draws a flowchart edge with (`edgePath`'s `stroke-width: 2px`).
const EDGE_WIDTH: f64 = 2.0;
/// The dash Mermaid gives a dashed flowchart edge (`stroke-dasharray: 3`).
const EDGE_DASH: [f64; 2] = [3.0, 3.0];

/// The colors the app draws a Mermaid scene with, built from its theme tokens.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Palette {
    pub node_fill: String,
    pub node_stroke: String,
    pub text: String,
    pub line: String,
    pub cluster_fill: String,
    pub cluster_stroke: String,
    pub label_background: String,
    /// The app's UI font, the family every label is measured and painted in.
    pub font_family: String,
}

/// The shapes a node with `kind` draws as when its text names none.
///
/// This is the one table for the editor's drawing, the library's insert, and the inspector's
/// kind changes; the core keeps the same mapping for the agent export.
pub(super) fn shape_for_kind(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "rectangle" => "rect",
        "rounded" => "rounded",
        "circle" => "circle",
        "diamond" => "diamond",
        "triangle" => "triangle",
        "hexagon" => "hex",
        "trapezoid" => "trapezoid",
        "cylinder" | "database" | "cache" | "storage" | "warehouse" | "dataset" | "logs" => "cyl",
        "cloud" | "cdn" | "cluster" | "firewall" | "kms" => "cloud",
        "document" | "spec" => "doc",
        "process" => "process",
        "manual-input" => "manual-input",
        "delay" => "delay",
        "display" => "display",
        "queue" | "stream" => "datastore",
        "llm" | "agent" | "mcp" | "vector" => "hex",
        "user" | "team" => "stadium",
        "scheduler" | "pipeline" | "ci" => "lean-r",
        "note" => "rounded",
        "text" => "text",
        "domain" | "component" => "rounded",
        _ if component_of(kind).is_some() => "rounded",
        _ => return None,
    })
}

/// Whether a kind is drawn with a dashed outline: something the project does not own.
pub(super) fn dashed_kind(kind: &str) -> bool {
    matches!(
        kind,
        "external" | "payment" | "email" | "webhook" | "secrets" | "firewall"
    )
}

/// The source as the drawing reads it: a node whose text names no shape takes its kind's,
/// and a kind drawn as external gains the dashed style line it wears.
pub(super) fn styled_source(source: &str) -> String {
    let mut out = String::with_capacity(source.len() + 64);
    let mut dashed: Vec<String> = Vec::new();
    let styled: HashSet<String> = source
        .lines()
        .filter_map(|line| {
            let rest = line.trim_start().strip_prefix("style ")?;
            rest.split_whitespace().next().map(str::to_string)
        })
        .collect();
    for line in source.split_inclusive('\n') {
        let Some((id, open, close)) = node_block(line) else {
            out.push_str(line);
            continue;
        };
        let inner = &line[open + 1..close];
        let kind = field(inner, "kind");
        let mut line = line.to_string();
        if let Some(kind) = &kind
            && field(inner, "shape").is_none()
            && let Some(shape) = shape_for_kind(kind)
        {
            let head = line[..close].trim_end();
            line = format!("{head}, shape: {shape}{}", &line[close..]);
        }
        if let Some(kind) = &kind
            && dashed_kind(kind)
            && !styled.contains(&id)
        {
            dashed.push(id);
        }
        out.push_str(&line);
    }
    for id in dashed {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&format!("  style {id} stroke-dasharray: {DASHED_STROKE}\n"));
    }
    out
}

/// The canvas node line's block: its id and the byte range of the braces.
fn node_block(line: &str) -> Option<(String, usize, usize)> {
    let at = line.find("@{")?;
    let open = at + 1;
    let close = matching_brace(line, open)?;
    let id = id_before(line, at);
    (!id.is_empty()).then_some((id, open, close))
}

/// The palette the app draws a scene with, from its theme tokens.
pub(super) fn palette(theme: &Theme) -> Palette {
    let dark = luminance(theme.color_app_background) < 0.5;
    let mut overrides = serde_json::Map::new();
    let mut set = |key: &str, value: String| {
        overrides.insert(key.to_string(), serde_json::json!(value));
    };
    set("primaryColor", css(theme.color_surface_subtle));
    set("mainBkg", css(theme.color_surface));
    set("nodeBkg", css(theme.color_surface_subtle));
    set("primaryTextColor", css(theme.color_text_primary));
    set("textColor", css(theme.color_text_secondary));
    set("titleColor", css(theme.color_text_primary));
    set("primaryBorderColor", css(theme.color_text_muted));
    set("nodeBorder", css(theme.color_text_muted));
    set("lineColor", css(theme.color_text_muted));
    set("background", css(theme.color_app_background));
    set("secondaryColor", css(theme.color_surface_subtle));
    set("tertiaryColor", css(theme.color_surface_subtle));
    set("clusterBkg", css(theme.color_surface));
    set("clusterBorder", css(theme.color_text_muted));
    set("edgeLabelBackground", css(theme.color_surface_raised));
    set("actorBkg", css(theme.color_surface));
    set("actorBorder", css(theme.color_text_muted));
    set("actorTextColor", css(theme.color_text_primary));
    set("labelBoxBkgColor", css(theme.color_surface));
    set("labelBoxBorderColor", css(theme.color_text_muted));
    set("labelTextColor", css(theme.color_text_primary));
    set("fontFamily", theme.font_sans.to_string());
    set("fontSize", "13px".to_string());
    let mermaid = if dark {
        MermaidTheme::Dark
    } else {
        MermaidTheme::Base
    };
    let variables = mermaid.variables(&overrides).unwrap_or_default();
    let pick =
        |key: &str, fallback: String| variables.color(key).map(str::to_string).unwrap_or(fallback);
    Palette {
        node_fill: pick("primaryColor", css(theme.color_surface_subtle)),
        node_stroke: pick("primaryBorderColor", css(theme.color_text_muted)),
        text: pick("primaryTextColor", css(theme.color_text_primary)),
        line: pick("lineColor", css(theme.color_text_muted)),
        cluster_fill: pick("clusterBkg", css(theme.color_surface)),
        cluster_stroke: pick("clusterBorder", css(theme.color_text_muted)),
        label_background: pick("edgeLabelBackground", css(theme.color_surface_raised)),
        font_family: theme.font_sans.to_string(),
    }
}

/// A theme color as a CSS hex string.
pub(super) fn css(color: Rgba) -> String {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!(
        "#{:02x}{:02x}{:02x}",
        channel(color.r),
        channel(color.g),
        channel(color.b)
    )
}

fn luminance(color: Rgba) -> f32 {
    0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b
}

/// Replaces Mermaid's own theme colors with the app's, keeping colors a style line names.
fn apply_theme(scene: &mut Scene, palette: &Palette) {
    for item in &mut scene.items {
        theme_item(item, palette);
    }
    for marker in &mut scene.defs {
        for shape in &mut marker.shapes {
            theme_style(&mut shape.style, palette);
        }
    }
}

fn theme_item(item: &mut Item, palette: &Palette) {
    match item {
        Item::Group(group) => {
            for item in &mut group.items {
                theme_item(item, palette);
            }
        }
        Item::Rect(rect) => theme_style(&mut rect.style, palette),
        Item::Path(path) => theme_style(&mut path.style, palette),
        Item::Ellipse(ellipse) => theme_style(&mut ellipse.style, palette),
        Item::Text(text) => theme_text(text, palette),
        Item::Image(_) => {}
    }
}

fn theme_style(style: &mut polterware_mermaid::scene::PathStyle, palette: &Palette) {
    if let Some(fill) = &style.fill {
        style.fill = mapped_fill(&fill.0, palette);
    }
    if let Some(stroke) = &style.stroke {
        style.stroke = mapped_stroke(&stroke.0, palette);
    }
}

fn mapped_fill(css: &str, palette: &Palette) -> Option<Color> {
    let lower = css.trim().to_ascii_lowercase();
    let mapped = match lower.as_str() {
        MERMAID_FILL | "white" | "#ffffff" => palette.node_fill.clone(),
        MERMAID_SECOND_FILL => palette.cluster_fill.clone(),
        MERMAID_LABEL_BACKGROUND | "rgba(232,232,232,0.8)" => palette.label_background.clone(),
        "none" | "transparent" => return Some(Color::new(css.trim())),
        _ => css.to_string(),
    };
    Some(Color::new(mapped))
}

fn mapped_stroke(css: &str, palette: &Palette) -> Option<Color> {
    let lower = css.trim().to_ascii_lowercase();
    let mapped = match lower.as_str() {
        MERMAID_STROKE | "black" => palette.node_stroke.clone(),
        MERMAID_CLUSTER_STROKE => palette.cluster_stroke.clone(),
        MERMAID_LINE => palette.line.clone(),
        "none" | "transparent" => return Some(Color::new(css.trim())),
        _ => css.to_string(),
    };
    Some(Color::new(mapped))
}

fn theme_text(text: &mut TextRun, palette: &Palette) {
    let default = match text.fill.as_ref() {
        None => true,
        Some(fill) => matches!(
            fill.0.trim().to_ascii_lowercase().as_str(),
            "#333" | "#333333" | "#131300" | "black" | "#000021"
        ),
    };
    if default {
        text.fill = Some(Color::new(palette.text.clone()));
    }
}

/// Puts every label in the app's UI font.
///
/// The renderer asks for a family of its own (a monospace one) and bakes it into each run; the
/// canvas reads in the font the rest of the app uses, so the family is replaced here, after the
/// layout. A code span keeps its weight but not its own face, so the measure and the painter
/// shape the same runs.
fn theme_fonts(items: &mut [Item], palette: &Palette) {
    for item in items {
        match item {
            Item::Group(group) => theme_fonts(&mut group.items, palette),
            Item::Text(text) => {
                text.font.family = palette.font_family.clone();
                for line in &mut text.lines {
                    for span in &mut line.spans {
                        span.code = false;
                    }
                }
            }
            _ => {}
        }
    }
}

/// Lays a diagram out in the app's UI font.
///
/// The renderer measures through this while it asks for its own family; mapping every request
/// to the family the canvas paints makes a label wrap where it is drawn, so its box stays
/// centered on its shape.
struct UiFontMeasure<'a> {
    inner: &'a dyn TextMeasure,
    family: &'a str,
}

impl UiFontMeasure<'_> {
    fn font(&self, font: &Font) -> Font {
        Font {
            family: self.family.to_string(),
            ..font.clone()
        }
    }
}

impl TextMeasure for UiFontMeasure<'_> {
    fn width(&self, text: &str, font: &Font) -> f64 {
        self.inner.width(text, &self.font(font))
    }

    fn line_height(&self, font: &Font) -> f64 {
        self.inner.line_height(&self.font(font))
    }
}

/// The `style` and `classDef`/`class` declarations of a source, by node id.
fn css_styles(source: &str) -> HashMap<String, CssStyle> {
    let mut classes: HashMap<String, CssStyle> = HashMap::new();
    let mut members: Vec<(String, String)> = Vec::new();
    let mut styles: HashMap<String, CssStyle> = HashMap::new();
    for line in source.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("classDef ") {
            let mut parts = rest.splitn(2, char::is_whitespace);
            if let Some(name) = parts.next() {
                let style = parts.next().map(CssStyle::parse).unwrap_or_default();
                classes.insert(name.to_string(), style);
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("class ") {
            let mut parts = rest.splitn(2, char::is_whitespace);
            if let (Some(ids), Some(name)) = (parts.next(), parts.next()) {
                for id in ids.split(',') {
                    members.push((id.trim().to_string(), name.trim().to_string()));
                }
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("style ") {
            let mut parts = rest.splitn(2, char::is_whitespace);
            if let Some(id) = parts.next() {
                let style = parts.next().map(CssStyle::parse).unwrap_or_default();
                styles.insert(id.to_string(), style);
            }
        }
    }
    let mut merged: HashMap<String, CssStyle> = HashMap::new();
    for (id, name) in members {
        let style = classes.get(&name).cloned().unwrap_or_else(|| {
            // The export marks unverified nodes as dashed; that class has no definition.
            if name == "unverified" {
                CssStyle {
                    dash: Some(DASHED_STROKE.to_string()),
                    ..CssStyle::default()
                }
            } else {
                CssStyle::default()
            }
        });
        merged.entry(id).or_default().merge(style);
    }
    // A style line is written for one node and wins over its class.
    for (id, style) in styles {
        merged.entry(id).or_default().merge(style);
    }
    merged
}

/// A node's CSS-style declaration, as a `style` line or a class carries it.
#[derive(Clone, Debug, Default, PartialEq)]
struct CssStyle {
    fill: Option<String>,
    stroke: Option<String>,
    width: Option<f64>,
    dash: Option<String>,
}

impl CssStyle {
    fn parse(text: &str) -> Self {
        let mut style = Self::default();
        for part in text.split(',') {
            let Some((name, value)) = part.split_once(':') else {
                continue;
            };
            let value = value.trim().to_string();
            match name.trim() {
                "fill" => style.fill = Some(value),
                "stroke" => style.stroke = Some(value),
                "stroke-width" => {
                    style.width = value.trim_end_matches("px").trim().parse().ok();
                }
                "stroke-dasharray" => style.dash = Some(value),
                _ => {}
            }
        }
        style
    }

    fn merge(&mut self, other: Self) {
        if other.fill.is_some() {
            self.fill = other.fill;
        }
        if other.stroke.is_some() {
            self.stroke = other.stroke;
        }
        if other.width.is_some() {
            self.width = other.width;
        }
        if other.dash.is_some() {
            self.dash = other.dash;
        }
    }
}

/// Paints each `style` line onto its node's shapes, over the theme.
fn apply_styles(scene: &mut Scene, styles: &HashMap<String, CssStyle>) {
    if styles.is_empty() {
        return;
    }
    apply_item_styles(&mut scene.items, styles);
}

fn apply_item_styles(items: &mut [Item], styles: &HashMap<String, CssStyle>) {
    for item in items {
        let Item::Group(group) = item else {
            continue;
        };
        if let Some(source) = &group.source
            && source.kind == SourceKind::Node
            && let Some(style) = styles.get(&source.id)
        {
            for inner in &mut group.items {
                paint_style(inner, style);
            }
        }
        apply_item_styles(&mut group.items, styles);
    }
}

fn paint_style(item: &mut Item, style: &CssStyle) {
    let target = match item {
        Item::Group(group) => {
            for item in &mut group.items {
                paint_style(item, style);
            }
            return;
        }
        Item::Rect(rect) => &mut rect.style,
        Item::Path(path) => &mut path.style,
        Item::Ellipse(ellipse) => &mut ellipse.style,
        _ => return,
    };
    if let Some(fill) = &style.fill {
        target.fill = Some(Color::new(fill.clone()));
    }
    if let Some(stroke) = &style.stroke {
        target.stroke = Some(Color::new(stroke.clone()));
    }
    if let Some(width) = style.width {
        target.stroke_width = width;
    }
    if let Some(dash) = &style.dash {
        target.dash = dash
            .split_whitespace()
            .filter_map(|value| value.parse::<f64>().ok())
            .collect();
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ComponentCard {
    kind: String,
    title: String,
    subtitle: String,
    domain: bool,
}

/// Reads the visible label, kind, and technology for semantic component and Structure nodes.
fn component_cards(source: &str, entities: &[StoreEntity]) -> HashMap<String, ComponentCard> {
    let mut cards = HashMap::new();
    for line in source.split_inclusive('\n') {
        let Some((id, open, close)) = node_block(line) else {
            continue;
        };
        let metadata = &line[open + 1..close];
        let Some(raw_kind) =
            field(metadata, "kind").filter(|kind| component_of(kind).is_some() || structure_kind(kind))
        else {
            continue;
        };
        let reference = field(metadata, "ref").and_then(|raw| {
            let (kind, ref_id) = raw.split_once('/')?;
            (!kind.trim().is_empty() && !ref_id.trim().is_empty()).then(|| CanvasRef {
                kind: kind.trim().to_string(),
                id: ref_id.trim().to_string(),
            })
        });
        let title = if let Some(reference) = &reference {
            reference_state(Some(reference), entities).title
        } else {
            let Some(title) = bracket_label(line, open)
                .or_else(|| field(metadata, "label"))
                .filter(|title| !title.is_empty())
            else {
                continue;
            };
            title
        };
        let probe = Node {
            id: id.clone(),
            name: title.clone(),
            kind: raw_kind.clone(),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            z: 0,
            props: Props::new(),
            reference,
        };
        let linked = linked_facts(&probe, entities);
        let kind = linked
            .as_ref()
            .and_then(|facts| facts.kind.clone())
            .unwrap_or_else(|| raw_kind.clone());
        let technology = linked
            .and_then(|facts| facts.tech)
            .or_else(|| {
                field(metadata, "technology")
                    .or_else(|| field(metadata, "tech"))
                    .or_else(|| {
                        field(metadata, "props")
                            .and_then(|props| serde_json::from_str::<serde_json::Value>(&props).ok())
                            .and_then(|props| {
                                props
                                    .get("technology")
                                    .or_else(|| props.get("tech"))?
                                    .as_str()
                                    .map(str::to_string)
                            })
                    })
                    .filter(|technology| !technology.is_empty())
            });
        let type_label = component_of(&kind)
            .map(|item| item.label.to_string())
            .unwrap_or_else(|| kind_label(&raw_kind));
        let subtitle = technology.map_or(type_label.clone(), |technology| {
            format!("{type_label} / {technology}")
        });
        cards.insert(
            id,
            ComponentCard {
                kind,
                title,
                subtitle,
                domain: raw_kind == "domain",
            },
        );
    }
    cards
}

/// Finds a square-bracket flowchart label before its metadata block and returns its content.
fn bracket_label(line: &str, metadata_start: usize) -> Option<String> {
    let prefix = &line[..metadata_start];
    let open = prefix.find('[')?;
    let close = prefix.rfind(']')?;
    if close <= open {
        return None;
    }
    let content = prefix[open + 1..close].trim();
    let title = content
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .or_else(|| {
            content
                .strip_prefix('\'')
                .and_then(|text| text.strip_suffix('\''))
        })
        .unwrap_or(content)
        .replace("&quot;", "\"");
    Some(title)
}

/// Resizes component shapes after Mermaid lays them out, then reroutes their attached edges
/// against the new visible bounds. Mermaid's label wrapping caps some silhouettes before the
/// card's icon gutter can be reserved by the source label alone.
fn resize_component_nodes(
    items: &mut [Item],
    cards: &HashMap<String, ComponentCard>,
    measure: &dyn TextMeasure,
    palette: &Palette,
) -> HashSet<String> {
    let mut resized = HashSet::new();
    let title_font = Font {
        family: palette.font_family.clone(),
        size: 13.0,
        weight: 500,
        italic: false,
    };
    let subtitle_font = Font::new(palette.font_family.clone(), 11.0);
    let mut deltas = HashMap::new();
    for (id, card) in cards {
        let Some(current) = node_shape_bounds(items, id) else {
            continue;
        };
        let min_height = if card.domain {
            DOMAIN_MIN_HEIGHT
        } else {
            CARD_MIN_HEIGHT
        };
        let content = measure
            .width(&card.title, &title_font)
            .max(measure.width(&card.subtitle, &subtitle_font));
        let inset = card_inset(&card.kind, current.width, min_height.max(current.height));
        let width = (content
            + 2.0
            + CARD_PAD_LEFT
            + BADGE_SIZE
            + CARD_GAP
            + CARD_PAD_RIGHT
            + f64::from(inset.left)
            + f64::from(inset.right))
        .max(current.width);
        let height = (min_height + f64::from(inset.top) + f64::from(inset.bottom)).max(current.height);
        if width > current.width || height > current.height {
            resize_node(items, id, Some(width), Some(height));
            // The resize grows about the center; the box keeps its top-left, where a pin put it.
            deltas.insert(
                id.clone(),
                Point::new(
                    (width - current.width) / 2.0,
                    (height - current.height) / 2.0,
                ),
            );
            resized.insert(id.clone());
        }
    }
    shift_nodes(items, Transform::IDENTITY, &deltas);
    resized
}

/// A card's two text lines and its icon badge fit inside this height.
const CARD_MIN_HEIGHT: f64 = 52.0;
const DOMAIN_MIN_HEIGHT: f64 = 64.0;
/// The library card's spacing (`scene.rs`'s semantic card): the badge, the padding
/// before it and after the text, and the gap between them.
const BADGE_SIZE: f64 = 28.0;
const CARD_PAD_LEFT: f64 = 10.0;
const CARD_PAD_RIGHT: f64 = 12.0;
const CARD_GAP: f64 = 10.0;

/// Where a card's content sits clear of its silhouette's details.
fn card_inset(kind: &str, width: f64, height: f64) -> polterware_ui::canvas::shapes::Inset {
    silhouette(kind)
        .map(|family| (family.inset)(width as f32, height as f32))
        .unwrap_or_default()
}

fn node_shape_bounds(items: &[Item], id: &str) -> Option<Rect> {
    items.iter().find_map(|item| {
        let Item::Group(group) = item else {
            return None;
        };
        if group
            .source
            .as_ref()
            .is_some_and(|source| source.kind == SourceKind::Node && source.id == id)
        {
            shape_bounds(&group.items)
        } else {
            node_shape_bounds(&group.items, id)
        }
    })
}

/// Replaces component-node visuals with an unpainted hit-target box at their laid-out bounds:
/// the editor draws semantic cards via `scene::node_view` so they wear the exact lit material
/// of the shape library.
fn paint_component_cards(
    items: &mut [Item],
    cards: &HashMap<String, ComponentCard>,
    rendered_cards: &mut HashSet<String>,
) {
    for item in items {
        let Item::Group(group) = item else {
            continue;
        };
        if let Some(source) = group
            .source
            .as_ref()
            .filter(|source| source.kind == SourceKind::Node)
            && cards.contains_key(&source.id)
            && let Some(bounds) = shape_bounds(&group.items)
        {
            group.items.clear();
            group.items.push(Item::Rect(
                RectShape::new(
                    bounds,
                    0.0,
                    PathStyle {
                        fill: None,
                        ..PathStyle::default()
                    },
                )
                .with_source(SourceRef::node(source.id.clone())),
            ));
            rendered_cards.insert(source.id.clone());
        }
        paint_component_cards(&mut group.items, cards, rendered_cards);
    }
}

/// The laid-out Mermaid scene and the node ids drawn as semantic cards over it.
#[derive(Debug)]
pub(super) struct Rendered {
    pub scene: Scene,
    pub cards: HashSet<String>,
}

/// Renders `source` with the layout's pinned positions applied, in the app's colors.
pub(super) fn render(
    source: &str,
    layout_json: &str,
    entities: &[StoreEntity],
    measure: &dyn TextMeasure,
    palette: &Palette,
) -> Result<Rendered, RenderError> {
    let components = component_cards(source, entities);
    let styled = styled_source(source);
    let measure = UiFontMeasure {
        inner: measure,
        family: &palette.font_family,
    };
    let (mut scene, edges) = draw(&styled, &measure, palette)?;
    paint_scene_theme(&mut scene, &styled, &edges, &measure, palette);
    center_labels(&mut scene.items);
    apply_pins(&mut scene, &pins_for(&styled, layout_json));
    let resized = resize_component_nodes(&mut scene.items, &components, &measure, palette);
    let mut cards = HashSet::new();
    paint_component_cards(&mut scene.items, &components, &mut cards);
    if !resized.is_empty() {
        let mut boxes = HashMap::new();
        collect_node_boxes(&scene.items, Transform::IDENTITY, &mut boxes);
        reroute_edges(&mut scene.items, &boxes, &HashMap::new(), &resized);
    }
    paint_arrow_heads(&mut scene, &edges, palette);
    Ok(Rendered { scene, cards })
}

/// Renders `source` for a preview: no pins, in the app's colors.
pub(super) fn preview(source: &str, measure: &dyn TextMeasure, palette: &Palette) -> Option<Scene> {
    let styled = styled_source(source);
    let measure = UiFontMeasure {
        inner: measure,
        family: &palette.font_family,
    };
    let (mut scene, edges) = draw(&styled, &measure, palette).ok()?;
    paint_scene_theme(&mut scene, &styled, &edges, &measure, palette);
    center_labels(&mut scene.items);
    paint_arrow_heads(&mut scene, &edges, palette);
    Some(scene)
}

/// Puts the app's colors on a laid-out scene, its style lines over them, and finishes the
/// edges the flowchart pipeline leaves to the stylesheet.
fn paint_scene_theme(
    scene: &mut Scene,
    source: &str,
    edges: &Edges,
    measure: &dyn TextMeasure,
    palette: &Palette,
) {
    apply_theme(scene, palette);
    apply_styles(scene, &css_styles(source));
    theme_fonts(&mut scene.items, palette);
    paint_edges(scene, edges, measure, palette);
}

/// Lays a source out into a scene, as the renderer draws it, in the app's UI font.
///
/// A flowchart keeps the layout's edges: the pipeline pushes their paths with the stylesheet's
/// job undone (no stroke, no arrow ends, no label), so [`paint_edges`] completes them once the
/// app's colors are on the scene.
fn draw(
    source: &str,
    measure: &dyn TextMeasure,
    palette: &Palette,
) -> Result<(Scene, Edges), RenderError> {
    let preprocessed = polterware_mermaid::preprocess::preprocess(source);
    if let Some(message) = preprocessed.error {
        return Err(RenderError::Parse(ParseError::new(message, None)));
    }
    let kind = polterware_mermaid::diagrams::detect(&preprocessed.body)
        .ok_or_else(|| ParseError::new("No diagram type detected", None))?;
    let mut config = Config::default();
    config.apply(&preprocessed.config)?;
    config.font_family = palette.font_family.clone();
    match kind {
        DiagramKind::Flowchart => {
            let ast = flowchart::parse(&preprocessed.body)?;
            let (data, scene) = flowchart::render::draw_layout_ast(&ast, &config, measure)?;
            let items = data
                .edges
                .iter()
                .filter(|edge| edge.is_layout_only != Some(true))
                .map(EdgePaint::new)
                .collect();
            let edges = Edges {
                items,
                font_size: config.font_size,
            };
            Ok((scene, edges))
        }
        other => Err(RenderError::Unsupported(format!("{other:?}"))),
    }
}

/// The edges a flowchart laid out, and the label size the layout measured them with.
#[derive(Clone, Debug, PartialEq)]
struct Edges {
    items: Vec<EdgePaint>,
    font_size: f64,
}

/// What an edge needs that the flowchart pipeline leaves to the stylesheet: the line, its
/// arrow ends, and its label.
#[derive(Clone, Debug, PartialEq)]
struct EdgePaint {
    id: String,
    label: Option<String>,
    label_x: Option<f64>,
    label_y: Option<f64>,
    label_width: f64,
    label_height: f64,
    points: Vec<Point>,
    arrow_start: Option<String>,
    arrow_end: Option<String>,
    /// A color a `linkStyle` line names; the theme's line color otherwise.
    stroke: Option<String>,
    dashed: bool,
}

impl EdgePaint {
    fn new(edge: &polterware_mermaid::render::pipeline::types::Edge) -> Self {
        Self {
            id: edge.id.clone(),
            label: edge
                .label
                .as_deref()
                .map(polterware_mermaid::measure::decode_html_entities)
                .filter(|label| !label.is_empty()),
            label_x: edge.x,
            label_y: edge.y,
            label_width: edge.width.unwrap_or(0.0),
            label_height: edge.height.unwrap_or(0.0),
            points: edge.points.clone().unwrap_or_default(),
            arrow_start: edge.arrow_type_start.clone(),
            arrow_end: edge.arrow_type_end.clone(),
            stroke: edge.stroke.clone(),
            dashed: edge.pattern.as_deref() == Some("dashed"),
        }
    }
}

/// Gives each edge path its theme line and its arrow ends, then paints every edge's label.
///
/// The pipeline pushes one bare, unstyled [`Item::Path`] per layout edge, in edge order, so the
/// path and its edge line up by position. The paths are filled with `currentColor` and have no
/// stroke, and the painter draws a stroke only for a path that has one, so without this pass an
/// edge paints as an empty black area: nothing at all.
fn paint_edges(scene: &mut Scene, edges: &Edges, measure: &dyn TextMeasure, palette: &Palette) {
    if edges.items.is_empty() {
        return;
    }
    let mut index = 0;
    for item in &mut scene.items {
        let Item::Path(path) = item else {
            continue;
        };
        let Some(edge) = edges.items.get(index) else {
            break;
        };
        index += 1;
        paint_edge_path(path, edge, palette);
    }
    let mut labels: Vec<Item> = Vec::new();
    for edge in &edges.items {
        paint_edge_label(
            edge,
            edge_stroke(edge, palette),
            measure,
            edges.font_size,
            palette,
            &mut labels,
        );
    }
    scene.items.extend(labels);
}

/// The color an edge's line, arrow ends, and label wear.
fn edge_stroke(edge: &EdgePaint, palette: &Palette) -> Color {
    Color::new(edge.stroke.clone().unwrap_or_else(|| palette.line.clone()))
}

/// Puts an edge's line and its dash on its path.
///
/// A point arrowhead is not left to the marker list: the painter strokes every marker path as
/// it is defined upstream (a hairline with a `1,0` dash), which reads as a few dots instead of
/// Mermaid's solid triangle. [`paint_arrow_heads`] draws those, filled, once the pins have
/// placed the line.
fn paint_edge_path(
    path: &mut polterware_mermaid::scene::PathShape,
    edge: &EdgePaint,
    palette: &Palette,
) {
    let stroke = edge_stroke(edge, palette);
    path.style = PathStyle::stroked(stroke.clone(), EDGE_WIDTH);
    if edge.dashed {
        path.style.dash = EDGE_DASH.to_vec();
    }
    path.source = Some(SourceRef::edge(edge.id.clone()));
    let Ok(placement) = marker_placement(
        edge.arrow_start
            .as_deref()
            .filter(|value| !value.is_empty()),
        edge.arrow_end.as_deref().filter(|value| !value.is_empty()),
        flowchart::layout_data::DEFAULT_DIAGRAM_ID,
        flowchart::DIAGRAM_TYPE,
        false,
    ) else {
        return;
    };
    if let Some(marker) = placement.start
        && !wants_filled_head(edge.arrow_start.as_deref())
    {
        path.start_markers.push(marker.with_color(stroke.clone()));
    }
    if let Some(marker) = placement.end
        && !wants_filled_head(edge.arrow_end.as_deref())
    {
        path.end_markers.push(marker.with_color(stroke));
    }
}

/// Whether an arrow type is Mermaid's `point`: the filled triangle this painter draws itself.
fn wants_filled_head(arrow: Option<&str>) -> bool {
    arrow == Some("arrow_point")
}

/// Draws every point arrowhead as Mermaid's `pointEnd` marker: a filled triangle in the line's
/// color at the marker's real size, along the last segment, its tip on the target node's
/// border.
///
/// Runs after the pins, so the head sits on the line as it is finally drawn, and appends the
/// triangles to the scene; the edge lines are read first, because painting adds items.
fn paint_arrow_heads(scene: &mut Scene, edges: &Edges, palette: &Palette) {
    let mut heads: Vec<Item> = Vec::new();
    for item in &scene.items {
        let Item::Path(path) = item else {
            continue;
        };
        let Some(source) = path
            .source
            .as_ref()
            .filter(|source| source.kind == SourceKind::Edge)
        else {
            continue;
        };
        let Some(edge) = edges.items.iter().find(|edge| edge.id == source.id) else {
            continue;
        };
        let stroke = edge_stroke(edge, palette);
        for (arrow, at_start) in [
            (edge.arrow_start.as_deref(), true),
            (edge.arrow_end.as_deref(), false),
        ] {
            if !wants_filled_head(arrow) {
                continue;
            }
            let Some((at, tangent)) = line_end(&path.path, at_start) else {
                continue;
            };
            let Some(definition) = point_marker(scene, at_start) else {
                continue;
            };
            let angle = tangent.y.atan2(tangent.x).to_degrees();
            if let Some(head) = point_arrow(
                definition,
                at,
                angle,
                path.style.stroke_width,
                stroke.clone(),
                source.clone(),
            ) {
                heads.push(head);
            }
        }
    }
    scene.items.extend(heads);
}

/// The `point` marker definition of a scene, for an arrow at the start or the end of a line.
fn point_marker(scene: &Scene, at_start: bool) -> Option<&MarkerDef> {
    let placement = marker_placement(
        at_start.then_some("arrow_point"),
        (!at_start).then_some("arrow_point"),
        flowchart::layout_data::DEFAULT_DIAGRAM_ID,
        flowchart::DIAGRAM_TYPE,
        false,
    )
    .ok()?;
    let marker = if at_start {
        placement.start
    } else {
        placement.end
    }?;
    scene.marker_def(&marker.def)
}

/// Where a line ends (or starts) and the unit direction it travels there.
fn line_end(path: &Path, at_start: bool) -> Option<(Point, Point)> {
    let points: Vec<Point> = path.commands.iter().filter_map(command_end).collect();
    let (at, before) = if at_start {
        (points.first()?, points.get(1)?)
    } else {
        (points.last()?, points.get(points.len().checked_sub(2)?)?)
    };
    let dx = if at_start {
        before.x - at.x
    } else {
        at.x - before.x
    };
    let dy = if at_start {
        before.y - at.y
    } else {
        at.y - before.y
    };
    let length = dx.hypot(dy);
    (length > 1e-9).then(|| (*at, Point::new(dx / length, dy / length)))
}

/// A point arrowhead in scene coordinates: the marker's own triangle, scaled to its size and
/// reference point, turned along the line, and filled with the edge's color.
fn point_arrow(
    definition: &MarkerDef,
    at: Point,
    angle: f64,
    stroke_width: f64,
    color: Color,
    source: SourceRef,
) -> Option<Item> {
    let view_box = definition.view_box;
    if view_box.width <= 0.0 || view_box.height <= 0.0 {
        return None;
    }
    let units = match definition.units {
        MarkerUnits::StrokeWidth => stroke_width.max(0.0),
        MarkerUnits::UserSpaceOnUse => 1.0,
    };
    if units <= 0.0 {
        return None;
    }
    let scale_x = units * definition.width / view_box.width;
    let scale_y = units * definition.height / view_box.height;
    let world = Transform::translate(-view_box.x, -view_box.y)
        .then(Transform::scale(scale_x, scale_y))
        .then(Transform::translate(
            -(definition.reference.x - view_box.x) * scale_x,
            -(definition.reference.y - view_box.y) * scale_y,
        ))
        .then(Transform::rotate(angle))
        .then(Transform::translate(at.x, at.y));
    let shape = definition.shapes.first()?;
    Some(Item::Path(PathShape {
        source: Some(source),
        path: transformed_path(&shape.path, world),
        style: PathStyle::filled(color),
        ..PathShape::default()
    }))
}

/// A path moved into another coordinate system, every point of it.
fn transformed_path(path: &Path, world: Transform) -> Path {
    let mut out = Path::new();
    for command in &path.commands {
        match *command {
            PathCommand::MoveTo { x, y } => {
                out.move_to(world.apply(Point::new(x, y)));
            }
            PathCommand::LineTo { x, y } => {
                out.line_to(world.apply(Point::new(x, y)));
            }
            PathCommand::CubicTo {
                c1x,
                c1y,
                c2x,
                c2y,
                x,
                y,
            } => {
                out.cubic_to(
                    world.apply(Point::new(c1x, c1y)),
                    world.apply(Point::new(c2x, c2y)),
                    world.apply(Point::new(x, y)),
                );
            }
            PathCommand::QuadTo { cx, cy, x, y } => {
                out.quad_to(
                    world.apply(Point::new(cx, cy)),
                    world.apply(Point::new(x, y)),
                );
            }
            PathCommand::ArcTo {
                rx,
                ry,
                rotation,
                large_arc,
                sweep,
                x,
                y,
            } => {
                out.arc_to(
                    rx,
                    ry,
                    rotation,
                    large_arc,
                    sweep,
                    world.apply(Point::new(x, y)),
                );
            }
            PathCommand::Close => {
                out.close();
            }
        }
    }
    out
}

/// Paints an edge's label over its line, on the theme's label background.
fn paint_edge_label(
    edge: &EdgePaint,
    stroke: Color,
    measure: &dyn TextMeasure,
    font_size: f64,
    palette: &Palette,
    labels: &mut Vec<Item>,
) {
    let Some(label) = edge.label.as_deref() else {
        return;
    };
    let data = EdgeData {
        id: edge.id.clone(),
        points: edge.points.clone(),
        label: Some(label.to_string()),
        label_width: edge.label_width,
        label_height: edge.label_height,
        label_x: edge.label_x,
        label_y: edge.label_y,
        ..EdgeData::default()
    };
    let font = Font {
        family: palette.font_family.clone(),
        size: font_size,
        weight: NORMAL_WEIGHT,
        italic: false,
    };
    for placed in insert_edge_label(&data) {
        let bounds = placed.position.bounds;
        if bounds.width <= 0.0 || bounds.height <= 0.0 {
            continue;
        }
        let source = SourceRef::edge(edge.id.clone());
        let mut group = Group::new()
            .with_class("edgeLabel")
            .with_source(source.clone());
        group.transform = Transform::translate(bounds.x, bounds.y);
        if placed.background.is_some() {
            group.push(RectShape::new(
                Rect::new(0.0, 0.0, bounds.width, bounds.height),
                0.0,
                PathStyle {
                    fill: Some(Color::new(palette.label_background.clone())),
                    stroke: None,
                    ..PathStyle::default()
                },
            ));
        }
        let block = plain_text_block(&placed.text, &font, measure);
        let mut text = TextRun::centered(
            Point::new(bounds.width / 2.0, bounds.height / 2.0),
            font.clone(),
            block,
        );
        text.baseline = TextBaseline::Middle;
        text.fill = Some(stroke.clone());
        text.source = Some(source);
        group.push(Item::Text(text));
        labels.push(Item::Group(group));
    }
}

/// The pins the layout holds, by both the diagram id and the canvas id the source gives a node.
fn pins_for(source: &str, layout_json: &str) -> HashMap<String, Pin> {
    let mut pinned = pins(layout_json);
    let aliases: Vec<(String, String)> = node_ids(source)
        .into_iter()
        .filter(|(diagram, canvas)| diagram != canvas && pinned.contains_key(canvas))
        .collect();
    for (diagram, canvas) in aliases {
        if let Some(point) = pinned.get(&canvas).copied() {
            pinned.entry(diagram).or_insert(point);
        }
    }
    pinned
}

/// What a person reads when the source does not draw: the parser's place, or the renderer's
/// own answer.
pub(super) fn message(error: &RenderError) -> String {
    match error {
        RenderError::Parse(parse) => match parse.span {
            Some(span) => format!(
                "line {}, column {}: {}",
                span.line, span.column, parse.message
            ),
            None => parse.message.clone(),
        },
        other => other.to_string(),
    }
}

/// Whether the source is a diagram type this renderer does not draw yet.
pub(super) fn unsupported(error: &RenderError) -> bool {
    matches!(error, RenderError::Unsupported(_))
}

/// The pan and zoom the layout remembers, when it has them.
pub(super) fn viewport(layout_json: &str) -> Option<SceneViewport> {
    let layout: serde_json::Value = serde_json::from_str(layout_json).ok()?;
    let view = layout.get("viewport")?.as_object()?;
    let number = |key: &str| view.get(key).and_then(serde_json::Value::as_f64);
    let (x, y, zoom) = (number("x")?, number("y")?, number("zoom")?);
    let (x, y, zoom) = (x as f32, y as f32, zoom as f32);
    (x.is_finite() && y.is_finite() && zoom.is_finite()).then(|| SceneViewport::new(x, y, zoom))
}

/// Fits the drawing inside the free rectangle of the full canvas surface.
pub(super) fn fit_viewport_in_area(
    view_box: Rect,
    surface: (f32, f32),
    insets: (f32, f32, f32, f32),
    padding: f32,
) -> SceneViewport {
    let (left, right, top, bottom) = insets;
    let (width, height) = surface;
    let left = left.max(0.0).min(width.max(0.0));
    let right = right.max(0.0).min((width - left).max(0.0));
    let top = top.max(0.0).min(height.max(0.0));
    let bottom = bottom.max(0.0).min((height - top).max(0.0));
    let mut fitted = SceneViewport::fit(
        view_box,
        (width - left - right).max(1.0),
        (height - top - bottom).max(1.0),
        padding,
    );
    fitted.x += left;
    fitted.y += top;
    fitted
}

/// Converts between full-surface and free-area viewport coordinates.
pub(super) fn viewport_at_free_origin(
    viewport: SceneViewport,
    left: f32,
    top: f32,
) -> SceneViewport {
    SceneViewport::new(viewport.x - left, viewport.y - top, viewport.zoom)
}

pub(super) fn viewport_from_free_origin(
    viewport: SceneViewport,
    left: f32,
    top: f32,
) -> SceneViewport {
    SceneViewport::new(viewport.x + left, viewport.y + top, viewport.zoom)
}

/// The fraction of the transformed scene bounds that falls in the unobscured area.
pub(super) fn visible_fraction(view_box: Rect, viewport: SceneViewport, free_area: Rect) -> f32 {
    let left = view_box.x as f32 * viewport.zoom + viewport.x;
    let top = view_box.y as f32 * viewport.zoom + viewport.y;
    let right = (view_box.x + view_box.width) as f32 * viewport.zoom + viewport.x;
    let bottom = (view_box.y + view_box.height) as f32 * viewport.zoom + viewport.y;
    let width = (right - left).max(0.0);
    let height = (bottom - top).max(0.0);
    let area = width * height;
    if area <= 0.0 {
        return 0.0;
    }
    let visible_width =
        (right.min(free_area.right() as f32) - left.max(free_area.x as f32)).max(0.0);
    let visible_height =
        (bottom.min(free_area.bottom() as f32) - top.max(free_area.y as f32)).max(0.0);
    (visible_width * visible_height / area).clamp(0.0, 1.0)
}

pub(super) fn restored_view_needs_fit(
    view_box: Rect,
    viewport: SceneViewport,
    free_area: Rect,
) -> bool {
    visible_fraction(view_box, viewport, free_area) < 0.5
}

/// The pan and zoom a canvas takes so the selected node stays in the room it has.
///
/// A diagram that fits the room is fitted again: opening the inspector narrows the room, and a
/// fit centers the whole drawing in what is left, so no other node leaves the view. A diagram
/// too large for the room is panned as little as it takes to bring the node inside the margins.
/// The view that is already right is `None`, so a click on a node that shows moves nothing.
pub(super) fn reveal_view(
    current: SceneViewport,
    view_box: Rect,
    room: (f32, f32),
    focus: Rect,
    margin: f32,
) -> Option<SceneViewport> {
    let (width, height) = room;
    if !(width > 0.0 && height > 0.0) || current.zoom <= 0.0 {
        return None;
    }
    let zoom = current.zoom;
    let fits = view_box.width as f32 * zoom <= (width - margin * 2.0).max(1.0)
        && view_box.height as f32 * zoom <= (height - margin * 2.0).max(1.0);
    let fitted = SceneViewport::fit(view_box, width, height, margin);
    // A drawing that fits, or that fits once zoomed out a little (to no less than
    // `REVEAL_MIN_ZOOM`), is fitted whole into the room, so no node ends up under a panel; only
    // a drawing too large for that pans to the selected node.
    if fits || (fitted.zoom >= REVEAL_MIN_ZOOM && fitted.zoom < zoom) {
        return (fitted != current).then_some(fitted);
    }
    let left = focus.x as f32 * zoom + current.x;
    let top = focus.y as f32 * zoom + current.y;
    let right = left + focus.width as f32 * zoom;
    let bottom = top + focus.height as f32 * zoom;
    let dx = if right > width - margin {
        width - margin - right
    } else if left < margin {
        margin - left
    } else {
        0.0
    };
    let dy = if bottom > height - margin {
        height - margin - bottom
    } else if top < margin {
        margin - top
    } else {
        0.0
    };
    if dx.abs() < 0.5 && dy.abs() < 0.5 {
        return None;
    }
    Some(SceneViewport::new(current.x + dx, current.y + dy, zoom))
}

/// `layout_json` with `viewport` written in, keeping every node's pin and every other field.
pub(super) fn with_viewport(layout_json: &str, viewport: SceneViewport) -> String {
    let mut layout = serde_json::from_str::<serde_json::Value>(layout_json)
        .ok()
        .filter(serde_json::Value::is_object)
        .unwrap_or_else(|| serde_json::json!({}));
    if let Some(fields) = layout.as_object_mut() {
        fields.insert("version".into(), serde_json::json!(1));
        fields.insert(
            "viewport".into(),
            serde_json::json!({
                "x": f64::from(viewport.x),
                "y": f64::from(viewport.y),
                "zoom": f64::from(viewport.zoom),
            }),
        );
    }
    layout.to_string()
}

/// Every `@{ ... }` block in the source, by the id in front of it, each field unquoted.
pub(super) fn blocks(source: &str) -> HashMap<String, HashMap<String, String>> {
    let bytes = source.as_bytes();
    let mut found = HashMap::new();
    let mut index = 0;
    while index + 1 < bytes.len() {
        if bytes[index] != b'@' || bytes[index + 1] != b'{' {
            index += 1;
            continue;
        }
        let Some(close) = matching_brace(source, index + 1) else {
            break;
        };
        let id = id_before(source, index);
        if !id.is_empty() {
            let inner = &source[index + 2..close];
            let mut fields = HashMap::new();
            for part in split_top_level(inner) {
                let Some((name, _)) = part.split_once(':') else {
                    continue;
                };
                let name = name.trim();
                if name.is_empty() {
                    continue;
                }
                fields.insert(name.to_string(), field(inner, name).unwrap_or_default());
            }
            found.insert(id, fields);
        }
        index = close + 1;
    }
    found
}

/// The canvas node each diagram id stands for: the export gives a node whose id is not a
/// Mermaid id (`u1`) an `@{ id: ... }` block carrying the canvas's own, and a click in the
/// diagram has to follow it to the inspector.
pub(super) fn node_ids(source: &str) -> HashMap<String, String> {
    blocks(source)
        .into_iter()
        .filter_map(|(id, fields)| {
            let canvas = fields.get("id")?;
            (!canvas.is_empty() && canvas != &id).then(|| (id, canvas.clone()))
        })
        .collect()
}

/// The box each node's drawing covers, by diagram id, in scene coordinates.
pub(super) fn node_boxes(scene: &Scene) -> HashMap<String, (f32, f32, f32, f32)> {
    let mut boxes = HashMap::new();
    collect_node_boxes(&scene.items, Transform::IDENTITY, &mut boxes);
    boxes
        .into_iter()
        .map(|(id, rect)| {
            (
                id,
                (
                    rect.x as f32,
                    rect.y as f32,
                    rect.width as f32,
                    rect.height as f32,
                ),
            )
        })
        .collect()
}

/// The byte of the `}` that closes the `{` at `start`, skipping braces inside quoted text.
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
                    depth = depth.saturating_sub(1);
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

/// The id in front of an `@{ }` block: the last id on the line once quoted text and bracket
/// labels are skipped.
fn id_before(source: &str, at: usize) -> String {
    let start = source[..at].rfind('\n').map_or(0, |index| index + 1);
    let mut kept = String::new();
    let bytes = &source.as_bytes()[start..at];
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'"' | b'\'' => {
                let quote = bytes[index];
                index += 1;
                while index < bytes.len() && bytes[index] != quote {
                    index += if bytes[index] == b'\\' { 2 } else { 1 };
                }
                kept.push(' ');
                index += 1;
            }
            open @ (b'[' | b'(' | b'{') => {
                let close = match open {
                    b'[' => b']',
                    b'(' => b')',
                    _ => b'}',
                };
                index += 1;
                while index < bytes.len() && bytes[index] != close {
                    index += 1;
                }
                kept.push(' ');
                index += 1;
            }
            byte => {
                kept.push(if byte.is_ascii() { byte as char } else { ' ' });
                index += 1;
            }
        }
    }
    let kept = kept.trim_end();
    let start = kept.len().saturating_sub(
        kept.bytes()
            .rev()
            .take_while(|byte| is_id_byte(*byte))
            .count(),
    );
    kept[start..].to_string()
}

fn is_id_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}

/// A `key: value` field of a metadata block, unquoted.
fn field(block: &str, key: &str) -> Option<String> {
    for part in split_top_level(block) {
        let Some((name, value)) = part.split_once(':') else {
            continue;
        };
        if name.trim() != key {
            continue;
        }
        let value = value.trim();
        let unquoted = value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .map(|value| value.replace("\\\"", "\"").replace("\\\\", "\\"))
            .unwrap_or_else(|| value.to_string());
        return Some(unquoted);
    }
    None
}

/// The parts of a metadata block between its top-level commas.
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
    parts.push(&text[start..]);
    parts
}

/// A node's remembered place in the layout: its top-left, and the box size a person gave it.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Pin {
    x: f64,
    y: f64,
    width: Option<f64>,
    height: Option<f64>,
}

/// The nodes the layout pins, by the id the source gives them.
fn pins(layout_json: &str) -> HashMap<String, Pin> {
    let Ok(layout) = serde_json::from_str::<serde_json::Value>(layout_json) else {
        return HashMap::new();
    };
    let Some(nodes) = layout.get("nodes").and_then(serde_json::Value::as_object) else {
        return HashMap::new();
    };
    nodes
        .iter()
        .filter_map(|(id, value)| {
            let position = value.as_object()?;
            if !position
                .get("pinned")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                return None;
            }
            let x = position.get("x").and_then(serde_json::Value::as_f64)?;
            let y = position.get("y").and_then(serde_json::Value::as_f64)?;
            let size = |key: &str| {
                position
                    .get(key)
                    .and_then(serde_json::Value::as_f64)
                    .filter(|value| *value > 0.0)
            };
            Some((
                id.clone(),
                Pin {
                    x,
                    y,
                    width: size("width"),
                    height: size("height"),
                },
            ))
        })
        .collect()
}

/// Moves every pinned node onto its place, sizes it as the layout remembers it, and reroutes
/// the edges whose ends touch a pinned node.
fn apply_pins(scene: &mut Scene, pins: &HashMap<String, Pin>) {
    if pins.is_empty() {
        return;
    }
    for (id, pin) in pins {
        if pin.width.is_some() || pin.height.is_some() {
            resize_node(&mut scene.items, id, pin.width, pin.height);
        }
    }
    let mut boxes = HashMap::new();
    collect_node_boxes(&scene.items, Transform::IDENTITY, &mut boxes);
    let deltas: HashMap<String, Point> = pins
        .iter()
        .filter_map(|(id, at)| {
            let rect = boxes.get(id)?;
            Some((id.clone(), Point::new(at.x - rect.x, at.y - rect.y)))
        })
        .filter(|(_, delta)| delta.x != 0.0 || delta.y != 0.0)
        .collect();
    shift_nodes(&mut scene.items, Transform::IDENTITY, &deltas);
    reroute_edges(&mut scene.items, &boxes, &deltas, &HashSet::new());
    center_labels(&mut scene.items);
    refit(scene);
}

/// Sizes a pinned node's shape (not its label) to the box the layout remembers.
fn resize_node(items: &mut [Item], id: &str, width: Option<f64>, height: Option<f64>) {
    for item in items {
        let Item::Group(group) = item else {
            continue;
        };
        if group
            .source
            .as_ref()
            .is_some_and(|source| source.kind == SourceKind::Node && source.id == id)
        {
            resize_group(group, width, height);
            // The node group's box may sit under a translated parent; the group's own
            // transform puts the resized box's top-left back where the pin expects it.
            return;
        }
        resize_node(&mut group.items, id, width, height);
    }
}

/// Scales a node's shape about its center, then moves the group so the box keeps its place.
fn resize_group(group: &mut Group, width: Option<f64>, height: Option<f64>) {
    let Some(bounds) = shape_bounds(&group.items) else {
        return;
    };
    let sx = width.map_or(1.0, |width| width / bounds.width.max(1e-6));
    let sy = height.map_or(1.0, |height| height / bounds.height.max(1e-6));
    if (sx - 1.0).abs() < 1e-9 && (sy - 1.0).abs() < 1e-9 {
        return;
    }
    let center = Point::new(
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
    );
    scale_items(&mut group.items, center, sx, sy);
}

/// The bounds of the painted shape of a node's group, in the group's own coordinates.
fn shape_bounds(items: &[Item]) -> Option<Rect> {
    items
        .iter()
        .filter(|item| !matches!(item, Item::Text(_)) && !is_label_group(item))
        .filter_map(|item| item_bounds(item, Transform::IDENTITY))
        .reduce(Rect::union)
}

fn is_label_group(item: &Item) -> bool {
    matches!(
        item,
        Item::Group(group)
            if group.class.as_deref().is_some_and(|classes| {
                classes.split_whitespace().any(|class| class == "label")
            })
    )
}

/// Scales the shapes in a node (never its text) about `center`.
fn scale_items(items: &mut [Item], center: Point, sx: f64, sy: f64) {
    let scale = |point: Point| {
        Point::new(
            center.x + (point.x - center.x) * sx,
            center.y + (point.y - center.y) * sy,
        )
    };
    for item in items {
        match item {
            Item::Rect(rect) => {
                let at = scale(Point::new(rect.rect.x, rect.rect.y));
                rect.rect.x = at.x;
                rect.rect.y = at.y;
                rect.rect.width *= sx;
                rect.rect.height *= sy;
                rect.rx *= sx.min(sy);
                rect.ry *= sx.min(sy);
            }
            Item::Ellipse(ellipse) => {
                ellipse.center = scale(ellipse.center);
                ellipse.radius_x *= sx;
                ellipse.radius_y *= sy;
            }
            Item::Path(path) => scale_path(&mut path.path, scale),
            Item::Image(image) => {
                let at = scale(Point::new(image.rect.x, image.rect.y));
                image.rect.x = at.x;
                image.rect.y = at.y;
                image.rect.width *= sx;
                image.rect.height *= sy;
            }
            Item::Group(group) => scale_items(&mut group.items, center, sx, sy),
            Item::Text(_) => {}
        }
    }
}

fn scale_path(path: &mut polterware_mermaid::scene::Path, scale: impl Fn(Point) -> Point) {
    for command in &mut path.commands {
        match command {
            PathCommand::MoveTo { x, y } | PathCommand::LineTo { x, y } => {
                let at = scale(Point::new(*x, *y));
                *x = at.x;
                *y = at.y;
            }
            PathCommand::CubicTo {
                c1x,
                c1y,
                c2x,
                c2y,
                x,
                y,
            } => {
                let c1 = scale(Point::new(*c1x, *c1y));
                let c2 = scale(Point::new(*c2x, *c2y));
                let end = scale(Point::new(*x, *y));
                (*c1x, *c1y) = (c1.x, c1.y);
                (*c2x, *c2y) = (c2.x, c2.y);
                (*x, *y) = (end.x, end.y);
            }
            PathCommand::QuadTo { cx, cy, x, y } => {
                let control = scale(Point::new(*cx, *cy));
                let end = scale(Point::new(*x, *y));
                (*cx, *cy) = (control.x, control.y);
                (*x, *y) = (end.x, end.y);
            }
            PathCommand::ArcTo { x, y, .. } => {
                let end = scale(Point::new(*x, *y));
                (*x, *y) = (end.x, end.y);
            }
            PathCommand::Close => {}
        }
    }
}

/// Keeps every node's label centered on its shape, whatever the layout gave the node.
///
/// The renderer places a label group by its own box and the painter draws a text block from
/// its baseline; putting the run at the shape's center makes both agree, so a pinned node and
/// an automatically placed one read the same.
fn center_labels(items: &mut [Item]) {
    for item in items {
        let Item::Group(group) = item else {
            continue;
        };
        let node = group
            .source
            .as_ref()
            .is_some_and(|source| source.kind == SourceKind::Node);
        if node {
            center_node_labels(group);
        }
        center_labels(&mut group.items);
    }
}

fn center_node_labels(group: &mut Group) {
    for item in &mut group.items {
        let Item::Group(label) = item else {
            continue;
        };
        let label_group = label
            .class
            .as_deref()
            .is_some_and(|classes| classes.split_whitespace().any(|class| class == "label"));
        if !label_group {
            continue;
        }
        let mut top = 0.0;
        for item in &mut label.items {
            let Item::Text(text) = item else {
                continue;
            };
            text.position = Point::new(-text.size.width / 2.0, text.size.height / 2.0 + top);
            top += text.line_height.max(text.size.height);
        }
        label.transform = Transform::IDENTITY;
    }
}

/// The bounds of every node's drawing, in scene coordinates.
fn collect_node_boxes(items: &[Item], transform: Transform, out: &mut HashMap<String, Rect>) {
    for item in items {
        let world = match item {
            Item::Group(group) => group.transform.then(transform),
            _ => transform,
        };
        if let Some(source) = item.source()
            && source.kind == SourceKind::Node
            && let Some(bounds) = item_bounds(item, transform)
        {
            out.entry(source.id.clone())
                .and_modify(|rect| *rect = rect.union(bounds))
                .or_insert(bounds);
        }
        if let Item::Group(group) = item {
            collect_node_boxes(&group.items, world, out);
        }
    }
}

/// Moves each pinned node's own group, so its shape, its label, and its box travel together.
fn shift_nodes(items: &mut [Item], parent: Transform, deltas: &HashMap<String, Point>) {
    for item in items {
        let Item::Group(group) = item else {
            continue;
        };
        let pinned = group
            .source
            .as_ref()
            .filter(|source| source.kind == SourceKind::Node)
            .and_then(|source| deltas.get(&source.id));
        if let Some(delta) = pinned {
            let local = local_delta(parent, *delta);
            group.transform.e += local.x;
            group.transform.f += local.y;
            continue;
        }
        let world = group.transform.then(parent);
        shift_nodes(&mut group.items, world, deltas);
    }
}

/// A scene-space translation as the translation of a group under `parent`.
fn local_delta(parent: Transform, delta: Point) -> Point {
    let determinant = parent.a * parent.d - parent.b * parent.c;
    if determinant.abs() < 1e-9 {
        return delta;
    }
    Point::new(
        (parent.d * delta.x - parent.c * delta.y) / determinant,
        (-parent.b * delta.x + parent.a * delta.y) / determinant,
    )
}

/// Reroutes every edge whose ends touch a pinned node: a straight line clipped at the source
/// and target borders, and a label centered on the line's midpoint. An edge between two
/// unpinned nodes keeps the layout's path exactly as dagre left it.
fn reroute_edges(
    items: &mut [Item],
    boxes: &HashMap<String, Rect>,
    deltas: &HashMap<String, Point>,
    forced: &HashSet<String>,
) {
    let mut midpoints = HashMap::new();
    reroute_edge_paths(items, boxes, deltas, forced, &mut midpoints);
    center_edge_labels(items, &midpoints, Transform::IDENTITY);
}

/// Reroutes each edge that has a pinned node on either end, remembering its label's midpoint.
fn reroute_edge_paths(
    items: &mut [Item],
    boxes: &HashMap<String, Rect>,
    deltas: &HashMap<String, Point>,
    forced: &HashSet<String>,
    midpoints: &mut HashMap<String, Point>,
) {
    for item in items {
        match item {
            Item::Path(path) => {
                if !is_edge(path.source.as_ref()) {
                    continue;
                }
                let Some(source) = path.source.as_ref() else {
                    continue;
                };
                let points: Vec<Point> =
                    path.path.commands.iter().filter_map(command_end).collect();
                let (Some(first), Some(last)) = (points.first(), points.last()) else {
                    continue;
                };
                // Mermaid stops a line one arrowhead short of the node it points at, so an
                // edge's ends are tested at the arrow's own reach, not against the line only.
                let Some((from_id, from)) = node_at(first, boxes, deltas, ARROW_REACH) else {
                    continue;
                };
                let Some((to_id, to)) = node_at(last, boxes, deltas, ARROW_REACH) else {
                    continue;
                };
                let moved = deltas.contains_key(&from_id)
                    || deltas.contains_key(&to_id)
                    || forced.contains(&from_id)
                    || forced.contains(&to_id);
                if from_id == to_id || !moved {
                    continue;
                }
                let Some((start, end)) = border_line(from, to) else {
                    continue;
                };
                path.path = Path::polyline([start, end]);
                midpoints.insert(
                    source.id.clone(),
                    Point::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0),
                );
            }
            Item::Group(group) => {
                reroute_edge_paths(&mut group.items, boxes, deltas, forced, midpoints)
            }
            _ => {}
        }
    }
}

fn is_edge(source: Option<&polterware_mermaid::scene::SourceRef>) -> bool {
    source.is_some_and(|source| source.kind == SourceKind::Edge)
}

/// The node a point sits on, with its box moved to where a pin put it.
fn node_at(
    point: &Point,
    boxes: &HashMap<String, Rect>,
    deltas: &HashMap<String, Point>,
    reach: f64,
) -> Option<(String, Rect)> {
    boxes.iter().find_map(|(id, rect)| {
        let dx = (rect.x - point.x).max(point.x - rect.right()).max(0.0);
        let dy = (rect.y - point.y).max(point.y - rect.bottom()).max(0.0);
        if dx.hypot(dy) > END_TOLERANCE + reach {
            return None;
        }
        let delta = deltas.get(id).copied().unwrap_or(Point::ZERO);
        Some((
            id.clone(),
            Rect::new(rect.x + delta.x, rect.y + delta.y, rect.width, rect.height),
        ))
    })
}

/// The straight line between two boxes, clipped at each border.
fn border_line(from: Rect, to: Rect) -> Option<(Point, Point)> {
    let start = exit_point(from.center(), to.center(), from)?;
    let end = exit_point(to.center(), from.center(), to)?;
    ((end.x - start.x).hypot(end.y - start.y) > 1e-9).then_some((start, end))
}

/// Where a ray from a box's center toward `toward` leaves the box.
fn exit_point(center: Point, toward: Point, rect: Rect) -> Option<Point> {
    let dx = toward.x - center.x;
    let dy = toward.y - center.y;
    let mut best: Option<f64> = None;
    let mut keep = |t: f64| {
        if t > 1e-9 && best.is_none_or(|best| t < best) {
            best = Some(t);
        }
    };
    if dx.abs() > 1e-9 {
        for x in [rect.x, rect.right()] {
            let t = (x - center.x) / dx;
            let y = center.y + dy * t;
            if y >= rect.y - 1e-9 && y <= rect.bottom() + 1e-9 {
                keep(t);
            }
        }
    }
    if dy.abs() > 1e-9 {
        for y in [rect.y, rect.bottom()] {
            let t = (y - center.y) / dy;
            let x = center.x + dx * t;
            if x >= rect.x - 1e-9 && x <= rect.right() + 1e-9 {
                keep(t);
            }
        }
    }
    best.map(|t| Point::new(center.x + dx * t, center.y + dy * t))
}

/// Centers each edge's label background box, and its text with it, on the edge's midpoint.
fn center_edge_labels(items: &mut [Item], midpoints: &HashMap<String, Point>, parent: Transform) {
    for item in items {
        let Item::Group(group) = item else {
            continue;
        };
        let midpoint = group
            .source
            .as_ref()
            .filter(|source| source.kind == SourceKind::Edge)
            .and_then(|source| midpoints.get(&source.id));
        if let Some(midpoint) = midpoint {
            let world = group.transform.then(parent);
            let background = group.items.iter().find_map(|item| match item {
                Item::Rect(rect) => Some(transformed(rect.rect, world)),
                _ => None,
            });
            if let Some(background) = background {
                let center = background.center();
                let delta = Point::new(midpoint.x - center.x, midpoint.y - center.y);
                let local = local_delta(parent, delta);
                group.transform.e += local.x;
                group.transform.f += local.y;
                continue;
            }
        }
        let world = group.transform.then(parent);
        center_edge_labels(&mut group.items, midpoints, world);
    }
}

/// The point a command ends at, the one an edge's end sits on.
fn command_end(command: &PathCommand) -> Option<Point> {
    match *command {
        PathCommand::MoveTo { x, y } | PathCommand::LineTo { x, y } => Some(Point::new(x, y)),
        PathCommand::CubicTo { x, y, .. }
        | PathCommand::QuadTo { x, y, .. }
        | PathCommand::ArcTo { x, y, .. } => Some(Point::new(x, y)),
        PathCommand::Close => None,
    }
}

/// The content's box again, so a node moved past the fitted view still shows.
fn refit(scene: &mut Scene) {
    let mut bounds: Option<Rect> = None;
    for item in &scene.items {
        if let Some(rect) = item_bounds(item, Transform::IDENTITY) {
            bounds = Some(bounds.map_or(rect, |bounds| bounds.union(rect)));
        }
    }
    let Some(bounds) = bounds else {
        return;
    };
    scene.bounds = bounds;
    scene.view_box = bounds.expanded(REFIT_PADDING);
}

/// The box an item covers, with `transform` applied.
fn item_bounds(item: &Item, transform: Transform) -> Option<Rect> {
    match item {
        Item::Group(group) => {
            let world = group.transform.then(transform);
            group
                .items
                .iter()
                .filter_map(|item| item_bounds(item, world))
                .reduce(Rect::union)
        }
        Item::Rect(rect) => Some(transformed(rect.rect, transform)),
        Item::Ellipse(ellipse) => Some(transformed(
            Rect::new(
                ellipse.center.x - ellipse.radius_x,
                ellipse.center.y - ellipse.radius_y,
                ellipse.radius_x * 2.0,
                ellipse.radius_y * 2.0,
            ),
            transform,
        )),
        Item::Text(text) => Some(transformed(text.bounds(), transform)),
        Item::Image(image) => Some(transformed(image.rect, transform)),
        Item::Path(path) => {
            let mut bounds: Option<Rect> = None;
            for command in &path.path.commands {
                let points = command_points(command);
                for point in points {
                    let at = transform.apply(point);
                    let rect = Rect::new(at.x, at.y, 0.0, 0.0);
                    bounds = Some(bounds.map_or(rect, |bounds| bounds.union(rect)));
                }
            }
            bounds.map(|bounds| bounds.expanded(path.style.stroke_width / 2.0))
        }
    }
}

fn command_points(command: &PathCommand) -> Vec<Point> {
    match *command {
        PathCommand::MoveTo { x, y } | PathCommand::LineTo { x, y } => vec![Point::new(x, y)],
        PathCommand::CubicTo {
            c1x,
            c1y,
            c2x,
            c2y,
            x,
            y,
        } => vec![Point::new(c1x, c1y), Point::new(c2x, c2y), Point::new(x, y)],
        PathCommand::QuadTo { cx, cy, x, y } => vec![Point::new(cx, cy), Point::new(x, y)],
        PathCommand::ArcTo { x, y, .. } => vec![Point::new(x, y)],
        PathCommand::Close => Vec::new(),
    }
}

fn transformed(rect: Rect, transform: Transform) -> Rect {
    let corners = [
        Point::new(rect.x, rect.y),
        Point::new(rect.right(), rect.y),
        Point::new(rect.right(), rect.bottom()),
        Point::new(rect.x, rect.bottom()),
    ]
    .map(|corner| transform.apply(corner));
    corners
        .into_iter()
        .map(|point| Rect::new(point.x, point.y, 0.0, 0.0))
        .reduce(Rect::union)
        .unwrap_or(Rect::ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polterware_mermaid::measure::FallbackMeasure;

    fn test_palette() -> Palette {
        palette(&Theme::dark())
    }

    fn render_scene(source: &str, layout: &str, palette: &Palette) -> Rendered {
        render(source, layout, &[], &FallbackMeasure, palette).expect("a scene")
    }

    #[test]
    fn fit_centers_the_drawing_inside_the_free_area() {
        let view_box = Rect::new(20.0, 30.0, 400.0, 200.0);
        let viewport = (1000.0, 500.0);
        let insets = (200.0, 300.0, 50.0, 40.0);
        let fitted = fit_viewport_in_area(view_box, viewport, insets, 20.0);

        let drawing_left = view_box.x as f32 * fitted.zoom + fitted.x;
        let drawing_right = (view_box.x + view_box.width) as f32 * fitted.zoom + fitted.x;
        let drawing_top = view_box.y as f32 * fitted.zoom + fitted.y;
        let drawing_bottom = (view_box.y + view_box.height) as f32 * fitted.zoom + fitted.y;
        assert!(drawing_left >= insets.0 + 19.9, "left edge: {drawing_left}");
        assert!(
            drawing_right <= viewport.0 - insets.1 - 19.9,
            "right edge: {drawing_right}"
        );
        assert!(drawing_top >= insets.2 + 19.9, "top edge: {drawing_top}");
        assert!(
            drawing_bottom <= viewport.1 - insets.3 - 19.9,
            "bottom edge: {drawing_bottom}"
        );
    }

    #[test]
    fn viewport_save_and_restore_use_the_free_area_origin() {
        let screen = SceneViewport::new(260.0, 84.0, 1.25);
        let saved = viewport_at_free_origin(screen, 204.0, 56.0);
        assert_eq!(saved, SceneViewport::new(56.0, 28.0, 1.25));
        assert_eq!(viewport_from_free_origin(saved, 204.0, 56.0), screen);
    }

    #[test]
    fn restored_view_with_less_than_half_its_bounds_visible_needs_fit() {
        let drawing = Rect::new(0.0, 0.0, 400.0, 200.0);
        let free_area = Rect::new(204.0, 56.0, 500.0, 360.0);
        let mostly_hidden = SceneViewport::new(-100.0, 56.0, 1.0);
        let half_visible = SceneViewport::new(4.0, 56.0, 1.0);

        assert!(restored_view_needs_fit(drawing, mostly_hidden, free_area));
        assert!(!restored_view_needs_fit(drawing, half_visible, free_area));
    }

    #[test]
    fn a_flowchart_draws_and_a_pin_moves_its_node() {
        let source = "flowchart LR\n  a[Start] --> b[End]\n";
        let scene = render_scene(source, "{}", &test_palette()).scene;
        assert!(!scene.is_empty());

        let layout = r#"{"nodes":{"a":{"x":-40,"y":-30,"pinned":true}}}"#;
        let pinned = render_scene(source, layout, &test_palette()).scene;
        let mut boxes = HashMap::new();
        collect_node_boxes(&pinned.items, Transform::IDENTITY, &mut boxes);
        let start = boxes.get("a").expect("the pinned node");
        assert!((start.x + 40.0).abs() < 0.5, "x: {}", start.x);
        assert!((start.y + 30.0).abs() < 0.5, "y: {}", start.y);
    }

    #[test]
    fn component_nodes_resize_and_hand_off_to_card_view() {
        let source = "flowchart LR\n  db[\"Orders DB\"]@{ kind: database, technology: Postgres }\n";
        let rendered = render_scene(source, "{}", &test_palette());
        assert!(rendered.cards.contains("db"));
        let node = find_node_group(&rendered.scene.items, "db").expect("component node");
        assert!(!node.items.iter().any(|item| matches!(
            item,
            Item::Group(group)
                if group.class.as_deref().is_some_and(|classes| classes.split_whitespace().any(|class| class == "label"))
        )));
        let bounds = shape_bounds(&node.items).expect("hit box");
        let title_font = Font {
            family: test_palette().font_family,
            size: 13.0,
            weight: 500,
            italic: false,
        };
        let subtitle_font = Font::new(test_palette().font_family, 11.0);
        let text_width = FallbackMeasure
            .width("Orders DB", &title_font)
            .max(FallbackMeasure.width("Database / Postgres", &subtitle_font));
        assert!(
            bounds.width >= text_width + CARD_PAD_LEFT + BADGE_SIZE + CARD_GAP + CARD_PAD_RIGHT,
            "card width {} too narrow for text {}",
            bounds.width,
            text_width
        );
        assert!(bounds.height >= CARD_MIN_HEIGHT);
    }

    #[test]
    fn a_library_inserted_node_and_linked_structure_node_are_marked_as_cards() {
        // The library writes the name only in `label`, and the technology in `props`.
        let source = "flowchart LR\n  n1@{ id: \"n1-service\", label: \"Compute\", kind: service, shape: rounded, props: \"{\\\"technology\\\":\\\"Go\\\"}\" }\n  n2@{ kind: component, ref: node/core }\n";
        let entities = vec![StoreEntity {
            kind: "node".into(),
            id: "core".into(),
            title: "Local core server".into(),
            slug: "core".into(),
            level: "component".into(),
            component_kind: "service".into(),
            tech: "Rust".into(),
        }];
        let rendered = render(source, "{}", &entities, &FallbackMeasure, &test_palette())
            .expect("a scene");
        assert!(rendered.cards.contains("n1"));
        assert!(rendered.cards.contains("n2"));
        let n2 = find_node_group(&rendered.scene.items, "n2").expect("linked component");
        let bounds = shape_bounds(&n2.items).expect("hit box");
        assert!(bounds.height >= CARD_MIN_HEIGHT);
    }

    #[test]
    fn non_component_mermaid_nodes_keep_the_plain_shape_and_label() {
        let rendered = render_scene(
            "flowchart LR\n  box[\"Plain node\"]@{ shape: rect }\n",
            "{}",
            &test_palette(),
        );
        assert!(!rendered.cards.contains("box"));
        let node = find_node_group(&rendered.scene.items, "box").expect("plain node");
        assert!(node.items.iter().any(|item| matches!(item, Item::Group(group)
            if group.class.as_deref().is_some_and(|classes| classes.split_whitespace().any(|class| class == "label")))));
    }

    fn find_node_group<'a>(items: &'a [Item], id: &str) -> Option<&'a Group> {
        items.iter().find_map(|item| {
            let Item::Group(group) = item else {
                return None;
            };
            if group
                .source
                .as_ref()
                .is_some_and(|source| source.kind == SourceKind::Node && source.id == id)
            {
                Some(group)
            } else {
                find_node_group(&group.items, id)
            }
        })
    }

    #[test]
    fn kind_selects_its_shape_from_one_table() {
        assert_eq!(shape_for_kind("database"), Some("cyl"));
        assert_eq!(shape_for_kind("storage"), Some("cyl"));
        assert_eq!(shape_for_kind("queue"), Some("datastore"));
        assert_eq!(shape_for_kind("llm"), Some("hex"));
        assert_eq!(shape_for_kind("frontend"), Some("rounded"));
        assert_eq!(shape_for_kind("user"), Some("stadium"));
        assert_eq!(shape_for_kind("scheduler"), Some("lean-r"));
        assert_eq!(shape_for_kind("document"), Some("doc"));
        assert_eq!(shape_for_kind("cloud"), Some("cloud"));
        assert!(dashed_kind("external"));
        assert!(shape_for_kind("a-kind-that-does-not-exist").is_none());
    }

    #[test]
    fn a_render_adds_the_kind_shape_and_keeps_written_ones() {
        let source = "flowchart LR\n  a[\"Storage\"]@{ kind: database }\n  b[\"Box\"]@{ kind: database, shape: rect }\n";
        let styled = styled_source(source);
        assert!(styled.contains("kind: database, shape: cyl"));
        assert!(styled.contains("kind: database, shape: rect"));
        assert_eq!(styled.matches("shape: cyl").count(), 1);
    }

    #[test]
    fn a_dashed_kind_gains_a_style_line_once() {
        let source = "flowchart LR\n  a[\"Stripe\"]@{ kind: payment }\n";
        let styled = styled_source(source);
        assert!(styled.contains("style a stroke-dasharray: 4 4"));
        assert_eq!(styled.matches("stroke-dasharray").count(), 1);
    }

    #[test]
    fn a_style_line_reaches_its_node_and_parses_each_control() {
        let styles = css_styles(
            "flowchart LR\n  a[\"A\"]\n  style a fill:#ff0000,stroke:#00ff00,stroke-width:2px,stroke-dasharray:4 2\n",
        );
        let style = styles.get("a").expect("the style");
        assert_eq!(style.fill.as_deref(), Some("#ff0000"));
        assert_eq!(style.stroke.as_deref(), Some("#00ff00"));
        assert_eq!(style.width, Some(2.0));
        assert_eq!(style.dash.as_deref(), Some("4 2"));
        let scene = render_scene(
            "flowchart LR\n  a[\"A\"]\n  style a fill:#ff0000,stroke-width:2px\n",
            "{}",
            &test_palette(),
        )
        .scene;
        assert_eq!(
            first_rect_fill(&scene.items).as_deref(),
            Some("#ff0000"),
            "the style line did not reach the shape"
        );
    }

    #[test]
    fn the_theme_replaces_mermaids_default_fill_and_stroke() {
        let scene = render_scene("flowchart LR\n  a[Start]\n", "{}", &test_palette()).scene;
        let fill = first_rect_fill(&scene.items).expect("a node fill");
        assert_ne!(
            fill, MERMAID_FILL,
            "Mermaid's light fill is still on the dark app"
        );
    }

    #[test]
    fn nodes_take_a_surface_fill_above_the_canvas_background() {
        let theme = Theme::dark();
        let palette = palette(&theme);
        assert_eq!(palette.node_fill, css(theme.color_surface_subtle));
        assert_ne!(palette.node_fill, css(theme.color_app_background));
        assert_ne!(
            palette.cluster_fill, palette.node_fill,
            "a cluster and a node would read the same"
        );
        let scene = render_scene("flowchart LR\n  a[Start]\n", "{}", &palette).scene;
        assert_eq!(
            first_rect_fill(&scene.items).as_deref(),
            Some(palette.node_fill.as_str()),
            "the node is not filled with the surface token"
        );
    }

    #[test]
    fn labels_paint_in_the_apps_ui_font() {
        let theme = Theme::dark();
        let palette = palette(&theme);
        assert_eq!(palette.font_family, theme.font_sans.to_string());
        let scene = render_scene("flowchart LR\n  a[Start] --> b[End]\n", "{}", &palette).scene;
        let families = text_families(&scene.items);
        assert!(!families.is_empty(), "the diagram has no label");
        assert!(
            families.iter().all(|family| *family == palette.font_family),
            "a label kept its own family: {families:?}"
        );
    }

    fn text_families(items: &[Item]) -> Vec<String> {
        let mut families = Vec::new();
        for item in items {
            match item {
                Item::Text(text) => families.push(text.font.family.clone()),
                Item::Group(group) => families.extend(text_families(&group.items)),
                _ => {}
            }
        }
        families
    }

    #[test]
    fn a_pinned_node_keeps_its_label_centered_on_its_shape() {
        let source = "flowchart LR\n  a[\"Storage\"]\n";
        let unpinned = render_scene(source, "{}", &test_palette()).scene;
        assert_centered(&unpinned, "a");

        let layout = r#"{"nodes":{"a":{"x":40,"y":30,"width":220,"height":90,"pinned":true}}}"#;
        let pinned = render_scene(source, layout, &test_palette()).scene;
        let box_ = node_box(&pinned, "a");
        assert!((box_.0 - 40.0).abs() < 0.5, "x: {}", box_.0);
        assert!((box_.1 - 30.0).abs() < 0.5, "y: {}", box_.1);
        assert!((box_.2 - 220.0).abs() < 0.5, "width: {}", box_.2);
        assert!((box_.3 - 90.0).abs() < 0.5, "height: {}", box_.3);
        assert_centered(&pinned, "a");
    }

    #[test]
    fn the_reported_unverified_canvas_keeps_component_cards_and_dashed_outlines() {
        let source = "flowchart LR\n  a[\"Storage\"]@{ kind: database }\n  b[\"AI\"]@{ kind: llm }\n  c[\"Storage\"]@{ kind: storage }\n  d[\"Plain\"]@{ shape: rect }\n  class a,b,c,d unverified\n";
        let layout = r#"{"nodes":{"a":{"x":0,"y":0,"width":160,"height":64,"pinned":true},"b":{"x":200,"y":0,"width":160,"height":64,"pinned":true},"c":{"x":400,"y":0,"width":160,"height":64,"pinned":true}}}"#;
        let rendered = render_scene(source, layout, &test_palette());
        for id in ["a", "b", "c"] {
            assert!(rendered.cards.contains(id));
            let group = node_group(&rendered.scene.items, id).expect("the node group");
            assert!(
                !group.items.iter().any(|item| matches!(
                    item,
                    Item::Group(label)
                        if label.class.as_deref().is_some_and(|classes| classes.split_whitespace().any(|class| class == "label"))
                )),
                "component node {id} retained Mermaid's centered label"
            );
            assert!(
                group
                    .items
                    .iter()
                    .all(|item| matches!(item, Item::Rect(rect) if rect.style.fill.is_none())),
                "component node {id} retained painted shape items underneath card view"
            );
        }
        // The class line draws its dash on non-card Mermaid shapes even without a classDef.
        let group = node_group(&rendered.scene.items, "d").expect("the plain node");
        assert!(
            has_dashed_shape(&group.items),
            "the unverified class did not draw a dashed outline"
        );
    }

    fn has_dashed_shape(items: &[Item]) -> bool {
        items.iter().any(|item| match item {
            Item::Path(path) => !path.style.dash.is_empty(),
            Item::Rect(rect) => !rect.style.dash.is_empty(),
            Item::Group(group) => has_dashed_shape(&group.items),
            _ => false,
        })
    }

    fn node_box(scene: &Scene, id: &str) -> (f32, f32, f32, f32) {
        *node_boxes(scene).get(id).expect("the node box")
    }

    /// The node's shape and label centers agree in the scene, in the node's own coordinates.
    fn assert_centered(scene: &Scene, id: &str) {
        let group = node_group(&scene.items, id).expect("the node group");
        let shape = shape_bounds(&group.items).expect("the shape");
        let shape_center = (shape.x + shape.width / 2.0, shape.y + shape.height / 2.0);
        let label = group
            .items
            .iter()
            .find_map(|item| match item {
                Item::Group(label)
                    if label.class.as_deref().is_some_and(|classes| {
                        classes.split_whitespace().any(|class| class == "label")
                    }) =>
                {
                    let text = label.items.iter().find_map(|item| match item {
                        Item::Text(text) => Some(text),
                        _ => None,
                    })?;
                    Some((
                        label.transform.e + text.position.x + text.size.width / 2.0,
                        label.transform.f + text.position.y - text.size.height / 2.0,
                    ))
                }
                _ => None,
            })
            .expect("the label");
        assert!(
            (shape_center.0 - label.0).abs() < 0.5,
            "label x {} vs shape x {}",
            label.0,
            shape_center.0
        );
        assert!(
            (shape_center.1 - label.1).abs() < 0.5,
            "label y {} vs shape y {}",
            label.1,
            shape_center.1
        );
    }

    fn node_group<'a>(items: &'a [Item], id: &str) -> Option<&'a Group> {
        for item in items {
            if let Item::Group(group) = item {
                if group
                    .source
                    .as_ref()
                    .is_some_and(|source| source.kind == SourceKind::Node && source.id == id)
                {
                    return Some(group);
                }
                if let Some(found) = node_group(&group.items, id) {
                    return Some(found);
                }
            }
        }
        None
    }

    fn edge_label_group<'a>(items: &'a [Item], text: &str) -> Option<&'a Group> {
        for item in items {
            if let Item::Group(group) = item {
                let labelled = group
                    .class
                    .as_deref()
                    .is_some_and(|classes| classes.contains("edgeLabel"));
                if labelled && label_text(&group.items, text).is_some() {
                    return Some(group);
                }
                if let Some(found) = edge_label_group(&group.items, text) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Whether a point sits on a box's border, within the edge's end tolerance.
    fn on_border(rect: Rect, point: Point) -> bool {
        let inside = point.x >= rect.x - END_TOLERANCE
            && point.x <= rect.right() + END_TOLERANCE
            && point.y >= rect.y - END_TOLERANCE
            && point.y <= rect.bottom() + END_TOLERANCE;
        let on_x = (point.x - rect.x).abs() <= END_TOLERANCE
            || (point.x - rect.right()).abs() <= END_TOLERANCE;
        let on_y = (point.y - rect.y).abs() <= END_TOLERANCE
            || (point.y - rect.bottom()).abs() <= END_TOLERANCE;
        inside && (on_x || on_y)
    }

    fn first_rect_fill(items: &[Item]) -> Option<String> {
        for item in items {
            match item {
                Item::Rect(rect) => {
                    if let Some(fill) = &rect.style.fill {
                        return Some(fill.0.clone());
                    }
                }
                Item::Group(group) => {
                    if let Some(fill) = first_rect_fill(&group.items) {
                        return Some(fill);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// The canvas the report was filed with: two nodes and one labelled edge.
    const REPORTED: &str = "flowchart LR\n  api[\"Orders API\"]@{ kind: service, shape: rounded }\n  db[\"Orders DB\"]@{ kind: database, shape: cyl }\n  api -->|writes| db\n";

    #[test]
    fn a_reported_canvas_paints_its_edge_line_arrow_and_label() {
        let palette = test_palette();
        let scene = render_scene(REPORTED, "{}", &palette).scene;
        let path = first_edge(&scene).expect("an edge path");
        let stroke = path.style.stroke.as_ref().expect("the edge has no stroke");
        assert_ne!(stroke.0, "none", "the edge paints nothing");
        assert_ne!(stroke.0, "transparent", "the edge paints nothing");
        assert_eq!(stroke.0, palette.line, "the edge is not the theme's line");
        assert_eq!(
            path.style.fill, None,
            "an edge is a line, not a filled area"
        );
        assert!(path.style.stroke_width > 0.0);
        assert!(
            path.end_markers.is_empty(),
            "the line still leaves its arrowhead to an unfilled marker"
        );

        // The arrowhead is Mermaid's point marker: a solid triangle in the line's color, its
        // tip on the target's border and its base a real head's size back along the line.
        let head = arrow_head(&scene).expect("the edge has no arrowhead");
        assert_eq!(
            head.style.fill.as_ref().map(|fill| fill.0.as_str()),
            Some(palette.line.as_str()),
            "the arrowhead is not filled with the line's color"
        );
        assert!(
            head.style.stroke.is_none(),
            "the arrowhead is a hairline, not a filled triangle"
        );
        assert!(
            head.style.dash.is_empty(),
            "the arrowhead is dashed, so it reads as dots"
        );
        let area = head_area(head);
        assert!(area > 8.0, "the arrowhead is too small to see: {area}");

        assert!(
            edge_label(&scene, "writes").is_some(),
            "the edge label is missing"
        );

        // The ends touch both boxes: the line starts on the tail's box and stops one arrowhead
        // short of the target, where the head's tip lands.
        let (start, end) = edge_ends(path);
        let api = node_rect(&scene, "api");
        let db = node_rect(&scene, "db");
        assert!(
            gap(api, start) <= END_TOLERANCE,
            "the start {start:?} is not on {api:?}"
        );
        assert!(
            gap(db, head_tip(head, db)) <= END_TOLERANCE,
            "the arrowhead does not reach {db:?}"
        );
        assert!(gap(db, end) <= ARROW_REACH + END_TOLERANCE);
    }

    #[test]
    fn a_pinned_node_takes_its_edge_and_its_label_with_it() {
        let palette = test_palette();
        let unpinned = render_scene(REPORTED, "{}", &palette).scene;
        let layout = r#"{"nodes":{"db":{"x":400,"y":120,"pinned":true}}}"#;
        let scene = render_scene(REPORTED, layout, &palette).scene;

        let db = node_rect(&scene, "db");
        assert!((db.x - 400.0).abs() < 0.5, "the node did not pin: {db:?}");
        assert!((db.y - 120.0).abs() < 0.5, "the node did not pin: {db:?}");
        let path = first_edge(&scene).expect("an edge path");
        let head = arrow_head(&scene).expect("an arrowhead");
        assert!(
            gap(db, head_tip(head, db)) <= END_TOLERANCE,
            "the arrowhead stayed behind, {} from the pinned node",
            gap(db, head_tip(head, db))
        );
        let api = node_rect(&scene, "api");
        assert!(
            gap(api, edge_ends(path).0) <= END_TOLERANCE,
            "the tail left the node that stayed"
        );
        let before = edge_label(&unpinned, "writes").expect("the label");
        let after = edge_label(&scene, "writes").expect("the label");
        assert!(
            after.0 > before.0 + 1.0,
            "the label did not follow the edge: {} -> {}",
            before.0,
            after.0
        );
    }

    #[test]
    fn an_edge_label_text_is_centered_in_its_background_box() {
        let layout = r#"{"nodes":{"api":{"x":-40,"y":260,"pinned":true}}}"#;
        let scene = render_scene(REPORTED, layout, &test_palette()).scene;
        let group = edge_label_group(&scene.items, "writes").expect("the label group");
        let background = group
            .items
            .iter()
            .find_map(|item| match item {
                Item::Rect(rect) => Some(transformed(rect.rect, group.transform)),
                _ => None,
            })
            .expect("the label background");
        let text = group
            .items
            .iter()
            .find_map(|item| match item {
                Item::Text(text) if text.lines.iter().any(|line| line.text() == "writes") => {
                    Some(transformed(text.bounds(), group.transform))
                }
                _ => None,
            })
            .expect("the label text");
        let (text_center, box_center) = (text.center(), background.center());
        assert!(
            (text_center.x - box_center.x).abs() < 0.5
                && (text_center.y - box_center.y).abs() < 0.5,
            "the text {text:?} is not centered in its box {background:?}"
        );
        assert!(
            background.contains(Point::new(text.x, text.y))
                && background.contains(Point::new(text.right(), text.bottom())),
            "the text {text:?} leaves its box {background:?}"
        );
    }

    #[test]
    fn a_pinned_endpoint_reroutes_its_edge_as_a_clipped_straight_line() {
        let layout = r#"{"nodes":{"api":{"x":-40,"y":260,"pinned":true}}}"#;
        let scene = render_scene(REPORTED, layout, &test_palette()).scene;
        let path = first_edge(&scene).expect("an edge path");
        let points: Vec<Point> = path.path.commands.iter().filter_map(command_end).collect();
        assert_eq!(
            points.len(),
            2,
            "the edge kept dagre's middle points: {points:?}"
        );
        let api = node_rect(&scene, "api");
        let db = node_rect(&scene, "db");
        assert!(
            on_border(api, points[0]),
            "the line starts at {:?}, not on {api:?}",
            points[0]
        );
        assert!(
            on_border(db, points[1]),
            "the line ends at {:?}, not on {db:?}",
            points[1]
        );
    }

    #[test]
    fn a_fitting_diagram_is_fitted_again_when_the_room_narrows() {
        let source = "flowchart LR\n  api[\"Orders API\"]@{ kind: service }\n  db[\"Orders DB\"]@{ kind: database }\n  ui[\"Storefront\"]@{ kind: frontend }\n";
        let scene = render_scene(source, "{}", &test_palette()).scene;
        let view_box = scene.view_box;
        let wide = (
            view_box.width as f32 + 400.0,
            view_box.height as f32 + 200.0,
        );
        let narrow = (view_box.width as f32 + 80.0, view_box.height as f32 + 160.0);
        let opened = SceneViewport::fit(view_box, wide.0, wide.1, REVEAL_MARGIN);
        let focus = node_rect(&scene, "ui");
        let target = reveal_view(opened, view_box, narrow, focus, REVEAL_MARGIN)
            .expect("the narrower room refits the diagram");

        for id in ["api", "db", "ui"] {
            let rect = node_rect(&scene, id);
            let left = rect.x as f32 * target.zoom + target.x;
            let right = left + rect.width as f32 * target.zoom;
            let top = rect.y as f32 * target.zoom + target.y;
            let bottom = top + rect.height as f32 * target.zoom;
            assert!(
                left >= 0.0 && right <= narrow.0 && top >= 0.0 && bottom <= narrow.1,
                "{id} is outside the room: {left}..{right} x {top}..{bottom}"
            );
        }
    }

    /// The report's canvas, open with the inspector: the room left of the panel is narrower
    /// than the window, and the two nodes still fit in it.
    #[test]
    fn the_room_beside_the_inspector_refits_the_whole_canvas() {
        let scene = render_scene(REPORTED, "{}", &test_palette()).scene;
        let view_box = scene.view_box;
        // The 640px the report had, less the inspector's panel and its insets: the drawing
        // fits, with the reveal margin around it and nothing more.
        let room = (
            (view_box.width as f32 + 56.0).min(640.0),
            view_box.height as f32 + 120.0,
        );
        assert!(
            view_box.width as f32 <= room.0 - REVEAL_MARGIN * 2.0,
            "the report's drawing no longer fits the room beside the inspector"
        );
        let current = SceneViewport::new(0.0, 0.0, 1.0);
        let focus = node_rect(&scene, "api");
        let target = reveal_view(current, view_box, room, focus, REVEAL_MARGIN)
            .expect("the room beside the inspector refits the drawing");
        assert_eq!(
            target,
            SceneViewport::fit(view_box, room.0, room.1, REVEAL_MARGIN),
            "the drawing was panned, not fitted, in the room beside the inspector"
        );
        for id in ["api", "db"] {
            let rect = node_rect(&scene, id);
            let left = rect.x as f32 * target.zoom + target.x;
            let right = left + rect.width as f32 * target.zoom;
            assert!(
                left >= 0.0 && right <= room.0,
                "{id} is cut off: {left}..{right} in a room {room:?}"
            );
        }
    }

    #[test]
    fn a_diagram_too_large_for_the_room_pans_only_to_the_selected_node() {
        let view_box = Rect::new(0.0, 0.0, 1200.0, 200.0);
        let room = (600.0, 400.0);
        let focus = Rect::new(1000.0, 20.0, 100.0, 40.0);
        let current = SceneViewport::new(0.0, 0.0, 2.0);
        let target = reveal_view(current, view_box, room, focus, REVEAL_MARGIN)
            .expect("the selected node is panned into the room");
        assert_eq!(target.zoom, current.zoom, "a pan must not zoom");
        assert!(
            (target.x - (room.0 - REVEAL_MARGIN - 1200.0 * 2.0)).abs() > 1.0,
            "the pan should stop at the node, not at the drawing's edge"
        );
        let right = (focus.x + focus.width) as f32 * target.zoom + target.x;
        assert!(
            (right - (room.0 - REVEAL_MARGIN)).abs() < 0.5,
            "the node did not stop at the margin: {right}"
        );
        assert!(
            reveal_view(target, view_box, room, focus, REVEAL_MARGIN).is_none(),
            "a node already in the room moves nothing"
        );
    }

    #[test]
    fn a_diagram_a_little_too_wide_for_the_room_zooms_out_to_fit_whole() {
        // The E2E canvas at 100% beside the library and the inspector: 300px of drawing in a
        // 330px room does not fit with the margins, but does at 0.9x, so both nodes stay out
        // from under the panels.
        let view_box = Rect::new(0.0, 0.0, 300.0, 80.0);
        let room = (330.0, 500.0);
        let focus = Rect::new(0.0, 20.0, 120.0, 40.0);
        let current = SceneViewport::new(0.0, 0.0, 1.0);
        let target = reveal_view(current, view_box, room, focus, REVEAL_MARGIN)
            .expect("the drawing is refitted");
        assert!(target.zoom < 1.0 && target.zoom >= REVEAL_MIN_ZOOM);
        let left = target.x;
        let right = view_box.width as f32 * target.zoom + target.x;
        assert!(left >= 0.0 && right <= room.0, "cut off: {left}..{right}");
    }

    #[test]
    fn a_diagram_that_already_fits_where_it_is_keeps_its_view() {
        let view_box = Rect::new(0.0, 0.0, 200.0, 100.0);
        let room = (400.0, 300.0);
        let focus = Rect::new(20.0, 20.0, 80.0, 40.0);
        let fitted = SceneViewport::fit(view_box, room.0, room.1, REVEAL_MARGIN);
        assert!(
            reveal_view(fitted, view_box, room, focus, REVEAL_MARGIN).is_none(),
            "a fitted diagram was moved again"
        );
    }

    /// The first edge path of a scene, which the painter draws a line and arrowheads from.
    fn first_edge(scene: &Scene) -> Option<&polterware_mermaid::scene::PathShape> {
        scene.items.iter().find_map(|item| match item {
            Item::Path(path) if is_edge(path.source.as_ref()) => Some(path),
            _ => None,
        })
    }

    /// The filled arrowhead of a scene: the edge path that carries a fill.
    fn arrow_head(scene: &Scene) -> Option<&polterware_mermaid::scene::PathShape> {
        scene.items.iter().find_map(|item| match item {
            Item::Path(path) if is_edge(path.source.as_ref()) && path.style.fill.is_some() => {
                Some(path)
            }
            _ => None,
        })
    }

    /// An arrowhead's points, in scene coordinates.
    fn head_points(head: &polterware_mermaid::scene::PathShape) -> Vec<Point> {
        head.path.commands.iter().filter_map(command_end).collect()
    }

    /// The area an arrowhead covers: how much paint the head puts on the canvas.
    fn head_area(head: &polterware_mermaid::scene::PathShape) -> f64 {
        let points = head_points(head);
        let mut area = 0.0;
        for (index, point) in points.iter().enumerate() {
            let next = points[(index + 1) % points.len()];
            area += point.x * next.y - next.x * point.y;
        }
        area.abs() / 2.0
    }

    /// The point of an arrowhead nearest `rect`: its tip, on the node's border.
    fn head_tip(head: &polterware_mermaid::scene::PathShape, rect: Rect) -> Point {
        head_points(head)
            .into_iter()
            .min_by(|a, b| gap(rect, *a).total_cmp(&gap(rect, *b)))
            .expect("an arrowhead has points")
    }

    /// The place an edge's label paints, by its text: its box's left and top.
    fn edge_label(scene: &Scene, text: &str) -> Option<(f64, f64)> {
        fn find(items: &[Item], text: &str) -> Option<(f64, f64)> {
            for item in items {
                match item {
                    Item::Group(group) => {
                        let labels = group
                            .class
                            .as_deref()
                            .is_some_and(|classes| classes.contains("edgeLabel"));
                        if labels && let Some((left, top)) = label_text(&group.items, text) {
                            return Some((group.transform.e + left, group.transform.f + top));
                        }
                        if let Some(found) = find(&group.items, text) {
                            return Some(found);
                        }
                    }
                    Item::Text(run) if run.lines.iter().any(|line| line.text() == text) => {
                        return Some((run.position.x, run.position.y));
                    }
                    _ => {}
                }
            }
            None
        }
        find(&scene.items, text)
    }

    fn label_text(items: &[Item], text: &str) -> Option<(f64, f64)> {
        items.iter().find_map(|item| match item {
            Item::Text(run) if run.lines.iter().any(|line| line.text() == text) => {
                Some((run.position.x, run.position.y))
            }
            _ => None,
        })
    }

    /// The first and last points an edge path ends at.
    fn edge_ends(path: &polterware_mermaid::scene::PathShape) -> (Point, Point) {
        let start = path
            .path
            .commands
            .first()
            .and_then(command_end)
            .expect("an edge has a start");
        let end = path
            .path
            .commands
            .iter()
            .rev()
            .find_map(command_end)
            .expect("an edge has an end");
        (start, end)
    }

    fn node_rect(scene: &Scene, id: &str) -> Rect {
        let (x, y, width, height) = node_box(scene, id);
        Rect::new(
            f64::from(x),
            f64::from(y),
            f64::from(width),
            f64::from(height),
        )
    }

    /// How far a point sits from a box, zero inside it.
    fn gap(rect: Rect, point: Point) -> f64 {
        let dx = (rect.x - point.x).max(point.x - rect.right()).max(0.0);
        let dy = (rect.y - point.y).max(point.y - rect.bottom()).max(0.0);
        dx.hypot(dy)
    }

    #[test]
    fn a_parse_error_reads_as_a_line_and_column() {
        let error = render(
            "flowchart LR\n  a@{ shape: nonsense }\n",
            "{}",
            &[],
            &FallbackMeasure,
            &test_palette(),
        )
        .expect_err("a bad shape");
        assert_eq!(
            message(&error),
            "line 2, column 4: No such shape: nonsense."
        );
    }

    #[test]
    fn another_diagram_type_is_unsupported() {
        let error = render(
            "stateDiagram-v2\n  A --> B\n",
            "{}",
            &[],
            &FallbackMeasure,
            &test_palette(),
        )
        .expect_err("not a flowchart");
        assert!(unsupported(&error));
    }

    #[test]
    fn an_empty_canvas_draws_nothing_rather_than_failing() {
        let scene = render(
            "flowchart LR\n",
            "{}",
            &[],
            &FallbackMeasure,
            &test_palette(),
        )
        .expect("an empty scene")
        .scene;
        assert!(scene.is_empty());
    }

    #[test]
    fn a_diagram_id_follows_its_metadata_to_the_canvas_node() {
        let source = "flowchart LR\n  u1@{\"Orders\", id: \"01HORDERS\", kind: rectangle, ref: component/01HWEB }\n  u1 --> u2\n";
        let ids = node_ids(source);
        assert_eq!(ids.get("u1").map(String::as_str), Some("01HORDERS"));
        assert!(!ids.contains_key("u2"));
    }

    #[test]
    fn the_viewport_round_trips_through_the_layout() {
        let layout =
            r#"{"nodes":{"a":{"x":1,"y":2,"pinned":true}},"viewport":{"x":10,"y":20,"zoom":2}}"#;
        let view = viewport(layout).expect("a viewport");
        assert_eq!((view.x, view.y, view.zoom), (10.0, 20.0, 2.0));
        let written = with_viewport(layout, SceneViewport::new(1.0, 2.0, 1.5));
        let read = viewport(&written).expect("the written viewport");
        assert_eq!((read.x, read.y, read.zoom), (1.0, 2.0, 1.5));
        let layout: serde_json::Value = serde_json::from_str(&written).expect("json");
        assert_eq!(layout["nodes"]["a"]["pinned"], serde_json::json!(true));
    }
}
