//! The inspector (`canvas-inspector.tsx`, `inspector.tsx`): what the selection is and how to
//! change it, in the right panel.
//!
//! - One shape: its title and kind over the tabs Style (drawing shapes and components only),
//!   Text, Arrange, and Data. Style picks the family and type, the fill and line swatches, the
//!   line width, and its pattern; Text the note color, the label (unless a reference names
//!   it), a note's body, and the type size, alignment, and weight; Arrange the box in
//!   steppers, the stacking, and Snap to grid; Data a component's technology and description,
//!   the project reference, and Promote.
//! - One connection: `Edge: Kind`, its ends, its kind, and its label.
//! - Several shapes: how many, and the six alignments.
//!
//! The panel follows the selection by id, so an edit keeps the open tab and the focused field;
//! another shape starts on its first tab. Typed edits coalesce into one undo step.

use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, Focusable, FontWeight, SharedString,
    WeakEntity, Window, div, prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::canvas::{Capture, FAMILIES, GRID, shape_of};
use polterware_ui::chip::chip;
use polterware_ui::icons::{Icon, icon_colored};
use polterware_ui::menu::MenuItem;
use polterware_ui::number_stepper::{NumberStepper, StepperChanged};
use polterware_ui::patterns::{
    CollapsibleSection, Composer, ComposerDraft, ComposerEvent, ComposerSpec, SectionBody,
};
use polterware_ui::segmented::SegmentedControl;
use polterware_ui::tabs::Tabs;
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::{StatusTone, Theme};

use super::editor::CanvasEditor;
use super::scene::{entity_icon, fill_color, stroke_color};
use crate::canvas::catalog::{
    COMPONENTS, component_of, edge_palette, family_for, kind_label, structure_kind,
};
use crate::canvas::document::{
    COLOR_TAGS, Node, ShapeStyle, TextAlign, TextSize, property_text, read_style, set_reference,
    update_edge, update_node, write_style,
};
use crate::canvas::references::{
    PromotionKind, StoreEntity, build_ref, can_promote, derive_slug, linked_facts, ref_label,
    reference_state,
};
use crate::contracts::CanvasRef;
use crate::views::picker::Picker;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    Style,
    Text,
    Arrange,
    Data,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Geometry {
    X,
    Y,
    Width,
    Height,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Align {
    Left,
    Center,
    Right,
    Top,
    Middle,
    Bottom,
}

const ALIGNMENTS: [(Align, &str, &str); 6] = [
    (Align::Left, "left", "Left"),
    (Align::Center, "center", "Center"),
    (Align::Right, "right", "Right"),
    (Align::Top, "top", "Top"),
    (Align::Middle, "middle", "Middle"),
    (Align::Bottom, "bottom", "Bottom"),
];

pub(super) struct InspectorState {
    /// The shape or connection the panel was filled from.
    shown: Option<String>,
    tab: Tab,
    label: Entity<TextInput>,
    body: Entity<TextInput>,
    technology: Entity<TextInput>,
    description: Entity<TextInput>,
    edge_label: Entity<TextInput>,
    search: Entity<TextInput>,
    query: String,
    steppers: [(Geometry, Entity<NumberStepper>); 4],
    family: Picker,
    kind: Picker,
    edge_kind: Picker,
    component: Entity<CollapsibleSection>,
    reference: Entity<CollapsibleSection>,
    promote: Entity<CollapsibleSection>,
    align: Entity<CollapsibleSection>,
    composer: Entity<Composer>,
    /// The shape being promoted, and to what.
    promotion: Option<(String, PromotionKind)>,
    /// The promotion went through; the Composer submits no second one.
    applied: bool,
}

fn section(
    title: &'static str,
    mark: Icon,
    cx: &mut Context<CanvasEditor>,
) -> Entity<CollapsibleSection> {
    let section = cx.new(|cx| {
        let mut section = CollapsibleSection::new(title, cx);
        section.set_plain_icon(mark);
        section
    });
    cx.observe(&section, |_, _, cx| cx.notify()).detach();
    section
}

fn input(
    cx: &mut Context<CanvasEditor>,
    placeholder: Option<&'static str>,
    multiline: bool,
    on_change: impl Fn(&mut CanvasEditor, String, &mut Context<CanvasEditor>) + 'static,
) -> Entity<TextInput> {
    let field = cx.new(|cx| {
        let mut field = TextInput::new(cx);
        if let Some(placeholder) = placeholder {
            field = field.placeholder(placeholder);
        }
        if multiline {
            field = field.multiline();
        }
        field
    });
    cx.observe(&field, move |this, field, cx| {
        let text = field.read(cx).text().to_string();
        on_change(this, text, cx);
    })
    .detach();
    field
}

/// Writes `key` when `value` has text, and drops it when blank (`ComponentFields.setProp`).
fn set_or_clear(node: &mut Node, key: &str, value: &str) -> bool {
    let before = node.props.get(key).cloned();
    if value.trim().is_empty() {
        node.props.remove(key);
    } else {
        node.props.insert(key.into(), value.into());
    }
    node.props.get(key).cloned() != before
}

impl InspectorState {
    pub(super) fn new(window: &mut Window, cx: &mut Context<CanvasEditor>) -> Self {
        let _ = window;
        let label = input(cx, None, false, |this, text, cx| {
            this.edit_node(
                |node| {
                    if node.reference.is_some() || node.name == text {
                        return false;
                    }
                    node.name = text;
                    true
                },
                Capture::Eventually,
                cx,
            )
        });
        let body = input(cx, None, true, |this, text, cx| {
            this.edit_node(
                |node| {
                    if property_text(&node.props, "body") == text {
                        return false;
                    }
                    node.props.insert("body".into(), text.into());
                    true
                },
                Capture::Eventually,
                cx,
            )
        });
        let technology = input(
            cx,
            Some("e.g. Postgres, React, Go"),
            false,
            |this, text, cx| {
                this.edit_node(
                    |node| set_or_clear(node, "technology", &text),
                    Capture::Eventually,
                    cx,
                )
            },
        );
        let description = input(cx, None, true, |this, text, cx| {
            this.edit_node(
                |node| set_or_clear(node, "description", &text),
                Capture::Eventually,
                cx,
            )
        });
        let edge_label = input(cx, None, false, |this, text, cx| {
            let Some(id) = this.selected_edge() else {
                return;
            };
            if this
                .doc
                .edge(&id)
                .is_none_or(|edge| property_text(&edge.props, "label") == text)
            {
                return;
            }
            let next = update_edge(&this.doc, &id, |edge| {
                edge.props.insert("label".into(), text.into());
            });
            this.change(next, Capture::Eventually, cx);
        });
        let search = input(cx, None, false, |this, text, cx| {
            if this.inspector.query != text {
                this.inspector.query = text;
                cx.notify();
            }
        });
        let stepper = |geometry: Geometry, label: &'static str, cx: &mut Context<CanvasEditor>| {
            let stepper = cx.new(|cx| {
                let stepper = NumberStepper::new(format!("canvas-{label}"), cx).label(label);
                match geometry {
                    Geometry::Width => stepper.min(40.0),
                    Geometry::Height => stepper.min(32.0),
                    _ => stepper,
                }
            });
            cx.subscribe(&stepper, move |this, _, event: &StepperChanged, cx| {
                this.set_geometry(geometry, event.0, cx);
            })
            .detach();
            (geometry, stepper)
        };
        let steppers = [
            stepper(Geometry::X, "X", cx),
            stepper(Geometry::Y, "Y", cx),
            stepper(Geometry::Width, "Width", cx),
            stepper(Geometry::Height, "Height", cx),
        ];
        let composer = cx.new(Composer::new);
        cx.observe(&composer, |_, _, cx| cx.notify()).detach();
        cx.subscribe(&composer, CanvasEditor::promotion_composed)
            .detach();
        Self {
            shown: None,
            tab: Tab::Text,
            label,
            body,
            technology,
            description,
            edge_label,
            search,
            query: String::new(),
            steppers,
            family: Picker::new(cx),
            kind: Picker::new(cx),
            edge_kind: Picker::new(cx),
            component: section("Component", Icon::Server, cx),
            reference: section("Project reference", Icon::Link2, cx),
            promote: section("Promote", Icon::ArrowUp, cx),
            align: section("Align", Icon::LayoutDashboard, cx),
            composer,
            promotion: None,
            applied: false,
        }
    }

    /// The promotion Composer and any open select menu, over everything.
    pub(super) fn layers(&self, cx: &App) -> Vec<AnyElement> {
        let mut out: Vec<AnyElement> = [&self.family, &self.kind, &self.edge_kind]
            .into_iter()
            .filter_map(|picker| picker.overlay(cx))
            .collect();
        out.push(self.composer.clone().into_any_element());
        out
    }
}

