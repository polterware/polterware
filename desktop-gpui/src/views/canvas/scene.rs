//! How a canvas node and its connections are drawn (`node-renderer.tsx`, `canvas.css`),
//! shared by the editor, the list's miniatures, and the shape library's samples.
//!
//! Each node is its screen box: the outline painted by polterware-ui under its text, both at
//! the viewport's zoom, as the Electron editor scales its SVG group. Three kinds of node:
//!
//! - A component or a Structure node is a semantic card in its family's silhouette: the
//!   type's 28px icon tile, its label (13px/500, 1.25, at most two lines), and the subtitle
//!   naming the type and the technology (11px muted). A domain heads its group from the top,
//!   18px down. A referenced Structure node's title, kind, and technology win over the
//!   canvas's own.
//! - A drawing shape draws its outline; its label sits in the middle (from the top on a frame,
//!   under the drawing on an actor) in the style's size, weight, and alignment.
//! - A note is a sticky note: its title (14px/600) over its body (12px, 1.45) from the top
//!   left, 12px and 14px in, clear of the fold. Text has no surface; any other kind is a
//!   rounded card.

use gpui::{
    AnyElement, FontWeight, IntoElement, Rgba, SharedString, div, prelude::*, px, relative,
};
use polterware_ui::canvas::{
    EdgePaint, Line, Pt, ShapePaint, Viewport, edges_layer, route, selection_frame, shape_of,
    shape_surface, sticky_surface,
};
use polterware_ui::chip::chip_tone;
use polterware_ui::icon_tile::{IconTile, TileTone};
use polterware_ui::icons::{Icon, icon_colored};
use polterware_ui::material::face;
use polterware_ui::theme::{StatusTone, Theme};

use crate::canvas::catalog::{
    Variant, backdrop, component_of, edge_style, kind_label, node_icon, silhouette, structure_kind,
    variant,
};
use crate::canvas::document::{Doc, Edge, Node, ShapeStyle, TextAlign, read_style};
use crate::canvas::references::{StoreEntity, linked_facts, reference_state};
/// `.pw-status-chip`'s radius.
const CHIP_RADIUS: f32 = 6.0;

/// `fillToken`: the face a fill tag paints; `none` paints none.
pub fn fill_color(tag: &str, theme: &Theme) -> Option<Rgba> {
    Some(match tag {
        "none" => return None,
        "yellow" => theme.status_warning.background,
        "blue" => theme.status_info.background,
        "green" => theme.status_success.background,
        "red" => theme.status_danger.background,
        _ => theme.color_surface_subtle,
    })
}

/// `fillTextToken`: the ink on a fill tag.
pub fn fill_ink(tag: &str, theme: &Theme) -> Rgba {
    match tag {
        "yellow" => theme.status_warning.foreground,
        "blue" => theme.status_info.foreground,
        "green" => theme.status_success.foreground,
        "red" => theme.status_danger.foreground,
        _ => theme.color_text_primary,
    }
}

/// `strokeToken`: the line a stroke tag draws; `none` draws none.
pub fn stroke_color(tag: &str, theme: &Theme) -> Option<Rgba> {
    Some(match tag {
        "none" => return None,
        "neutral" => theme.color_text_muted,
        "yellow" => theme.color_warning,
        "blue" => theme.color_info,
        "green" => theme.color_brand,
        "red" => theme.color_danger,
        _ => theme.color_text_muted,
    })
}

/// The outline as polterware-ui paints it (`ShapeSurface`).
fn surface(
    path: String,
    node: &Node,
    style: &ShapeStyle,
    stroke_only: bool,
    family_dashed: bool,
    marks: Option<String>,
    theme: &Theme,
) -> ShapePaint {
    let dashed = style.dashed || family_dashed;
    let face = if stroke_only {
        None
    } else if style.fill == "neutral" {
        Some(theme.color_surface)
    } else {
        fill_color(&style.fill, theme)
    };
    let lit = !stroke_only && style.stroke == "neutral" && style.stroke_width == 1 && !dashed;
    let line = if lit {
        Line::Lit {
            top: theme.material_rim_strong,
            rest: theme.material_highlight_strong,
        }
    } else if style.stroke == "neutral" {
        Line::Color(theme.tone_neutral.rim)
    } else {
        stroke_color(&style.stroke, theme).map_or(Line::None, Line::Color)
    };
    let mark_color = if style.stroke == "neutral" {
        theme.tone_neutral.rim
    } else {
        stroke_color(&style.stroke, theme).unwrap_or(theme.tone_neutral.rim)
    };
    ShapePaint {
        path,
        size: (node.width, node.height),
        marks,
        face,
        line,
        width: f32::from(style.stroke_width),
        dashed,
        mark_color,
        drop: lit.then_some(theme.material_drop),
    }
}

