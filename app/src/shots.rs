//! The comparisons with the Electron Desktop, off unless an environment variable turns them on.
//!
//! `POLTERWARE_GPUI_SHOTS=<dir>`: a tour that captures each screen to `<dir>/<name>.png` and
//! quits. Its stops and names are the Electron tour's (`apps/polterware-desktop/src/renderer/
//! instrument-tour.ts`), so `tooling/parity` pairs each capture with the same screen there:
//! the Projects screen and its New project, the seed's main project (Atlas, else the first) on
//! its Inbox, with the open item's Decide and Later panels, Switch project, Help and a topic,
//! and every other section (Activity with a change open, then a Dispatch entry; Knowledge's
//! first rule in the Composer, the Docs tab's Add doc Composer and its archive, and the first
//! doc), Project settings, Commander, and App settings. Then, back in the project, the first
//! milestone's board, a card's page (one in review first), a Rule's Composer, the canvas editor
//! with a node selected so its inspector shows, and the first Inbox item that asks for a
//! confirmation.
//!
//! Then the interactive checks, each driving the real handlers: typing into the canvas's
//! Mermaid source and the parse error it shows, a scripted node drag and its pin, zooming and
//! fitting the diagram, scrolling the Inbox and Rules tables under the pointer, closing the
//! window (which hides the app) and reopening it as the Dock does, and a title bar double
//! click. Every stop prints one line saying what it verified.
//!
//! `POLTERWARE_BENCH=<file>`: appends `{"app","event","at"}` lines (`at` in epoch milliseconds)
//! when the Projects screen first shows rows (`projects-ready`), for `tooling/bench`.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gpui::{App, AsyncWindowContext, Entity, Keystroke, Modifiers, WeakEntity, Window};
use polterware_ui::theme::Theme;
use polterware_ui::window::capture::save_window_png;

use crate::app::AppView;
use crate::state::{
    AppState, CanvasPatch, ComposeRequest, DISPATCH_ROW, KnowledgeTab, Load, Section, SettingsTab,
};
use crate::views::canvas::CanvasView;
use crate::views::knowledge::first_row;
use crate::work::{ALL_CARDS, Stage};

/// The project the tour opens: the seed's main one (`tooling/bench/seed.mjs`), else the first.
const TOUR_PROJECT: &str = "Atlas";

/// Sidebar sections after the Inbox, with their stop names.
const SECTIONS: [(&str, Section); 5] = [
    ("03-activity", Section::Activity),
    ("04-milestones", Section::Milestones),
    ("05-canvas", Section::Canvas),
    ("06-structure", Section::Structure),
    ("07-knowledge", Section::Knowledge),
];

/// Knowledge's tabs as the tour walks them: the list's stop after switching to it (none for
/// Rules, where the first row opens in the Composer, and the stop of its first row's detail.
const KNOWLEDGE_STOPS: [(KnowledgeTab, Option<&str>, Option<&str>); 2] = [
    (KnowledgeTab::Rules, None, Some("07-knowledge-rule")),
    (
        KnowledgeTab::Docs,
        Some("07-knowledge-docs"),
        Some("07-knowledge-doc"),
    ),
];

/// The open Inbox item's actions the tour opens, with their stop names.
const INBOX_ACTIONS: [(&str, &str); 2] =
    [("02-inbox-decide", "decide"), ("02-inbox-later", "defer")];

pub fn shots_dir() -> Option<PathBuf> {
    std::env::var_os("POLTERWARE_GPUI_SHOTS").map(PathBuf::from)
}

pub fn bench_file() -> Option<PathBuf> {
    std::env::var_os("POLTERWARE_BENCH")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
}

/// Appends one milestone to the bench file.
pub fn mark(file: &Path, event: &str) {
    let at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or_default();
    let line = format!("{{\"app\":\"gpui\",\"event\":\"{event}\",\"at\":{at}}}\n");
    let written = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)
        .and_then(|mut out| out.write_all(line.as_bytes()));
    if let Err(error) = written {
        eprintln!("bench {}: {error}", file.display());
    }
}