/// Shows `text` in `field` unless someone is typing in it.
fn sync(field: &Entity<TextInput>, text: &str, window: &Window, cx: &mut App) {
    let current = field.read(cx);
    if current.text() == text || current.focus_handle(cx).is_focused(window) {
        return;
    }
    let text = text.to_string();
    field.update(cx, |field, cx| field.set_text(text, cx));
}

fn heading(title: String, detail: Option<String>, theme: &Theme) -> AnyElement {
    let line = |text: String, size: f32, weight: FontWeight, color| {
        div()
            .w_full()
            .overflow_hidden()
            .text_ellipsis()
            .whitespace_nowrap()
            .text_size(px(size))
            .line_height(px(size * 1.3))
            .font_weight(weight)
            .text_color(color)
            .child(text)
    };
    div()
        .flex()
        .flex_col()
        .min_w_0()
        .gap(px(2.0))
        .pt(px(12.0))
        .px(px(12.0))
        .pb(px(8.0))
        .child(line(
            title,
            13.0,
            FontWeight::SEMIBOLD,
            theme.color_text_primary,
        ))
        .children(
            detail
                .filter(|detail| !detail.is_empty())
                .map(|detail| line(detail, 11.0, FontWeight::NORMAL, theme.color_text_muted)),
        )
        .into_any_element()
}

fn body(id: &'static str, content: impl IntoElement) -> AnyElement {
    div()
        .id(id)
        .min_h_0()
        .flex_1()
        .overflow_y_scroll()
        .pt(px(4.0))
        .px(px(12.0))
        .pb(px(12.0))
        .child(content)
        .into_any_element()
}

/// `FormStack`: fields 14px apart.
fn form(top: f32) -> gpui::Div {
    div().w_full().flex().flex_col().gap(px(14.0)).pt(px(top))
}

/// `Field`: an 11px/500 muted label over its control, 6px apart, and an optional hint.
fn field(label: &str, control: impl IntoElement, theme: &Theme) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .text_size(px(11.0))
                .line_height(px(11.0 * 1.3))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.color_text_muted)
                .child(label.to_string()),
        )
        .child(control)
}

fn hint(text: impl Into<SharedString>, theme: &Theme) -> gpui::Div {
    div()
        .text_size(px(11.0))
        .line_height(px(11.0 * 1.35))
        .text_color(theme.color_text_muted)
        .child(text.into())
}

fn muted(text: impl Into<SharedString>, size: f32, theme: &Theme) -> gpui::Div {
    div()
        .text_size(px(size))
        .line_height(px(size * 1.4))
        .text_color(theme.color_text_muted)
        .child(text.into())
}