/// What a node draws besides its outline.
pub struct NodeDecor {
    pub selected: bool,
    /// The editor's chips and reference icons; the miniatures draw none.
    pub decorations: bool,
    /// The label being typed in, in place of the label.
    pub editing: Option<AnyElement>,
}

/// The node's label: the cited record's title, or its own name.
pub fn node_title(node: &Node, entities: &[StoreEntity]) -> String {
    match &node.reference {
        Some(reference) => reference_state(Some(reference), entities).title,
        None => node.name.clone(),
    }
}

/// Only shapes that stand for something can be unverified; drawings carry no claim.
pub fn unverified(node: &Node, entities: &[StoreEntity]) -> bool {
    if node.reference.is_none() && !structure_kind(&node.kind) {
        return false;
    }
    !reference_state(node.reference.as_ref(), entities).referenced
}

/// `EntityIcon`: the sign of what a node cites.
pub fn entity_icon(kind: &str) -> Icon {
    match kind {
        "card" => Icon::ListChecks,
        "decision" => Icon::ShieldCheck,
        "node" => Icon::GitBranch,
        _ => Icon::ScrollText,
    }
}

/// A node's screen box at `viewport`, with its outline, text, and selection.
pub fn node_view(
    node: &Node,
    entities: &[StoreEntity],
    viewport: Viewport,
    decor: NodeDecor,
    theme: &Theme,
) -> AnyElement {
    let zoom = viewport.zoom;
    let at = viewport.to_screen(Pt::new(node.x, node.y));
    let style = read_style(&node.props);
    let semantic = component_of(&node.kind).is_some() || structure_kind(&node.kind);
    let title = node_title(node, entities);
    let flag = decor.decorations && unverified(node, entities);
    let chip = flag.then(|| {
        warn_chip(
            if node.reference.is_some() {
                "Missing / Unverified"
            } else {
                "Unverified"
            },
            zoom,
            theme,
        )
    });
    let content = if semantic {
        semantic_card(
            node,
            entities,
            &style,
            title,
            chip,
            decor.editing,
            zoom,
            theme,
        )
    } else if let Some(definition) = shape_of(&node.kind) {
        let path = (definition.path)(node.width, node.height);
        let outline = surface(
            path,
            node,
            &style,
            definition.stroke_only,
            false,
            None,
            theme,
        );
        let ink = if definition.label_below {
            theme.color_text_primary
        } else {
            fill_ink(&style.fill, theme)
        };
        let icon = (decor.decorations && node.reference.is_some()).then(|| {
            let kind = node
                .reference
                .as_ref()
                .map(|r| r.kind.as_str())
                .unwrap_or("");
            icon_colored(entity_icon(kind), 14.0 * zoom, ink)
        });
        let label = decor
            .editing
            .unwrap_or_else(|| styled_label(title, &style, ink, zoom).into_any_element());
        let text = div()
            .absolute()
            .left_0()
            .w(px(node.width * zoom))
            .when(definition.label_below, |text| {
                text.top(px((node.height + 4.0) * zoom)).h(px(32.0 * zoom))
            })
            .when(!definition.label_below, |text| text.top_0().h_full())
            .flex()
            .flex_col()
            .gap(px(4.0 * zoom))
            .p(px(8.0 * zoom))
            .map(|text| {
                if definition.label_top {
                    text.justify_start()
                } else {
                    text.justify_center()
                }
            })
            .text_color(ink)
            .children(icon)
            .children(chip)
            .child(label);
        div()
            .size_full()
            .child(shape_surface(outline, zoom))
            .child(text)
            .into_any_element()
    } else if variant(&node.kind) == Variant::Sticky {
        let color_tag = match node.text("color") {
            color if color.is_empty() => "yellow".to_string(),
            color => color,
        };
        let color = fill_color(&color_tag, theme).unwrap_or(theme.color_surface_subtle);
        let ink = fill_ink(&color_tag, theme);
        let body = node.text("body");
        let label = decor.editing.unwrap_or_else(|| {
            div()
                .w_full()
                .text_size(px(14.0 * zoom))
                .font_weight(FontWeight::SEMIBOLD)
                .line_height(relative(1.3))
                .child(title)
                .into_any_element()
        });
        div()
            .size_full()
            .child(sticky_surface((node.width, node.height), color, ink, zoom))
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .flex_col()
                    .gap(px(4.0 * zoom))
                    .pt(px(12.0 * zoom))
                    .px(px(14.0 * zoom))
                    .pb(px(22.0 * zoom))
                    .overflow_hidden()
                    .text_color(ink)
                    .children(chip)
                    .child(label)
                    .when(!body.is_empty(), |note| {
                        note.child(
                            div()
                                .min_h_0()
                                .flex_1()
                                .overflow_hidden()
                                .text_size(px(12.0 * zoom))
                                .line_height(relative(1.45))
                                .child(body),
                        )
                    }),
            )
            .into_any_element()
    } else {
        let card = variant(&node.kind) != Variant::Text;
        let ink = fill_ink(&style.fill, theme);
        let label = decor
            .editing
            .unwrap_or_else(|| styled_label(title, &style, ink, zoom).into_any_element());
        div()
            .size_full()
            .when(card, |node_box| {
                node_box.child(shape_surface(
                    surface(
                        polterware_ui::canvas::shapes::rounded_by(8.0, node.width, node.height),
                        node,
                        &style,
                        false,
                        false,
                        None,
                        theme,
                    ),
                    zoom,
                ))
            })
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(4.0 * zoom))
                    .p(px(12.0 * zoom))
                    .text_color(ink)
                    .children(chip)
                    .child(label),
            )
            .into_any_element()
    };
    div()
        .absolute()
        .left(px(at.x))
        .top(px(at.y))
        .w(px(node.width * zoom))
        .h(px(node.height * zoom))
        .child(content)
        .when(decor.selected, |node_box| {
            node_box.child(selection_frame(
                (node.width, node.height),
                zoom,
                theme.color_info,
            ))
        })
        .into_any_element()
}