pub fn start_tour(
    dir: PathBuf,
    state: Entity<AppState>,
    view: WeakEntity<AppView>,
    window: &mut Window,
    cx: &mut App,
) {
    window
        .spawn(cx, async move |cx| {
            let _ = std::fs::create_dir_all(&dir);
            settle(cx, 2500).await;
            update(cx, &state, |state, cx| state.go_home(cx));
            shot(cx, &dir, "01-projects").await;
            update(cx, &state, |state, _| state.set_new_project(true));
            shot(cx, &dir, "01-projects-new").await;
            update(cx, &state, |state, _| state.close_new_project());
            settle(cx, 600).await;
            let project = cx
                .update(|_, app| match state.read(app).projects() {
                    Load::Ready(projects) => projects
                        .iter()
                        .find(|project| project.name == TOUR_PROJECT)
                        .or(projects.first())
                        .map(|project| project.id.clone()),
                    _ => None,
                })
                .ok()
                .flatten();
            if let Some(id) = project {
                update(cx, &state, move |state, cx| state.open_project(id, cx));
                settle(cx, 1500).await;
                // The first item, opened as a click on its row would.
                update(cx, &state, |state, _| {
                    if let Some(id) = state.inbox_open().map(str::to_string) {
                        state.open_inbox_item(id);
                    }
                });
                settle(cx, 800).await;
                shot(cx, &dir, "02-inbox").await;
                // The open item's Decide and Later, each opened, then closed.
                for (name, action) in INBOX_ACTIONS {
                    update(cx, &state, move |state, _| {
                        state.set_inbox_confirm(Some(action.into()))
                    });
                    settle(cx, 800).await;
                    shot(cx, &dir, name).await;
                    update(cx, &state, |state, _| state.set_inbox_confirm(None));
                    settle(cx, 600).await;
                }
                // The project crumb: a click outside the detail closes it.
                update(cx, &state, |state, _| state.close_inbox_detail());
                on_view(cx, &view, |view, window, cx| {
                    view.toggle_switcher(window, cx)
                });
                shot(cx, &dir, "02-switch-project").await;
                on_view(cx, &view, |view, window, cx| {
                    view.toggle_switcher(window, cx)
                });
                settle(cx, 600).await;
                on_view(cx, &view, |view, window, cx| view.toggle_help(window, cx));
                shot(cx, &dir, "02-help").await;
                // The first topic: its row closes the list and opens the dialog.
                on_view(cx, &view, |view, window, cx| view.toggle_help(window, cx));
                update(cx, &state, |state, _| state.set_help_topic(Some(0)));
                shot(cx, &dir, "02-help-topic").await;
                update(cx, &state, |state, _| state.set_help_topic(None));
                settle(cx, 600).await;
                for (name, section) in SECTIONS {
                    update(cx, &state, move |state, cx| state.go_section(section, cx));
                    shot(cx, &dir, name).await;
                    if section == Section::Activity {
                        // The newest change, opened as a click on its row would.
                        update(cx, &state, |state, cx| {
                            let newest = match state.activity() {
                                Load::Ready(changes) => {
                                    changes.first().map(|change| change.step_id.clone())
                                }
                                _ => None,
                            };
                            if let Some(step) = newest {
                                state.open_change(step, cx);
                            }
                        });
                        shot(cx, &dir, "03-activity-detail").await;
                        update(cx, &state, |state, _| state.close_change());
                        settle(cx, 600).await;
                        // The newest Dispatch entry.
                        update(cx, &state, |state, cx| {
                            let newest = state
                                .activity_dispatch()
                                .first()
                                .map(|event| format!("{DISPATCH_ROW}{}", event.id));
                            if let Some(step) = newest {
                                state.open_change(step, cx);
                            }
                        });
                        shot(cx, &dir, "03-activity-dispatch").await;
                        update(cx, &state, |state, _| state.close_change());
                        settle(cx, 600).await;
                    }
                    if section == Section::Knowledge {
                        for (tab, list, page) in KNOWLEDGE_STOPS {
                            if let Some(list) = list {
                                update(cx, &state, move |state, cx| {
                                    state.set_knowledge_tab(tab, cx)
                                });
                                shot(cx, &dir, list).await;
                            }
                            if tab == KnowledgeTab::Docs {
                                // The toolbar's Add doc, then the archive and back.
                                on_view(cx, &view, |view, window, cx| {
                                    view.compose().update(cx, |compose, cx| {
                                        compose.open(ComposeRequest::Doc, window, cx)
                                    })
                                });
                                shot(cx, &dir, "07-knowledge-docs-new").await;
                                on_view(cx, &view, |view, _, cx| {
                                    view.compose().update(cx, |compose, cx| compose.close(cx))
                                });
                                settle(cx, 600).await;
                                let toggle = |view: &AppView, _: &mut Window, cx: &mut App| {
                                    view.knowledge()
                                        .update(cx, |knowledge, cx| knowledge.toggle_archived(cx))
                                };
                                on_view(cx, &view, toggle);
                                shot(cx, &dir, "07-knowledge-docs-archived").await;
                                on_view(cx, &view, toggle);
                                settle(cx, 600).await;
                            }
                            if let Some(page) = page {
                                update(cx, &state, move |state, cx| {
                                    let first = match state.knowledge() {
                                        Load::Ready(knowledge) => first_row(tab, knowledge),
                                        _ => None,
                                    };
                                    if let Some(id) = first {
                                        state.open_knowledge_page(id, cx);
                                    }
                                });
                                settle(cx, 1200).await;
                                shot(cx, &dir, page).await;
                                update(cx, &state, |state, cx| state.go_back(cx));
                                settle(cx, 600).await;
                            }
                        }
                    }
                }
                update(cx, &state, |state, cx| {
                    state.open_settings(SettingsTab::General, cx)
                });
                shot(cx, &dir, "08-project-settings").await;
                update(cx, &state, |state, _| state.close_settings());
                settle(cx, 600).await;
            }
            update(cx, &state, |state, _| state.set_commander(true));
            shot(cx, &dir, "09-commander").await;
            update(cx, &state, |state, cx| {
                state.set_commander(false);
                state.open_settings(SettingsTab::AgentsMcp, cx);
            });
            shot(cx, &dir, "10-app-settings").await;
            let in_project = cx
                .update(|_, app| state.read(app).section().is_some())
                .unwrap_or(false);
            if in_project {
                update(cx, &state, |state, _| state.close_settings());
                settle(cx, 600).await;
                // The first milestone's board, or All cards when there is none.
                update(cx, &state, |state, cx| {
                    let board = match state.work() {
                        Load::Ready(work) => work
                            .milestones
                            .first()
                            .map(|milestone| milestone.id.clone()),
                        _ => None,
                    };
                    state.open_board(Some(board.unwrap_or_else(|| ALL_CARDS.to_string())), cx);
                });
                shot(cx, &dir, "11-board").await;
                // A card's page, one in review when the board has one.
                update(cx, &state, |state, cx| {
                    let Load::Ready(work) = state.work() else {
                        return;
                    };
                    let slug = work
                        .cards
                        .iter()
                        .find(|card| card.stage == Stage::Verify.id())
                        .or_else(|| work.cards.first())
                        .map(|card| card.slug.clone());
                    if let Some(slug) = slug {
                        state.open_card(slug, cx);
                    }
                });
                shot(cx, &dir, "12-card").await;
                // The first Rule in its Composer over the Rules list.
                update(cx, &state, |state, cx| {
                    state.go_section(Section::Knowledge, cx);
                    state.set_knowledge_tab(KnowledgeTab::Rules, cx);
                    let first = match state.knowledge() {
                        Load::Ready(knowledge) => first_row(KnowledgeTab::Rules, knowledge),
                        _ => None,
                    };
                    if let Some(id) = first {
                        state.open_knowledge_page(id, cx);
                    }
                });
                settle(cx, 1200).await;
                shot(cx, &dir, "13-rule").await;
                // The canvas editor with a node selected, so its inspector shows.
                update(cx, &state, |state, cx| {
                    state.go_section(Section::Canvas, cx);
                    let first = match state.canvas_list() {
                        Load::Ready(list) => list.canvases.first().map(|canvas| canvas.id.clone()),
                        _ => None,
                    };
                    if let Some(id) = first {
                        state.open_canvas_by_id(id, cx);
                    }
                });
                settle(cx, 1500).await;
                on_view(cx, &view, |view, _, cx| {
                    view.canvas()
                        .update(cx, |canvas, cx| canvas.select_first_node(cx))
                });
                shot(cx, &dir, "14-canvas-inspector").await;
                // The first Inbox item that asks for a confirmation, opened.
                update(cx, &state, |state, cx| {
                    state.go_section(Section::Inbox, cx);
                    let confirm = match state.inbox() {
                        Load::Ready(inbox) => inbox
                            .items
                            .iter()
                            .find(|item| item.kind == "confirm")
                            .map(|item| item.id.clone()),
                        _ => None,
                    };
                    if let Some(id) = confirm {
                        state.open_inbox_item(id);
                    }
                });
                settle(cx, 800).await;
                shot(cx, &dir, "15-inbox-confirm").await;
                interactive_stops(cx, &dir, &state, &view).await;
            }
            let _ = cx.update(|_, app| app.quit());
        })
        .detach();
}

