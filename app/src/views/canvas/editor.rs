//! The canvas editor (`editor.tsx`): the shape library on the left, the canvas well, and the
//! inspector on the right, which opens only for a selected shape or connection. Both panels
//! resize (192px, 168 to 256; 272px, 256 to 336).
//!
//! The toolbar floats over the well: Canvases (back), the name in a 224px pill field, and the
//! save state (Unsaved, Saving…, Not saved in the danger color, Saved; 11px muted). The tool
//! dock floats at the bottom center: Select, Pan, Connect; Undo, Redo; Delete selection,
//! Arrange; Zoom out, the zoom level, Zoom in, Fit canvas; Agent-visible and Canvas rules.
//! While Connect waits for its target, a hint floats under the toolbar.
//!
//! The well is the app background with the dotted grid following the viewport. A press on a
//! shape selects it (Shift adds or removes it) and drags the selection; on a connection
//! selects it; on the empty well clears the selection and pans, as Pan and the middle button
//! always do. A selected shape's eight handles resize it from its pre-gesture box. Scrolling
//! pans; Ctrl or ⌘ with it zooms about the pointer. Double-click or Enter edits a label (not a
//! referenced shape's). Delete or Backspace removes the selection, Escape returns to Select
//! and clears it, the arrows nudge it 10 units (40 with Shift), and ⌘Z / ⇧⌘Z undo and redo.
//!
//! Every edit saves on its own after 500ms, one writer per canvas: new edits coalesce, and an
//! older answer never replaces local state. Leaving saves first; a refusal stays on the page
//! with Retry save. An editor closed from outside saves what it holds.

use std::time::Duration;

use gpui::{
    AnyElement, App, AppContext as _, Context, CursorStyle, DispatchPhase, Entity, FocusHandle,
    Focusable, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Render, ScrollWheelEvent, SharedString, Task, Window, canvas, div, prelude::*,
    px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::canvas::{
    Capture, History, Hit, Pt, Viewport, arrange_horizontally, bounds_of, canvas_dock, dock_group,
    dock_separator, dot_grid, hit_test, zoom_level,
};
use polterware_ui::icons::Icon;
use polterware_ui::patterns::{AnchorSlot, PanelSide, ResizablePanel};
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::Theme;
use polterware_ui::window::window_drag::WindowDragRegion;

use super::scene::{NodeDecor, edge_label, edge_routes, edges_view, layers, node_view};
use crate::app::CloseOverlay;
use crate::canvas::catalog::{
    PaletteGroup, backdrop, component_of, default_props, edge_palette, initial_geometry,
    node_palette, palette_groups, palette_label,
};
use crate::canvas::document::{
    Doc, HANDLES, Handle, Selection, connect_nodes, insert_node, move_nodes, remove_selection,
    resize_from, serialize, set_reference, to_document, update_node,
};
use crate::canvas::references::{StoreEntity, build_ref};
use crate::contracts::{Canvas, CanvasTypeDefinition};
use crate::state::{AppState, CanvasPatch, OpenCanvas};

/// How long an edit waits before it saves.
const SAVE_DELAY: Duration = Duration::from_millis(500);
/// A resize handle's side on screen.
const HANDLE_SIZE: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Tool {
    Select,
    Pan,
    Connect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SaveState {
    Saved,
    Pending,
    Saving,
    Error,
}

#[derive(Clone, Debug)]
enum GestureMode {
    Pan,
    Move,
    Resize(Handle),
}

#[derive(Clone, Debug)]
struct Gesture {
    mode: GestureMode,
    start: Pt,
    before: Doc,
    ids: Vec<String>,
}

/// What the library hands a drop on the well.
#[derive(Clone, Debug)]
pub(super) struct LibraryDrag {
    pub kind: String,
    pub entity: Option<StoreEntity>,
}

/// The tile a library drag shows under the pointer.
/// What a library drag carries under the pointer: the tile itself, its drawing over its
/// name, faded as the browser's drag image is.
pub(super) struct DragPreview {
    pub node: crate::canvas::document::Node,
    pub name: SharedString,
    pub width: f32,
}

impl Render for DragPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *Theme::global(cx);
        div()
            .opacity(0.75)
            .flex()
            .flex_col()
            .gap(px(4.0))
            .pt(px(6.0))
            .px(px(6.0))
            .pb(px(4.0))
            .child(super::library::node_sample(
                &self.node,
                self.width,
                super::library::ART_HEIGHT,
                &[],
                &theme,
            ))
            .child(
                div()
                    .w(px(self.width))
                    .truncate()
                    .text_center()
                    .text_size(px(11.0))
                    .text_color(theme.color_text_secondary)
                    .child(self.name.clone()),
            )
    }
}