/// The warn `StatusChip` at the canvas's zoom (11px/500 ink, 3px 9px in, radius 6), drawn
/// here because a chip inside a node scales with it.
fn warn_chip(label: &'static str, zoom: f32, theme: &Theme) -> AnyElement {
    let set = chip_tone(theme, StatusTone::Warn);
    let radius = CHIP_RADIUS * zoom;
    div()
        .relative()
        .flex_none()
        .flex()
        .items_center()
        .rounded(px(radius))
        .border(px(zoom))
        .border_color(set.frame)
        .bg(face(set.top, set.bottom))
        .px(px(9.0 * zoom))
        .py(px(3.0 * zoom))
        .text_size(px(11.0 * zoom))
        .line_height(px(13.2 * zoom))
        .font_weight(FontWeight::MEDIUM)
        .whitespace_nowrap()
        .text_color(set.ink)
        .child(
            div()
                .absolute()
                .inset_0()
                .rounded(px((radius - zoom).max(0.0)))
                .border(px(zoom))
                .border_color(set.rim),
        )
        .child(label)
        .into_any_element()
}

/// A drawing's label in its style: 12, 14, or 16px, semibold when bold, aligned.
pub fn styled_label(title: String, style: &ShapeStyle, ink: Rgba, zoom: f32) -> impl IntoElement {
    div()
        .w_full()
        .text_size(px(style.text_size.px() * zoom))
        .when(style.bold, |label| label.font_weight(FontWeight::SEMIBOLD))
        .text_color(ink)
        .map(|label| match style.text_align {
            TextAlign::Left => label.text_left(),
            TextAlign::Center => label.text_center(),
            TextAlign::Right => label.text_right(),
        })
        .child(title)
}