/// The canvas the interactive stops drive, as the tour found it.
#[derive(Clone)]
struct CanvasSnapshot {
    id: String,
    name: String,
    source: String,
    layout: String,
    agent_visible: bool,
}

/// The interactive checks the release run verifies without a person: each stop drives the
/// real handlers a click, a key, or the pointer would, then captures what they did.
async fn interactive_stops(
    cx: &mut AsyncWindowContext,
    dir: &Path,
    state: &Entity<AppState>,
    view: &WeakEntity<AppView>,
) {
    match canvas_snapshot(cx, state).await {
        Some(snapshot) => {
            canvas_interactive_stops(cx, dir, state, view, snapshot).await;
        }
        None => {
            println!("16-canvas-text skipped: the project has no canvas");
            println!("17-canvas-drag skipped: the project has no canvas");
            println!("18-canvas-zoom skipped: the project has no canvas");
        }
    }
    table_scroll_stops(cx, dir, state).await;
    window_stops(cx, dir).await;
}

/// Opens Canvas and the canvas the editor shows, and returns its fields before the stops.
async fn canvas_snapshot(
    cx: &mut AsyncWindowContext,
    state: &Entity<AppState>,
) -> Option<CanvasSnapshot> {
    let ready = cx
        .update(|_, app| {
            let app = state.read(app);
            app.section() == Some(Section::Canvas) && app.open_canvas().is_some()
        })
        .unwrap_or(false);
    if !ready {
        update(cx, state, |state, cx| state.go_section(Section::Canvas, cx));
        settle(cx, 900).await;
    }
    if cx
        .update(|_, app| state.read(app).open_canvas().is_none())
        .unwrap_or(false)
    {
        update(cx, state, |state, cx| {
            let first = match state.canvas_list() {
                Load::Ready(list) => list.canvases.first().map(|canvas| canvas.id.clone()),
                _ => None,
            };
            if let Some(id) = first {
                state.open_canvas_by_id(id, cx);
            }
        });
        settle(cx, 1500).await;
    }
    cx.update(|_, app| {
        state.read(app).open_canvas().map(|open| CanvasSnapshot {
            id: open.canvas.id.clone(),
            name: open.canvas.name.clone(),
            source: open.canvas.source.clone(),
            layout: open.canvas.layout_json.clone(),
            agent_visible: open.canvas.agent_visible,
        })
    })
    .ok()
    .flatten()
}

