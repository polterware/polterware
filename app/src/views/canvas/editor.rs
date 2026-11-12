//! The canvas editor (`editor.tsx`): the shape library on the left, the diagram in the well,
//! and the inspector on the right, which opens only for a selected node or edge. Both panels
//! resize (192px, 168 to 256; 272px, 256 to 336).
//!
//! The canvas is Mermaid text, so the well draws the source: `polterware_mermaid` lays the
//! flowchart out with the window's text measurement and `MermaidView` paints it, with pan,
//! zoom around the pointer, a double-click to fit, a click that selects the node or edge under
//! it, and a drag that moves a node. Pins from `layout_json` override the automatic layout, and
//! its viewport restores the pan and zoom the canvas was left at. A source the renderer does
//! not draw yet shows the text and a note instead of the diagram.
//!
//! Every edit writes the source, by line: the library appends a node, the inspector rewrites a
//! node's label, kind, reference, or properties, deleting removes the lines, connecting appends
//! a link, and the text pane types over it. The pins are the one thing the text does not carry,
//! so a dropped node writes its place into `layout_json` and Reset layout clears them.
//!
//! The toolbar floats over the well: Canvases (back), the name in a 224px pill field, the save
//! state (Unsaved, Saving…, Not saved in the danger color, Saved; 11px muted), and Mermaid
//! source, which opens the text pane beside the diagram. Typing there re-renders after a short
//! pause; a parse error reads as `line L, column C: message` under the text and keeps the last
//! diagram that drew. The tool dock floats at the bottom center: Select, Pan, Connect; Undo,
//! Redo; Delete selection, Arrange; Zoom out, the zoom level, Zoom in, Fit canvas, Reset layout;
//! Agent-visible and Canvas rules. While Connect waits for its target, a hint floats under the
//! toolbar.
//!
//! Every edit saves on its own after 500ms, one writer per canvas: new edits coalesce, and an
//! older answer never replaces local state. Leaving saves first; a refusal stays on the page
//! with Retry save. An editor closed from outside saves what it holds.

use std::collections::HashMap;
use std::time::Duration;

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, FocusHandle, Focusable, IntoElement,
    KeyDownEvent, MouseButton, MouseDownEvent, Render, SharedString, Task, Window, div, point,
    prelude::*, px,
};
use polterware_mermaid::scene::{Rect, Scene};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::canvas::{
    Capture, History, arrange_horizontally, canvas_dock, dock_group, dock_separator, zoom_level,
};
use polterware_ui::card::card;
use polterware_ui::icons::Icon;
use polterware_ui::mermaid::{MermaidEvent, MermaidView, SceneMeasure, SceneViewport};
use polterware_ui::patterns::{AnchorSlot, PanelSide, ResizablePanel};
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::Theme;
use polterware_ui::window::window_drag::WindowDragRegion;

use super::mermaid as source;
use super::source_edit;
use crate::app::CloseOverlay;
use crate::canvas::catalog::{
    PaletteGroup, component_of, default_props, edge_palette, node_palette, palette_groups,
    palette_label,
};
use crate::canvas::document::{Doc, Selection, update_node};
use crate::canvas::references::{StoreEntity, build_ref};
use crate::contracts::{Canvas, CanvasTypeDefinition};
use crate::state::{AppState, CanvasPatch, OpenCanvas};

