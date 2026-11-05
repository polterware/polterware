//! Canvas (`CanvasView` in `@polterware/feature-canvas`): the project's canvases as a grid of
//! cards, and the editor for the one open.
//!
//! The 44px toolbar holds Search canvases, the archive button, and New canvas ("+"). The grid
//! is three columns 12px apart, at most 1120px wide and 16px in. A card (`Card
//! variant="texture"`, at least 200px tall, 16px in) is the canvas's miniature filling its face
//! inside the 1px frame, fading into the face under the title row: its 24px tile, the name
//! (14px/500), the Agent-visible eye, and the "…" menu (Open, Rename, Make agent-visible or
//! Hide from agents, Archive). The pointer brightens the miniature (`brightness(1.12)`); a
//! press opens the canvas.
//!
//! New canvas and Rename take the shared Composer with the name as its title. Archive happens
//! at once; the archive button lists the archived canvases (`ArchivedList`), each restored or,
//! once confirmed, deleted for good. With no canvas, the page says so and offers New canvas;
//! a search that matches nothing says No matches.

mod editor;
mod inspector;
mod library;
pub mod scene;

use std::collections::HashMap;
use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, FocusHandle, FontWeight, IntoElement,
    Render, SharedString, Window, div, linear_color_stop, linear_gradient, prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::canvas::{Box2, Viewport, bounds_of, dot_grid};
use polterware_ui::card::texture_card;
use polterware_ui::icon_tile::IconTile;
use polterware_ui::icons::{Icon, icon_colored};
use polterware_ui::menu::MenuItem;
use polterware_ui::patterns::{
    AnchorSlot, ArchivedItem, ArchivedList, ArchivedListEvent, Composer, ComposerDraft,
    ComposerEvent, ComposerSpec, ConfirmDialog, ContextMenu, MascotMood, modal_layer, page_empty,
    page_empty_action, window_layer,
};
use polterware_ui::skeleton::{SkeletonLayout, page_skeleton};
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::Theme;
use polterware_ui::window::window_drag::WindowDragRegion;

use crate::canvas::document::to_document;
use crate::canvas::references::StoreEntity;
use crate::contracts::Canvas;
use crate::format::archive_date;
use crate::state::{AppState, CanvasPatch, Load};
use editor::CanvasEditor;
use scene::{NodeDecor, edges_view, layers, node_view};

/// The grid's most width, its padding, and the gap between cards.
const GRID_MAX: f32 = 1120.0;
const GRID_PAD: f32 = 16.0;
const GRID_GAP: f32 = 12.0;
const CARD_MIN_HEIGHT: f32 = 200.0;
/// The miniature's margin around the drawing, in document units.
const MINI_MARGIN: f32 = 48.0;
/// The room the title row takes at the card's foot: 16px in, a 24px row, 8px above it.
const MINI_RESERVE: f32 = 48.0;

/// What the shared Composer is writing.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Dialog {
    Create,
    Rename { id: String, name: String },
}

pub struct CanvasView {
    state: Entity<AppState>,
    search: Entity<TextInput>,
    query: String,
    show_archived: bool,
    archived: Entity<ArchivedList>,
    composer: Entity<Composer>,
    dialog: Option<Dialog>,
    menu: Entity<ContextMenu>,
    /// The archived canvas waiting for Delete permanently: its id and name.
    delete_ask: Option<(String, String)>,
    delete_focus: FocusHandle,
    delete_focus_due: bool,
    /// Each card's bounds, which its miniature fits.
    cards: HashMap<String, AnchorSlot>,
    editor: Option<(u64, Entity<CanvasEditor>)>,
}