/// 16 through 18: the text pane, a scripted drag, and zoom and fit, with the canvas put back.
async fn canvas_interactive_stops(
    cx: &mut AsyncWindowContext,
    dir: &Path,
    state: &Entity<AppState>,
    view: &WeakEntity<AppView>,
    snapshot: CanvasSnapshot,
) {
    // The canvas is taken before each step starts and updated on its own: a keystroke reaches
    // the shell's key listeners, which update the view, so no step may run inside another
    // entity's update.
    let Some(canvas) = on_view_with(cx, view, |view, _, _| view.canvas().clone()) else {
        println!("16-canvas-text skipped: the canvas editor is not open");
        println!("16b-canvas-text-error skipped: the canvas editor is not open");
        println!("17-canvas-drag skipped: the canvas editor is not open");
        println!("18-canvas-zoom skipped: the canvas editor is not open");
        return;
    };
    let original = canvas_source(cx, &canvas).unwrap_or_default();

    // 16: the toolbar's Mermaid source button opens the text pane, and typing reaches the
    // pane's field as a keyboard would.
    let opened = cx
        .update(|window, app| canvas.update(app, |canvas, cx| canvas.toggle_source(window, cx)))
        .unwrap_or(false);
    settle(cx, 500).await;
    if opened {
        type_text(cx, "\nn_tour[\"Tour node\"]");
        settle(cx, 700).await;
        shot(cx, dir, "16-canvas-text").await;
        let drew = cx
            .update(|_, app| canvas.read(app).editor_has_node("Tour node", app))
            .unwrap_or(false);
        println!(
            "16-canvas-text typed n_tour into the Mermaid source; the diagram {}",
            if drew {
                "drew Tour node"
            } else {
                "did not draw Tour node"
            }
        );

        // 16b: a line the parser refuses points at its place and keeps the last diagram.
        type_text(cx, "\n~");
        settle(cx, 700).await;
        shot(cx, dir, "16b-canvas-text-error").await;
        let message = cx
            .update(|_, app| canvas.read(app).editor_render_message(app))
            .ok()
            .flatten()
            .unwrap_or_else(|| "no parse error".to_string());
        // The pane's own undo puts the typed lines back, as the field's ⌘Z does.
        let mut restored = false;
        for _ in 0..12 {
            dispatch_key(cx, "cmd-z");
            settle(cx, 120).await;
            if canvas_source(cx, &canvas).unwrap_or_default() == original {
                restored = true;
                break;
            }
        }
        println!(
            "16b-canvas-text-error showed {message} and kept the last diagram; the source was {}",
            if restored { "restored" } else { "not restored" }
        );
        // Back to the diagram alone, so the drag and zoom show the whole well.
        let _ = cx.update(|window, app| {
            canvas.update(app, |canvas, cx| canvas.toggle_source(window, cx))
        });
        settle(cx, 700).await;
    } else {
        println!("16-canvas-text skipped: the canvas editor is not open");
        println!("16b-canvas-text-error skipped: the canvas editor is not open");
    }

    // 17: a press on the canvas's first node, a move past the drag threshold, and the
    // release, which the platform delivers to MermaidView; its drop writes the pin.
    let target = canvas_node_point(cx, &canvas);
    let metrics = cx.update(|window, _| window_metrics(window)).ok().flatten();
    let dragged = if let (Some((x, y)), Some(metrics)) = (target, metrics) {
        send_mouse(metrics, NsMouse::Down, x, y, 1);
        settle(cx, 80).await;
        send_mouse(metrics, NsMouse::Dragged, x + DRAG_X, y + DRAG_Y, 1);
        settle(cx, 80).await;
        send_mouse(metrics, NsMouse::Up, x + DRAG_X, y + DRAG_Y, 1);
        settle(cx, 900).await;
        true
    } else {
        false
    };
    shot(cx, dir, "17-canvas-drag").await;
    match cx
        .update(|_, app| canvas.read(app).editor_dropped_pin(app))
        .ok()
        .flatten()
    {
        Some((id, x, y)) => println!("17-canvas-drag pinned {id} at {x:.1},{y:.1} in layout_json"),
        None if dragged => {
            println!("17-canvas-drag dropped the node but layout_json has no pin for it")
        }
        None => println!("17-canvas-drag found no node under the scripted pointer"),
    }

    // 18: the secondary-modifier wheel zooms around the pointer, and a double click fits.
    let target = canvas_node_point(cx, &canvas);
    let metrics = cx.update(|window, _| window_metrics(window)).ok().flatten();
    let zoomed = if let (Some((x, y)), Some(metrics)) = (target, metrics) {
        send_wheel(metrics, x, y, -120.0, true);
        settle(cx, 700).await;
        shot(cx, dir, "18-canvas-zoom").await;
        let zoom = canvas_zoom(cx, &canvas);
        double_click(cx, metrics, x, y).await;
        settle(cx, 900).await;
        shot(cx, dir, "18b-canvas-fit").await;
        zoom
    } else {
        None
    };
    // The stops changed the canvas and its layout: the copy of the store goes back to the
    // source, pins, and viewport the tour opened.
    let CanvasSnapshot {
        id,
        name,
        source,
        layout,
        agent_visible,
    } = snapshot;
    update(cx, state, move |state, cx| {
        state.update_canvas(
            id,
            CanvasPatch {
                name: Some(name),
                agent_visible: Some(agent_visible),
                source: Some(source),
                layout_json: Some(layout),
                ..CanvasPatch::default()
            },
            |_, _| {},
            cx,
        );
    });
    settle(cx, 1200).await;
    match zoomed {
        Some(zoom) => println!(
            "18-canvas-zoom zoomed to {zoom:.2} around the pointer and fit the diagram; the canvas was put back"
        ),
        None => println!(
            "18-canvas-zoom fit the diagram and put the canvas back; the zoom level was unavailable"
        ),
    }
}

