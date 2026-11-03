//! The Projects screen, reached through Home: every local project in one table. A row
//! enters the project on its Inbox.

use std::rc::Rc;

use gpui::{
    AnyElement, App, Entity, FontWeight, IntoElement, Rgba, SharedString, div, prelude::*, px,
};
use polterware_ui::data_table::{Column, DataTable, primary_cell, sans};
use polterware_ui::icon_button::icon_button;
use polterware_ui::icons::{Icon, icon_colored};
use polterware_ui::patterns::{MascotMood, page_empty_with, toolbar};
use polterware_ui::scroll::page_fill;
use polterware_ui::theme::{Theme, tw};

use crate::client::Project;
use crate::state::{AppState, Load};
use crate::views::core_status::core_status;

pub fn projects_page(state: &Entity<AppState>, cx: &App) -> AnyElement {
    if let Some(page) = core_status(state, cx) {
        return page;
    }
    let theme = *Theme::global(cx);
    let reload = state.clone();
    let refresh = icon_button(
        "projects-refresh",
        Icon::RefreshCw,
        "Reload",
        Rc::new(move |_, cx: &mut App| {
            reload.update(cx, |state, cx| {
                state.reload(cx);
                cx.notify();
            });
        }),
        cx,
    );
    let title = div()
        .px(px(theme.space_2))
        .text_size(px(theme.font_size_md))
        .font_weight(FontWeight::MEDIUM)
        .child("All projects");
    let bar = toolbar(title, refresh, cx);
    let page = |body: AnyElement| {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(body)
            .into_any_element()
    };
    match state.read(cx).projects() {
        Load::Idle | Load::Loading => page(
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(bar)
                .child(page_empty_with(None, MascotMood::Lost, None, cx))
                .into_any_element(),
        ),
        Load::Failed(message) => page(
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(bar)
                .child(page_empty_with(
                    Some("Could not read the projects"),
                    MascotMood::Sad,
                    Some(muted(message.clone(), &theme)),
                    cx,
                ))
                .into_any_element(),
        ),
        Load::Ready(projects) if projects.is_empty() => page(
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(bar)
                .child(page_empty_with(
                    Some("No projects yet"),
                    MascotMood::Sleeping,
                    Some(muted(
                        "Create one with polterware init in a repository, or in the Electron Desktop."
                            .into(),
                        &theme,
                    )),
                    cx,
                ))
                .into_any_element(),
        ),
        Load::Ready(projects) => {
            let table = projects_table(state, projects.clone(), &theme);
            page(page_fill(bar, tw(4.0), table, cx))
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

fn text(value: impl Into<SharedString>, ink: Rgba) -> AnyElement {
    div()
        .text_size(px(12.0))
        .font_weight(FontWeight::MEDIUM)
        .text_color(ink)
        .whitespace_nowrap()
        .child(value.into())
        .into_any_element()
}

fn projects_table(state: &Entity<AppState>, projects: Vec<Project>, theme: &Theme) -> DataTable {
    let projects = Rc::new(projects);
    let columns = vec![
        Column::new("Name").min_width(240.0),
        Column::new("Status").width(120.0),
        Column::new("Updated").width(140.0),
    ];
    let cells = projects.clone();
    let font = sans(theme, FontWeight::MEDIUM);
    let open = state.clone();
    DataTable::new(
        "projects",
        columns,
        projects.len(),
        move |row, _window, cx| {
            let theme = *Theme::global(cx);
            let project = &cells[row.index];
            vec![
                primary_cell(
                    icon_colored(Icon::Folder, 16.0, theme.color_text_muted),
                    SharedString::from(project.name.clone()),
                    font.clone(),
                    None,
                    cx,
                ),
                text(
                    if project.status.is_empty() {
                        "active".to_string()
                    } else {
                        project.status.clone()
                    },
                    theme.color_text_muted,
                ),
                text(day(&project.updated_at), theme.color_text_muted),
            ]
        },
    )
    .row_height(48.0)
    .empty("No projects yet")
    .on_activate(move |index, _window, cx| {
        let id = projects[index].id.clone();
        open.update(cx, |state, cx| {
            state.open_project(id, cx);
            cx.notify();
        });
    })
}

/// The date of an RFC 3339 timestamp.
fn day(timestamp: &str) -> String {
    timestamp.get(..10).unwrap_or(timestamp).to_string()
}