impl CanvasView {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| TextInput::new(cx).placeholder("Search canvases").pill());
        cx.observe(&search, |this, search, cx| {
            this.query = search.read(cx).text().to_string();
            cx.notify();
        })
        .detach();
        let archived = cx.new(|cx| ArchivedList::new("canvas-archived", Icon::SchemeIcon, cx));
        cx.subscribe(&archived, Self::archive_picked).detach();
        let composer = cx.new(Composer::new);
        cx.subscribe(&composer, Self::composed).detach();
        let menu = cx.new(ContextMenu::new);
        cx.observe(&menu, |_, _, cx| cx.notify()).detach();
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        Self {
            state,
            search,
            query: String::new(),
            show_archived: false,
            archived,
            composer,
            dialog: None,
            menu,
            delete_ask: None,
            delete_focus: cx.focus_handle(),
            delete_focus_due: false,
            cards: HashMap::new(),
            editor: None,
        }
    }

    fn toggle_archived(&mut self, cx: &mut Context<Self>) {
        self.show_archived = !self.show_archived;
        if self.show_archived {
            self.state
                .update(cx, |state, cx| state.load_archived_canvases(cx));
        }
        cx.notify();
    }

    fn archive_picked(
        &mut self,
        _: Entity<ArchivedList>,
        event: &ArchivedListEvent,
        cx: &mut Context<Self>,
    ) {
        let (id, delete) = match event {
            ArchivedListEvent::Restore(id) => (id.to_string(), false),
            ArchivedListEvent::Delete(id) => (id.to_string(), true),
        };
        let Load::Ready(canvases) = self.state.read(cx).archived_canvases() else {
            return;
        };
        let Some(name) = canvases
            .iter()
            .find(|canvas| canvas.id == id)
            .map(|canvas| canvas.name.clone())
        else {
            return;
        };
        if delete {
            self.delete_ask = Some((id, name));
            self.delete_focus_due = true;
        } else {
            self.state
                .update(cx, |state, cx| state.restore_canvas_by_id(id, name, cx));
        }
        cx.notify();
    }

    fn open_dialog(&mut self, dialog: Dialog, window: &mut Window, cx: &mut Context<Self>) {
        let spec =
            match &dialog {
                Dialog::Create => ComposerSpec::new("Canvas name", "Create canvas"),
                Dialog::Rename { name, .. } => ComposerSpec::new("Canvas name", "Save")
                    .edit()
                    .initial(ComposerDraft {
                        title: name.clone(),
                        ..ComposerDraft::default()
                    }),
            };
        self.dialog = Some(dialog);
        self.composer.update(cx, |composer, cx| {
            composer.open(spec, Vec::new(), window, cx)
        });
    }

    fn composed(
        &mut self,
        composer: Entity<Composer>,
        event: &ComposerEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            ComposerEvent::Submit(draft) => {
                let name = draft.title.trim().to_string();
                let done = {
                    let composer = composer.downgrade();
                    move |written: bool, cx: &mut App| {
                        let _ = composer.update(cx, |composer, cx| composer.finish(written, cx));
                    }
                };
                if name.is_empty() {
                    done(false, cx);
                    return;
                }
                match self.dialog.clone() {
                    Some(Dialog::Create) => self
                        .state
                        .update(cx, |state, cx| state.create_canvas(name, done, cx)),
                    Some(Dialog::Rename { id, .. }) => self.state.update(cx, |state, cx| {
                        state.update_canvas(
                            id,
                            CanvasPatch {
                                name: Some(name),
                                ..CanvasPatch::default()
                            },
                            done,
                            cx,
                        )
                    }),
                    None => done(false, cx),
                }
            }
            ComposerEvent::Closed => self.dialog = None,
            _ => {}
        }
    }

    fn open_card_menu(&mut self, canvas: &Canvas, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.entity().downgrade();
        let state = self.state.clone();
        let (id, name, visible) = (canvas.id.clone(), canvas.name.clone(), canvas.agent_visible);
        let open = {
            let (state, id) = (state.clone(), id.clone());
            Rc::new(move |_: &mut Window, cx: &mut App| {
                let id = id.clone();
                state.update(cx, |state, cx| state.open_canvas_by_id(id, cx));
            })
        };
        let rename = {
            let (view, id, name) = (view.clone(), id.clone(), name.clone());
            Rc::new(move |window: &mut Window, cx: &mut App| {
                let dialog = Dialog::Rename {
                    id: id.clone(),
                    name: name.clone(),
                };
                let _ = view.update(cx, |this, cx| this.open_dialog(dialog, window, cx));
            })
        };
        let agents = {
            let (state, id) = (state.clone(), id.clone());
            Rc::new(move |_: &mut Window, cx: &mut App| {
                let id = id.clone();
                state.update(cx, |state, cx| {
                    state.update_canvas(
                        id,
                        CanvasPatch {
                            agent_visible: Some(!visible),
                            ..CanvasPatch::default()
                        },
                        |_, _| {},
                        cx,
                    )
                });
            })
        };
        let archive = {
            let (state, id, name) = (state.clone(), id.clone(), name.clone());
            Rc::new(move |_: &mut Window, cx: &mut App| {
                let (id, name) = (id.clone(), name.clone());
                state.update(cx, |state, cx| state.archive_canvas(id, name, cx));
            })
        };
        let items = vec![
            MenuItem::action("Open", Some(Icon::SchemeIcon), open),
            MenuItem::action("Rename", Some(Icon::Pencil), rename),
            MenuItem::action(
                if visible {
                    "Hide from agents"
                } else {
                    "Make agent-visible"
                },
                Some(if visible { Icon::PinOff } else { Icon::Eye }),
                agents,
            ),
            MenuItem::action("Archive", Some(Icon::Archive), archive),
        ];
        let at = window.mouse_position();
        self.menu.update(cx, |menu, cx| {
            menu.set_width(160.0);
            menu.open(at, items, window, cx)
        });
    }

    fn toolbar(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let busy = self.state.read(cx).canvas_busy();
        let has_types = matches!(self.state.read(cx).canvas_list(), Load::Ready(list) if !list.types.is_empty());
        let view = cx.entity().downgrade();
        let actions = div()
            .flex_none()
            .flex()
            .items_center()
            .gap(px(8.0))
            .when(!self.show_archived, |actions| {
                actions.child(div().w(px(200.0)).child(self.search.clone()))
            })
            .child(
                Button::new("canvas-archived-toggle", ButtonKind::Secondary)
                    .size(ButtonSize::Icon)
                    .icon(Icon::Archive)
                    .selected(self.show_archived)
                    .tooltip(if self.show_archived {
                        "Back to canvases"
                    } else {
                        "Archived canvases"
                    })
                    .on_click({
                        let view = view.clone();
                        move |_, cx| {
                            let _ = view.update(cx, |this, cx| this.toggle_archived(cx));
                        }
                    }),
            )
            .when(!self.show_archived, |actions| {
                actions.child(
                    Button::new("canvas-new", ButtonKind::Primary)
                        .size(ButtonSize::Icon)
                        .icon(Icon::Plus)
                        .tooltip("New canvas")
                        .disabled(busy || !has_types)
                        .on_click(move |window, cx| {
                            let _ = view.update(cx, |this, cx| {
                                this.open_dialog(Dialog::Create, window, cx)
                            });
                        }),
                )
            });
        div()
            .flex_none()
            .h(px(theme.page_header_height))
            .pt(px(theme.space_2))
            .px(px(theme.space_2))
            .pb(px(theme.space_1))
            .flex()
            .items_center()
            .gap(px(theme.space_3))
            .child(div().flex_1().h_full().window_drag_region())
            .child(actions)
            .into_any_element()
    }

    fn archived_page(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match self.state.read(cx).archived_canvases().clone() {
            Load::Ready(canvases) => {
                let items = canvases
                    .iter()
                    .map(|canvas| ArchivedItem {
                        id: canvas.id.clone().into(),
                        title: canvas.name.clone().into(),
                        detail: "Canvas".into(),
                        archived_at: archive_date(&canvas.archived_at).into(),
                    })
                    .collect();
                let busy = self.state.read(cx).canvas_busy();
                self.archived.update(cx, |list, cx| {
                    list.set_items(items, cx);
                    list.set_busy(busy, cx);
                });
                div()
                    .min_h_0()
                    .flex_1()
                    .child(self.archived.clone())
                    .into_any_element()
            }
            Load::Failed(error) => page_empty(&error, MascotMood::Sad, cx).into_any_element(),
            _ => {
                let _ = window;
                page_skeleton(SkeletonLayout::Table)
                    .without_toolbar()
                    .into_any_element()
            }
        }
    }

    fn grid(&mut self, canvases: Vec<Canvas>, cx: &mut Context<Self>) -> AnyElement {
        let theme = *Theme::global(cx);
        let state = self.state.read(cx);
        let entities = state.canvas_entities().to_vec();
        let types = match state.canvas_list() {
            Load::Ready(list) => list.types.clone(),
            _ => Vec::new(),
        };
        let cards: Vec<AnyElement> = canvases
            .iter()
            .map(|canvas| {
                let known = types
                    .iter()
                    .any(|definition| definition.r#type == canvas.r#type);
                let slot = self.cards.entry(canvas.id.clone()).or_default().clone();
                self.card(canvas, known, &entities, slot, cx)
            })
            .collect();
        let mut rows = Vec::new();
        let mut cards = cards.into_iter().peekable();
        while cards.peek().is_some() {
            let mut row: Vec<AnyElement> = cards.by_ref().take(3).collect();
            while row.len() < 3 {
                row.push(div().flex_1().into_any_element());
            }
            rows.push(
                div().flex().gap(px(GRID_GAP)).children(
                    row.into_iter()
                        .map(|card| div().flex_1().min_w_0().child(card)),
                ),
            );
        }
        div()
            .id("canvas-grid")
            .min_h_0()
            .flex_1()
            .overflow_y_scroll()
            .child(
                div()
                    .w_full()
                    .max_w(px(GRID_MAX))
                    .p(px(GRID_PAD))
                    .flex()
                    .flex_col()
                    .gap(px(GRID_GAP))
                    .children(rows),
            )
            .text_color(theme.color_text_primary)
            .into_any_element()
    }

    fn card(
        &self,
        canvas: &Canvas,
        known: bool,
        entities: &[StoreEntity],
        slot: AnchorSlot,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = *Theme::global(cx);
        let id = canvas.id.clone();
        let group: SharedString = format!("canvas-card-{id}").into();
        let size = slot.get().map(|bounds| {
            (
                f32::from(bounds.size.width) - 2.0,
                f32::from(bounds.size.height) - 2.0,
            )
        });
        let preview = div()
            .absolute()
            .top(px(1.0))
            .left(px(1.0))
            .right(px(1.0))
            .bottom(px(1.0))
            .rounded(px(theme.radius_card - 2.0))
            .overflow_hidden()
            .child(miniature(canvas, known, entities, size, &theme))
            // The miniature fades into the face under the title row.
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .h(px(96.0))
                    .bg(fade_edge(&theme)),
            )
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .invisible()
                    .group_hover(group.clone(), |veil| veil.visible())
                    .bg(gpui::Rgba {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.05,
                    }),
            );
        let open_state = self.state.clone();
        let open_id = id.clone();
        let menu_view = cx.entity().downgrade();
        let menu_canvas = canvas.clone();
        let busy = self.state.read(cx).canvas_busy();
        let head = div()
            .relative()
            .mt_auto()
            .flex()
            .min_w_0()
            .items_center()
            .gap(px(8.0))
            .child(IconTile::new(Icon::SchemeIcon).size(24.0, 14.0))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(px(14.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.color_text_primary)
                    .child(canvas.name.clone()),
            )
            .when(canvas.agent_visible, |head| {
                head.child(icon_colored(Icon::Eye, 14.0, theme.color_text_muted))
            })
            .child(
                div().flex_none().my(px(-6.0)).child(
                    Button::new(
                        SharedString::from(format!("canvas-card-menu-{id}")),
                        ButtonKind::Secondary,
                    )
                    .size(ButtonSize::Icon)
                    .icon(Icon::MoreHorizontal)
                    .icon_size(14.0)
                    .disabled(busy)
                    .on_click(move |window, cx| {
                        let canvas = menu_canvas.clone();
                        let _ = menu_view
                            .update(cx, |this, cx| this.open_card_menu(&canvas, window, cx));
                    }),
                ),
            );
        texture_card(cx)
            .id(SharedString::from(format!("canvas-card-{id}")))
            .group(group)
            .flex()
            .flex_col()
            .gap(px(4.0))
            .min_h(px(CARD_MIN_HEIGHT))
            .h_full()
            .p(px(16.0))
            .overflow_hidden()
            .cursor_pointer()
            .child(slot.probe())
            .child(preview)
            .child(head)
            .on_click(move |_, _, cx| {
                let id = open_id.clone();
                open_state.update(cx, |state, cx| state.open_canvas_by_id(id, cx));
            })
            .into_any_element()
    }

    fn delete_confirm(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let (id, title) = self.delete_ask.clone()?;
        if std::mem::take(&mut self.delete_focus_due) {
            self.delete_focus.focus(window);
        }
        let theme = *Theme::global(cx);
        let view = cx.entity().downgrade();
        let close: polterware_ui::Handler = {
            let view = view.clone();
            Rc::new(move |_, cx: &mut App| {
                let _ = view.update(cx, |this, cx| {
                    this.delete_ask = None;
                    cx.notify();
                });
            })
        };
        let dialog = ConfirmDialog::new(
            "canvas-delete",
            format!("Delete \u{201c}{title}\u{201d} permanently?"),
            "The canvas, its shapes, and its layout are removed from this project.",
        )
        .recovery("This cannot be undone.")
        .confirm_label("Delete permanently")
        .focus(&self.delete_focus)
        .on_cancel(close.clone())
        .on_confirm(Rc::new(move |_, cx: &mut App| {
            let _ = view.update(cx, |this, cx| {
                this.delete_ask = None;
                let (id, title) = (id.clone(), title.clone());
                this.state
                    .update(cx, |state, cx| state.delete_canvas(id, title, cx));
                cx.notify();
            });
        }));
        Some(window_layer(modal_layer(1.0, true, close, dialog, &theme)))
    }

    /// Keeps the editor on the canvas the state has open.
    fn follow_open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let open = self.state.read(cx).open_canvas().cloned();
        match (open, &self.editor) {
            (None, Some((_, editor))) => {
                editor.update(cx, |editor, cx| editor.save_on_close(cx));
                self.editor = None;
            }
            (Some(open), current)
                if current.as_ref().map(|(generation, _)| *generation) != Some(open.generation) =>
            {
                if let Some((_, editor)) = current {
                    editor.update(cx, |editor, cx| editor.save_on_close(cx));
                }
                let state = self.state.clone();
                let generation = open.generation;
                let editor = cx.new(|cx| CanvasEditor::new(state, open, window, cx));
                self.editor = Some((generation, editor));
            }
            _ => {}
        }
    }
}