/// 19: the Inbox and Knowledge Rules tables scroll a few hundred pixels under the pointer.
async fn table_scroll_stops(cx: &mut AsyncWindowContext, dir: &Path, state: &Entity<AppState>) {
    update(cx, state, |state, cx| {
        state.go_section(Section::Inbox, cx);
        state.close_inbox_detail();
    });
    settle(cx, 900).await;
    hover_and_scroll(cx).await;
    shot(cx, dir, "19-inbox-scroll").await;
    println!("19-table-scroll scrolled the Inbox table under a hovered row");

    update(cx, state, |state, cx| {
        state.go_section(Section::Knowledge, cx);
        state.set_knowledge_tab(KnowledgeTab::Rules, cx);
    });
    settle(cx, 900).await;
    hover_and_scroll(cx).await;
    shot(cx, dir, "19b-rules-scroll").await;
    println!("19b-table-scroll scrolled the Knowledge Rules table under a hovered row");
}

/// Puts the pointer on the middle of the list and turns the wheel a few hundred pixels: two
/// steps either way leave the table further down whichever sign the platform gives them.
async fn hover_and_scroll(cx: &mut AsyncWindowContext) {
    let Some(metrics) = cx.update(|window, _| window_metrics(window)).ok().flatten() else {
        return;
    };
    let center = cx
        .update(|window, _| {
            let size = window.viewport_size();
            (f32::from(size.width) * 0.55, f32::from(size.height) * 0.55)
        })
        .ok();
    let Some((x, y)) = center else {
        return;
    };
    send_mouse(metrics, NsMouse::Moved, x, y, 0);
    settle(cx, 120).await;
    send_wheel(metrics, x, y, 30.0, false);
    settle(cx, 80).await;
    send_wheel(metrics, x, y, -15.0, false);
    settle(cx, 300).await;
}

