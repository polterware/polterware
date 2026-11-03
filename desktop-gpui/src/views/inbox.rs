//! Inbox, the first surface read from the core (`WorkService.ListInbox`): items that need a
//! person, grouped by kind (Conflicts, Deploys, Questions, Setup), open ones before those
//! moved to Later, with the active item's detail on the right. The first item starts
//! active and a row opens another. Deciding, Later, and Dismiss are not ported yet.

use std::rc::Rc;

use gpui::{AnyElement, App, Entity, FontWeight, IntoElement, SharedString, div, prelude::*, px};
use polterware_ui::chip::chip;
use polterware_ui::data_table::{Column, DataTable, primary_cell, sans};
use polterware_ui::icon_button::icon_button;
use polterware_ui::icons::{Icon, icon_colored};
use polterware_ui::patterns::{MascotMood, page_empty_with, toolbar};
use polterware_ui::scroll::page_fill;
use polterware_ui::theme::{StatusTone, Theme, tw};

use crate::client::{Inbox, InboxItem};
use crate::state::{AppState, Load};
use crate::views::core_status::core_status;

/// The detail's width, the documents' properties rail.
const DETAIL_WIDTH: f32 = 376.0;

/// The kinds in the order their groups are listed.
const KINDS: [&str; 4] = ["conflict", "deploy", "decision", "setup"];

pub fn kind_label(kind: &str) -> &'static str {
    match kind {
        "conflict" => "Conflict",
        "deploy" => "Deploy",
        "decision" => "Question",
        "setup" => "Setup",
        _ => "Other",
    }
}

fn kind_icon(kind: &str) -> Icon {
    match kind {
        "conflict" => Icon::Scale,
        "deploy" => Icon::AlertDiamond,
        "decision" => Icon::HelpCircle,
        "setup" => Icon::Plug,
        _ => Icon::Inbox,
    }
}

/// Items by kind group; inside a group, open before Later, each in the core's order.
pub fn ordered(inbox: &Inbox) -> Vec<InboxItem> {
    let rank = |item: &InboxItem| {
        let kind = KINDS
            .iter()
            .position(|kind| *kind == item.kind)
            .unwrap_or(KINDS.len());
        (kind, item.state == "deferred")
    };
    let mut items = inbox.items.clone();
    items.sort_by_key(rank);
    items
}

fn status(item: &InboxItem) -> (&'static str, StatusTone) {
    if item.state == "deferred" {
        ("Later", StatusTone::Muted)
    } else if item.blocking {
        ("Blocking", StatusTone::Danger)
    } else {
        ("Open", StatusTone::Warn)
    }
}

