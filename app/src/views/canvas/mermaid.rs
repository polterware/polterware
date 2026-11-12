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

use std::collections::HashMap;

use polterware_mermaid::DiagramKind;
use polterware_mermaid::config::Config;
use polterware_mermaid::error::{ParseError, RenderError};
use polterware_mermaid::measure::TextMeasure;
use polterware_mermaid::scene::{Item, PathCommand, Point, Rect, Scene, SourceKind, Transform};
use polterware_ui::mermaid::SceneViewport;

/// How near an edge end has to be to a node's box to be read as that node's.
const END_TOLERANCE: f64 = 0.75;
/// The room a scene keeps around its content when pins move nodes past the fitted view box.
const REFIT_PADDING: f64 = 24.0;

/// Renders `source` with the layout's pinned positions applied.
pub(super) fn render(
    source: &str,
    layout_json: &str,
    measure: &dyn TextMeasure,
) -> Result<Scene, RenderError> {
    let preprocessed = polterware_mermaid::preprocess::preprocess(source);
    if let Some(message) = preprocessed.error {
        return Err(RenderError::Parse(ParseError::new(message, None)));
    }
    let kind = polterware_mermaid::diagrams::detect(&preprocessed.body)
        .ok_or_else(|| ParseError::new("No diagram type detected", None))?;
    let mut config = Config::default();
    config.apply(&preprocessed.config)?;
    let mut scene = match kind {
        DiagramKind::Flowchart => polterware_mermaid::diagrams::flowchart::render::draw(
            &preprocessed.body,
            &config,
            measure,
        )?,
        other => return Err(RenderError::Unsupported(format!("{other:?}"))),
    };
    apply_pins(&mut scene, &pins(layout_json));
    Ok(scene)
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

/// The canvas node each diagram id stands for: the export gives a node whose id is not a
/// Mermaid id (`u1`) an `@{ id: ... }` block carrying the canvas's own, and a click in the
/// diagram has to follow it to the inspector.
pub(super) fn node_ids(source: &str) -> HashMap<String, String> {
    let bytes = source.as_bytes();
    let mut ids = HashMap::new();
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
        if let Some(canvas) = field(&source[index + 2..close], "id")
            && !id.is_empty()
            && canvas != id
        {
            ids.insert(id, canvas);
        }
        index = close + 1;
    }
    ids
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

/// The nodes the layout pins, by the id the source gives them.
fn pins(layout_json: &str) -> HashMap<String, Point> {
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
            Some((id.clone(), Point::new(x, y)))
        })
        .collect()
}

/// Moves every pinned node onto its place, and the edge ends that touch it with it.
fn apply_pins(scene: &mut Scene, pins: &HashMap<String, Point>) {
    if pins.is_empty() {
        return;
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
    if deltas.is_empty() {
        return;
    }
    shift_nodes(&mut scene.items, Transform::IDENTITY, &deltas);
    shift_edges(&mut scene.items, &boxes, &deltas);
    refit(scene);
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

    #[test]
    fn a_flowchart_draws_and_a_pin_moves_its_node() {
        let source = "flowchart LR\n  a[Start] --> b[End]\n";
        let scene = render(source, "{}", &FallbackMeasure).expect("a scene");
        assert!(!scene.is_empty());

        let layout = r#"{"nodes":{"a":{"x":-40,"y":-30,"pinned":true}}}"#;
        let pinned = render(source, layout, &FallbackMeasure).expect("a scene");
        let mut boxes = HashMap::new();
        collect_node_boxes(&pinned.items, Transform::IDENTITY, &mut boxes);
        let start = boxes.get("a").expect("the pinned node");
        assert!((start.x + 40.0).abs() < 0.5, "x: {}", start.x);
        assert!((start.y + 30.0).abs() < 0.5, "y: {}", start.y);
    }

    #[test]
    fn a_parse_error_reads_as_a_line_and_column() {
        let error = render(
            "flowchart LR\n  a@{ shape: nonsense }\n",
            "{}",
            &FallbackMeasure,
        )
        .expect_err("a bad shape");
        assert_eq!(
            message(&error),
            "line 2, column 4: No such shape: nonsense."
        );
    }

    #[test]
    fn another_diagram_type_is_unsupported() {
        let error = render("stateDiagram-v2\n  A --> B\n", "{}", &FallbackMeasure)
            .expect_err("not a flowchart");
        assert!(unsupported(&error));
    }

    #[test]
    fn an_empty_canvas_draws_nothing_rather_than_failing() {
        let scene = render("flowchart LR\n", "{}", &FallbackMeasure).expect("an empty scene");
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
