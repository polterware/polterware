//! The comparisons with the Electron Desktop, off unless an environment variable turns them on.
//!
//! `POLTERWARE_GPUI_SHOTS=<dir>`: a tour that captures each screen to `<dir>/<name>.png` and
//! quits. Its stops and names are the Electron tour's (`apps/polterware-desktop/src/renderer/
//! instrument-tour.ts`), so `tooling/parity` pairs each capture with the same screen there:
//! the Projects screen and its New project, the seed's main project (Atlas, else the first) on
//! its Inbox, with the open item's Decide and Later panels, Switch project, Help and a topic,
//! and every other section (Activity with a change open, then a Dispatch entry; Knowledge's
//! first decision, its first rule in the Composer, the Docs tab's Add doc Composer and its
//! archive, and the first doc), Project settings, Commander, and App settings.
//!
//! `POLTERWARE_BENCH=<file>`: appends `{"app","event","at"}` lines (`at` in epoch milliseconds)
//! when the Projects screen first shows rows (`projects-ready`), for `tooling/bench`.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gpui::{App, AsyncWindowContext, Entity, WeakEntity, Window};
use polterware_ui::window::capture::save_window_png;

use crate::app::AppView;
use crate::state::{
    AppState, ComposeRequest, DISPATCH_ROW, KnowledgeTab, Load, Section, SettingsTab,
};
use crate::views::knowledge::first_row;

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
/// Decisions, where Knowledge opens) and the stop of its first row's page.
const KNOWLEDGE_STOPS: [(KnowledgeTab, Option<&str>, Option<&str>); 3] = [
    (KnowledgeTab::Decisions, None, Some("07-knowledge-decision")),
    (
        KnowledgeTab::Rules,
        Some("07-knowledge-rules"),
        Some("07-knowledge-rule"),
    ),
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
                shot(cx, &dir, "02-inbox").await;
                // The first item, opened as a click on its row would.
                update(cx, &state, |state, _| {
                    if let Some(id) = state.inbox_open().map(str::to_string) {
                        state.open_inbox_item(id);
                    }
                });
                settle(cx, 600).await;
                // The open item's Decide and Later, each opened, then closed.
                for (name, action) in INBOX_ACTIONS {
                    update(cx, &state, move |state, _| {
                        state.set_inbox_confirm(Some(action.into()))
                    });
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
            let _ = cx.update(|_, app| app.quit());
        })
        .detach();
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