pub struct CanvasEditor {
    pub(super) state: Entity<AppState>,
    pub(super) canvas: Canvas,
    pub(super) definition: CanvasTypeDefinition,
    pub(super) palette: Vec<String>,
    pub(super) groups: Vec<PaletteGroup>,
    pub(super) doc: Doc,
    pub(super) selection: Selection,
    history: History<Doc>,
    pub(super) tool: Tool,
    connect_from: Option<String>,
    editing: Option<String>,
    label_input: Entity<TextInput>,
    name_input: Entity<TextInput>,
    pub(super) name: String,
    agent_visible: bool,
    save_state: SaveState,
    error: Option<String>,
    leaving: bool,
    version: u64,
    saved_version: u64,
    saving: bool,
    timer: Option<Task<()>>,
    gesture: Option<Gesture>,
    hover_edge: Option<String>,
    host: AnchorSlot,
    focus: FocusHandle,
    pub(super) library_panel: Entity<ResizablePanel>,
    pub(super) inspector_panel: Entity<ResizablePanel>,
    pub(super) library: super::library::LibraryState,
    pub(super) inspector: super::inspector::InspectorState,
}

impl CanvasEditor {
    pub fn new(
        state: Entity<AppState>,
        open: OpenCanvas,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let palette = node_palette(&open.definition);
        let groups = palette_groups(&palette);
        let doc = to_document(&open.canvas);
        let name = open.canvas.name.clone();
        let name_input = cx.new(|cx| TextInput::new(cx).pill());
        name_input.update(cx, |input, cx| input.set_text(name.clone(), cx));
        cx.observe(&name_input, |this, input, cx| {
            let text = input.read(cx).text().to_string();
            if text != this.name {
                this.name = text;
                this.schedule(cx);
            }
        })
        .detach();
        let editor = cx.entity().downgrade();
        let label_input = cx.new(|cx| {
            let submit = editor.clone();
            let cancel = editor.clone();
            // The label's own text at the zoom, not a field (`.pw-canvas-label-input`).
            TextInput::new(cx)
                .bare()
                .on_submit(move |_, _, cx| {
                    let _ = submit.update(cx, |this, cx| this.finish_label(cx));
                })
                .on_cancel(move |_, cx| {
                    let _ = cancel.update(cx, |this, cx| this.finish_label(cx));
                })
        });
        cx.observe(&label_input, |this, input, cx| {
            let Some(id) = this.editing.clone() else {
                return;
            };
            let text = input.read(cx).text().to_string();
            if this.doc.node(&id).is_some_and(|node| node.name != text) {
                let next = update_node(&this.doc, &id, |node| node.name = text);
                this.change(next, Capture::Eventually, cx);
            }
        })
        .detach();
        let label_focus = label_input.focus_handle(cx);
        cx.on_blur(&label_focus, window, |this, _, cx| this.finish_label(cx))
            .detach();
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        // The panels open at the widths last dragged to, kept across launches.
        let widths = crate::prefs::canvas_prefs(&crate::prefs::user_data_dir());
        let library_panel = cx.new(|_| {
            ResizablePanel::new(
                PanelSide::Left,
                widths.library.unwrap_or(192.0),
                168.0,
                256.0,
            )
        });
        let inspector_panel = cx.new(|_| {
            ResizablePanel::new(
                PanelSide::Right,
                widths.inspector.unwrap_or(272.0),
                256.0,
                336.0,
            )
            .hidden()
        });
        for (panel, library) in [(&library_panel, true), (&inspector_panel, false)] {
            let mut saved = panel.read(cx).width();
            cx.observe(panel, move |_, panel, cx| {
                let width = panel.read(cx).width();
                if (width - saved).abs() > 0.5 {
                    saved = width;
                    crate::prefs::update_canvas_prefs(&crate::prefs::user_data_dir(), |prefs| {
                        if library {
                            prefs.library = Some(width);
                        } else {
                            prefs.inspector = Some(width);
                        }
                    });
                }
                cx.notify();
            })
            .detach();
        }
        let library = super::library::LibraryState::new(&groups, cx);
        let inspector = super::inspector::InspectorState::new(window, cx);
        let focus = cx.focus_handle();
        Self {
            state,
            agent_visible: open.canvas.agent_visible,
            canvas: open.canvas,
            definition: open.definition,
            palette,
            groups,
            doc,
            selection: Selection::default(),
            history: History::new(100),
            tool: Tool::Select,
            connect_from: None,
            editing: None,
            label_input,
            name_input,
            name,
            save_state: SaveState::Saved,
            error: None,
            leaving: false,
            version: 0,
            saved_version: 0,
            saving: false,
            timer: None,
            gesture: None,
            hover_edge: None,
            host: AnchorSlot::default(),
            focus,
            library_panel,
            inspector_panel,
            library,
            inspector,
        }
    }

    pub(super) fn entities(&self, cx: &App) -> Vec<StoreEntity> {
        self.state.read(cx).canvas_entities().to_vec()
    }

    /// The well's size, once a frame has laid it out.
    fn host_size(&self) -> (f32, f32) {
        self.host.get().map_or((960.0, 640.0), |bounds| {
            (f32::from(bounds.size.width), f32::from(bounds.size.height))
        })
    }

    fn local(&self, position: gpui::Point<gpui::Pixels>) -> Pt {
        let origin = self
            .host
            .get()
            .map_or(gpui::point(px(0.0), px(0.0)), |bounds| bounds.origin);
        Pt::new(
            f32::from(position.x - origin.x),
            f32::from(position.y - origin.y),
        )
    }

    // Saving.