/// `--material-fade-edge-top`: the face at the bottom, half of it solid, fading out upward.
fn fade_edge(theme: &Theme) -> gpui::Background {
    let surface = theme.color_surface;
    let at = |alpha: f32| gpui::Rgba {
        a: surface.a * alpha,
        ..surface
    };
    // GPUI's gradients take two stops; the curve is held by its solid half and its fade.
    let _ = at(0.45);
    linear_gradient(
        0.0,
        linear_color_stop(surface, 0.48),
        linear_color_stop(at(0.0), 1.0),
    )
}

/// A read-only miniature of a canvas (`CanvasMinimap`): its shapes, connections, and dotted
/// grid, fitted to the card above the title row's reserve, on the app background.
fn miniature(
    canvas: &Canvas,
    known: bool,
    entities: &[StoreEntity],
    size: Option<(f32, f32)>,
    theme: &Theme,
) -> AnyElement {
    let doc = known
        .then(|| to_document(canvas))
        .filter(|doc| !doc.nodes.is_empty());
    let (min_x, min_y, max_x, max_y) = match &doc {
        Some(doc) => bounds_of(doc.nodes.iter().map(|node| node.rect())),
        None => (
            MINI_MARGIN,
            MINI_MARGIN,
            320.0 - MINI_MARGIN,
            160.0 - MINI_MARGIN,
        ),
    };
    let fit = Box2::new(
        min_x - MINI_MARGIN,
        min_y - MINI_MARGIN,
        max_x - min_x + MINI_MARGIN * 2.0,
        max_y - min_y + MINI_MARGIN * 2.0,
    );
    let (width, height) = size.unwrap_or((320.0, CARD_MIN_HEIGHT));
    let free = height - MINI_RESERVE;
    let viewport = if width > 0.0 && free > 0.0 {
        let scale = (width / fit.width).min(free / fit.height);
        let x = fit.x - (width / scale - fit.width) / 2.0;
        let y = fit.y - (free / scale - fit.height) / 2.0;
        Viewport {
            x: -x * scale,
            y: -y * scale,
            zoom: scale,
        }
    } else {
        Viewport::default()
    };
    let mut layer = div()
        .absolute()
        .inset_0()
        .bg(theme.color_app_background)
        .child(dot_grid(
            viewport,
            viewport.zoom,
            theme.color_border_default,
        ));
    if let Some(doc) = &doc {
        let (back, front) = layers(doc);
        let draw = |node: &crate::canvas::document::Node| {
            node_view(
                node,
                entities,
                viewport,
                NodeDecor {
                    selected: false,
                    decorations: false,
                    editing: None,
                },
                theme,
            )
        };
        layer = layer
            .children(back.into_iter().map(draw))
            .child(edges_view(doc, viewport, |_| theme.color_text_muted))
            .children(front.into_iter().map(draw));
    }
    layer.into_any_element()
}