fn act(
    editor: &WeakEntity<CanvasEditor>,
    action: impl Fn(&mut CanvasEditor, &mut Window, &mut Context<CanvasEditor>) + 'static,
) -> impl Fn(&mut Window, &mut App) + 'static {
    let editor = editor.clone();
    move |window, cx| {
        let _ = editor.update(cx, |this, cx| {
            action(this, window, cx);
            cx.notify();
        });
    }
}

/// A row of swatches, one per color tag and `none` (`Swatches`).
#[allow(clippy::too_many_arguments)]
fn swatches(
    id: &'static str,
    label: &str,
    value: &str,
    none: bool,
    token: fn(&str, &Theme) -> Option<gpui::Rgba>,
    pick: impl Fn(&mut CanvasEditor, String, &mut Context<CanvasEditor>) + 'static,
    editor: &WeakEntity<CanvasEditor>,
    theme: &Theme,
) -> AnyElement {
    let pick = Rc::new(pick);
    let tags = COLOR_TAGS
        .iter()
        .copied()
        .chain(none.then_some("none"))
        .map(|tag| {
            let pick = pick.clone();
            let mut button =
                Button::new(SharedString::from(format!("{id}-{tag}")), ButtonKind::Ghost)
                    .size(ButtonSize::Icon)
                    .tooltip(format!("{label}: {tag}"))
                    .selected(value == tag)
                    .on_click(act(editor, move |this, _, cx| {
                        pick(this, tag.to_string(), cx)
                    }));
            button = match token(tag, theme).filter(|_| tag != "none") {
                Some(color) => button.leading(
                    div()
                        .size(px(14.0))
                        .rounded_full()
                        .border_1()
                        .border_color(theme.color_border_strong)
                        .bg(color),
                ),
                None => button.icon(Icon::X).icon_size(14.0),
            };
            button.into_any_element()
        });
    field(
        label,
        div().flex().flex_wrap().gap(px(4.0)).children(tags),
        theme,
    )
    .into_any_element()
}