#[allow(clippy::too_many_arguments)]
fn semantic_card(
    node: &Node,
    entities: &[StoreEntity],
    style: &ShapeStyle,
    title: String,
    chip: Option<AnyElement>,
    editing: Option<AnyElement>,
    zoom: f32,
    theme: &Theme,
) -> AnyElement {
    let Some(outline) = silhouette(&node.kind) else {
        return div().into_any_element();
    };
    let (w, h) = (node.width, node.height);
    let inset = (outline.inset)(w, h);
    let linked = linked_facts(node, entities);
    let kind = linked
        .as_ref()
        .and_then(|facts| facts.kind.clone())
        .unwrap_or_else(|| node.kind.clone());
    let technology = linked
        .as_ref()
        .and_then(|facts| facts.tech.clone())
        .unwrap_or_else(|| node.text("technology"));
    let icon = node_icon(&kind)
        .or_else(|| node_icon(&node.kind))
        .unwrap_or(Icon::Blockchain01Icon);
    let type_label = component_of(&kind)
        .map(|item| item.label.to_string())
        .unwrap_or_else(|| kind_label(&node.kind));
    let subtitle: SharedString = [type_label, technology]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" / ")
        .into();
    let paint = surface(
        (outline.path)(w, h),
        node,
        style,
        false,
        outline.dashed,
        outline.marks.map(|marks| marks(w, h)),
        theme,
    );
    let group = node.kind == "domain";
    let ink = fill_ink(&style.fill, theme);
    let mut tile = IconTile::new(icon)
        .size(28.0 * zoom, 16.0 * zoom)
        .tone(TileTone::Architecture);
    if outline.round {
        tile = tile.round();
    }
    let label = editing.unwrap_or_else(|| {
        div()
            .max_w_full()
            .text_size(px(13.0 * zoom))
            .font_weight(FontWeight::MEDIUM)
            .line_height(relative(1.25))
            .line_clamp(2)
            .child(title)
            .into_any_element()
    });
    let card = div()
        .absolute()
        .left(px(inset.left * zoom))
        .top(px(inset.top * zoom))
        .w(px((w - inset.left - inset.right).max(0.0) * zoom))
        .h(px((h - inset.top - inset.bottom).max(0.0) * zoom))
        .flex()
        .map(|card| {
            if group {
                card.items_start().pt(px(18.0 * zoom))
            } else {
                card.items_center().pt(px(8.0 * zoom))
            }
        })
        .gap(px(10.0 * zoom))
        .pr(px(12.0 * zoom))
        .pb(px(8.0 * zoom))
        .pl(px(10.0 * zoom))
        .overflow_hidden()
        .text_color(ink)
        .child(tile)
        .child(
            div()
                .min_w_0()
                .flex_1()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(2.0 * zoom))
                .children(chip)
                .child(label)
                .child(
                    div()
                        .max_w_full()
                        .text_size(px(11.0 * zoom))
                        .line_height(relative(1.3))
                        .text_color(theme.color_text_muted)
                        .truncate()
                        .child(subtitle),
                ),
        );
    div()
        .size_full()
        .child(shape_surface(paint, zoom))
        .child(card)
        .into_any_element()
}

/// A connection's label: its own, else its kind's (none for `relates`).
pub fn edge_label(edge: &Edge) -> String {
    let own = crate::canvas::document::property_text(&edge.props, "label");
    if !own.is_empty() {
        own
    } else if edge.kind == "relates" {
        String::new()
    } else {
        kind_label(&edge.kind)
    }
}

/// The routes of a document's connections, in document units.
pub fn edge_routes(doc: &Doc) -> Vec<(String, Vec<Pt>)> {
    doc.edges
        .iter()
        .filter_map(|edge| {
            let source = doc.node(&edge.from)?;
            let target = doc.node(&edge.to)?;
            Some((edge.id.clone(), route(source.rect(), target.rect())))
        })
        .collect()
}

/// The connections painted at `viewport`, each in the color `color` gives it.
pub fn edges_view(
    doc: &Doc,
    viewport: Viewport,
    color: impl Fn(&Edge) -> Rgba,
) -> impl IntoElement {
    let routes = edge_routes(doc);
    let edges = doc
        .edges
        .iter()
        .filter_map(|edge| {
            let points = routes.iter().find(|(id, _)| *id == edge.id)?.1.clone();
            let (arrow, dashed) = edge_style(&edge.kind);
            Some(EdgePaint {
                points,
                color: color(edge),
                dashed,
                arrow,
            })
        })
        .collect();
    edges_layer(edges, viewport)
}

/// The document's nodes in paint order: frames first, the rest after the connections.
pub fn layers(doc: &Doc) -> (Vec<&Node>, Vec<&Node>) {
    doc.nodes.iter().partition(|node| backdrop(&node.kind))
}