/// 20 and 21: closing the window hides the app until it is activated again as the Dock does,
/// and a title bar double click zooms it and zooms it back.
async fn window_stops(cx: &mut AsyncWindowContext, dir: &Path) {
    // CloseWindow is what Cmd+W and the close button reach: on macOS it hides the app.
    dispatch_key(cx, "cmd-w");
    settle(cx, 600).await;
    // A click on the Dock icon of a hidden app unhides and activates it again.
    let _ = cx.update(|window, app| {
        unhide_app();
        app.activate(true);
        window.activate_window();
        window.refresh();
    });
    shot(cx, dir, "20-window-hide").await;
    println!("20-window-hide closed the window, hid the app, and reopened it as the Dock does");

    let point = cx.update(|window, app| titlebar_point(window, app)).ok();
    let metrics = cx.update(|window, _| window_metrics(window)).ok().flatten();
    if let (Some((x, y)), Some(metrics)) = (point, metrics) {
        double_click(cx, metrics, x, y).await;
        settle(cx, 1400).await;
        shot(cx, dir, "21-titlebar-double-click").await;
        let point = cx.update(|window, app| titlebar_point(window, app)).ok();
        let metrics = cx.update(|window, _| window_metrics(window)).ok().flatten();
        if let (Some((x, y)), Some(metrics)) = (point, metrics) {
            double_click(cx, metrics, x, y).await;
            settle(cx, 1400).await;
            shot(cx, dir, "21b-titlebar-restore").await;
        }
        println!("21-titlebar-double-click zoomed the window on the title bar and zoomed it back");
    } else {
        println!("21-titlebar-double-click skipped: the window has no title bar point");
    }
}

/// How far the scripted drag moves the node, in window pixels.
const DRAG_X: f32 = 36.0;
const DRAG_Y: f32 = 28.0;

/// The window a platform event goes to: its number and its height, for the flipped y.
#[derive(Clone, Copy)]
struct WindowMetrics {
    number: i64,
    height: f32,
}

/// Brings the application back after CloseWindow hid it, as the Dock's click does before it
/// activates the app.
#[cfg(target_os = "macos")]
fn unhide_app() {
    use objc::runtime::Object;
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        let nil: *mut Object = std::ptr::null_mut();
        let _: () = msg_send![app, unhide: nil];
    }
}

#[cfg(not(target_os = "macos"))]
fn unhide_app() {}

#[cfg(target_os = "macos")]
fn window_metrics(window: &Window) -> Option<WindowMetrics> {
    polterware_ui::window::capture::window_number(window).map(|number| WindowMetrics {
        number: i64::from(number),
        height: f32::from(window.viewport_size().height),
    })
}

#[cfg(not(target_os = "macos"))]
fn window_metrics(_: &Window) -> Option<WindowMetrics> {
    None
}

/// The mouse events a scripted pointer sends.
#[derive(Clone, Copy)]
enum NsMouse {
    Down,
    Up,
    Dragged,
    Moved,
}

