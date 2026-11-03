//! The title bar: Home ends on the sidebar's right edge, where the main frame begins, then the
//! breadcrumb (the project, whose crumb returns to the Projects screen, then the section), drag
//! space, Commander, and App settings. The whole bar drags the window.

use std::rc::Rc;

use gpui::{App, Entity, FontWeight, IntoElement, SharedString, div, prelude::*, px};
use polterware_ui::icon_button::icon_button;
use polterware_ui::icons::{Icon, icon_colored};
use polterware_ui::theme::Theme;
use polterware_ui::window::window_drag::WindowDragRegion;

use crate::state::{AppState, Route};

pub fn titlebar(state: &Entity<AppState>, cx: &App) -> impl IntoElement {
    let theme = *Theme::global(cx);
    let app = state.read(cx);

    let home_state = state.clone();
    let home = icon_button(
        "titlebar-home",
        Icon::House,
        "Projects",
        Rc::new(move |_, cx: &mut App| {
            home_state.update(cx, |state, cx| {
                state.go_home();
                cx.notify();
            });
        }),
        cx,
    );

    let mut crumbs: Vec<(SharedString, bool)> = Vec::new();
    match app.route() {
        Route::Projects => crumbs.push(("Projects".into(), false)),
        Route::AppSettings { .. } => crumbs.push(("App settings".into(), false)),
        Route::Project { section, .. } => {
            let name = app
                .project()
                .map(|project| project.name.clone())
                .unwrap_or_else(|| "Project".into());
            crumbs.push((name.into(), true));
            crumbs.push((section.label().into(), false));
        }
    }
    let mut trail = div()
        .flex()
        .items_center()
        .gap(px(theme.space_1))
        .min_w(px(0.0));
    for (index, (label, opens_projects)) in crumbs.into_iter().enumerate() {
        if index > 0 {
            trail = trail.child(icon_colored(
                Icon::ChevronRight,
                12.0,
                theme.color_text_muted,
            ));
        }
        let last = index > 0 || !opens_projects;
        let mut crumb = div()
            .id(("crumb", index))
            .px(px(theme.space_2))
            .h(px(theme.control_height_sm))
            .flex()
            .items_center()
            .rounded(px(theme.radius_control))
            .text_size(px(theme.font_size_sm))
            .font_weight(FontWeight::MEDIUM)
            .text_color(if last {
                theme.color_text_primary
            } else {
                theme.color_text_secondary
            })
            .whitespace_nowrap()
            .overflow_hidden()
            .text_ellipsis()
            .child(label);
        if opens_projects {
            let crumb_state = state.clone();
            crumb = crumb
                .cursor_pointer()
                .hover(|style| style.bg(theme.color_surface_hover))
                .on_click(move |_, _, cx| {
                    crumb_state.update(cx, |state, cx| {
                        state.go_home();
                        cx.notify();
                    });
                });
        }
        trail = trail.child(crumb);
    }

    let commander_state = state.clone();
    let commander = icon_button(
        "titlebar-commander",
        Icon::Search,
        "Commander",
        Rc::new(move |_, cx: &mut App| {
            commander_state.update(cx, |state, cx| {
                let open = state.commander_open();
                state.set_commander(!open);
                cx.notify();
            });
        }),
        cx,
    );
    let settings_state = state.clone();
    let settings = icon_button(
        "titlebar-settings",
        Icon::Settings,
        "App settings",
        Rc::new(move |_, cx: &mut App| {
            settings_state.update(cx, |state, cx| {
                state.open_app_settings();
                cx.notify();
            });
        }),
        cx,
    );

    div()
        .id("titlebar")
        .h(px(theme.titlebar_height))
        .w_full()
        .flex_none()
        .flex()
        .items_center()
        .window_drag_region()
        .child(
            // The traffic lights sit at the left; Home ends on the sidebar's right edge.
            div()
                .w(px(theme.sidebar_width))
                .flex_none()
                .flex()
                .justify_end()
                .pr(px(theme.space_2))
                .child(home),
        )
        .child(div().flex_1().min_w(px(0.0)).flex().child(trail))
        .child(
            div()
                .flex_none()
                .flex()
                .items_center()
                .gap(px(theme.space_1))
                .pr(px(theme.space_3))
                .child(commander)
                .child(settings),
        )
}