pub fn inbox_page(state: &Entity<AppState>, cx: &App) -> AnyElement {
    if let Some(page) = core_status(state, cx) {
        return page;
    }
    let theme = *Theme::global(cx);
    let app = state.read(cx);
    let reload = state.clone();
    let refresh = icon_button(
        "inbox-refresh",
        Icon::RefreshCw,
        "Reload",
        Rc::new(move |_, cx: &mut App| {
            reload.update(cx, |state, cx| {
                state.load_inbox(cx);
                cx.notify();
            });
        }),
        cx,
    );
    let count = match app.inbox() {
        Load::Ready(inbox) if inbox.open_count > 0 => format!("{} open", inbox.open_count),
        _ => String::new(),
    };
    let title = div()
        .px(px(theme.space_2))
        .flex()
        .items_center()
        .gap(px(theme.space_2))
        .child(
            div()
                .text_size(px(theme.font_size_md))
                .font_weight(FontWeight::MEDIUM)
                .child("Inbox"),
        )
        .child(
            div()
                .text_size(px(theme.font_size_sm))
                .text_color(theme.color_text_muted)
                .child(SharedString::from(count)),
        );
    let bar = toolbar(title, refresh, cx).into_any_element();
    let column = |bar: AnyElement, body: AnyElement| {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(bar)
            .child(body)
            .into_any_element()
    };
    match app.inbox() {
        Load::Idle | Load::Loading => column(
            bar,
            page_empty_with(None, MascotMood::Lost, None, cx).into_any_element(),
        ),
        Load::Failed(message) => column(
            bar,
            page_empty_with(
                Some("Could not read the Inbox"),
                MascotMood::Sad,
                Some(muted(message.clone(), &theme)),
                cx,
            )
            .into_any_element(),
        ),
        Load::Ready(inbox) if inbox.items.is_empty() => column(
            bar,
            page_empty_with(
                Some("Nothing needs you"),
                MascotMood::Sleeping,
                Some(muted(
                    "Conflicts, failed deploys, card questions, and setup land here.".into(),
                    &theme,
                )),
                cx,
            )
            .into_any_element(),
        ),
        Load::Ready(inbox) => {
            let items = ordered(inbox);
            let open = app
                .inbox_open()
                .and_then(|id| items.iter().find(|item| item.id == id))
                .cloned();
            let table = inbox_table(
                state,
                items,
                open.as_ref().map(|item| item.id.clone()),
                &theme,
            );
            let mut row = div()
                .flex_1()
                .min_h(px(0.0))
                .w_full()
                .flex()
                .flex_row()
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .h_full()
                        .flex()
                        .flex_col()
                        .child(page_fill(bar, tw(4.0), table, cx)),
                );
            if let Some(item) = open {
                row = row.child(detail(state, &item, &theme, cx));
            }
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(row)
                .into_any_element()
        }
    }
}

fn muted(text: String, theme: &Theme) -> AnyElement {
    div()
        .text_size(px(theme.font_size_sm))
        .text_color(theme.color_text_muted)
        .child(SharedString::from(text))
        .into_any_element()
}

fn inbox_table(
    state: &Entity<AppState>,
    items: Vec<InboxItem>,
    open: Option<String>,
    theme: &Theme,
) -> DataTable {
    let items = Rc::new(items);
    let columns = vec![
        Column::new("Item").min_width(240.0),
        Column::new("Kind").width(104.0),
        Column::new("Status").content(96.0),
    ];
    let cells = items.clone();
    let font = sans(theme, FontWeight::MEDIUM);
    let activate = state.clone();
    DataTable::new("inbox", columns, items.len(), move |row, _window, cx| {
        let theme = *Theme::global(cx);
        let item = &cells[row.index];
        let active = open.as_deref() == Some(item.id.as_str());
        let (label, tone) = status(item);
        vec![
            primary_cell(
                icon_colored(
                    kind_icon(&item.kind),
                    14.0,
                    if active {
                        theme.color_text_primary
                    } else {
                        theme.color_text_muted
                    },
                ),
                SharedString::from(item.title.clone()),
                font.clone(),
                None,
                cx,
            ),
            div()
                .text_size(px(12.0))
                .text_color(theme.color_text_muted)
                .child(kind_label(&item.kind))
                .into_any_element(),
            chip(label, tone, cx).into_any_element(),
        ]
    })
    .empty("Nothing needs you")
    .on_activate(move |index, _window, cx| {
        let id = items[index].id.clone();
        activate.update(cx, |state, cx| {
            state.open_inbox_item(Some(id));
            cx.notify();
        });
    })
}

