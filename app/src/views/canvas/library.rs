//! The shape library (`shape-library.tsx`): a search over the kinds this canvas allows, then a
//! section per group of them, then Project structure with the project's domains and
//! components. Each tile draws its kind as it lands on the canvas, fitted to a 52px box with
//! its name under it (`.pw-canvas-library-tile`: 6px 6px 4px padding, 11px secondary name,
//! primary on hover); a click adds it at the well's center, a drag drops it where it lands.
//! Technology names match the search too, so "redis" finds Storage.

use std::collections::HashMap;
use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, SharedString, WeakEntity, Window, div,
    prelude::*, px,
};
use polterware_ui::canvas::{Viewport, shape_of};
use polterware_ui::icons::{Icon, icon_colored};
use polterware_ui::patterns::{CollapsibleSection, SectionBody};
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::Theme;

use super::editor::{CanvasEditor, DragPreview, LibraryDrag};
use super::scene::{NodeDecor, node_view};
use crate::canvas::catalog::{
    PaletteGroup, component_of, default_props, family_for, initial_geometry, kind_label,
    palette_label, palette_search_text,
};
use crate::canvas::document::{Node, Props};
use crate::canvas::references::{StoreEntity, structure_entities};

/// Room around a drawing in its tile, and under a shape whose label sits below it.
const PAD: f32 = 12.0;
const LABEL_BELOW: f32 = 36.0;
pub(super) const ART_HEIGHT: f32 = 52.0;
const STRUCTURE: &str = "structure";
const LEVELS: [(&str, &str); 2] = [("domain", "Domains"), ("component", "Components")];

pub(super) struct LibraryState {
    search: Entity<TextInput>,
    query: String,
    sections: HashMap<&'static str, Entity<CollapsibleSection>>,
}

impl LibraryState {
    pub(super) fn new(groups: &[PaletteGroup], cx: &mut Context<CanvasEditor>) -> Self {
        let search = cx.new(|cx| TextInput::new(cx).placeholder("Search shapes").pill());
        cx.observe(&search, |this, input, cx| {
            let text = input.read(cx).text().to_string();
            if text != this.library.query {
                this.library.query = text;
                cx.notify();
            }
        })
        .detach();
        let mut sections = HashMap::new();
        let titles = groups
            .iter()
            .map(|group| (group.id, group.title))
            .chain([(STRUCTURE, "Project structure")]);
        for (id, title) in titles {
            let section = cx.new(|cx| {
                let mut section = CollapsibleSection::new(title, cx);
                section.set_compact(true);
                if id == STRUCTURE {
                    section.set_plain_icon(Icon::Network);
                }
                section
            });
            cx.observe(&section, |_, _, cx| cx.notify()).detach();
            sections.insert(id, section);
        }
        Self {
            search,
            query: String::new(),
            sections,
        }
    }
}

/// The text a sample of `kind` shows, so a tile reads as it will on a canvas (`samples.tsx`).
fn sample_name(kind: &str) -> String {
    let own = match kind {
        "note" => Some("Retry budget"),
        "text" => Some("Heading text"),
        "domain" => Some("Billing"),
        "component" => Some("Checkout"),
        _ => None,
    };
    let family = family_for(kind).and_then(|family| {
        Some(match family.id {
            "client" => "Storefront",
            "compute" => "Orders",
            "workflow" => "Nightly sync",
            "storage" => "Orders DB",
            "messaging" => "Order events",
            "ai" => "Assistant",
            "edge" => "Public edge",
            "security" => "Sign-in",
            "platform" => "Production",
            "observability" => "Health",
            "external" => "Stripe",
            "people" => "Customers",
            "document" => "Orders API",
            _ => return None,
        })
    });
    own.or(family)
        .map(str::to_string)
        .unwrap_or_else(|| kind_label(kind))
}

/// The node the editor would insert for `kind`, at the origin, with sample text.
pub(super) fn sample_node(kind: &str, title: Option<&str>, id: &str) -> Node {
    let (width, height, _) = initial_geometry(kind);
    let mut props: Props = default_props(kind);
    if let Some(technology) = component_of(kind).and_then(|item| item.technologies.first()) {
        props.insert("technology".into(), (*technology).into());
    }
    if kind == "note" {
        props.insert(
            "body".into(),
            "Cap retries at three per request and alert past one percent.".into(),
        );
    }
    Node {
        id: id.to_string(),
        name: title.map_or_else(|| sample_name(kind), str::to_string),
        kind: kind.to_string(),
        x: 0.0,
        y: 0.0,
        width,
        height,
        z: 0,
        props,
        reference: None,
    }
}

