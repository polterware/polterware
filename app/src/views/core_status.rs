//! What a page shows while the core is not ready: looking for it, not running (with Start the
//! core), or failed.

use gpui::{AnyElement, App, Entity, IntoElement, SharedString, div, prelude::*, px};
use polterware_ui::button::{Button, ButtonKind};
use polterware_ui::icons::Icon;
use polterware_ui::patterns::{MascotMood, page_empty_with};
use polterware_ui::theme::Theme;

use crate::client;
use crate::state::{AppState, CoreState};

/// The page for a core that is not ready, or `None` when it is.
pub fn core_status(state: &Entity<AppState>, cx: &App) -> Option<AnyElement> {
    let theme = *Theme::global(cx);
    let app = state.read(cx);
    let line = |text: String| {
        div()
            .text_size(px(theme.font_size_sm))
            .text_color(theme.color_text_muted)
            .child(SharedString::from(text))
    };
    let page = match app.core() {
        CoreState::Ready(_) => return None,
        CoreState::Connecting => page_empty_with(
            Some("Looking for the local core"),
            MascotMood::Lost,
            None,
            cx,
        ),
        CoreState::Missing { starting } => {
            let starting = *starting;
            let start = state.clone();
            let mut body = div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(theme.space_3))
                .child(line(format!(
                    "No core serves {}.",
                    app.database().display()
                )));
            match client::cli_binary() {
                Some(binary) => {
                    body = body.child(
                        Button::new("core-start", ButtonKind::Primary)
                            .label(if starting {
                                "Starting…"
                            } else {
                                "Start the core"
                            })
                            .icon(Icon::Play)
                            .disabled(starting)
                            .on_click(move |_, cx| {
                                start.update(cx, |state, cx| {
                                    state.start_core(cx);
                                    cx.notify();
                                });
                            }),
                    );
                    body = body.child(line(format!("It runs {} serve.", binary.display())));
                }
                None => {
                    body = body.child(line(
                        "Open the Polterware Desktop, or set POLTERWARE_BIN to a polterware binary."
                            .into(),
                    ));
                }
            }
            page_empty_with(
                Some("The local core is not running"),
                MascotMood::Sleeping,
                Some(body.into_any_element()),
                cx,
            )
        }
        CoreState::Failed(message) => {
            let retry = state.clone();
            let body = div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(theme.space_3))
                .child(line(message.clone()))
                .child(
                    Button::new("core-retry", ButtonKind::Secondary)
                        .label("Try again")
                        .icon(Icon::RefreshCw)
                        .on_click(move |_, cx| {
                            retry.update(cx, |state, cx| {
                                state.connect(cx);
                                cx.notify();
                            });
                        }),
                );
            page_empty_with(
                Some("Could not reach the local core"),
                MascotMood::Sad,
                Some(body.into_any_element()),
                cx,
            )
        }
    };
    Some(
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(page)
            .into_any_element(),
    )
}