/// `Segments`: a segmented control under its label.
fn segments(
    id: &'static str,
    label: &str,
    options: &[&'static str],
    selected: usize,
    pick: impl Fn(&mut CanvasEditor, usize, &mut Context<CanvasEditor>) + 'static,
    editor: &WeakEntity<CanvasEditor>,
    theme: &Theme,
) -> AnyElement {
    let editor = editor.clone();
    let control = SegmentedControl::new(id)
        .options(options.iter().copied())
        .selected(selected)
        .on_select(move |index, _, cx| {
            let _ = editor.update(cx, |this, cx| {
                pick(this, index, cx);
                cx.notify();
            });
        });
    field(label, control, theme).into_any_element()
}

fn small(id: impl Into<SharedString>, label: impl Into<SharedString>, kind: ButtonKind) -> Button {
    Button::new(id.into(), kind)
        .size(ButtonSize::Sm)
        .label(label)
}

impl CanvasEditor {
    pub(super) fn library_panel_width(&self, cx: &App) -> f32 {
        self.library_panel.read(cx).width()
    }

    fn selected_node(&self) -> Option<String> {
        match self.selection.nodes.as_slice() {
            [id] => Some(id.clone()),
            _ => None,
        }
    }

    fn selected_edge(&self) -> Option<String> {
        if !self.selection.nodes.is_empty() {
            return None;
        }
        self.selection.edges.first().cloned()
    }

    /// Changes the one selected shape, when `change` says it changed.
    fn edit_node(
        &mut self,
        change: impl FnOnce(&mut Node) -> bool,
        capture: Capture,
        cx: &mut Context<Self>,
    ) {
        let Some(id) = self.selected_node() else {
            return;
        };
        let Some(mut node) = self.doc.node(&id).cloned() else {
            return;
        };
        if !change(&mut node) {
            return;
        }
        let next = update_node(&self.doc, &id, |current| *current = node);
        self.change(next, capture, cx);
    }

    fn set_style(&mut self, change: impl FnOnce(&mut ShapeStyle), cx: &mut Context<Self>) {
        self.edit_node(
            |node| {
                let mut style = read_style(&node.props);
                change(&mut style);
                write_style(&mut node.props, &style);
                true
            },
            Capture::Immediately,
            cx,
        );
    }

    fn set_geometry(&mut self, geometry: Geometry, value: f64, cx: &mut Context<Self>) {
        if !value.is_finite() {
            return;
        }
        let value = value as f32;
        self.edit_node(
            |node| {
                let (slot, min) = match geometry {
                    Geometry::X => (&mut node.x, f32::NEG_INFINITY),
                    Geometry::Y => (&mut node.y, f32::NEG_INFINITY),
                    Geometry::Width => (&mut node.width, 40.0),
                    Geometry::Height => (&mut node.height, 32.0),
                };
                let next = value.round().max(min);
                if *slot == next {
                    return false;
                }
                *slot = next;
                true
            },
            Capture::Eventually,
            cx,
        );
    }

    /// The component types of `family` this canvas allows.
    fn allowed(&self, family: &str) -> Vec<&'static crate::canvas::catalog::Component> {
        COMPONENTS
            .iter()
            .filter(|item| {
                item.family == family && self.palette.iter().any(|kind| kind == item.kind)
            })
            .collect()
    }

    /// Moves a component to another family, at that family's default type and size.
    fn switch_family(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(next) = FAMILIES.iter().find(|item| item.id == id) else {
            return;
        };
        let kinds: Vec<&str> = self.allowed(id).iter().map(|item| item.kind).collect();
        let Some(first) = kinds.first().copied() else {
            return;
        };
        let current = self
            .selected_node()
            .and_then(|node| self.doc.node(&node).map(|node| node.kind.clone()));
        if current
            .as_deref()
            .and_then(family_for)
            .map(|family| family.id)
            == Some(id)
        {
            return;
        }
        let kind = if kinds.contains(&next.kind) {
            next.kind
        } else {
            first
        };
        self.edit_node(
            |node| {
                node.kind = kind.to_string();
                (node.width, node.height) = next.geometry;
                true
            },
            Capture::Immediately,
            cx,
        );
    }

    /// Brings the shape to the front or sends it to the back: the graph order is the stacking.
    fn order(&mut self, front: bool, cx: &mut Context<Self>) {
        let Some(id) = self.selected_node() else {
            return;
        };
        let Some(node) = self.doc.node(&id).cloned() else {
            return;
        };
        let mut next = self.doc.clone();
        let rest: Vec<Node> = next.nodes.drain(..).filter(|item| item.id != id).collect();
        next.nodes = if front {
            rest.into_iter().chain([node]).collect()
        } else {
            [node].into_iter().chain(rest).collect()
        };
        for (z, node) in next.nodes.iter_mut().enumerate() {
            node.z = z as i32;
        }
        self.change(next, Capture::Immediately, cx);
    }

    fn link(&mut self, reference: Option<CanvasRef>, cx: &mut Context<Self>) {
        let Some(id) = self.selected_node() else {
            return;
        };
        let next = set_reference(&self.doc, &id, reference);
        self.change(next, Capture::Immediately, cx);
    }

    fn align(&mut self, align: Align, cx: &mut Context<Self>) {
        let ids = self.selection.nodes.clone();
        let nodes: Vec<&Node> = ids.iter().filter_map(|id| self.doc.node(id)).collect();
        if nodes.is_empty() {
            return;
        }
        let left = nodes
            .iter()
            .map(|node| node.x)
            .fold(f32::INFINITY, f32::min);
        let top = nodes
            .iter()
            .map(|node| node.y)
            .fold(f32::INFINITY, f32::min);
        let right = nodes
            .iter()
            .map(|node| node.x + node.width)
            .fold(f32::NEG_INFINITY, f32::max);
        let bottom = nodes
            .iter()
            .map(|node| node.y + node.height)
            .fold(f32::NEG_INFINITY, f32::max);
        let mut next = self.doc.clone();
        for node in next.nodes.iter_mut().filter(|node| ids.contains(&node.id)) {
            match align {
                Align::Left => node.x = left,
                Align::Center => node.x = (left + right) / 2.0 - node.width / 2.0,
                Align::Right => node.x = right - node.width,
                Align::Top => node.y = top,
                Align::Middle => node.y = (top + bottom) / 2.0 - node.height / 2.0,
                Align::Bottom => node.y = bottom - node.height,
            }
        }
        self.change(next, Capture::Immediately, cx);
    }

    fn open_promotion(&mut self, kind: PromotionKind, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.selected_node() else {
            return;
        };
        let Some(node) = self.doc.node(&id) else {
            return;
        };
        let entities = self.entities(cx);
        let existing: Vec<String> = entities
            .iter()
            .filter(|entity| entity.kind == kind.wire())
            .map(|entity| entity.slug.clone())
            .collect();
        let slug = derive_slug(&node.name, &existing);
        let mut spec = ComposerSpec::new("Title", "Promote").edit();
        let slugged = kind != PromotionKind::Decision;
        if slugged {
            spec = spec.field("Slug", "lowercase-kebab");
        }
        spec = spec.initial(ComposerDraft {
            title: node.name.clone(),
            fields: if slugged { vec![slug] } else { Vec::new() },
            ..ComposerDraft::default()
        });
        self.inspector.promotion = Some((id, kind));
        self.inspector.applied = false;
        self.inspector.composer.update(cx, |composer, cx| {
            composer.open(spec, Vec::new(), window, cx)
        });
    }

    fn promotion_composed(
        &mut self,
        composer: Entity<Composer>,
        event: &ComposerEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            ComposerEvent::Submit(draft) => {
                let finish = |written: bool, cx: &mut Context<Self>| {
                    composer.update(cx, |composer, cx| composer.finish(written, cx));
                };
                let Some((id, kind)) = self.inspector.promotion.clone() else {
                    return finish(false, cx);
                };
                if self.inspector.applied {
                    return finish(false, cx);
                }
                let title = draft.title.clone();
                let slug = draft.fields.first().cloned().unwrap_or_default();
                let canvas_name = self.name.clone();
                let task = self.state.update(cx, |state, cx| {
                    state.promote_canvas_item(kind, title, slug, canvas_name, cx)
                });
                let composer = composer.downgrade();
                cx.spawn(async move |this, cx| {
                    let result = task.await;
                    let _ = this.update(cx, |this, cx| {
                        let written = match result {
                            Ok(entity) => {
                                this.inspector.applied = true;
                                this.inspector.promotion = None;
                                let reference = build_ref(&entity);
                                this.state
                                    .update(cx, |state, _| state.add_canvas_entity(entity));
                                let next = set_reference(&this.doc, &id, Some(reference));
                                this.change(next, Capture::Immediately, cx);
                                true
                            }
                            Err(message) => {
                                this.state.update(cx, |state, cx| {
                                    state.toast(message, true);
                                    cx.notify();
                                });
                                false
                            }
                        };
                        let _ = composer.update(cx, |composer, cx| composer.finish(written, cx));
                        cx.notify();
                    });
                })
                .detach();
            }
            ComposerEvent::Closed => self.inspector.promotion = None,
            _ => {}
        }
    }

    /// Fills the fields from what is selected, and starts another shape on its first tab.
    fn sync_inspector(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let shown = self.selected_node().or_else(|| self.selected_edge());
        if shown != self.inspector.shown {
            self.inspector.shown = shown.clone();
            let library = shown
                .as_deref()
                .and_then(|id| self.doc.node(id))
                .is_some_and(|node| {
                    shape_of(&node.kind).is_some() || component_of(&node.kind).is_some()
                });
            self.inspector.tab = if library { Tab::Style } else { Tab::Text };
            self.inspector.query.clear();
            let search = self.inspector.search.clone();
            search.update(cx, |search, cx| search.set_text("", cx));
            for picker in [
                &self.inspector.family,
                &self.inspector.kind,
                &self.inspector.edge_kind,
            ] {
                picker.close(cx);
            }
        }
        if let Some(node) = self
            .selected_node()
            .and_then(|id| self.doc.node(&id).cloned())
        {
            sync(&self.inspector.label, &node.name, window, cx);
            sync(
                &self.inspector.body,
                &property_text(&node.props, "body"),
                window,
                cx,
            );
            sync(
                &self.inspector.technology,
                &property_text(&node.props, "technology"),
                window,
                cx,
            );
            sync(
                &self.inspector.description,
                &property_text(&node.props, "description"),
                window,
                cx,
            );
            for (geometry, stepper) in &self.inspector.steppers {
                let value = f64::from(match geometry {
                    Geometry::X => node.x,
                    Geometry::Y => node.y,
                    Geometry::Width => node.width,
                    Geometry::Height => node.height,
                })
                .round();
                if stepper.read(cx).value() != value {
                    stepper.update(cx, |stepper, cx| stepper.set_value(value, window, cx));
                }
            }
        } else if let Some(edge) = self
            .selected_edge()
            .and_then(|id| self.doc.edge(&id).cloned())
        {
            sync(
                &self.inspector.edge_label,
                &property_text(&edge.props, "label"),
                window,
                cx,
            );
        }
    }

    pub(super) fn inspector_view(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.sync_inspector(window, cx);
        let theme = *Theme::global(cx);
        let width = (self.inspector_panel.read(cx).width() - 24.0).max(120.0);
        let editor = cx.entity().downgrade();
        let content = if let Some(node) = self
            .selected_node()
            .and_then(|id| self.doc.node(&id).cloned())
        {
            self.node_panel(&node, width, &editor, &theme, cx)
        } else if let Some(edge) = self
            .selected_edge()
            .filter(|id| self.doc.edge(id).is_some())
        {
            self.edge_panel(&edge, width, &editor, &theme, cx)
        } else if self.selection.nodes.len() > 1 {
            self.multi_panel(&editor, &theme, cx)
        } else {
            div().into_any_element()
        };
        div()
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .child(content)
            .into_any_element()
    }

    fn node_panel(
        &mut self,
        node: &Node,
        width: f32,
        editor: &WeakEntity<Self>,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let entities = self.entities(cx);
        let library = shape_of(&node.kind).is_some() || component_of(&node.kind).is_some();
        let semantic = component_of(&node.kind).is_some() || structure_kind(&node.kind);
        let is_note = node.kind == "note";
        let style = read_style(&node.props);
        let linked = linked_facts(node, &entities);
        let title = match &node.reference {
            Some(reference) => reference_state(Some(reference), &entities).title,
            None if !node.name.is_empty() => node.name.clone(),
            None => kind_label(&node.kind),
        };
        let detail = [
            kind_label(
                linked
                    .as_ref()
                    .and_then(|facts| facts.kind.clone())
                    .as_deref()
                    .unwrap_or(&node.kind),
            ),
            linked
                .as_ref()
                .and_then(|facts| facts.tech.clone())
                .unwrap_or_else(|| property_text(&node.props, "technology")),
        ]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" / ");
        let mut tabs: Vec<(Tab, &str)> = Vec::new();
        if library {
            tabs.push((Tab::Style, "Style"));
        }
        tabs.extend([
            (Tab::Text, "Text"),
            (Tab::Arrange, "Arrange"),
            (Tab::Data, "Data"),
        ]);
        let tab = if tabs.iter().any(|(item, _)| *item == self.inspector.tab) {
            self.inspector.tab
        } else {
            Tab::Text
        };
        let selected = tabs.iter().position(|(item, _)| *item == tab).unwrap_or(0);
        let ids: Vec<Tab> = tabs.iter().map(|(item, _)| *item).collect();
        let mut bar = Tabs::new("canvas-inspector-tabs").selected(selected);
        for (_, label) in &tabs {
            bar = bar.tab(*label);
        }
        let bar = bar.on_select({
            let editor = editor.clone();
            move |index, _, cx| {
                let Some(tab) = ids.get(index).copied() else {
                    return;
                };
                let _ = editor.update(cx, |this, cx| {
                    this.inspector.tab = tab;
                    cx.notify();
                });
            }
        });
        let page = match tab {
            Tab::Style => self.style_tab(node, &style, width, editor, theme, cx),
            Tab::Text => self.text_tab(node, &style, is_note, semantic, editor, theme),
            Tab::Arrange => self.arrange_tab(editor, theme),
            Tab::Data => self.data_tab(
                node,
                &entities,
                linked.and_then(|facts| facts.tech),
                editor,
                theme,
                cx,
            ),
        };
        div()
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .child(heading(title, Some(detail), theme))
            .child(div().px(px(8.0)).pb(px(4.0)).child(bar))
            .child(body("canvas-inspector-body", page))
            .into_any_element()
    }

    fn style_tab(
        &self,
        node: &Node,
        style: &ShapeStyle,
        width: f32,
        editor: &WeakEntity<Self>,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut out = form(12.0);
        if let Some(current) = family_for(&node.kind) {
            let families: Vec<MenuItem> = FAMILIES
                .iter()
                .filter(|family| !self.allowed(family.id).is_empty())
                .map(|family| {
                    let id = family.id;
                    MenuItem::radio(
                        family.label,
                        family.id == current.id,
                        Rc::new(act(editor, move |this, _, cx| this.switch_family(id, cx))),
                    )
                })
                .collect();
            let kinds: Vec<MenuItem> = self
                .allowed(current.id)
                .into_iter()
                .map(|item| {
                    let kind = item.kind;
                    MenuItem::radio(
                        item.label,
                        item.kind == node.kind,
                        Rc::new(act(editor, move |this, _, cx| {
                            this.edit_node(
                                |node| {
                                    node.kind = kind.to_string();
                                    true
                                },
                                Capture::Immediately,
                                cx,
                            )
                        })),
                    )
                })
                .collect();
            let type_label = component_of(&node.kind).map_or("", |item| item.label);
            out = out
                .child(field(
                    "Family",
                    self.inspector.family.trigger(
                        "canvas-family",
                        current.label,
                        width,
                        families,
                        false,
                        cx,
                    ),
                    theme,
                ))
                .child(field(
                    "Type",
                    self.inspector
                        .kind
                        .trigger("canvas-type", type_label, width, kinds, false, cx),
                    theme,
                ));
        }
        let widths = ["1 px", "2 px", "3 px"];
        out.child(swatches(
            "canvas-fill",
            "Fill",
            &style.fill,
            true,
            fill_color,
            |this, tag, cx| this.set_style(|style| style.fill = tag, cx),
            editor,
            theme,
        ))
        .child(swatches(
            "canvas-line",
            "Line",
            &style.stroke,
            true,
            stroke_color,
            |this, tag, cx| this.set_style(|style| style.stroke = tag, cx),
            editor,
            theme,
        ))
        .child(segments(
            "canvas-line-width",
            "Line width",
            &widths,
            usize::from(style.stroke_width.clamp(1, 3) - 1),
            |this, index, cx| this.set_style(|style| style.stroke_width = index as u8 + 1, cx),
            editor,
            theme,
        ))
        .child(segments(
            "canvas-line-pattern",
            "Line pattern",
            &["Solid", "Dashed"],
            usize::from(style.dashed),
            |this, index, cx| this.set_style(|style| style.dashed = index == 1, cx),
            editor,
            theme,
        ))
        .into_any_element()
    }

    fn text_tab(
        &self,
        node: &Node,
        style: &ShapeStyle,
        is_note: bool,
        semantic: bool,
        editor: &WeakEntity<Self>,
        theme: &Theme,
    ) -> AnyElement {
        let mut out = form(12.0);
        if is_note {
            let color = property_text(&node.props, "color");
            let color = if color.is_empty() {
                "yellow".to_string()
            } else {
                color
            };
            out = out.child(swatches(
                "canvas-note-color",
                "Note color",
                &color,
                false,
                fill_color,
                |this, tag, cx| {
                    this.edit_node(
                        |node| {
                            node.props.insert("color".into(), tag.into());
                            true
                        },
                        Capture::Immediately,
                        cx,
                    )
                },
                editor,
                theme,
            ));
        }
        out = match &node.reference {
            Some(reference) => out.child(muted(
                format!(
                    "The label follows the referenced {}.",
                    kind_label(&reference.kind)
                ),
                12.0,
                theme,
            )),
            None => out.child(field(
                if is_note { "Title" } else { "Label" },
                self.inspector.label.clone(),
                theme,
            )),
        };
        if is_note {
            return out
                .child(field(
                    "Body",
                    div()
                        .min_h(px(5.0 * 20.0))
                        .child(self.inspector.body.clone()),
                    theme,
                ))
                .into_any_element();
        }
        let size = match style.text_size {
            TextSize::Sm => 0,
            TextSize::Md => 1,
            TextSize::Lg => 2,
        };
        out = out.child(segments(
            "canvas-text-size",
            "Text size",
            &["Small", "Medium", "Large"],
            size,
            |this, index, cx| {
                let size = [TextSize::Sm, TextSize::Md, TextSize::Lg][index.min(2)];
                this.set_style(|style| style.text_size = size, cx)
            },
            editor,
            theme,
        ));
        if !semantic {
            let align = match style.text_align {
                TextAlign::Left => 0,
                TextAlign::Center => 1,
                TextAlign::Right => 2,
            };
            out = out.child(segments(
                "canvas-text-align",
                "Alignment",
                &["Left", "Center", "Right"],
                align,
                |this, index, cx| {
                    let align =
                        [TextAlign::Left, TextAlign::Center, TextAlign::Right][index.min(2)];
                    this.set_style(|style| style.text_align = align, cx)
                },
                editor,
                theme,
            ));
        }
        out.child(segments(
            "canvas-text-weight",
            "Weight",
            &["Regular", "Bold"],
            usize::from(style.bold),
            |this, index, cx| this.set_style(|style| style.bold = index == 1, cx),
            editor,
            theme,
        ))
        .into_any_element()
    }

    fn arrange_tab(&self, editor: &WeakEntity<Self>, theme: &Theme) -> AnyElement {
        let [x, y, w, h] = self.inspector.steppers.clone().map(|(_, stepper)| stepper);
        let row = |a: Entity<NumberStepper>, b: Entity<NumberStepper>| {
            div()
                .flex()
                .gap(px(8.0))
                .child(div().flex_1().min_w_0().child(a))
                .child(div().flex_1().min_w_0().child(b))
        };
        form(12.0)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(row(x, y))
                    .child(row(w, h)),
            )
            .child(field(
                "Order",
                div()
                    .flex()
                    .gap(px(8.0))
                    .child(
                        small("canvas-front", "Bring to front", ButtonKind::Secondary)
                            .on_click(act(editor, |this, _, cx| this.order(true, cx))),
                    )
                    .child(
                        small("canvas-back-order", "Send to back", ButtonKind::Secondary)
                            .on_click(act(editor, |this, _, cx| this.order(false, cx))),
                    ),
                theme,
            ))
            .child(div().child(
                small("canvas-snap", "Snap to grid", ButtonKind::Ghost).on_click(act(
                    editor,
                    |this, _, cx| {
                        this.edit_node(
                            |node| {
                                let (x, y) = (
                                    (node.x / GRID).round() * GRID,
                                    (node.y / GRID).round() * GRID,
                                );
                                let changed = (x, y) != (node.x, node.y);
                                (node.x, node.y) = (x, y);
                                changed
                            },
                            Capture::Immediately,
                            cx,
                        )
                    },
                )),
            ))
            .into_any_element()
    }

    fn data_tab(
        &self,
        node: &Node,
        entities: &[StoreEntity],
        linked_tech: Option<String>,
        editor: &WeakEntity<Self>,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut out = div().w_full().flex().flex_col().pt(px(12.0));
        if let Some(component) = component_of(&node.kind) {
            let current = property_text(&node.props, "technology");
            let suggestions: Vec<&'static str> = component
                .technologies
                .iter()
                .copied()
                .filter(|item| *item != current)
                .collect();
            let technology = self.inspector.technology.clone();
            let description = self.inspector.description.clone();
            let editor = editor.clone();
            let body: SectionBody = Rc::new(move |_: &mut Window, cx: &mut App| {
                let theme = *Theme::global(cx);
                let mut stack = form(0.0);
                stack = match &linked_tech {
                    Some(tech) => stack.child(
                        field(
                            "Technology",
                            div()
                                .h(px(28.0))
                                .px(px(10.0))
                                .flex()
                                .items_center()
                                .rounded(px(6.0))
                                .bg(theme.color_surface_subtle)
                                .opacity(0.6)
                                .text_size(px(13.0))
                                .text_color(theme.color_text_secondary)
                                .child(tech.clone()),
                            &theme,
                        )
                        .child(hint("Set on the linked Structure node.", &theme)),
                    ),
                    None => stack.child(field("Technology", technology.clone(), &theme)),
                };
                if linked_tech.is_none() && !suggestions.is_empty() {
                    stack = stack.child(div().flex().flex_wrap().gap(px(4.0)).children(
                        suggestions.iter().map(|item| {
                            let item = *item;
                            small(
                                SharedString::from(format!("canvas-tech-{item}")),
                                item,
                                ButtonKind::Ghost,
                            )
                            .tooltip(format!("Use {item}"))
                            .on_click(act(
                                &editor,
                                move |this, _, cx| {
                                    this.edit_node(
                                        |node| set_or_clear(node, "technology", item),
                                        Capture::Immediately,
                                        cx,
                                    )
                                },
                            ))
                        }),
                    ));
                }
                stack
                    .child(field(
                        "Description",
                        div().min_h(px(3.0 * 20.0)).child(description.clone()),
                        &theme,
                    ))
                    .into_any_element()
            });
            self.inspector
                .component
                .update(cx, |section, _| section.set_body(body));
            out = out.child(self.inspector.component.clone());
        }
        out = out.child(self.reference_section(node, entities, editor, cx));
        if can_promote(node) {
            let editor = editor.clone();
            let body: SectionBody = Rc::new(move |_: &mut Window, cx: &mut App| {
                let theme = *Theme::global(cx);
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(px(4.0))
                    .children(PromotionKind::ALL.into_iter().map(|kind| {
                        small(
                            SharedString::from(format!("canvas-promote-{}", kind.wire())),
                            format!("Promote to {}", ref_label(kind.wire())),
                            ButtonKind::Ghost,
                        )
                        .leading(icon_colored(
                            entity_icon(kind.wire()),
                            14.0,
                            theme.color_text_secondary,
                        ))
                        .on_click(act(&editor, move |this, window, cx| {
                            this.open_promotion(kind, window, cx)
                        }))
                    }))
                    .into_any_element()
            });
            self.inspector
                .promote
                .update(cx, |section, _| section.set_body(body));
            out = out.child(self.inspector.promote.clone());
        }
        let _ = theme;
        out.into_any_element()
    }

    fn reference_section(
        &self,
        node: &Node,
        entities: &[StoreEntity],
        editor: &WeakEntity<Self>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let required = structure_kind(&node.kind);
        let reference = node.reference.clone();
        let state = reference_state(reference.as_ref(), entities);
        let trusted = state.referenced;
        let chip_label = match &reference {
            Some(reference) if trusted => ref_label(&reference.kind),
            Some(_) => "Missing reference / Unverified".to_string(),
            None => "Unverified".to_string(),
        };
        let referenced_title = state.title.clone();
        let missing = state.entity.is_none();
        let query = self.inspector.query.to_lowercase();
        let matches: Vec<StoreEntity> = entities
            .iter()
            .filter(|entity| !required || entity.kind == "node")
            .filter(|entity| {
                format!(
                    "{} {} {}",
                    ref_label(&entity.kind),
                    entity.title,
                    entity.slug
                )
                .to_lowercase()
                .contains(&query)
            })
            .cloned()
            .collect();
        let search = self.inspector.search.clone();
        let placeholder = if required {
            "Search Structure"
        } else {
            "Search cards, Decisions, Rules or Structure"
        };
        search.update(cx, |search, _| search.set_placeholder(placeholder));
        let editor = editor.clone();
        let body: SectionBody = Rc::new(move |_: &mut Window, cx: &mut App| {
            let theme = *Theme::global(cx);
            let mut out = div().flex().flex_col().gap(px(8.0));
            if reference.is_some() || required {
                out = out.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(chip(
                            chip_label.clone(),
                            if trusted {
                                StatusTone::Muted
                            } else {
                                StatusTone::Warn
                            },
                            cx,
                        ))
                        .when(reference.is_some() && !required, |row| {
                            row.child(
                                small("canvas-unlink", "Unlink", ButtonKind::Ghost)
                                    .on_click(act(&editor, |this, _, cx| this.link(None, cx))),
                            )
                        }),
                );
            }
            if reference.is_some() {
                out = out.child(
                    div()
                        .text_size(px(13.0))
                        .text_color(theme.color_text_primary)
                        .child(referenced_title.clone()),
                );
                if missing {
                    out = out.child(muted(
                        "The referenced entity is missing from the loaded project data. It may have been deleted or excluded by the current scope.",
                        12.0,
                        &theme,
                    ));
                }
            }
            if required {
                out = out.child(muted(
                    "This shape requires a Structure reference. Choose another Structure node to replace it.",
                    12.0,
                    &theme,
                ));
            }
            let rows = matches.iter().map(|entity| {
                let link = build_ref(entity);
                div()
                    .id(SharedString::from(format!(
                        "canvas-entity-{}-{}",
                        entity.kind, entity.id
                    )))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .px(px(8.0))
                    .py(px(6.0))
                    .rounded(px(theme.radius_control))
                    .cursor_pointer()
                    .hover(|row| row.bg(theme.color_surface_hover))
                    .text_size(px(13.0))
                    .child(icon_colored(
                        entity_icon(&entity.kind),
                        14.0,
                        theme.color_text_secondary,
                    ))
                    .child(
                        div()
                            .min_w_0()
                            .overflow_hidden()
                            .text_ellipsis()
                            .whitespace_nowrap()
                            .text_color(theme.color_text_primary)
                            .child(entity.title.clone()),
                    )
                    .child(
                        div()
                            .flex_none()
                            .text_size(px(12.0))
                            .text_color(theme.color_text_muted)
                            .child(ref_label(&entity.kind)),
                    )
                    .on_click({
                        let link_to = act(&editor, move |this, _, cx| {
                            this.link(Some(link.clone()), cx)
                        });
                        move |_, window, cx| link_to(window, cx)
                    })
            });
            out.child(search.clone())
                .child(
                    div()
                        .id("canvas-entities")
                        .max_h(px(256.0))
                        .overflow_y_scroll()
                        .flex()
                        .flex_col()
                        .children(rows)
                        .when(matches.is_empty(), |list| {
                            list.child(muted("No matching project entities.", 12.0, &theme))
                        }),
                )
                .into_any_element()
        });
        self.inspector
            .reference
            .update(cx, |section, _| section.set_body(body));
        self.inspector.reference.clone().into_any_element()
    }

    fn edge_panel(
        &self,
        id: &str,
        width: f32,
        editor: &WeakEntity<Self>,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(edge) = self.doc.edge(id) else {
            return div().into_any_element();
        };
        let name = |id: &str| {
            self.doc
                .node(id)
                .map(|node| node.name.clone())
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| "?".into())
        };
        let kinds: Vec<MenuItem> = edge_palette(&self.definition)
            .into_iter()
            .map(|kind| {
                let checked = kind == edge.kind;
                let (label, id) = (kind_label(&kind), id.to_string());
                MenuItem::radio(
                    label,
                    checked,
                    Rc::new(act(editor, move |this, _, cx| {
                        let next = update_edge(&this.doc, &id, |edge| edge.kind = kind.clone());
                        this.change(next, Capture::Immediately, cx);
                    })),
                )
            })
            .collect();
        let page = form(8.0)
            .child(field(
                "Edge kind",
                self.inspector.edge_kind.trigger(
                    "canvas-edge-kind",
                    kind_label(&edge.kind),
                    width,
                    kinds,
                    false,
                    cx,
                ),
                theme,
            ))
            .child(field("Label", self.inspector.edge_label.clone(), theme));
        div()
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .child(heading(
                format!("Edge: {}", kind_label(&edge.kind)),
                Some(format!("{} \u{2192} {}", name(&edge.from), name(&edge.to))),
                theme,
            ))
            .child(body("canvas-inspector-edge", page))
            .into_any_element()
    }

    fn multi_panel(
        &self,
        editor: &WeakEntity<Self>,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let editor = editor.clone();
        let body_fn: SectionBody = Rc::new(move |_: &mut Window, _: &mut App| {
            let mut rows = div().flex().flex_col().gap(px(4.0));
            for chunk in ALIGNMENTS.chunks(3) {
                rows = rows.child(div().flex().gap(px(4.0)).children(chunk.iter().map(
                    |(align, id, label)| {
                        let align = *align;
                        div().flex_1().min_w_0().child(
                            small(
                                SharedString::from(format!("canvas-align-{id}")),
                                *label,
                                ButtonKind::Secondary,
                            )
                            .full_width()
                            .tooltip(format!("Align {id}"))
                            .on_click(act(&editor, move |this, _, cx| this.align(align, cx))),
                        )
                    },
                )));
            }
            rows.into_any_element()
        });
        self.inspector
            .align
            .update(cx, |section, _| section.set_body(body_fn));
        div()
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .child(heading(
                format!("{} shapes selected", self.selection.nodes.len()),
                None,
                theme,
            ))
            .child(body("canvas-inspector-multi", self.inspector.align.clone()))
            .into_any_element()
    }
}