/// How long an edit waits before it saves.
const SAVE_DELAY: Duration = Duration::from_millis(500);
/// How long typing in the source pane waits before the diagram is drawn again.
const RENDER_DELAY: Duration = Duration::from_millis(250);
/// The source pane's floating width.
const SOURCE_PANE_WIDTH: f32 = 380.0;
/// How many frames the view may take to measure itself before the remembered viewport is
/// dropped.
const RESTORE_FRAMES: u8 = 4;
/// How many frames a reveal runs for: the inspector's panel slides in over the diagram, and
/// every frame it moves leaves a different room to fit the drawing into.
const REVEAL_FRAMES: u8 = 12;

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
    /// The canvas as Mermaid text: what the diagram draws and what saving writes.
    pub(super) source: String,
    /// The pins and the viewport the canvas layout keeps; the text carries neither.
    pub(super) layout: String,
    /// The canvas node each diagram id stands for, read from the source's `@{ }` blocks.
    node_ids: HashMap<String, String>,
    /// The box each diagram node's drawing covered on the last render, by Mermaid id.
    boxes: HashMap<String, (f32, f32, f32, f32)>,
    /// The colors the diagram was last drawn with.
    colors: source::Palette,
    /// The selected node wants to be panned into view.
    reveal: bool,
    /// How many frames the reveal may still run for: the inspector's panel slides over the
    /// diagram, so the room it leaves is measured again for as long as it moves.
    reveal_waits: u8,
    source_input: Entity<TextInput>,
    source_open: bool,
    /// The node a drag dropped last, as the diagram id and its new top-left: the screenshot
    /// tour reads it to check the pin it wrote.
    last_drop: Option<(String, f32, f32)>,
    /// The source does not draw: the parser's place or the renderer's answer.
    render_error: Option<String>,
    /// The source is a diagram type the renderer does not draw yet, and the note saying so.
    unsupported: Option<String>,
    /// The diagram is waiting to be drawn again, and the pause it waits out.
    render_due: bool,
    render_timer: Option<Task<()>>,
    view: Entity<MermaidView>,
    /// The pan and zoom the canvas was opened at, put on screen once the view has measured
    /// itself, and the frames left to wait for that.
    restore_viewport: Option<SceneViewport>,
    restore_waits: u8,
    /// The diagram has never been fitted: it opens fitted to the whole drawing, and a canvas
    /// with a saved viewport restores that instead.
    fit_on_open: bool,
    /// How many frames the opening fit may still be asked for. The view measures itself on the
    /// frame it paints, so the wish outlives a frame that measured nothing.
    fit_waits: u8,
    /// The pan and zoom the canvas was opened at, so leaving can tell whether it moved.
    opened_viewport: Option<SceneViewport>,
    history: History<String>,
    pub(super) tool: Tool,
    connect_from: Option<String>,
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
        // The canvas is Mermaid text: the pane edits it, and the view draws it.
        let text = if open.canvas.source.trim().is_empty() {
            "flowchart LR\n".to_string()
        } else {
            open.canvas.source.clone()
        };
        let source_input = cx.new(|cx| {
            TextInput::new(cx)
                .multiline()
                .nowrap()
                .mono()
                .bare()
                .placeholder("flowchart LR\n  start[Start] --> end[End]")
        });
        source_input.update(cx, |input, cx| input.set_text(text.clone(), cx));
        cx.observe(&source_input, |this, input, cx| {
            let text = input.read(cx).text().to_string();
            if text != this.source {
                this.source = text;
                this.draw_soon(cx);
                this.schedule(cx);
            }
        })
        .detach();
        let view = cx.new(|cx| {
            MermaidView::new(Scene::default()).background(Theme::global(cx).color_app_background)
        });
        cx.subscribe(&view, |this, _, event: &MermaidEvent, cx| match event {
            MermaidEvent::Selected(hit) => this.select(hit.as_ref(), cx),
            MermaidEvent::NodeDropped { id, position } => this.drop_node(id, *position, cx),
            _ => {}
        })
        .detach();
        let layout = open.canvas.layout_json.clone();
        let opened_viewport = source::viewport(&layout);
        let doc = source_edit::document(&text, &layout, &HashMap::new());
        Self {
            state,
            agent_visible: open.canvas.agent_visible,
            canvas: open.canvas,
            definition: open.definition,
            palette,
            groups,
            doc,
            selection: Selection::default(),
            node_ids: source::node_ids(&text),
            boxes: HashMap::new(),
            colors: source::palette(Theme::global(cx)),
            reveal: false,
            reveal_waits: 0,
            source: text,
            layout,
            source_input,
            source_open: false,
            last_drop: None,
            render_error: None,
            unsupported: None,
            render_due: true,
            render_timer: None,
            view,
            restore_viewport: opened_viewport,
            restore_waits: RESTORE_FRAMES,
            fit_on_open: opened_viewport.is_none(),
            fit_waits: RESTORE_FRAMES,
            opened_viewport,
            history: History::new(100),
            tool: Tool::Select,
            connect_from: None,
            name_input,
            name,
            save_state: SaveState::Saved,
            error: None,
            leaving: false,
            version: 0,
            saved_version: 0,
            saving: false,
            timer: None,
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

    /// The canvas as the diagram draws it: the source, the pins, and the window's text
    /// measurement, so a label wraps where the painter draws it. A source that does not draw
    /// keeps the last diagram on screen and says where the parser stopped.
    fn draw(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let theme = *Theme::global(cx);
        let palette = source::palette(&theme);
        let measure = SceneMeasure::new(
            window.text_system().clone(),
            theme.font_sans,
            theme.font_mono,
        );
        let drawn = source::render(&self.source, &self.layout, &measure, &palette);
        self.node_ids = source::node_ids(&self.source);
        match drawn {
            Ok(scene) => {
                self.boxes = source::node_boxes(&scene);
                self.doc = source_edit::document(&self.source, &self.layout, &self.boxes);
                self.colors = palette;
                let first = self.view.read(cx).viewport().is_none();
                self.view.update(cx, |view, cx| {
                    if first {
                        view.set_scene(scene, cx);
                    } else {
                        view.update_scene(scene, cx);
                    }
                });
                self.show_selection(cx);
                self.render_error = None;
                self.unsupported = None;
            }
            Err(error) => {
                let message = source::message(&error);
                self.render_error = Some(message.clone());
                self.unsupported = source::unsupported(&error).then(|| {
                    format!(
                        "{message}. This canvas shows its source until the renderer draws that diagram type."
                    )
                });
                self.source_open = true;
            }
        }
        self.render_due = false;
    }

    /// Draws the source again after a short pause, so typing does not lay the diagram out on
    /// every keystroke.
    fn draw_soon(&mut self, cx: &mut Context<Self>) {
        self.render_timer = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RENDER_DELAY).await;
            let _ = this.update(cx, |this, cx| {
                this.render_due = true;
                cx.notify();
            });
        }));
        cx.notify();
    }

    /// Puts the pan and zoom waiting to be restored on screen. `MermaidView` measures itself
    /// on its first paint, so the wish is kept until it has.
    fn place_viewport(&mut self, cx: &mut Context<Self>) {
        let Some(target) = self.restore_viewport else {
            return;
        };
        if self.view.read(cx).viewport().is_none() {
            return;
        }
        self.restore_viewport = None;
        self.apply_view(target, cx);
    }

    /// Moves the pan and zoom to `target`. `MermaidView` zooms around a point rather than
    /// taking a viewport, so this picks the point that lands the scene where `target` has it;
    /// a ratio of one cannot move the pan, so a same-zoom move goes out a step and back in.
    fn apply_view(&mut self, target: SceneViewport, cx: &mut Context<Self>) {
        let Some(current) = self.view.read(cx).viewport() else {
            return;
        };
        let ratio = target.zoom / current.zoom;
        if (ratio - 1.0).abs() < 1e-4 {
            let half = 0.5;
            let focus = point(
                px((target.x - current.x * half) / (1.0 - half)),
                px((target.y - current.y * half) / (1.0 - half)),
            );
            self.view.update(cx, |view, cx| {
                view.zoom_at(focus, current.zoom * half, cx);
                view.zoom_at(point(px(target.x), px(target.y)), target.zoom, cx);
            });
        } else {
            let focus = point(
                px((target.x - current.x * ratio) / (1.0 - ratio)),
                px((target.y - current.y * ratio) / (1.0 - ratio)),
            );
            self.view
                .update(cx, |view, cx| view.zoom_at(focus, target.zoom, cx));
        }
    }

    /// Keeps the selected node in view in the uncovered part of the diagram. A fitting drawing
    /// is fitted into that room; otherwise the view pans only as far as the node needs.
    fn reveal_selection(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(id) = self.selection.nodes.first() else {
            return false;
        };
        let Some(mermaid) = self.mermaid(id) else {
            return false;
        };
        let Some((bx, by, bw, bh)) = self.boxes.get(&mermaid).copied() else {
            return false;
        };
        let Some(current) = self.view.read(cx).viewport() else {
            return false;
        };
        // The diagram sits beside both panels, so its own box is the room a diagram may use.
        let Some(bounds) = self.host.get() else {
            return true;
        };
        let full_width = f32::from(bounds.size.width);
        let height = f32::from(bounds.size.height);
        let left = self.open_panel_width(PanelSide::Left, cx);
        let right = self.open_panel_width(PanelSide::Right, cx);
        let room_width = (full_width - left - right).max(1.0);
        let focus = Rect::new(f64::from(bx), f64::from(by), f64::from(bw), f64::from(bh));
        let view_box = self.view.read(cx).scene().view_box;
        let local_current = SceneViewport::new(current.x - left, current.y, current.zoom);
        let Some(mut target) = source::reveal_view(
            local_current,
            view_box,
            (room_width, height),
            focus,
            source::REVEAL_MARGIN,
        ) else {
            return false;
        };
        target.x += left;
        self.apply_view(target, cx);
        true
    }

    fn open_panel_width(&self, side: PanelSide, cx: &App) -> f32 {
        if side == PanelSide::Right && self.source_open {
            return SOURCE_PANE_WIDTH + PANEL_INSET;
        }
        let panel = match side {
            PanelSide::Left => &self.library_panel,
            PanelSide::Right => &self.inspector_panel,
        };
        if panel.read(cx).is_open() {
            panel.read(cx).width()
        } else {
            0.0
        }
    }

    fn fit_in_open_area(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(bounds) = self.host.get() else {
            return false;
        };
        if self.view.read(cx).viewport().is_none() {
            return false;
        }
        let target = source::fit_viewport_inset(
            self.view.read(cx).scene().view_box,
            (f32::from(bounds.size.width), f32::from(bounds.size.height)),
            self.open_panel_width(PanelSide::Left, cx),
            self.open_panel_width(PanelSide::Right, cx),
            source::REVEAL_MARGIN,
        );
        self.apply_view(target, cx);
        true
    }

    /// The pan and zoom on screen, as the layout keeps them.
    fn viewport(&self, cx: &App) -> Option<SceneViewport> {
        self.view.read(cx).viewport()
    }

    /// Counts a moved viewport as an edit, so leaving writes where the canvas was left.
    fn touch_viewport(&mut self, cx: &App) {
        if self.viewport(cx) == self.opened_viewport {
            return;
        }
        self.version += 1;
        self.save_state = SaveState::Pending;
    }

    /// Follows a click in the diagram: the node or edge it hit opens in the inspector, and
    /// empty space closes it. A diagram id that stands for a canvas node (`u1@{ id: ... }`)
    /// is followed to that node, which is the one the inspector knows. While Connect waits,
    /// a click on a node is the connection's other end.
    fn select(&mut self, hit: Option<&polterware_ui::mermaid::SceneHit>, cx: &mut Context<Self>) {
        use polterware_mermaid::scene::SourceKind;
        let canvas_id = |id: &str| {
            self.node_ids
                .get(id)
                .cloned()
                .unwrap_or_else(|| id.to_string())
        };
        if self.tool == Tool::Connect
            && let Some(hit) = hit.filter(|hit| hit.source.kind == SourceKind::Node)
        {
            self.connect_to(canvas_id(&hit.source.id), cx);
            return;
        }
        let selection = match hit {
            Some(hit) if hit.source.kind == SourceKind::Node => {
                let id = canvas_id(&hit.source.id);
                self.doc
                    .node(&id)
                    .map_or(Selection::default(), |_| Selection::node(&id))
            }
            Some(hit) if hit.source.kind == SourceKind::Edge => {
                let id = canvas_id(&hit.source.id);
                self.doc
                    .edge(&id)
                    .map_or(Selection::default(), |_| Selection::edge(&id))
            }
            _ => Selection::default(),
        };
        if selection == self.selection {
            return;
        }
        if selection.is_empty() {
            self.connect_from = None;
        }
        self.selection = selection;
        self.show_selection(cx);
        self.reveal = !self.selection.nodes.is_empty();
        self.reveal_waits = REVEAL_FRAMES;
        self.inspector_panel
            .update(cx, |panel, _| panel.set_open(!self.selection.is_empty()));
        cx.notify();
    }

    /// Makes the diagram outline the selection the editor holds, so a selection made in the
    /// inspector, in the text pane, or by a palette insert shows in the drawing, not only one
    /// made by a click on it.
    fn show_selection(&mut self, cx: &mut Context<Self>) {
        let id = self
            .selection
            .nodes
            .first()
            .and_then(|id| self.mermaid(id))
            .or_else(|| {
                self.selection
                    .edges
                    .first()
                    .and_then(|id| self.edge_link(id))
            });
        self.view
            .update(cx, |view, cx| view.select_id(id.as_deref(), cx));
    }

    /// Pins the dropped node where the drawing left it, so the next render keeps it there.
    fn drop_node(
        &mut self,
        id: &str,
        position: polterware_mermaid::scene::Point,
        cx: &mut Context<Self>,
    ) {
        let x = position.x as f32;
        let y = position.y as f32;
        if !x.is_finite() || !y.is_finite() {
            return;
        }
        self.layout = source_edit::with_pin(&self.layout, id, x, y);
        self.last_drop = Some((id.to_string(), x, y));
        let canvas = self
            .node_ids
            .get(id)
            .cloned()
            .unwrap_or_else(|| id.to_string());
        self.doc = update_node(&self.doc, &canvas, |node| {
            node.x = x;
            node.y = y;
        });
        self.schedule(cx);
    }

    /// The diagram id a canvas node id stands for.
    pub(super) fn mermaid(&self, canvas_id: &str) -> Option<String> {
        source_edit::mermaid_id(&self.source, canvas_id)
    }

    /// The diagram id a connection's canvas id stands for.
    pub(super) fn edge_link(&self, canvas_id: &str) -> Option<String> {
        source_edit::edge_mermaid_id(&self.source, canvas_id)
    }

    /// Opens or closes the text pane, as the toolbar's Mermaid source button does, and puts
    /// the caret at the source's end when it opens.
    pub(super) fn toggle_source(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.source_open = !self.source_open;
        if self.source_open {
            let text = self.source.clone();
            self.source_input.update(cx, |input, cx| {
                input.set_text(text, cx);
                input.focus_handle(cx).focus(window);
            });
        }
        self.fit(cx);
        cx.notify();
    }

    /// The canvas as the text pane holds it (the screenshot tour's stops).
    pub(super) fn source(&self) -> &str {
        &self.source
    }

    /// The parser's answer while the source does not draw.
    pub(super) fn render_message(&self) -> Option<&str> {
        self.render_error.as_deref()
    }

    /// Whether the last drawing has a node called `name`.
    pub(super) fn has_node(&self, name: &str) -> bool {
        self.doc.nodes.iter().any(|node| node.name == name)
    }

    /// The pin the last drag wrote, when `layout` carries it.
    pub(super) fn dropped_pin(&self) -> Option<(String, f32, f32)> {
        let id = self.last_drop.as_ref()?.0.clone();
        let layout: serde_json::Value = serde_json::from_str(&self.layout).ok()?;
        let pin = layout.get("nodes")?.get(&id)?;
        if !pin.get("pinned")?.as_bool()? {
            return None;
        }
        let x = pin.get("x")?.as_f64()? as f32;
        let y = pin.get("y")?.as_f64()? as f32;
        Some((id, x, y))
    }

    /// The diagram the editor draws, for the tour's scripted pointer.
    pub(super) fn diagram(&self) -> &Entity<MermaidView> {
        &self.view
    }

    /// Where the first node's middle is in the window, so the tour's pointer presses the node
    /// the diagram draws: the view's place between the panels, the pan and zoom it holds, and
    /// the node's box.
    pub(super) fn node_point(&self, cx: &App) -> Option<(f32, f32)> {
        let node = self.doc.nodes.first()?;
        let mermaid = self.mermaid(&node.id)?;
        let (x, y, width, height) = self.boxes.get(&mermaid).copied()?;
        let viewport = self.view.read(cx).viewport()?;
        let theme = Theme::global(cx);
        let origin_x = theme.sidebar_width + self.library_panel.read(cx).width();
        let origin_y = theme.titlebar_height;
        Some((
            origin_x + x * viewport.zoom + viewport.x + width * viewport.zoom / 2.0,
            origin_y + y * viewport.zoom + viewport.y + height * viewport.zoom / 2.0,
        ))
    }

    /// Selects the canvas's first node, as a click on it would, so its inspector shows (the
    /// screenshot tour's stop).
    pub(crate) fn select_first_node(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.doc.nodes.first().map(|node| node.id.clone()) else {
            return;
        };
        self.selection = Selection::node(&id);
        self.reveal = true;
        self.reveal_waits = REVEAL_FRAMES;
        self.show_selection(cx);
        self.inspector_panel
            .update(cx, |panel, _| panel.set_open(true));
        cx.notify();
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

    fn patch(&self, cx: &App) -> CanvasPatch {
        let layout_json = match self.viewport(cx) {
            Some(viewport) => source::with_viewport(&self.layout, viewport),
            None => self.layout.clone(),
        };
        CanvasPatch {
            name: Some(self.name.trim().to_string()),
            agent_visible: Some(self.agent_visible),
            source: Some(self.source.clone()),
            layout_json: Some(layout_json),
            ..CanvasPatch::default()
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
        let patch = self.patch(cx);
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
                        this.opened_viewport = this.viewport(cx);
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
        self.touch_viewport(cx);
        if self.version <= self.saved_version || self.name.trim().is_empty() {
            return;
        }
        self.saved_version = self.version;
        let patch = self.patch(cx);
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
        self.touch_viewport(cx);
        self.leaving = true;
        if self.version <= self.saved_version && !self.saving {
            self.leaving = false;
            self.state.update(cx, |state, cx| state.close_canvas(cx));
            return;
        }
        self.flush(cx);
    }

    // Editing.

    /// Writes an edited source, showing it in the pane and drawing it again.
    pub(super) fn apply_source(&mut self, next: String, capture: Capture, cx: &mut Context<Self>) {
        if next == self.source {
            return;
        }
        self.history.commit(&self.source, &next, capture);
        self.source = next;
        let text = self.source.clone();
        self.source_input.update(cx, |input, cx| {
            if input.text() != text {
                input.set_text(text, cx);
            }
        });
        self.render_due = true;
        self.schedule(cx);
    }

    /// Writes an edited layout (pins or viewport) and saves it.
    pub(super) fn apply_layout(&mut self, next: String, cx: &mut Context<Self>) {
        if next == self.layout {
            return;
        }
        self.layout = next;
        self.render_due = true;
        self.schedule(cx);
    }

    fn undo(&mut self, redo: bool, cx: &mut Context<Self>) {
        // Settle a pending typed edit first, so undo reverts it instead of dropping it.
        if self.history.has_pending() {
            let source = self.source.clone();
            self.history.commit(&source, &source, Capture::Immediately);
        }
        let next = if redo {
            self.history.redo(&self.source)
        } else {
            self.history.undo(&self.source)
        };
        let Some(next) = next else {
            return;
        };
        self.source = next;
        let text = self.source.clone();
        self.source_input.update(cx, |input, cx| {
            if input.text() != text {
                input.set_text(text, cx);
            }
        });
        self.render_due = true;
        self.selection = Selection::default();
        self.show_selection(cx);
        self.schedule(cx);
    }

    pub(super) fn delete_selected(&mut self, cx: &mut Context<Self>) {
        if self.selection.is_empty() {
            return;
        }
        let mut source = self.source.clone();
        let mut layout = self.layout.clone();
        for id in &self.selection.nodes {
            if let Some(mermaid) = self.mermaid(id) {
                source = source_edit::remove_node(&source, &mermaid);
                layout = source_edit::without_pin(&layout, &mermaid);
            }
        }
        for id in &self.selection.edges {
            if let Some(link) = self.edge_link(id) {
                source = source_edit::remove_edge(&source, &link);
            }
        }
        self.layout = layout;
        self.apply_source(source, Capture::Immediately, cx);
        self.selection = Selection::default();
        self.show_selection(cx);
        self.connect_from = None;
    }

    fn zoom_by(&mut self, factor: f32, cx: &mut Context<Self>) {
        let Some(viewport) = self.view.read(cx).viewport() else {
            return;
        };
        let zoom = viewport.zoom * factor;
        self.view.update(cx, |view, cx| view.zoom_to(zoom, cx));
    }

    fn fit(&mut self, cx: &mut Context<Self>) {
        if !self.fit_in_open_area(cx) {
            self.fit_on_open = true;
            self.fit_waits = RESTORE_FRAMES;
            cx.notify();
        }
    }

    /// Clears the pins, so the automatic layout places every node again.
    fn reset_layout(&mut self, cx: &mut Context<Self>) {
        let layout = source_edit::clear_pins(&self.layout);
        self.apply_layout(layout, cx);
    }

    /// Pins each node at the place `place` gives it.
    fn pin_nodes(
        &mut self,
        cx: &mut Context<Self>,
        mut place: impl FnMut(&crate::canvas::document::Node) -> (f32, f32),
    ) {
        let mut layout = self.layout.clone();
        let mut moved = false;
        for node in &self.doc.nodes {
            let (x, y) = place(node);
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            let Some(mermaid) = self.mermaid(&node.id) else {
                continue;
            };
            layout = source_edit::with_pin(&layout, &mermaid, x, y);
            moved = true;
        }
        if moved {
            self.apply_layout(layout, cx);
        }
    }

    fn arrange(&mut self, cx: &mut Context<Self>) {
        let widths: Vec<f32> = self.doc.nodes.iter().map(|node| node.width).collect();
        let places = arrange_horizontally(&widths, 48.0);
        let mut index = 0;
        self.pin_nodes(cx, |_| {
            let at = places.get(index).copied().unwrap_or_default();
            index += 1;
            (at.x, at.y)
        });
        self.fit(cx);
    }

    /// Inserts a node of `kind` into the source, citing `entity` when one is given.
    pub(super) fn insert(
        &mut self,
        kind: &str,
        entity: Option<StoreEntity>,
        cx: &mut Context<Self>,
    ) {
        if !self.palette.iter().any(|item| item == kind) {
            self.error = Some("This shape is not allowed by the core.".into());
            cx.notify();
            return;
        }
        let named = entity
            .as_ref()
            .map(|entity| entity.title.clone())
            .or_else(|| {
                (polterware_ui::canvas::shape_of(kind).is_some() || component_of(kind).is_some())
                    .then(|| palette_label(kind))
            })
            .unwrap_or_else(|| palette_label(kind));
        let props = default_props(kind);
        let reference = entity.as_ref().map(build_ref);
        let (source, _mermaid, canvas) =
            source_edit::insert_node(&self.source, kind, &named, reference.as_ref(), &props);
        self.error = None;
        self.apply_source(source, Capture::Immediately, cx);
        self.selection = Selection::node(&canvas);
        self.reveal = true;
        self.reveal_waits = REVEAL_FRAMES;
        self.show_selection(cx);
        self.inspector_panel
            .update(cx, |panel, _| panel.set_open(true));
        cx.notify();
    }

    /// Connects the selected node to `to`, or starts the connection at it.
    fn connect_to(&mut self, to: String, cx: &mut Context<Self>) {
        let Some(from) = self.connect_from.take() else {
            self.selection = Selection::node(&to);
            self.show_selection(cx);
            self.connect_from = Some(to);
            cx.notify();
            return;
        };
        if from == to {
            self.error = Some("Choose two different shapes.".into());
            cx.notify();
            return;
        }
        let (Some(a), Some(b)) = (self.mermaid(&from), self.mermaid(&to)) else {
            self.error = Some("Those shapes are not in the diagram.".into());
            cx.notify();
            return;
        };
        let kind = edge_palette(&self.definition)
            .first()
            .cloned()
            .unwrap_or_default();
        let edge_id = uuid();
        let next = source_edit::connect(&self.source, &a, &b, &kind, &edge_id);
        self.error = None;
        self.apply_source(next, Capture::Immediately, cx);
        self.selection = Selection::edge(&edge_id);
        self.show_selection(cx);
    }

    /// Changes a connection's kind, rewriting its link text.
    pub(super) fn set_edge_kind(&mut self, id: &str, kind: &str, cx: &mut Context<Self>) {
        let Some(link) = self.edge_link(id) else {
            return;
        };
        let next = source_edit::set_edge_kind(&self.source, &link, kind);
        self.doc = crate::canvas::document::update_edge(&self.doc, id, |edge| {
            edge.kind = kind.to_string();
        });
        self.apply_source(next, Capture::Immediately, cx);
    }

    // The keys.

    fn key(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let keys = &event.keystroke;
        let command = keys.modifiers.platform || keys.modifiers.control;
        match keys.key.as_str() {
            "z" if command => self.undo(keys.modifiers.shift, cx),
            "delete" | "backspace" if !command => self.delete_selected(cx),
            "left" | "right" | "up" | "down" if !self.selection.nodes.is_empty() => {
                let step = if keys.modifiers.shift { 40.0 } else { 10.0 };
                let (dx, dy) = match keys.key.as_str() {
                    "left" => (-step, 0.0),
                    "right" => (step, 0.0),
                    "up" => (0.0, -step),
                    _ => (0.0, step),
                };
                let ids = self.selection.nodes.clone();
                let doc = self.doc.clone();
                let mut layout = self.layout.clone();
                for id in &ids {
                    let Some(node) = doc.node(id) else {
                        continue;
                    };
                    let Some(mermaid) = self.mermaid(id) else {
                        continue;
                    };
                    layout = source_edit::with_pin(&layout, &mermaid, node.x + dx, node.y + dy);
                }
                self.doc = crate::canvas::document::Doc {
                    nodes: self
                        .doc
                        .nodes
                        .iter()
                        .map(|node| {
                            if ids.contains(&node.id) {
                                crate::canvas::document::Node {
                                    x: node.x + dx,
                                    y: node.y + dy,
                                    ..node.clone()
                                }
                            } else {
                                node.clone()
                            }
                        })
                        .collect(),
                    ..self.doc.clone()
                };
                self.apply_layout(layout, cx);
            }
            _ => return,
        }
        cx.stop_propagation();
    }

    fn escape(&mut self, cx: &mut Context<Self>) {
        self.tool = Tool::Select;
        self.connect_from = None;
        self.selection = Selection::default();
        self.show_selection(cx);
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
        let back = view.clone();
        let toolbar = card(cx)
            .id("canvas-toolbar")
            .occlude()
            .w(px(420.0))
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
                        let _ = back.update(cx, |this, cx| this.back(cx));
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
            .child(
                Button::new("canvas-source-toggle", ButtonKind::Secondary)
                    .size(ButtonSize::Icon)
                    .icon(Icon::SourceCodeIcon)
                    .selected(self.source_open)
                    .tooltip(if self.source_open {
                        "Hide Mermaid source"
                    } else {
                        "Mermaid source"
                    })
                    .on_click({
                        let view = view.clone();
                        move |window, cx| {
                            let _ = view.update(cx, |this, cx| {
                                this.toggle_source(window, cx);
                            });
                        }
                    }),
            )
            .child(div().flex_1().h_full().window_drag_region());
        div()
            .absolute()
            .top(px(PANEL_INSET))
            .left_0()
            .right_0()
            .flex()
            .justify_center()
            .child(toolbar)
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
            .child(zoom_level(
                self.view
                    .read(cx)
                    .viewport()
                    .map_or(1.0, |viewport| viewport.zoom),
                cx,
            ))
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
            .child(self.dock_button(
                "canvas-reset-layout",
                Icon::RefreshCw,
                "Reset layout",
                false,
                false,
                cx,
                |this, _, cx| this.reset_layout(cx),
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

    /// The diagram, and the source pane beside it when it is open. The view draws the
    /// canvas's Mermaid source with the window's measurement, so labels wrap as they paint,
    /// and a click in it selects the node or edge under the pointer.
    fn well(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        if self.render_due {
            self.draw(window, cx);
        }
        self.place_viewport(cx);
        // A canvas opened without a saved viewport opens on the whole drawing, so a wide
        // diagram does not open at 100% with its right-hand node cut off. The view fits
        // itself against the room it measures when it paints, so the wish is repeated for a
        // few frames in case the first one measured nothing.
        if self.fit_on_open {
            if self.fit_waits > 0 {
                self.fit_waits -= 1;
                if !self.fit_in_open_area(cx) {
                    cx.notify();
                }
            } else {
                self.fit_on_open = false;
            }
        }
        // The view measures itself on its first paint, so the remembered pan and zoom waits
        // for the frame after it; asking for that frame, a few times at most, keeps the wish
        // from waiting forever.
        if self.restore_viewport.is_some() && self.unsupported.is_none() {
            if self.restore_waits > 0 {
                self.restore_waits -= 1;
                cx.notify();
            } else {
                self.restore_viewport = None;
            }
        }
        let diagram = match self.unsupported.clone() {
            Some(note) => div()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(6.0))
                .p(px(24.0))
                .text_center()
                .child(
                    div()
                        .text_size(px(13.0))
                        .text_color(theme.color_text_primary)
                        .child(note),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(theme.color_text_muted)
                        .child("The source stays editable in the pane."),
                )
                .into_any_element(),
            None => div()
                .size_full()
                .child(self.view.clone())
                .into_any_element(),
        };
        div()
            .id("canvas-well")
            .track_focus(&self.focus)
            .key_context("CanvasEditor")
            .on_key_down(cx.listener(Self::key))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _: &MouseDownEvent, window, _| {
                    this.focus.focus(window);
                }),
            )
            .relative()
            .size_full()
            .overflow_hidden()
            .bg(theme.color_app_background)
            .child(self.host.probe())
            .child(diagram)
            .on_action(cx.listener(|this, _: &CloseOverlay, _, cx| this.escape(cx)))
            .into_any_element()
    }

    /// The canvas's Mermaid text beside the diagram, with the place the parser stopped when
    /// the text does not draw.
    fn source_pane(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        card(cx)
            .id("canvas-source-pane-card")
            .occlude()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .id("canvas-source-pane")
                    .size_full()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .flex_none()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .h(px(32.0))
                            .px(px(12.0))
                            .text_size(px(12.0))
                            .text_color(theme.color_text_secondary)
                            .child("Mermaid source"),
                    )
                    .child(
                        div()
                            .id("canvas-source-scroll")
                            .min_h_0()
                            .flex_1()
                            .overflow_y_scroll()
                            .px(px(12.0))
                            .pt(px(12.0))
                            .pb(px(12.0))
                            .child(self.source_input.clone()),
                    )
                    .when_some(self.render_error.clone(), |pane, error| {
                        pane.child(
                            div()
                                .flex_none()
                                .border_t_1()
                                .border_color(theme.color_border_default)
                                .px(px(12.0))
                                .py(px(8.0))
                                .text_size(px(12.0))
                                .text_color(theme.color_danger)
                                .child(error),
                        )
                    }),
            )
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
        // The text pane floats where the inspector does, so the inspector steps aside while the
        // source is open instead of showing through behind it.
        let inspecting = !self.selection.is_empty() && !self.source_open;
        let inspector_was_open = self.inspector_panel.read(cx).is_open();
        self.inspector_panel
            .update(cx, |panel, _| panel.set_open(inspecting));
        if inspector_was_open && !inspecting {
            self.fit(cx);
        }
        if source::palette(&theme) != self.colors {
            self.render_due = true;
        }
        if self.reveal {
            if self.reveal_waits > 0 {
                self.reveal_waits -= 1;
                // The inspector's panel is still sliding over the diagram: keep measuring the
                // room it leaves until it stops, or the fit is one for a room nobody has.
                if self.reveal_selection(cx) {
                    cx.notify();
                } else {
                    self.reveal = false;
                }
            } else {
                self.reveal = false;
            }
        }
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
        let library = self.library_view(window, cx);
        let library_overlay = self.library_panel.update(cx, |panel, cx| {
            panel.render(
                &self.library_panel,
                panel_frame("canvas-library-panel", library, PanelSide::Left, cx),
                div().size_0().into_any_element(),
                window,
                cx,
            )
        });
        let overlays = div()
            .absolute()
            .top_0()
            .bottom_0()
            .left_0()
            .w(px(self.library_panel.read(cx).width() + 12.0))
            .child(library_overlay);
        let inspector = self.inspector_view(window, cx);
        let inspector_panel = self.inspector_panel.clone();
        let inspector_overlay = inspector_panel.update(cx, |panel, cx| {
            panel.render(
                &inspector_panel,
                panel_frame("canvas-inspector-panel", inspector, PanelSide::Right, cx),
                div().size_0().into_any_element(),
                window,
                cx,
            )
        });
        let mut layer = div()
            .relative()
            .size_full()
            .child(well)
            .child(self.toolbar(cx))
            .child(self.tools_dock(cx))
            .children(hint)
            .child(overlays)
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .right_0()
                    .w(px(self.inspector_panel.read(cx).width() + 12.0))
                    .child(inspector_overlay),
            );
        if self.source_open {
            layer = layer.child(
                div()
                    .absolute()
                    .top(px(56.0))
                    .bottom(px(64.0))
                    .right(px(PANEL_INSET))
                    .w(px(SOURCE_PANE_WIDTH))
                    .child(self.source_pane(cx)),
            );
        }
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
            .relative()
            .size_full()
            .child(layer)
            .children(alert.map(|alert| div().absolute().top(px(48.0)).left(px(12.0)).child(alert)))
            .children(self.inspector.layers(cx))
    }
}

pub(super) const PANEL_INSET: f32 = 12.0;

/// A floating side panel: inset from the edges with the panel gap, rounded with the card
/// radius token, wearing the card surface and quiet edge.
fn panel_frame(id: &'static str, content: AnyElement, _side: PanelSide, cx: &App) -> AnyElement {
    let frame = card(cx)
        .id(id)
        .occlude()
        .size_full()
        .flex()
        .flex_col()
        .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
        .child(content);

    div()
        .size_full()
        .p(px(PANEL_INSET))
        .child(frame)
        .into_any_element()
}