impl Render for CanvasView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.follow_open(window, cx);
        if let Some((_, editor)) = &self.editor {
            return div().size_full().child(editor.clone()).into_any_element();
        }
        let toolbar = self.toolbar(cx);
        let body = if self.show_archived {
            self.archived_page(window, cx)
        } else {
            match self.state.read(cx).canvas_list().clone() {
                Load::Ready(list) if list.types.is_empty() => {
                    page_empty("Canvas is unavailable", MascotMood::Sad, cx).into_any_element()
                }
                Load::Ready(list) => {
                    let needle = self.query.trim().to_lowercase();
                    let visible: Vec<Canvas> = list
                        .canvases
                        .into_iter()
                        .filter(|canvas| {
                            needle.is_empty() || canvas.name.to_lowercase().contains(&needle)
                        })
                        .collect();
                    if visible.is_empty() {
                        if needle.is_empty() {
                            let view = cx.entity().downgrade();
                            page_empty_action(
                                Some("No canvases yet"),
                                MascotMood::Sleeping,
                                Button::new("canvas-empty-new", ButtonKind::Secondary)
                                    .icon(Icon::Plus)
                                    .label("New canvas")
                                    .on_click(move |window, cx| {
                                        let _ = view.update(cx, |this, cx| {
                                            this.open_dialog(Dialog::Create, window, cx)
                                        });
                                    })
                                    .into_any_element(),
                                cx,
                            )
                            .into_any_element()
                        } else {
                            page_empty("No matches", MascotMood::Lost, cx).into_any_element()
                        }
                    } else {
                        self.grid(visible, cx)
                    }
                }
                Load::Failed(error) => page_empty(&error, MascotMood::Sad, cx).into_any_element(),
                _ => page_skeleton(SkeletonLayout::Table)
                    .without_toolbar()
                    .into_any_element(),
            }
        };
        let confirm = self.delete_confirm(window, cx);
        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .child(toolbar)
            .child(body)
            .child(self.composer.clone())
            .child(self.menu.clone())
            .children(confirm)
            .into_any_element()
    }
}
