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
use polterware_mermaid::error::{ParseError, RenderError};
use polterware_mermaid::measure::{Font, TextMeasure};
use polterware_mermaid::scene::{
    Color, Group, Item, PathCommand, Point, Rect, Scene, SourceKind, TextRun, Transform,
};
use polterware_mermaid::theme::Theme as MermaidTheme;
use polterware_ui::mermaid::SceneViewport;
use polterware_ui::theme::Theme;

use crate::canvas::catalog::component_of;

/// How near an edge end has to be to a node's box to be read as that node's.
const END_TOLERANCE: f64 = 0.75;
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
    pub muted: String,
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
        muted: css(theme.color_text_muted),
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

/// Renders `source` with the layout's pinned positions applied, in the app's colors.
pub(super) fn render(
    source: &str,
    layout_json: &str,
    measure: &dyn TextMeasure,
    palette: &Palette,
) -> Result<Scene, RenderError> {
    let styled = styled_source(source);
    let measure = UiFontMeasure {
        inner: measure,
        family: &palette.font_family,
    };
    let mut scene = draw(&styled, &measure, palette)?;
    apply_theme(&mut scene, palette);
    apply_styles(&mut scene, &css_styles(&styled));
    theme_fonts(&mut scene.items, palette);
    center_labels(&mut scene.items);
    apply_pins(&mut scene, &pins_for(&styled, layout_json));
    Ok(scene)
}

/// Renders `source` for a preview: no pins, in the app's colors.
pub(super) fn preview(source: &str, measure: &dyn TextMeasure, palette: &Palette) -> Option<Scene> {
    let styled = styled_source(source);
    let measure = UiFontMeasure {
        inner: measure,
        family: &palette.font_family,
    };
    let mut scene = draw(&styled, &measure, palette).ok()?;
    apply_theme(&mut scene, palette);
    apply_styles(&mut scene, &css_styles(&styled));
    theme_fonts(&mut scene.items, palette);
    center_labels(&mut scene.items);
    Some(scene)
}

/// Lays a source out into a scene, as the renderer draws it, in the app's UI font.
fn draw(source: &str, measure: &dyn TextMeasure, palette: &Palette) -> Result<Scene, RenderError> {
    let preprocessed = polterware_mermaid::preprocess::preprocess(source);
    if let Some(message) = preprocessed.error {
        return Err(RenderError::Parse(ParseError::new(message, None)));
    }
    let kind = polterware_mermaid::diagrams::detect(&preprocessed.body)
        .ok_or_else(|| ParseError::new("No diagram type detected", None))?;
    let mut config = Config::default();
    config.apply(&preprocessed.config)?;
    config.font_family = palette.font_family.clone();
    let scene = match kind {
        DiagramKind::Flowchart => polterware_mermaid::diagrams::flowchart::render::draw(
            &preprocessed.body,
            &config,
            measure,
        )?,
        other => return Err(RenderError::Unsupported(format!("{other:?}"))),
    };
    Ok(scene)
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

/// Moves every pinned node onto its place, sizes it as the layout remembers it, and the edge
/// ends that touch it with it.
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
    shift_edges(&mut scene.items, &boxes, &deltas);
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

/// Moves the edge ends that sit on a pinned node's box, and the edge's label with them.
fn shift_edges(items: &mut [Item], boxes: &HashMap<String, Rect>, deltas: &HashMap<String, Point>) {
    let mut moved: HashMap<String, Vec<Point>> = HashMap::new();
    shift_edge_paths(items, boxes, deltas, &mut moved);
    shift_edge_labels(items, &moved);
}

/// Moves each edge's ends, remembering by how much, so the label follows them.
fn shift_edge_paths(
    items: &mut [Item],
    boxes: &HashMap<String, Rect>,
    deltas: &HashMap<String, Point>,
    moved: &mut HashMap<String, Vec<Point>>,
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
                for command in &mut path.path.commands {
                    let Some(point) = command_end(command) else {
                        continue;
                    };
                    let Some(delta) = touching(&point, boxes, deltas) else {
                        continue;
                    };
                    shift_command(command, delta);
                    moved.entry(source.id.clone()).or_default().push(delta);
                }
            }
            Item::Group(group) => shift_edge_paths(&mut group.items, boxes, deltas, moved),
            _ => {}
        }
    }
}

/// Moves an edge's label by how far its ends moved on average.
fn shift_edge_labels(items: &mut [Item], moved: &HashMap<String, Vec<Point>>) {
    for item in items {
        match item {
            Item::Text(text) => {
                if !is_edge(text.source.as_ref()) {
                    continue;
                }
                let Some(deltas) = text
                    .source
                    .as_ref()
                    .and_then(|source| moved.get(&source.id))
                else {
                    continue;
                };
                if deltas.is_empty() {
                    continue;
                }
                let (dx, dy) = deltas
                    .iter()
                    .fold((0.0, 0.0), |(x, y), delta| (x + delta.x, y + delta.y));
                let count = deltas.len() as f64;
                text.position =
                    Point::new(text.position.x + dx / count, text.position.y + dy / count);
            }
            Item::Group(group) => shift_edge_labels(&mut group.items, moved),
            _ => {}
        }
    }
}

fn is_edge(source: Option<&polterware_mermaid::scene::SourceRef>) -> bool {
    source.is_some_and(|source| source.kind == SourceKind::Edge)
}

