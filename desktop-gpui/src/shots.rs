//! `POLTERWARE_GPUI_SHOTS=<dir>`: a tour that captures each screen to a PNG and quits. It shows
//! the Projects screen, opens the first project's Inbox and every other section, then
//! Commander, App settings, and the Projects screen again.

use std::path::{Path, PathBuf};
use std::time::Duration;

use gpui::{App, AsyncWindowContext, Entity, Window};
use polterware_ui::window::capture::save_window_png;

use crate::state::{AppState, Load, Section};

pub fn shots_dir() -> Option<PathBuf> {
    std::env::var_os("POLTERWARE_GPUI_SHOTS").map(PathBuf::from)
}

pub fn start_tour(dir: PathBuf, state: Entity<AppState>, window: &mut Window, cx: &mut App) {
    window
        .spawn(cx, async move |cx| {
            let _ = std::fs::create_dir_all(&dir);
            settle(cx, 2500).await;
            shot(cx, &dir, "01-projects").await;
            let first = cx
                .update(|_, app| match state.read(app).projects() {
                    Load::Ready(projects) => projects.first().map(|project| project.id.clone()),
                    _ => None,
                })
                .ok()
                .flatten();
            if let Some(id) = first {
                update(cx, &state, move |state, cx| state.open_project(id, cx));
                settle(cx, 1500).await;
                shot(cx, &dir, "02-inbox").await;
                for (index, section) in Section::ALL.iter().skip(1).enumerate() {
                    let section = *section;
                    update(cx, &state, move |state, cx| state.go_section(section, cx));
                    shot(cx, &dir, &format!("03-section{index}")).await;
                }
                update(cx, &state, |state, cx| state.go_section(Section::Inbox, cx));
            }
            update(cx, &state, |state, _| state.set_commander(true));
            shot(cx, &dir, "04-commander").await;
            update(cx, &state, |state, _| {
                state.set_commander(false);
                state.open_app_settings();
            });
            shot(cx, &dir, "05-app-settings").await;
            update(cx, &state, |state, _| state.go_home());
            shot(cx, &dir, "06-projects").await;
            let _ = cx.update(|_, app| app.quit());
        })
        .detach();
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