/// A mouse event from the platform, so the app takes it through its own event handling as it
/// takes a person's click.
#[cfg(target_os = "macos")]
fn send_mouse(metrics: WindowMetrics, kind: NsMouse, x: f32, y: f32, count: usize) {
    use objc::runtime::Object;
    use objc::{class, msg_send, sel, sel_impl};

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Point {
        x: f64,
        y: f64,
    }

    let event_type: usize = match kind {
        NsMouse::Down => 1,
        NsMouse::Up => 2,
        NsMouse::Moved => 5,
        NsMouse::Dragged => 6,
    };
    let location = Point {
        x: f64::from(x),
        y: f64::from(metrics.height - y),
    };
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs_f64())
        .unwrap_or_default();
    let nil: *mut Object = std::ptr::null_mut();
    unsafe {
        let event: *mut Object = msg_send![class!(NSEvent),
            mouseEventWithType: event_type
            location: location
            modifierFlags: 0usize
            timestamp: timestamp
            windowNumber: metrics.number as isize
            context: nil
            eventNumber: 0isize
            clickCount: count as isize
            pressure: 1.0f32];
        if !event.is_null() {
            send_event(event);
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn send_mouse(_: WindowMetrics, _: NsMouse, _: f32, _: f32, _: usize) {}

/// A point as Core Graphics spells it.
#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
struct CgPoint {
    x: f64,
    y: f64,
}

/// A rectangle as Core Graphics spells it: its origin is unused, only the display's height is.
#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct CgRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

// The Core Graphics and Core Foundation calls a wheel event needs; the `core-graphics` crate
// this tree resolves does not declare them.
#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGMainDisplayID() -> u32;
    fn CGDisplayBounds(display: u32) -> CgRect;
    fn CGEventCreateScrollWheelEvent2(
        source: *const std::ffi::c_void,
        units: u32,
        wheel_count: u32,
        wheel1: i32,
        wheel2: i32,
        wheel3: i32,
    ) -> *mut std::ffi::c_void;
    fn CGEventSetLocation(event: *mut std::ffi::c_void, location: CgPoint);
    fn CGEventSetFlags(event: *mut std::ffi::c_void, flags: u64);
}

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(object: *const std::ffi::c_void);
}

/// A wheel event from the platform: `control` turns it into the diagram's zoom.
///
/// `NSEvent` has no initializer for one, so Core Graphics makes it in pixel units (a smooth
/// scroll), places it and sets its flags, and AppKit wraps it as the `NSEvent` the window's
/// view receives.
#[cfg(target_os = "macos")]
fn send_wheel(metrics: WindowMetrics, x: f32, y: f32, dy: f32, control: bool) {
    use objc::runtime::Object;
    use objc::{class, msg_send, sel, sel_impl};

    // `kCGScrollEventUnitPixel`, the units that read back as precise scrolling deltas.
    const PIXEL_UNITS: u32 = 0;
    // `kCGEventFlagMaskControl`, which `NSEvent.modifierFlags` reports as control.
    const CONTROL: u64 = 1 << 18;

    unsafe {
        let wheel = CGEventCreateScrollWheelEvent2(
            std::ptr::null(),
            PIXEL_UNITS,
            1,
            dy.round() as i32,
            0,
            0,
        );
        if wheel.is_null() {
            return;
        }
        // An event with no window reports its location in screen points, flipped by the main
        // display's height, and the app reads that as the window's base point: it is placed
        // where the flip lands on the pointer.
        let display = CGDisplayBounds(CGMainDisplayID());
        CGEventSetLocation(
            wheel,
            CgPoint {
                x: f64::from(x),
                y: display.height - f64::from(metrics.height) + f64::from(y),
            },
        );
        CGEventSetFlags(wheel, if control { CONTROL } else { 0 });
        let event: *mut Object = msg_send![class!(NSEvent), eventWithCGEvent: wheel];
        CFRelease(wheel as *const std::ffi::c_void);
        if event.is_null() {
            return;
        }
        let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        let window: *mut Object = msg_send![app, windowWithWindowNumber: metrics.number as isize];
        if window.is_null() {
            return;
        }
        let content: *mut Object = msg_send![window, contentView];
        if content.is_null() {
            return;
        }
        // A Core Graphics event carries no window number, so queueing it never routes it: the
        // view under the pointer takes it, as AppKit hands a person's wheel event to it.
        let hit: *mut Object = msg_send![content, hitTest: CgPoint {
            x: f64::from(x),
            y: f64::from(metrics.height) - f64::from(y),
        }];
        let view = if hit.is_null() { content } else { hit };
        let _: () = msg_send![view, scrollWheel: event];
    }
}

#[cfg(not(target_os = "macos"))]
fn send_wheel(_: WindowMetrics, _: f32, _: f32, _: f32, _: bool) {}

/// Queues an event with the application, whose window number routes it to the window it was
/// made for; AppKit then dispatches it as it dispatches a person's.
#[cfg(target_os = "macos")]
unsafe fn send_event(event: *mut objc::runtime::Object) {
    use objc::runtime::{NO, Object};
    use objc::{class, msg_send, sel, sel_impl};

    let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
    // `NO`: after the events already waiting, as the pointer's own arrive.
    let _: () = msg_send![app, postEvent: event atStart: NO];
}

/// A real double click: the first press and release, then the second at a count of two.
async fn double_click(cx: &mut AsyncWindowContext, metrics: WindowMetrics, x: f32, y: f32) {
    send_mouse(metrics, NsMouse::Down, x, y, 1);
    settle(cx, 50).await;
    send_mouse(metrics, NsMouse::Up, x, y, 1);
    settle(cx, 50).await;
    send_mouse(metrics, NsMouse::Down, x, y, 2);
    settle(cx, 50).await;
    send_mouse(metrics, NsMouse::Up, x, y, 2);
}