    pub(super) fn schedule(&mut self, cx: &mut Context<Self>) {
        self.version += 1;
        self.save_state = SaveState::Pending;
        self.timer = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SAVE_DELAY).await;
            let _ = this.update(cx, |this, cx| this.flush(cx));
        }));
        cx.notify();
    }

    fn patch(&self) -> CanvasPatch {
        let (graph, layout_json) = serialize(&self.doc, &self.canvas.layout_json);
        CanvasPatch {
            name: Some(self.name.trim().to_string()),
            agent_visible: Some(self.agent_visible),
            graph: Some(graph),
            layout_json: Some(layout_json),
        }
    }

    /// Saves until the latest edit is written, or the core refuses one.
    fn flush(&mut self, cx: &mut Context<Self>) {
        self.timer = None;
        if self.saving || self.version <= self.saved_version {
            if !self.saving && self.save_state == SaveState::Pending {
                self.save_state = SaveState::Saved;
                cx.notify();
            }
            return;
        }
        if self.name.trim().is_empty() {
            self.save_state = SaveState::Error;
            self.error = Some("Canvas name is required.".into());
            cx.notify();
            return;
        }
        let version = self.version;
        let patch = self.patch();
        let (project, id) = (self.canvas.project_id.clone(), self.canvas.id.clone());
        self.saving = true;
        self.save_state = SaveState::Saving;
        let task = self
            .state
            .update(cx, |state, cx| state.save_canvas(project, id, patch, cx));
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.saving = false;
                match result {
                    Ok(()) => {
                        this.saved_version = this.saved_version.max(version);
                        if this.version > this.saved_version {
                            this.flush(cx);
                        } else {
                            this.save_state = SaveState::Saved;
                            this.error = None;
                            if this.leaving {
                                this.leaving = false;
                                this.state.update(cx, |state, cx| state.close_canvas(cx));
                            }
                        }
                    }
                    Err(message) => {
                        this.save_state = SaveState::Error;
                        this.error = Some(message);
                        this.leaving = false;
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    /// Writes what is unsaved when the editor goes away without Back; a refusal is a toast.
    pub fn save_on_close(&mut self, cx: &mut Context<Self>) {
        if self.version <= self.saved_version || self.name.trim().is_empty() {
            return;
        }
        self.saved_version = self.version;
        let patch = self.patch();
        let (project, id, name) = (
            self.canvas.project_id.clone(),
            self.canvas.id.clone(),
            self.name.clone(),
        );
        let state = self.state.clone();
        let task = state.update(cx, |state, cx| state.save_canvas(project, id, patch, cx));
        cx.spawn(async move |_, cx| {
            if let Err(message) = task.await {
                let _ = state.update(cx, |state, cx| {
                    state.toast(
                        format!("Could not save canvas \u{201c}{name}\u{201d}: {message}"),
                        true,
                    );
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn back(&mut self, cx: &mut Context<Self>) {
        self.finish_label(cx);
        self.leaving = true;
        if self.version <= self.saved_version && !self.saving {
            self.leaving = false;
            self.state.update(cx, |state, cx| state.close_canvas(cx));
            return;
        }
        self.flush(cx);
    }

    // Editing.

    pub(super) fn change(&mut self, next: Doc, capture: Capture, cx: &mut Context<Self>) {
        self.history.commit(&self.doc, &next, capture);
        self.doc = next;
        self.schedule(cx);
    }

    pub(super) fn finish_label(&mut self, cx: &mut Context<Self>) {
        if self.editing.take().is_some() {
            let doc = self.doc.clone();
            self.change(doc, Capture::Immediately, cx);
        }
    }

    /// The label being edited takes its shape's text style at the viewport's zoom: its size,
    /// weight, and centering, so the text does not jump when editing starts.
    fn sync_label_type(&mut self, cx: &mut Context<Self>) {
        let Some(node) = self.editing.as_deref().and_then(|id| self.doc.node(id)) else {
            return;
        };
        let style = crate::canvas::document::read_style(&node.props);
        let size = style.text_size.px() * self.doc.viewport.zoom;
        let weight = if style.bold {
            gpui::FontWeight::SEMIBOLD
        } else {
            gpui::FontWeight::NORMAL
        };
        let centered = style.text_align == crate::canvas::document::TextAlign::Center;
        self.label_input.update(cx, |input, cx| {
            input.set_prose(size, size * 1.3, weight, cx);
            input.set_centered(centered, cx);
        });
    }

    fn start_label(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(node) = self.doc.node(id) else {
            return;
        };
        if node.reference.is_some() {
            return;
        }
        let name = node.name.clone();
        self.editing = Some(id.to_string());
        self.label_input.update(cx, |input, cx| {
            input.set_text(name.clone(), cx);
            input.select(0..name.len(), cx);
        });
        self.label_input.focus_handle(cx).focus(window);
        cx.notify();
    }

    fn undo(&mut self, redo: bool, cx: &mut Context<Self>) {
        self.finish_label(cx);
        // Settle a pending typed edit first, so undo reverts it instead of dropping it.
        if self.history.has_pending() {
            let doc = self.doc.clone();
            self.history.commit(&doc, &doc, Capture::Immediately);
        }
        let next = if redo {
            self.history.redo(&self.doc)
        } else {
            self.history.undo(&self.doc)
        };
        if let Some(next) = next {
            self.doc = next;
        }
        self.selection = Selection::default();
        self.schedule(cx);
    }

    pub(super) fn delete_selected(&mut self, cx: &mut Context<Self>) {
        let next = remove_selection(&self.doc, &self.selection);
        self.change(next, Capture::Immediately, cx);
        self.selection = Selection::default();
        self.connect_from = None;
    }

    fn zoom_by(&mut self, factor: f32, cx: &mut Context<Self>) {
        let (w, h) = self.host_size();
        let view = self.doc.viewport;
        self.doc.viewport = view.zoom_at(Pt::new(w / 2.0, h / 2.0), view.zoom * factor);
        self.schedule(cx);
    }

    fn fit(&mut self, cx: &mut Context<Self>) {
        if self.doc.nodes.is_empty() {
            return;
        }
        let (w, h) = self.host_size();
        let bounds = bounds_of(self.doc.nodes.iter().map(|node| node.rect()));
        self.doc.viewport = Viewport::fit(bounds, w, h, 48.0);
        self.schedule(cx);
    }

    fn arrange(&mut self, cx: &mut Context<Self>) {
        let widths: Vec<f32> = self.doc.nodes.iter().map(|node| node.width).collect();
        let places = arrange_horizontally(&widths, 48.0);
        let mut next = self.doc.clone();
        for (node, at) in next.nodes.iter_mut().zip(places) {
            node.x = at.x;
            node.y = at.y;
        }
        self.change(next, Capture::Immediately, cx);
        self.fit(cx);
    }

    /// Inserts a node of `kind` centered on `at`, citing `entity` when one is given.
    pub(super) fn insert(
        &mut self,
        kind: &str,
        at: Option<Pt>,
        entity: Option<StoreEntity>,
        cx: &mut Context<Self>,
    ) {
        if !self.palette.iter().any(|item| item == kind) {
            self.error = Some("This shape is not allowed by the core.".into());
            cx.notify();
            return;
        }
        let center = at.unwrap_or_else(|| {
            let (w, h) = self.host_size();
            self.doc.viewport.to_document(Pt::new(w / 2.0, h / 2.0))
        });
        let (width, height, z) = initial_geometry(kind);
        let id = uuid();
        let origin = (
            (center.x - width / 2.0).round(),
            (center.y - height / 2.0).round(),
        );
        let next = match insert_node(&self.doc, &self.definition, kind, origin, id.clone()) {
            Ok(next) => next,
            Err(message) => {
                self.error = Some(message);
                cx.notify();
                return;
            }
        };
        let named = entity
            .as_ref()
            .map(|entity| entity.title.clone())
            .or_else(|| {
                (polterware_ui::canvas::shape_of(kind).is_some() || component_of(kind).is_some())
                    .then(|| palette_label(kind))
            });
        let mut next = update_node(&next, &id, |node| {
            node.width = width;
            node.height = height;
            if let Some(z) = z {
                node.z = z;
            }
            if let Some(name) = named {
                node.name = name;
            }
            for (key, value) in default_props(kind) {
                node.props.insert(key, value);
            }
        });
        if let Some(entity) = &entity {
            next = set_reference(&next, &id, Some(build_ref(entity)));
        }
        self.change(next, Capture::Immediately, cx);
        self.error = None;
        self.selection = Selection::node(&id);
    }

    // The pointer.

    fn hit(&self, at: Pt) -> Hit {
        let document = self.doc.viewport.to_document(at);
        let routes = edge_routes(&self.doc);
        hit_test(
            self.doc.nodes.iter().map(|node| {
                (
                    node.id.as_str(),
                    node.rect(),
                    if backdrop(&node.kind) { -1 } else { node.z },
                )
            }),
            routes
                .iter()
                .map(|(id, points)| (id.as_str(), points.clone())),
            document,
        )
    }

    fn pressed(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
        self.focus.focus(window);
        let at = self.local(event.position);
        let hit = self.hit(at);
        let middle = event.button == MouseButton::Middle;
        if self.tool == Tool::Connect
            && !middle
            && let Hit::Node(id) = hit
        {
            match self.connect_from.take() {
                None => {
                    self.selection = Selection::node(&id);
                    self.connect_from = Some(id);
                }
                Some(from) => {
                    let kind = edge_palette(&self.definition)
                        .first()
                        .cloned()
                        .unwrap_or_default();
                    match connect_nodes(&self.doc, &self.definition, &from, &id, &kind, uuid()) {
                        Ok(next) => {
                            self.change(next, Capture::Immediately, cx);
                            self.error = None;
                        }
                        Err(message) => self.error = Some(message),
                    }
                }
            }
            cx.notify();
            return;
        }
        let mut ids = Vec::new();
        let mut mode = GestureMode::Pan;
        if self.tool != Tool::Pan && !middle {
            match hit {
                Hit::Node(id) => {
                    if event.click_count == 2 {
                        self.selection = Selection::node(&id);
                        self.start_label(&id, window, cx);
                        return;
                    }
                    let shift = event.modifiers.shift;
                    let next = if !shift && self.selection.nodes.contains(&id) {
                        self.selection.clone()
                    } else {
                        self.selection.click_node(&id, shift)
                    };
                    ids = next.nodes.clone();
                    self.selection = next;
                    mode = GestureMode::Move;
                }
                Hit::Edge(id) => {
                    self.selection = Selection::edge(&id);
                    cx.notify();
                    return;
                }
                Hit::Empty => {
                    self.selection = Selection::default();
                    self.connect_from = None;
                }
            }
        }
        self.gesture = Some(Gesture {
            mode,
            start: at,
            before: self.doc.clone(),
            ids,
        });
        cx.notify();
    }

    fn resize_pressed(
        &mut self,
        id: String,
        handle: Handle,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus.focus(window);
        self.gesture = Some(Gesture {
            mode: GestureMode::Resize(handle),
            start: self.local(event.position),
            before: self.doc.clone(),
            ids: vec![id],
        });
        cx.notify();
    }

    fn dragged(&mut self, position: gpui::Point<gpui::Pixels>, cx: &mut Context<Self>) {
        let Some(gesture) = self.gesture.clone() else {
            return;
        };
        let at = self.local(position);
        let (dx, dy) = (at.x - gesture.start.x, at.y - gesture.start.y);
        let base = &gesture.before;
        let zoom = base.viewport.zoom;
        match gesture.mode {
            GestureMode::Pan => {
                self.doc.viewport = base.viewport.pan_by(dx, dy);
            }
            GestureMode::Move => {
                let mut next = move_nodes(base, &gesture.ids, dx / zoom, dy / zoom);
                next.viewport = self.doc.viewport;
                self.doc = next;
            }
            GestureMode::Resize(handle) => {
                if let Some(node) = gesture.ids.first().and_then(|id| base.node(id)) {
                    let mut next = resize_from(base, node, handle, dx / zoom, dy / zoom);
                    next.viewport = self.doc.viewport;
                    self.doc = next;
                }
            }
        }
        cx.notify();
    }

    fn released(&mut self, cx: &mut Context<Self>) {
        let Some(gesture) = self.gesture.take() else {
            return;
        };
        if !matches!(gesture.mode, GestureMode::Pan) {
            let mut before = gesture.before;
            before.viewport = self.doc.viewport;
            self.history
                .commit(&before, &self.doc, Capture::Immediately);
        }
        self.schedule(cx);
    }

    fn hovered(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        if self.gesture.is_some() {
            return;
        }
        let hover = match self.hit(self.local(event.position)) {
            Hit::Edge(id) => Some(id),
            _ => None,
        };
        if hover != self.hover_edge {
            self.hover_edge = hover;
            cx.notify();
        }
    }

    fn scrolled(&mut self, event: &ScrollWheelEvent, window: &Window, cx: &mut Context<Self>) {
        let delta = event.delta.pixel_delta(window.line_height());
        let (dx, dy) = (f32::from(delta.x), f32::from(delta.y));
        let view = self.doc.viewport;
        self.doc.viewport = if event.modifiers.control || event.modifiers.platform {
            view.zoom_at(self.local(event.position), view.zoom * (dy * 0.004).exp())
        } else {
            view.pan_by(dx, dy)
        };
        self.schedule(cx);
    }

    fn key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.editing.is_some() {
            return;
        }
        let keys = &event.keystroke;
        let command = keys.modifiers.platform || keys.modifiers.control;
        match keys.key.as_str() {
            "z" if command => self.undo(keys.modifiers.shift, cx),
            "delete" | "backspace" if !command => self.delete_selected(cx),
            "enter" => {
                if let [id] = self.selection.nodes.as_slice() {
                    let id = id.clone();
                    self.start_label(&id, window, cx);
                }
            }
            "left" | "right" | "up" | "down" if !self.selection.nodes.is_empty() => {
                let step = if keys.modifiers.shift { 40.0 } else { 10.0 };
                let (dx, dy) = match keys.key.as_str() {
                    "left" => (-step, 0.0),
                    "right" => (step, 0.0),
                    "up" => (0.0, -step),
                    _ => (0.0, step),
                };
                let next = move_nodes(&self.doc, &self.selection.nodes, dx, dy);
                self.change(next, Capture::Immediately, cx);
            }
            _ => return,
        }
        cx.stop_propagation();
    }

    fn escape(&mut self, cx: &mut Context<Self>) {
        self.tool = Tool::Select;
        self.connect_from = None;
        self.selection = Selection::default();
        cx.notify();
    }

    // Rendering.

    fn save_label(&self) -> &'static str {
        match self.save_state {
            SaveState::Pending => "Unsaved",
            SaveState::Saving => "Saving\u{2026}",
            SaveState::Error => "Not saved",
            SaveState::Saved => "Saved",
        }
    }

    fn toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *Theme::global(cx);
        let view = cx.entity().downgrade();
        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .h(px(theme.page_header_height))
            .pt(px(theme.space_2))
            .px(px(theme.space_2))
            .pb(px(theme.space_1))
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(
                Button::new("canvas-back", ButtonKind::Secondary)
                    .size(ButtonSize::Icon)
                    .icon(Icon::ChevronLeft)
                    .tooltip("Canvases")
                    .disabled(self.leaving)
                    .on_click(move |_, cx| {
                        let _ = view.update(cx, |this, cx| this.back(cx));
                    }),
            )
            .child(div().w(px(224.0)).child(self.name_input.clone()))
            .child(
                div()
                    .flex_none()
                    .whitespace_nowrap()
                    .text_size(px(11.0))
                    .text_color(if self.save_state == SaveState::Error {
                        theme.color_danger
                    } else {
                        theme.color_text_muted
                    })
                    .child(self.save_label()),
            )
            .child(div().flex_1().h_full().window_drag_region())
    }

    #[allow(clippy::too_many_arguments)]
    fn dock_button(
        &self,
        id: &'static str,
        icon: Icon,
        tip: impl Into<SharedString>,
        selected: bool,
        disabled: bool,
        cx: &mut Context<Self>,
        action: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> Button {
        let view = cx.entity().downgrade();
        Button::new(id, ButtonKind::Ghost)
            .size(ButtonSize::Icon)
            .icon(icon)
            .icon_size(14.0)
            .tooltip(tip)
            .selected(selected)
            .disabled(disabled)
            .on_click(move |window, cx| {
                let _ = view.update(cx, |this, cx| {
                    action(this, window, cx);
                    cx.notify();
                });
            })
    }

    fn tools_dock(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let tools = [
            (
                Tool::Select,
                "canvas-tool-select",
                Icon::MousePointer2,
                "Select",
            ),
            (Tool::Pan, "canvas-tool-pan", Icon::Hand, "Pan"),
            (
                Tool::Connect,
                "canvas-tool-connect",
                Icon::ConnectIcon,
                "Connect",
            ),
        ];
        let mut picks = dock_group();
        for (tool, id, icon, label) in tools {
            picks = picks.child(self.dock_button(
                id,
                icon,
                label,
                self.tool == tool,
                false,
                cx,
                move |this, _, _| {
                    this.tool = tool;
                    this.connect_from = None;
                },
            ));
        }
        let empty = self.selection.is_empty();
        let rules = format!(
            "{} Drag to move, scroll to pan, Ctrl/\u{2318}+scroll to zoom.",
            self.definition.rules.join(" ")
        );
        let rules = rules.trim().to_string();
        let actions = dock_group()
            .child(self.dock_button(
                "canvas-undo",
                Icon::Undo2,
                "Undo",
                false,
                !self.history.can_undo(),
                cx,
                |this, _, cx| this.undo(false, cx),
            ))
            .child(self.dock_button(
                "canvas-redo",
                Icon::Redo2,
                "Redo",
                false,
                !self.history.can_redo(),
                cx,
                |this, _, cx| this.undo(true, cx),
            ))
            .child(dock_separator(cx))
            .child(self.dock_button(
                "canvas-delete",
                Icon::Delete,
                "Delete selection",
                false,
                empty,
                cx,
                |this, _, cx| this.delete_selected(cx),
            ))
            .child(self.dock_button(
                "canvas-arrange",
                Icon::LayoutDashboard,
                "Arrange",
                false,
                self.doc.nodes.is_empty(),
                cx,
                |this, _, cx| this.arrange(cx),
            ))
            .child(dock_separator(cx))
            .child(self.dock_button(
                "canvas-zoom-out",
                Icon::MinusSignIcon,
                "Zoom out",
                false,
                false,
                cx,
                |this, _, cx| this.zoom_by(0.8, cx),
            ))
            .child(zoom_level(self.doc.viewport.zoom, cx))
            .child(self.dock_button(
                "canvas-zoom-in",
                Icon::Plus,
                "Zoom in",
                false,
                false,
                cx,
                |this, _, cx| this.zoom_by(1.25, cx),
            ))
            .child(self.dock_button(
                "canvas-fit",
                Icon::Maximize2,
                "Fit canvas",
                false,
                false,
                cx,
                |this, _, cx| this.fit(cx),
            ))
            .child(dock_separator(cx))
            .child(self.dock_button(
                "canvas-agent-visible",
                Icon::Eye,
                if self.agent_visible {
                    "Agent-visible (on)"
                } else {
                    "Agent-visible (off)"
                },
                self.agent_visible,
                false,
                cx,
                |this, _, cx| {
                    this.agent_visible = !this.agent_visible;
                    this.schedule(cx);
                },
            ))
            .child(self.dock_button(
                "canvas-rules",
                Icon::HelpCircle,
                rules,
                false,
                false,
                cx,
                |_, _, _| {},
            ));
        div()
            .absolute()
            .bottom(px(12.0))
            .left_0()
            .right_0()
            .flex()
            .justify_center()
            .child(
                canvas_dock(cx)
                    .occlude()
                    .child(picks)
                    .child(dock_separator(cx))
                    .child(actions),
            )
    }

    fn handles(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let [id] = self.selection.nodes.as_slice() else {
            return Vec::new();
        };
        let Some(node) = self.doc.node(id) else {
            return Vec::new();
        };
        let theme = *Theme::global(cx);
        let view = self.doc.viewport;
        let at = view.to_screen(Pt::new(node.x, node.y));
        let (w, h) = (node.width * view.zoom, node.height * view.zoom);
        HANDLES
            .iter()
            .map(|handle| {
                let x = if handle.west() {
                    0.0
                } else if handle.east() {
                    w
                } else {
                    w / 2.0
                };
                let y = if handle.north() {
                    0.0
                } else if handle.south() {
                    h
                } else {
                    h / 2.0
                };
                let cursor = match handle {
                    Handle::Nw | Handle::Se => CursorStyle::ResizeUpLeftDownRight,
                    Handle::Ne | Handle::Sw => CursorStyle::ResizeUpRightDownLeft,
                    Handle::N | Handle::S => CursorStyle::ResizeUpDown,
                    Handle::E | Handle::W => CursorStyle::ResizeLeftRight,
                };
                let editor = cx.entity().downgrade();
                let (node_id, handle) = (id.clone(), *handle);
                div()
                    .id(SharedString::from(format!(
                        "canvas-handle-{}",
                        handle.label()
                    )))
                    .absolute()
                    .left(px(at.x + x - HANDLE_SIZE / 2.0))
                    .top(px(at.y + y - HANDLE_SIZE / 2.0))
                    .size(px(HANDLE_SIZE))
                    .border_1()
                    .border_color(theme.color_info)
                    .bg(theme.color_surface_raised)
                    .hover(|handle| handle.bg(theme.color_info))
                    .cursor(cursor)
                    .on_mouse_down(
                        MouseButton::Left,
                        move |event: &MouseDownEvent, window, cx| {
                            cx.stop_propagation();
                            let _ = editor.update(cx, |this, cx| {
                                this.resize_pressed(node_id.clone(), handle, event, window, cx)
                            });
                        },
                    )
                    .into_any_element()
            })
            .collect()
    }

    fn edge_labels(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let theme = *Theme::global(cx);
        let view = self.doc.viewport;
        let routes = edge_routes(&self.doc);
        self.doc
            .edges
            .iter()
            .filter_map(|edge| {
                let label = edge_label(edge);
                if label.is_empty() {
                    return None;
                }
                let points = &routes.iter().find(|(id, _)| *id == edge.id)?.1;
                let middle = points.get(points.len() / 2)?;
                let at = view.to_screen(Pt::new(middle.x, middle.y - 8.0));
                let selected = self.selection.edges.contains(&edge.id);
                let size = 12.0 * view.zoom;
                Some(
                    div()
                        .absolute()
                        .left(px(at.x))
                        .top(px(at.y - size))
                        .whitespace_nowrap()
                        .text_size(px(size))
                        .text_color(if selected {
                            theme.color_text_primary
                        } else {
                            theme.color_text_muted
                        })
                        .child(label)
                        .into_any_element(),
                )
            })
            .collect()
    }

    fn well(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let view = self.doc.viewport;
        self.sync_label_type(cx);
        let entities = self.entities(cx);
        let (back, front) = layers(&self.doc);
        let label = |this: &Self, id: &str| {
            (this.editing.as_deref() == Some(id)).then(|| {
                div()
                    .w_full()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(this.label_input.clone())
                    .into_any_element()
            })
        };
        let draw = |this: &Self, node: &crate::canvas::document::Node| {
            node_view(
                node,
                &entities,
                view,
                NodeDecor {
                    selected: this.selection.nodes.contains(&node.id),
                    decorations: true,
                    editing: label(this, &node.id),
                },
                &theme,
            )
        };
        let back: Vec<AnyElement> = back.into_iter().map(|node| draw(self, node)).collect();
        let front: Vec<AnyElement> = front.into_iter().map(|node| draw(self, node)).collect();
        let (selected, hovered) = (self.selection.edges.clone(), self.hover_edge.clone());
        let edges = edges_view(&self.doc, view, move |edge| {
            if selected.contains(&edge.id) {
                theme.color_info
            } else if hovered.as_deref() == Some(edge.id.as_str()) {
                theme.color_text_secondary
            } else {
                theme.color_text_muted
            }
        });
        let editor = cx.entity().downgrade();
        let tracker = self.gesture.is_some().then(|| {
            let moves = editor.clone();
            let ups = editor.clone();
            canvas(
                |_, _, _| {},
                move |_, _, window, _| {
                    window.on_mouse_event({
                        let editor = moves.clone();
                        move |event: &MouseMoveEvent, phase, _, cx| {
                            if phase != DispatchPhase::Bubble {
                                return;
                            }
                            let _ = editor.update(cx, |this, cx| {
                                if event.pressed_button.is_none() {
                                    this.released(cx);
                                } else {
                                    this.dragged(event.position, cx);
                                }
                            });
                        }
                    });
                    window.on_mouse_event({
                        let editor = ups.clone();
                        move |_: &MouseUpEvent, phase, _, cx| {
                            if phase == DispatchPhase::Bubble {
                                let _ = editor.update(cx, |this, cx| this.released(cx));
                            }
                        }
                    });
                },
            )
            .absolute()
            .size_0()
        });
        let empty = self.doc.nodes.is_empty().then(|| {
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(13.0))
                .text_color(theme.color_text_muted)
                .child("This canvas is empty. Click or drag a shape from the library.")
        });
        let handles = self.handles(cx);
        let labels = self.edge_labels(cx);
        let cursor = match (&self.gesture, self.tool) {
            (
                Some(Gesture {
                    mode: GestureMode::Pan,
                    ..
                }),
                _,
            ) => CursorStyle::ClosedHand,
            (_, Tool::Pan) => CursorStyle::OpenHand,
            _ if self.hover_edge.is_some() => CursorStyle::PointingHand,
            _ => CursorStyle::Arrow,
        };
        let _ = window;
        div()
            .id("canvas-well")
            .track_focus(&self.focus)
            .key_context("CanvasEditor")
            .relative()
            .size_full()
            .overflow_hidden()
            .bg(theme.color_app_background)
            .cursor(cursor)
            .child(self.host.probe())
            .child(dot_grid(view, 1.0, theme.color_border_default))
            .children(empty)
            .children(back)
            .child(edges)
            .children(labels)
            .children(front)
            .children(handles)
            .children(tracker)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::pressed))
            .on_mouse_down(MouseButton::Middle, cx.listener(Self::pressed))
            .on_mouse_move(cx.listener(|this, event, _, cx| this.hovered(event, cx)))
            .on_scroll_wheel(
                cx.listener(|this, event, window, cx| this.scrolled(event, window, cx)),
            )
            .on_key_down(cx.listener(Self::key))
            .on_action(cx.listener(|this, _: &CloseOverlay, _, cx| this.escape(cx)))
            .on_drop(cx.listener(|this, drag: &LibraryDrag, window, cx| {
                let at = this
                    .doc
                    .viewport
                    .to_document(this.local(window.mouse_position()));
                let drag = drag.clone();
                this.insert(&drag.kind, Some(at), drag.entity, cx);
                cx.notify();
            }))
            .into_any_element()
    }
}

/// A fresh id for a node or a connection (`crypto.randomUUID`).
fn uuid() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|time| time.as_nanos())
        .unwrap_or_default();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mixed = (nanos as u64)
        ^ count.wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ (u64::from(std::process::id()) << 32);
    let high = (nanos >> 64) as u64 ^ count.rotate_left(17);
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (mixed >> 32) as u32,
        (mixed >> 16) as u16,
        (mixed & 0x0fff) as u16,
        (0x8000 | (high & 0x3fff)) as u16,
        (high >> 16) & 0xffff_ffff_ffff
    )
}