fn detail(state: &Entity<AppState>, item: &InboxItem, theme: &Theme, cx: &App) -> AnyElement {
    let close_state = state.clone();
    let close = icon_button(
        "inbox-detail-close",
        Icon::X,
        "Close",
        Rc::new(move |_, cx: &mut App| {
            close_state.update(cx, |state, cx| {
                state.open_inbox_item(None);
                cx.notify();
            });
        }),
        cx,
    );
    let (label, tone) = status(item);
    let property = |name: &str, value: String| {
        div()
            .flex()
            .gap(px(theme.space_3))
            .text_size(px(theme.font_size_sm))
            .child(
                div()
                    .w(px(88.0))
                    .flex_none()
                    .text_color(theme.color_text_muted)
                    .child(SharedString::from(name.to_string())),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .text_color(theme.color_text_secondary)
                    .child(SharedString::from(value)),
            )
    };
    let mut body = div()
        .id("inbox-detail-body")
        .flex_1()
        .min_h(px(0.0))
        .overflow_y_scroll()
        .px(px(theme.space_5))
        .pb(px(theme.space_5))
        .flex()
        .flex_col()
        .gap(px(theme.space_4))
        .child(
            div()
                .text_size(px(15.0))
                .line_height(px(22.0))
                .font_weight(FontWeight::SEMIBOLD)
                .child(SharedString::from(item.title.clone())),
        )
        .child(
            div()
                .flex()
                .gap(px(theme.space_2))
                .child(chip(kind_label(&item.kind), StatusTone::Muted, cx))
                .child(chip(label, tone, cx)),
        );
    if !item.detail.is_empty() {
        body = body.child(
            div()
                .text_size(px(theme.font_size_md))
                .line_height(px(20.0))
                .text_color(theme.color_text_secondary)
                .child(SharedString::from(item.detail.clone())),
        );
    }
    let mut properties = div().flex().flex_col().gap(px(theme.space_2));
    if !item.location.is_empty() {
        properties = properties.child(property("Where", item.location.clone()));
    }
    if !item.source.is_empty() {
        properties = properties.child(property("Source", item.source.clone()));
    }
    for (key, value) in &item.details {
        properties = properties.child(property(&capitalize(key), value.clone()));
    }
    properties = properties.child(property("Id", item.id.clone()));
    body = body.child(properties);
    if !item.options.is_empty() {
        body = body.child(
            div()
                .flex()
                .flex_col()
                .gap(px(theme.space_2))
                .child(
                    div()
                        .text_size(px(theme.font_size_sm))
                        .font_weight(FontWeight::MEDIUM)
                        .child("Options"),
                )
                .children(item.options.iter().enumerate().map(|(index, option)| {
                    div()
                        .flex()
                        .gap(px(theme.space_2))
                        .text_size(px(theme.font_size_sm))
                        .text_color(theme.color_text_secondary)
                        .child(
                            div()
                                .text_color(theme.color_text_muted)
                                .child(SharedString::from(format!("{}.", index + 1))),
                        )
                        .child(SharedString::from(option.clone()))
                })),
        );
    }
    body = body.child(
        div()
            .text_size(px(theme.font_size_xs))
            .text_color(theme.color_text_muted)
            .child("Decide, Later, and Dismiss are not ported yet: use the Electron Desktop or polterware inbox."),
    );
    div()
        .w(px(DETAIL_WIDTH))
        .h_full()
        .flex_none()
        .flex()
        .flex_col()
        .border_l_1()
        .border_color(theme.color_border_subtle)
        .child(
            div()
                .h(px(theme.page_header_height))
                .flex_none()
                .flex()
                .items_center()
                .justify_end()
                .px(px(theme.space_2))
                .child(close),
        )
        .child(body)
        .into_any_element()
}

fn capitalize(key: &str) -> String {
    let mut chars = key.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, kind: &str, state: &str) -> InboxItem {
        InboxItem {
            id: id.into(),
            kind: kind.into(),
            state: state.into(),
            ..Default::default()
        }
    }

    #[test]
    fn groups_by_kind_with_later_last_in_each() {
        let inbox = Inbox {
            items: vec![
                item("s", "setup", "open"),
                item("q2", "decision", "deferred"),
                item("q1", "decision", "open"),
                item("c", "conflict", "open"),
                item("d", "deploy", "open"),
            ],
            ..Default::default()
        };
        let ids: Vec<_> = ordered(&inbox).into_iter().map(|item| item.id).collect();
        assert_eq!(ids, ["c", "d", "q1", "q2", "s"]);
    }
}