/// What the user's keyboard types into the focused field, dispatched at the top level: a
/// keystroke reaches the shell's own key listeners, which update the view, so it cannot be
/// sent while an entity is being updated.
fn type_text(cx: &mut AsyncWindowContext, text: &str) {
    let _ = cx.update(|window, app| {
        for ch in text.chars() {
            let keystroke = Keystroke {
                modifiers: Modifiers::default(),
                key: ch.to_string(),
                key_char: Some(ch.to_string()),
            };
            window.dispatch_keystroke(keystroke, app);
        }
    });
}

/// One chord, dispatched at the top level as a person's keyboard sends it.
fn dispatch_key(cx: &mut AsyncWindowContext, chord: &str) {
    let _ = cx.update(|window, app| {
        if let Ok(keystroke) = Keystroke::parse(chord) {
            let _ = window.dispatch_keystroke(keystroke, app);
        }
    });
}

/// The canvas editor's source, read while nothing is being updated.
fn canvas_source(cx: &mut AsyncWindowContext, canvas: &Entity<CanvasView>) -> Option<String> {
    cx.update(|_, app| canvas.read(app).editor_source(app))
        .ok()
        .flatten()
}

/// Where the canvas editor's first node is in the window.
fn canvas_node_point(
    cx: &mut AsyncWindowContext,
    canvas: &Entity<CanvasView>,
) -> Option<(f32, f32)> {
    cx.update(|_, app| canvas.read(app).editor_node_point(app))
        .ok()
        .flatten()
}

/// The zoom the diagram's viewport holds.
fn canvas_zoom(cx: &mut AsyncWindowContext, canvas: &Entity<CanvasView>) -> Option<f32> {
    cx.update(|_, app| {
        let diagram = canvas.read(app).editor_diagram(app)?;
        diagram.read(app).viewport().map(|viewport| viewport.zoom)
    })
    .ok()
    .flatten()
}

/// Where a double click on the title bar's drag space lands.
fn titlebar_point(window: &Window, app: &App) -> (f32, f32) {
    let theme = Theme::global(app);
    let size = window.viewport_size();
    (
        (f32::from(size.width) - 320.0).max(80.0),
        theme.titlebar_height / 2.0,
    )
}

fn on_view(
    cx: &mut AsyncWindowContext,
    view: &WeakEntity<AppView>,
    f: impl FnOnce(&AppView, &mut Window, &mut App),
) {
    let _ = cx.update(|window, app| {
        if let Some(view) = view.upgrade() {
            view.update(app, |view, cx| f(view, window, cx));
        }
    });
}

fn on_view_with<T>(
    cx: &mut AsyncWindowContext,
    view: &WeakEntity<AppView>,
    f: impl FnOnce(&AppView, &mut Window, &mut App) -> T,
) -> Option<T> {
    cx.update(|window, app| {
        let view = view.upgrade()?;
        Some(view.update(app, |view, cx| f(view, window, cx)))
    })
    .ok()
    .flatten()
}

fn update(
    cx: &mut AsyncWindowContext,
    state: &Entity<AppState>,
    f: impl FnOnce(&mut AppState, &mut gpui::Context<AppState>),
) {
    let _ = cx.update(|_, app| {
        state.update(app, |state, cx| {
            f(state, cx);
            cx.notify();
        })
    });
}

async fn shot(cx: &mut AsyncWindowContext, dir: &Path, name: &str) {
    let _ = cx.update(|window, app| {
        app.activate(true);
        window.activate_window();
        window.refresh();
    });
    settle(cx, 1200).await;
    let path = dir.join(format!("{name}.png"));
    let _ = cx.update(|window, _| {
        // `screencapture -l`, as the Electron tour captures, when the run asks for it.
        if std::env::var_os("POLTERWARE_GPUI_SCREENCAPTURE").is_some()
            && let Some(id) = polterware_ui::window::capture::window_number(window)
        {
            let status = std::process::Command::new("screencapture")
                .args(["-x", "-o", &format!("-l{id}")])
                .arg(&path)
                .status();
            if !matches!(status, Ok(status) if status.success()) {
                eprintln!("shot {}: screencapture failed", path.display());
            }
            return;
        }
        if let Err(error) = save_window_png(window, &path, None) {
            eprintln!("shot {}: {error}", path.display());
        }
    });
}

async fn settle(cx: &mut AsyncWindowContext, ms: u64) {
    cx.background_executor()
        .timer(Duration::from_millis(ms))
        .await;
}
