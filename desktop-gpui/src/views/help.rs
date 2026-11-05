//! Help (`help-menu.tsx`, `help-topics.ts`, `help-topic-dialog.tsx`, `help-topic-body.tsx`): the
//! sidebar footer's "?" opens a 260px list of the topics, over the button, and a topic opens
//! in a dialog: its icon tile before the title, then its rows, each a 14px icon beside 13.5px
//! secondary text on a relaxed line, 20px apart; a topic with a link ends on a button that
//! opens it.

use std::rc::Rc;

use gpui::{AnyElement, App, Entity, IntoElement, SharedString, div, prelude::*, px};
use polterware_ui::icons::{Icon, icon_colored};
use polterware_ui::list_popover::ListRow;
use polterware_ui::patterns::dialog::{DialogSize, dialog};
use polterware_ui::theme::{Theme, tw};
use polterware_ui::{Button, ButtonKind, ButtonSize, IconTile, TileTone};

use crate::state::AppState;

/// `HELP_GITHUB_URL`.
const GITHUB_URL: &str = "https://github.com/polterware/polterware";

pub struct HelpTopic {
    pub label: &'static str,
    pub title: &'static str,
    pub icon: Icon,
    pub body: &'static [(Icon, &'static str)],
    pub link: Option<(&'static str, &'static str)>,
}

/// `HELP_TOPICS`, in their order.
pub const HELP_TOPICS: [HelpTopic; 4] = [
    HelpTopic {
        label: "What it's for",
        title: "What Polterware is for",
        icon: Icon::Bot,
        body: &[
            (
                Icon::Bot,
                "Polterware is Project Context & Change Intelligence for AI Coding. It gives every coding agent the same understanding of your project, with the Rules they must follow, and shows on each card the commits that delivered it.",
            ),
            (
                Icon::Layers,
                "Architecture and Knowledge are ways for people to inspect the project model. They are not the product category \u{2014} the durable value is verifiable context for agents.",
            ),
        ],
        link: None,
    },
    HelpTopic {
        label: "Agents & context",
        title: "Agents & context",
        icon: Icon::FingerAccessIcon,
        body: &[
            (
                Icon::FingerAccessIcon,
                "Under Agents \u{2192} Access, connect coding agents through MCP so they read the same project model people see in the Desktop.",
            ),
            (
                Icon::FileText,
                "Decisions, Rules, and Docs live under Knowledge. Agents read the same model through the CLI and MCP \u{2014} humans and agents work from one model, not a hidden agent-only view.",
            ),
        ],
        link: None,
    },
    HelpTopic {
        label: "Commits & history",
        title: "Commits & history",
        icon: Icon::GitCompare,
        body: &[
            (
                Icon::GitCompare,
                "A card shows the commits that delivered its tasks: the ones an agent linked, and the ones whose subject cites a task as slug#T2, with a link to GitHub when the repository is there.",
            ),
            (
                Icon::History,
                "Agents \u{2192} History is a unified trail of justified changes from people and agents \u{2014} what changed, why, and the audit chain behind it.",
            ),
        ],
        link: None,
    },
    HelpTopic {
        label: "How to contribute",
        title: "How to contribute",
        icon: Icon::GitPullRequest,
        body: &[
            (
                Icon::GitPullRequest,
                "Polterware is open source. Issues, discussions, and pull requests are welcome on GitHub.",
            ),
            (
                Icon::FileText,
                "Start from the repository README for setup, contribution guidelines, and how to report problems.",
            ),
        ],
        link: Some(("Open on GitHub", GITHUB_URL)),
    },
];

/// The popover's rows: each topic, opening its dialog.
pub fn help_rows(state: &Entity<AppState>) -> Vec<ListRow> {
    HELP_TOPICS
        .iter()
        .enumerate()
        .map(|(index, topic)| {
            let state = state.clone();
            ListRow::new(
                topic.label,
                Rc::new(move |_, cx: &mut App| {
                    state.update(cx, |state, cx| {
                        state.set_help_topic(Some(index));
                        cx.notify();
                    });
                }),
            )
            .icon(topic.icon)
        })
        .collect()
}

/// The open topic's dialog.
pub fn help_dialog(state: &Entity<AppState>, index: usize, cx: &App) -> AnyElement {
    let theme = Theme::global(cx);
    let Some(topic) = HELP_TOPICS.get(index) else {
        return div().into_any_element();
    };
    let close_state = state.clone();
    let close = Rc::new(move |_: &mut gpui::Window, cx: &mut App| {
        close_state.update(cx, |state, cx| {
            state.set_help_topic(None);
            cx.notify();
        });
    });
    let rows = topic.body.iter().map(|(icon, text)| {
        div()
            .flex()
            .items_start()
            .gap(px(tw(3.0)))
            .child(div().flex_none().mt(px(tw(0.5))).child(icon_colored(
                *icon,
                14.0,
                theme.color_text_secondary,
            )))
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .text_size(px(13.5))
                    .line_height(px(13.5 * 1.625))
                    .text_color(theme.color_text_secondary)
                    .child(SharedString::from(*text)),
            )
    });
    let footer: Vec<AnyElement> = topic
        .link
        .map(|(label, url)| {
            Button::new("help-link", ButtonKind::Primary)
                .size(ButtonSize::Sm)
                .icon(Icon::ExternalLink)
                .label(label)
                .radius(8.0)
                .on_click(move |_, cx| cx.open_url(url))
                .into_any_element()
        })
        .into_iter()
        .collect();
    dialog(topic.title, 1.0)
        .size(DialogSize::Md)
        .title_leading(
            IconTile::new(topic.icon)
                .size(24.0, 14.0)
                .tone(TileTone::Neutral),
        )
        .on_close(close)
        .footer(footer)
        .child(div().flex().flex_col().gap(px(tw(5.0))).children(rows))
        .into_any_element()
}
