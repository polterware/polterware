//! The surfaces not ported yet: each page names what it will hold, from the Electron Desktop.

use gpui::{AnyElement, App, IntoElement, SharedString, div, prelude::*, px};
use polterware_ui::patterns::{MascotMood, page_empty_with, toolbar};
use polterware_ui::theme::Theme;

use crate::state::Section;

fn holds(section: Section) -> &'static [&'static str] {
    match section {
        Section::Inbox => &[],
        Section::Activity => &[
            "Every justified change, grouped by day",
            "Filters by person, agent, and tier",
            "Dispatch entries read from Polterware Dispatch",
        ],
        Section::Milestones => &[
            "The Roadmap: All cards, then each milestone with its progress",
            "A milestone's Board by stage and its List",
            "A card's page: tasks, gates, verification, commits, and the agent brief",
        ],
        Section::Canvas => &[
            "Every canvas of the project",
            "The editor: shape library, components linked to Structure, and the inspector",
        ],
        Section::Structure => &[
            "The project tree of domains and components",
            "A node's document: kind, technology, checkout, deploys, Rules, and what cites it",
        ],
        Section::Knowledge => &["Decisions with their lifecycle", "Rules", "Docs"],
        Section::ProjectSettings => &[
            "General, with the AGENTS.md sync switch",
            "Environments",
            "Platforms: each provider CLI and its sign-in",
            "Danger zone: archive the project",
        ],
    }
}

pub fn section_page(section: Section, cx: &App) -> AnyElement {
    pending_page(
        &format!("{} is not ported yet", section.label()),
        holds(section),
        cx,
    )
}

pub fn app_settings_page(cx: &App) -> AnyElement {
    pending_page(
        "App settings are not ported yet",
        &[
            "Agents: the MCP server and clients, Agent setup, Hooks, the AI skill, the CLI reference",
            "App: Shortcuts, Appearance, Local core, Privacy, Danger zone",
        ],
        cx,
    )
}

fn pending_page(title: &str, items: &[&str], cx: &App) -> AnyElement {
    let theme = *Theme::global(cx);
    let list = div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(theme.space_1))
        .text_size(px(theme.font_size_sm))
        .text_color(theme.color_text_muted)
        .children(
            items
                .iter()
                .map(|item| div().child(SharedString::from(item.to_string()))),
        );
    div()
        .size_full()
        .flex()
        .flex_col()
        .child(toolbar(div(), div(), cx))
        .child(page_empty_with(
            Some(title),
            MascotMood::Sleeping,
            Some(list.into_any_element()),
            cx,
        ))
        .into_any_element()
}