impl Render for CanvasEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *Theme::global(cx);
        let inspecting = !self.selection.nodes.is_empty()
            || self
                .selection
                .edges
                .first()
                .is_some_and(|id| self.doc.edge(id).is_some());
        self.inspector_panel
            .update(cx, |panel, _| panel.set_open(inspecting));
        let well = self.well(window, cx);
        let hint = self.connect_from.as_ref().map(|_| {
            let kind = edge_palette(&self.definition)
                .first()
                .cloned()
                .unwrap_or_default();
            let kind = if kind.is_empty() {
                "new".to_string()
            } else {
                kind
            };
            div().absolute().top(px(52.0)).left(px(12.0)).child(
                canvas_dock(cx)
                    .px(px(13.0))
                    .py(px(6.0))
                    .text_size(px(12.0))
                    .text_color(theme.color_text_secondary)
                    .child(format!("Pick the target shape for the {kind} connection.")),
            )
        });
        let stage = div()
            .relative()
            .size_full()
            .child(well)
            .child(self.toolbar(cx))
            .children(hint)
            .child(self.tools_dock(cx))
            .into_any_element();
        let inspector = self.inspector_view(window, cx);
        let inspector_panel = self.inspector_panel.clone();
        let inner = inspector_panel.update(cx, |panel, cx| {
            panel.render(
                &inspector_panel,
                panel_frame(inspector, PanelSide::Right, &theme),
                stage,
                window,
                cx,
            )
        });
        let library = self.library_view(window, cx);
        let library_panel = self.library_panel.clone();
        let workspace = library_panel.update(cx, |panel, cx| {
            panel.render(
                &library_panel,
                panel_frame(library, PanelSide::Left, &theme),
                inner,
                window,
                cx,
            )
        });
        let alert = self.error.clone().map(|message| {
            let view = cx.entity().downgrade();
            div()
                .flex_none()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(8.0))
                .pt(px(4.0))
                .px(px(12.0))
                .pb(px(8.0))
                .text_size(px(13.0))
                .text_color(theme.color_danger)
                .child(message)
                .child(
                    Button::new("canvas-retry-save", ButtonKind::Ghost)
                        .size(ButtonSize::Sm)
                        .label("Retry save")
                        .on_click(move |_, cx| {
                            let _ = view.update(cx, |this, cx| {
                                this.version = this.version.max(this.saved_version + 1);
                                this.flush(cx);
                            });
                        }),
                )
        });
        div()
            .size_full()
            .flex()
            .flex_col()
            .children(alert)
            .child(div().min_h_0().flex_1().child(workspace))
            .children(self.inspector.layers(cx))
    }
}

/// A side panel's face: the surface with a rule on its inner edge.
fn panel_frame(content: AnyElement, side: PanelSide, theme: &Theme) -> AnyElement {
    div()
        .size_full()
        .bg(theme.color_surface)
        .map(|panel| match side {
            PanelSide::Left => panel.border_r_1(),
            PanelSide::Right => panel.border_l_1(),
        })
        .border_color(theme.color_border_default)
        .child(content)
        .into_any_element()
}
