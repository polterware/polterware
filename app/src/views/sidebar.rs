//! The project sidebar (`sidebar.tsx`, `sidebar.css`): Now (Inbox with its open count,
//! Activity) and Project (Milestones, Canvas, Structure, Knowledge), 20px apart, each under a
//! label that collapses it (a collapsed group keeps its open row). Settings (the project's) closes
//! the Project group: it opens the settings dialog on General, and is lit while that dialog shows
//! the project's settings. The Inbox count is the danger badge while something blocks.

use gpui::{App, Entity, IntoElement, div, prelude::*, px};
use polterware_ui::count_badge::{CountBadge, CountTone};
use polterware_ui::icons::Icon;
use polterware_ui::patterns::{SidebarItem, SidebarSectionLabel};
use polterware_ui::theme::Theme;

use crate::state::{AppState, Load, Section, SettingsScope, SettingsTab};

/// `.pw-sidebar-nav`'s gap between its groups.
const GROUP_GAP: f32 = 20.0;
/// The frame's `pt-2` and the nav's 4px, on the 13px root.
const TOP: f32 = 6.5 + 4.0;
pub fn sidebar(state: &Entity<AppState>, cx: &App) -> impl IntoElement + use<> {
    let theme = *Theme::global(cx);
    let app = state.read(cx);
    let active = app.section();
    let (open_count, blocking) = match app.inbox() {
        Load::Ready(inbox) => (inbox.open_count, inbox.blocking_count > 0),
        _ => (0, false),
    };

    let item = |section: Section| {
        let go = state.clone();
        let mut row = SidebarItem::new(("sidebar", section as usize), section.label())
            .icon(section.icon())
            .active(active == Some(section))
            .on_click(move |_, cx| {
                go.update(cx, |state, cx| {
                    state.go_section(section, cx);
                    cx.notify();
                });
            });
        if section == Section::Inbox && open_count > 0 {
            row = row.trailing(CountBadge::new(open_count.max(0) as u64).sidebar().tone(
                if blocking {
                    CountTone::Danger
                } else {
                    CountTone::Neutral
                },
            ));
        }
        row
    };
    let settings_open = app
        .settings_tab()
        .is_some_and(|tab| tab.scope() == SettingsScope::Project);
    let settings_state = state.clone();
    let project_settings = move || {
        SidebarItem::new("sidebar-project-settings", "Settings")
            .icon(Icon::Sliders)
            .active(settings_open)
            .on_click({
                let settings_state = settings_state.clone();
                move |_, cx| {
                    settings_state.update(cx, |state, cx| {
                        state.open_settings(SettingsTab::General, cx);
                        cx.notify();
                    });
                }
            })
    };

    let group = |label: &'static str, sections: &[Section], settings: bool| {
        let collapsed = app.group_collapsed(label);
        let toggle = state.clone();
        div()
            .flex()
            .flex_col()
            .gap(px(theme.sidebar_row_gap))
            .child(
                SidebarSectionLabel::new(
                    gpui::ElementId::Name(format!("sidebar-group-{label}").into()),
                    label,
                    collapsed,
                )
                .quiet_chevron()
                .on_toggle(move |_, cx| {
                    toggle.update(cx, |state, cx| {
                        state.toggle_group(label);
                        cx.notify();
                    });
                }),
            )
            .children(
                sections
                    .iter()
                    .filter(|section| !collapsed || active == Some(**section))
                    .map(|section| item(*section)),
            )
            .when(settings && (!collapsed || settings_open), |group| {
                group.child(project_settings())
            })
    };

    div()
        .id("sidebar")
        .w(px(theme.sidebar_width))
        .h_full()
        .flex_none()
        .overflow_hidden()
        .flex()
        .flex_col()
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .pt(px(TOP))
                .px(px(theme.space_2))
                .pb(px(theme.space_1))
                .flex()
                .flex_col()
                .gap(px(GROUP_GAP))
                .child(group("Now", &Section::NOW, false))
                .child(group("Project", &Section::PROJECT, true)),
        )
}
