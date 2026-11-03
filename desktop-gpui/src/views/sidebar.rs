//! The project sidebar: Now (Inbox with its open count, Activity) and Project (Milestones,
//! Canvas, Structure, Knowledge), with Project settings fixed in the footer.

use gpui::{App, Entity, IntoElement, SharedString, div, prelude::*, px};
use polterware_ui::patterns::{SidebarItem, SidebarSectionLabel};
use polterware_ui::theme::Theme;

use crate::state::{AppState, Load, Section};

pub fn sidebar(state: &Entity<AppState>, cx: &App) -> impl IntoElement {
    let theme = *Theme::global(cx);
    let app = state.read(cx);
    let active = app.section();
    let open_count = match app.inbox() {
        Load::Ready(inbox) => inbox.open_count,
        _ => 0,
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
            row = row.trailing(
                div()
                    .text_size(px(theme.font_size_xs))
                    .text_color(theme.color_text_muted)
                    .child(SharedString::from(open_count.to_string())),
            );
        }
        if section == Section::ProjectSettings {
            row = row.footer();
        }
        row
    };

    let group = |label: &'static str, sections: &[Section]| {
        div()
            .flex()
            .flex_col()
            .gap(px(theme.sidebar_row_gap))
            .child(SidebarSectionLabel::new(
                SharedString::from(format!("sidebar-group-{label}")),
                label,
                false,
            ))
            .children(sections.iter().map(|section| item(*section)))
    };

    div()
        .id("sidebar")
        .w(px(theme.sidebar_width))
        .h_full()
        .flex_none()
        .overflow_hidden()
        .pt(px(theme.space_2))
        .px(px(theme.space_2))
        .pb(px(theme.space_2))
        .flex()
        .flex_col()
        .gap(px(theme.section_gap))
        .child(group("Now", &Section::NOW))
        .child(group("Project", &Section::PROJECT))
        .child(div().flex_1())
        .child(item(Section::ProjectSettings))
}
