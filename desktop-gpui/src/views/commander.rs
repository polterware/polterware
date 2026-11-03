//! Commander (⌘K): the sections of the open project, the projects to switch to, App
//! settings, and Reload, filtered by what is typed. Enter runs the first match; Escape closes.

use std::rc::Rc;

use gpui::{
    AnyElement, App, Context, Entity, FocusHandle, Focusable, IntoElement, Render, ScrollHandle,
    SharedString, Window, div, prelude::*, px,
};
use polterware_ui::icons::{Icon, icon_colored};
use polterware_ui::patterns::{commander_panel, commander_size};
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::Theme;

use crate::state::{AppState, Load, Section};

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Section(Section),
    Project { id: String, name: String },
    Projects,
    AppSettings,
    Reload,
}

impl Command {
    pub fn label(&self) -> String {
        match self {
            Command::Section(section) => format!("Go to {}", section.label()),
            Command::Project { name, .. } => format!("Switch to {name}"),
            Command::Projects => "Projects".into(),
            Command::AppSettings => "App settings".into(),
            Command::Reload => "Reload".into(),
        }
    }

    fn icon(&self) -> Icon {
        match self {
            Command::Section(section) => section.icon(),
            Command::Project { .. } => Icon::Folder,
            Command::Projects => Icon::House,
            Command::AppSettings => Icon::Settings,
            Command::Reload => Icon::RefreshCw,
        }
    }
}

/// The commands that match `query` (every word, in any order, ignoring case).
pub fn commands(state: &AppState, query: &str) -> Vec<Command> {
    let mut all = Vec::new();
    if state.project().is_some() {
        all.extend(
            Section::ALL
                .iter()
                .map(|section| Command::Section(*section)),
        );
    }
    all.push(Command::Projects);
    if let Load::Ready(projects) = state.projects() {
        let open = state.project().map(|project| project.id.clone());
        all.extend(
            projects
                .iter()
                .filter(|project| Some(&project.id) != open.as_ref())
                .map(|project| Command::Project {
                    id: project.id.clone(),
                    name: project.name.clone(),
                }),
        );
    }
    all.push(Command::AppSettings);
    all.push(Command::Reload);
    let words: Vec<String> = query
        .split_whitespace()
        .map(|word| word.to_lowercase())
        .collect();
    all.into_iter()
        .filter(|command| {
            let label = command.label().to_lowercase();
            words.iter().all(|word| label.contains(word))
        })
        .collect()
}

pub fn run(state: &Entity<AppState>, command: Command, cx: &mut App) {
    state.update(cx, |state, cx| {
        match command {
            Command::Section(section) => state.go_section(section, cx),
            Command::Project { id, .. } => state.open_project(id, cx),
            Command::Projects => state.go_home(),
            Command::AppSettings => state.open_app_settings(),
            Command::Reload => {
                state.set_commander(false);
                state.connect(cx);
            }
        }
        state.set_commander(false);
        cx.notify();
    });
}

pub struct CommanderView {
    state: Entity<AppState>,
    input: Entity<TextInput>,
    scroll: ScrollHandle,
}

impl CommanderView {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let submit = state.clone();
        let cancel = state.clone();
        let input = cx.new(|cx| {
            TextInput::new(cx)
                .placeholder("Type a command or search")
                .bare()
                .on_submit(move |text, _window, cx| {
                    let first = commands(submit.read(cx), text).into_iter().next();
                    if let Some(command) = first {
                        run(&submit, command, cx);
                    }
                })
                .on_cancel(move |_window, cx| {
                    cancel.update(cx, |state, cx| {
                        state.set_commander(false);
                        cx.notify();
                    });
                })
        });
        cx.observe(&input, |_, _, cx| cx.notify()).detach();
        Self {
            state,
            input,
            scroll: ScrollHandle::new(),
        }
    }

    /// Clears the query and focuses the input, as Commander opens.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.update(cx, |input, cx| input.set_text("", cx));
        let handle = self.input.read(cx).focus_handle(cx);
        handle.focus(window);
    }

    fn row(&self, index: usize, command: Command, theme: &Theme) -> AnyElement {
        let state = self.state.clone();
        let label = SharedString::from(command.label());
        let icon = command.icon();
        div()
            .id(("commander-row", index))
            .h(px(36.0))
            .px(px(theme.space_3))
            .flex()
            .items_center()
            .gap(px(theme.space_3))
            .rounded(px(theme.radius_item))
            .cursor_pointer()
            .text_size(px(theme.font_size_md))
            .text_color(theme.color_text_secondary)
            .hover(|style| {
                style
                    .bg(theme.color_surface_hover)
                    .text_color(theme.color_text_primary)
            })
            .child(icon_colored(icon, 16.0, theme.color_text_muted))
            .child(label)
            .on_click(move |_, _, cx| run(&state, command.clone(), cx))
            .into_any_element()
    }
}

impl Focusable for CommanderView {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.input.read(cx).focus_handle(cx)
    }
}

impl Render for CommanderView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *Theme::global(cx);
        let query = self.input.read(cx).text().to_string();
        let matches = commands(self.state.read(cx), &query);
        let mut results = div()
            .id("commander-results")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .px(px(theme.space_3))
            .py(px(theme.space_2))
            .flex()
            .flex_col()
            .gap(px(2.0));
        if matches.is_empty() {
            results = results.child(
                div()
                    .px(px(theme.space_3))
                    .py(px(theme.space_2))
                    .text_size(px(theme.font_size_sm))
                    .text_color(theme.color_text_muted)
                    .child("No command matches"),
            );
        }
        for (index, command) in matches.into_iter().enumerate() {
            results = results.child(self.row(index, command, &theme));
        }
        let close = self.state.clone();
        commander_panel(
            "commander",
            commander_size(window),
            1.0,
            div().flex_1().child(self.input.clone()),
            results,
            &self.scroll,
            Rc::new(move |_, cx: &mut App| {
                close.update(cx, |state, cx| {
                    state.set_commander(false);
                    cx.notify();
                });
            }),
            &theme,
            cx,
        )
    }
}