/// The pinned node whose box holds `point`, and how far that node moved.
fn touching(
    point: &Point,
    boxes: &HashMap<String, Rect>,
    deltas: &HashMap<String, Point>,
) -> Option<Point> {
    boxes.iter().find_map(|(id, rect)| {
        let delta = deltas.get(id)?;
        let dx = (rect.x - point.x).max(point.x - rect.right()).max(0.0);
        let dy = (rect.y - point.y).max(point.y - rect.bottom()).max(0.0);
        (dx.hypot(dy) <= END_TOLERANCE).then_some(*delta)
    })
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

/// Moves a command's end point, keeping its control points where the layout put them.
fn shift_command(command: &mut PathCommand, delta: Point) {
    let (x, y) = match command {
        PathCommand::MoveTo { x, y } | PathCommand::LineTo { x, y } => (x, y),
        PathCommand::CubicTo { x, y, .. }
        | PathCommand::QuadTo { x, y, .. }
        | PathCommand::ArcTo { x, y, .. } => (x, y),
        PathCommand::Close => return,
    };
    *x += delta.x;
    *y += delta.y;
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

    #[test]
    fn a_flowchart_draws_and_a_pin_moves_its_node() {
        let source = "flowchart LR\n  a[Start] --> b[End]\n";
        let scene = render(source, "{}", &FallbackMeasure, &test_palette()).expect("a scene");
        assert!(!scene.is_empty());

        let layout = r#"{"nodes":{"a":{"x":-40,"y":-30,"pinned":true}}}"#;
        let pinned = render(source, layout, &FallbackMeasure, &test_palette()).expect("a scene");
        let mut boxes = HashMap::new();
        collect_node_boxes(&pinned.items, Transform::IDENTITY, &mut boxes);
        let start = boxes.get("a").expect("the pinned node");
        assert!((start.x + 40.0).abs() < 0.5, "x: {}", start.x);
        assert!((start.y + 30.0).abs() < 0.5, "y: {}", start.y);
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
        let scene = render(
            "flowchart LR\n  a[\"A\"]\n  style a fill:#ff0000,stroke-width:2px\n",
            "{}",
            &FallbackMeasure,
            &test_palette(),
        )
        .expect("a scene");
        assert_eq!(
            first_rect_fill(&scene.items).as_deref(),
            Some("#ff0000"),
            "the style line did not reach the shape"
        );
    }

    #[test]
    fn the_theme_replaces_mermaids_default_fill_and_stroke() {
        let scene = render(
            "flowchart LR\n  a[Start]\n",
            "{}",
            &FallbackMeasure,
            &test_palette(),
        )
        .expect("a scene");
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
        let scene = render(
            "flowchart LR\n  a[Start]\n",
            "{}",
            &FallbackMeasure,
            &palette,
        )
        .expect("a scene");
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
        let scene = render(
            "flowchart LR\n  a[Start] --> b[End]\n",
            "{}",
            &FallbackMeasure,
            &palette,
        )
        .expect("a scene");
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
        let unpinned = render(source, "{}", &FallbackMeasure, &test_palette()).expect("a scene");
        assert_centered(&unpinned, "a");

        let layout = r#"{"nodes":{"a":{"x":40,"y":30,"width":220,"height":90,"pinned":true}}}"#;
        let pinned = render(source, layout, &FallbackMeasure, &test_palette()).expect("a scene");
        let box_ = node_box(&pinned, "a");
        assert!((box_.0 - 40.0).abs() < 0.5, "x: {}", box_.0);
        assert!((box_.1 - 30.0).abs() < 0.5, "y: {}", box_.1);
        assert!((box_.2 - 220.0).abs() < 0.5, "width: {}", box_.2);
        assert!((box_.3 - 90.0).abs() < 0.5, "height: {}", box_.3);
        assert_centered(&pinned, "a");
    }

    #[test]
    fn the_reported_unverified_canvas_keeps_every_label_centered() {
        let source = "flowchart LR\n  a[\"Storage\"]@{ kind: database }\n  b[\"AI\"]@{ kind: llm }\n  c[\"Storage\"]@{ kind: storage }\n  class a,b,c unverified\n";
        let layout = r#"{"nodes":{"a":{"x":0,"y":0,"width":160,"height":64,"pinned":true},"b":{"x":200,"y":0,"width":160,"height":64,"pinned":true},"c":{"x":400,"y":0,"width":160,"height":64,"pinned":true}}}"#;
        let scene = render(source, layout, &FallbackMeasure, &test_palette()).expect("a scene");
        for id in ["a", "b", "c"] {
            assert_centered(&scene, id);
        }
        // The class line draws its dash even before the export writes a classDef.
        let group = node_group(&scene.items, "a").expect("the node");
        assert!(
            has_dashed_shape(&group.items),
            "the unverified class did not draw a dashed outline"
        );
    }

    fn has_dashed_shape(items: &[Item]) -> bool {
        items.iter().any(|item| match item {
            Item::Path(path) => !path.style.dash.is_empty(),
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

    #[test]
    fn a_parse_error_reads_as_a_line_and_column() {
        let error = render(
            "flowchart LR\n  a@{ shape: nonsense }\n",
            "{}",
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
            &FallbackMeasure,
            &test_palette(),
        )
        .expect_err("not a flowchart");
        assert!(unsupported(&error));
    }

    #[test]
    fn an_empty_canvas_draws_nothing_rather_than_failing() {
        let scene = render("flowchart LR\n", "{}", &FallbackMeasure, &test_palette())
            .expect("an empty scene");
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