/// `node` drawn by the real renderer, fitted into a `width`×`height` box (`NodeSample`:
/// a `PAD` margin, room for a label below, `xMidYMid meet`).
pub(super) fn node_sample(
    node: &Node,
    width: f32,
    height: f32,
    entities: &[StoreEntity],
    theme: &Theme,
) -> AnyElement {
    let below = if shape_of(&node.kind).is_some_and(|shape| shape.label_below) {
        LABEL_BELOW
    } else {
        0.0
    };
    let (box_w, box_h) = (node.width + PAD * 2.0, node.height + below + PAD * 2.0);
    let zoom = (width / box_w).min(height / box_h);
    let viewport = Viewport {
        x: (width - box_w * zoom) / 2.0 + PAD * zoom,
        y: (height - box_h * zoom) / 2.0 + PAD * zoom,
        zoom,
    };
    div()
        .relative()
        .flex_none()
        .w(px(width))
        .h(px(height))
        .child(node_view(
            node,
            entities,
            viewport,
            NodeDecor {
                selected: false,
                decorations: false,
                editing: None,
            },
            theme,
        ))
        .into_any_element()
}

/// One library entry: the drawing over its name; a click adds it, a drag drops it.
fn tile(
    id: SharedString,
    node: Node,
    name: String,
    drag: LibraryDrag,
    art_width: f32,
    editor: WeakEntity<CanvasEditor>,
    cx: &App,
) -> AnyElement {
    let theme = *Theme::global(cx);
    let art = node_sample(&node, art_width, ART_HEIGHT, &[], &theme);
    let (preview_node, preview_name) = (node.clone(), SharedString::from(name.clone()));
    let added = drag.clone();
    let group = SharedString::from(format!("{id}-group"));
    div()
        .id(id)
        .group(group.clone())
        .w_full()
        .min_w_0()
        .overflow_hidden()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .pt(px(6.0))
        .px(px(6.0))
        .pb(px(4.0))
        .rounded(px(theme.radius_control))
        .cursor_pointer()
        .hover(|tile| tile.bg(theme.color_surface_hover))
        .child(art)
        .child(
            // A known width: a percentage leaves the ellipsis nothing to measure against.
            div()
                .w(px(art_width))
                .truncate()
                .text_center()
                .text_size(px(11.0))
                .text_color(theme.color_text_secondary)
                .group_hover(group, |name| name.text_color(theme.color_text_primary))
                .child(name),
        )
        .on_drag(drag, move |_, _, _, cx| {
            let (node, name) = (preview_node.clone(), preview_name.clone());
            cx.new(|_| DragPreview {
                node,
                name,
                width: art_width,
            })
        })
        .on_click(move |_, _, cx| {
            let added = added.clone();
            let _ = editor.update(cx, |editor, cx| {
                editor.insert(&added.kind, None, added.entity, cx);
                cx.notify();
            });
        })
        .into_any_element()
}

/// Tiles two to a row, 4px apart (`.pw-canvas-tiles`).
fn tiles(items: Vec<AnyElement>) -> AnyElement {
    let mut rows = Vec::new();
    let mut items = items.into_iter().peekable();
    while items.peek().is_some() {
        let row: Vec<AnyElement> = items.by_ref().take(2).collect();
        let filler = (row.len() == 1).then(|| div().flex_1().min_w_0());
        rows.push(
            div()
                .flex()
                .gap(px(4.0))
                .children(
                    row.into_iter()
                        .map(|item| div().flex_1().min_w_0().child(item)),
                )
                .children(filler)
                .into_any_element(),
        );
    }
    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .children(rows)
        .into_any_element()
}

fn note(text: impl Into<SharedString>, theme: &Theme) -> gpui::Div {
    div()
        .text_size(px(12.0))
        .line_height(px(12.0 * 1.4))
        .text_color(theme.color_text_muted)
        .child(text.into())
}

impl CanvasEditor {
    pub(super) fn library_view(&mut self, _: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let query = self.library.query.trim().to_lowercase();
        let width = self.library_panel_width(cx);
        // Body padding 12 on each side, two columns 4 apart, each tile padded 6.
        let art_width = ((width - 24.0 - 4.0) / 2.0 - 12.0).max(24.0);
        let editor = cx.entity().downgrade();
        let mut sections: Vec<AnyElement> = Vec::new();
        let mut shown = 0;
        for group in &self.groups {
            let kinds: Vec<String> = group
                .kinds
                .iter()
                .filter(|kind| palette_search_text(kind).contains(&query))
                .cloned()
                .collect();
            if kinds.is_empty() {
                continue;
            }
            shown += kinds.len();
            let Some(section) = self.library.sections.get(group.id).cloned() else {
                continue;
            };
            let editor = editor.clone();
            let body: SectionBody = Rc::new(move |_: &mut Window, cx: &mut App| {
                let items = kinds
                    .iter()
                    .map(|kind| {
                        let label = palette_label(kind);
                        tile(
                            SharedString::from(format!("canvas-library-{kind}")),
                            sample_node(kind, None, &format!("sample-{kind}")),
                            label.clone(),
                            LibraryDrag {
                                kind: kind.clone(),
                                entity: None,
                            },
                            art_width,
                            editor.clone(),
                            cx,
                        )
                    })
                    .collect();
                tiles(items)
            });
            section.update(cx, |section, _| section.set_body(body));
            sections.push(div().w_full().child(section).into_any_element());
        }
        let entities = self.entities(cx);
        let structure: Vec<StoreEntity> = structure_entities(&entities)
            .into_iter()
            .filter(|entity| {
                format!("{} {}", entity.title, entity.level)
                    .to_lowercase()
                    .contains(&query)
            })
            .cloned()
            .collect();
        let structure_count = structure.len();
        if let Some(section) = self.library.sections.get(STRUCTURE).cloned() {
            let editor = editor.clone();
            let body: SectionBody = Rc::new(move |_: &mut Window, cx: &mut App| {
                let theme = *Theme::global(cx);
                let mut out = div().flex().flex_col();
                for (index, (level, title)) in LEVELS.iter().enumerate() {
                    let items: Vec<AnyElement> = structure
                        .iter()
                        .filter(|entity| entity.level == *level)
                        .map(|entity| {
                            tile(
                                SharedString::from(format!("canvas-library-entity-{}", entity.id)),
                                sample_node(level, Some(&entity.title), &entity.id),
                                entity.title.clone(),
                                LibraryDrag {
                                    kind: level.to_string(),
                                    entity: Some(entity.clone()),
                                },
                                art_width,
                                editor.clone(),
                                cx,
                            )
                        })
                        .collect();
                    if items.is_empty() {
                        continue;
                    }
                    out = out
                        .child(
                            div()
                                .mt(px(if index == 0 { 0.0 } else { 8.0 }))
                                .mb(px(4.0))
                                .mx(px(2.0))
                                .text_size(px(11.0))
                                .text_color(theme.color_text_muted)
                                .child(*title),
                        )
                        .child(tiles(items));
                }
                if structure.is_empty() {
                    out = out
                        .child(note("No matching domain or component nodes.", &theme).px(px(8.0)));
                }
                out.into_any_element()
            });
            section.update(cx, |section, _| section.set_body(body));
            sections.push(div().w_full().child(section).into_any_element());
        }
        let nothing = (!query.is_empty() && shown == 0 && structure_count == 0).then(|| {
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(4.0))
                .child(icon_colored(Icon::Search, 14.0, theme.color_text_muted))
                .child(note(
                    format!("No shapes match \u{201c}{}\u{201d}.", self.library.query),
                    &theme,
                ))
        });
        div()
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .pt(px(12.0))
                    .px(px(12.0))
                    .pb(px(8.0))
                    .child(self.library.search.clone()),
            )
            .child(
                div()
                    .id("canvas-library-body")
                    .min_h_0()
                    .flex_1()
                    .overflow_y_scroll()
                    .pt(px(4.0))
                    .px(px(12.0))
                    .pb(px(12.0))
                    .children(sections)
                    .children(nothing),
            )
            .into_any_element()
    }
}
