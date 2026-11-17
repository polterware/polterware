//! New project Composer.

use std::rc::Rc;

use gpui::{
    AnyElement, App, Context, Entity, Focusable, IntoElement, KeyDownEvent, Render, Window, div,
    prelude::*,
};
use polterware_ui::patterns::{composer_card, composer_footer};
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::Theme;
use polterware_ui::{Button, ButtonKind, ButtonSize};

use crate::state::AppState;

pub struct NewProjectView {
    state: Entity<AppState>,
    title: Entity<TextInput>,
}

impl NewProjectView {
    pub fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let view = cx.entity().downgrade();
        let title = cx.new(|cx| {
            TextInput::new(cx)
                .placeholder("Project name")
                .bare()
                .title()
                .on_submit(move |_, _, cx| {
                    if let Some(view) = view.upgrade() {
                        view.update(cx, |this, cx| this.submit(cx));
                    }
                })
        });
        cx.observe(&title, |_, _, cx| cx.notify()).detach();
        let mut view = Self { state, title };
        view.reset(cx);
        view
    }

    /// Starts again with no name, with the name focused.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset(cx);
        let focus = self.title.focus_handle(cx);
        focus.focus(window);
    }

    fn reset(&mut self, cx: &mut Context<Self>) {
        self.title.update(cx, |title, cx| title.set_text("", cx));
    }

    fn name(&self, cx: &App) -> String {
        self.title.read(cx).text().trim().to_string()
    }

    fn submit(&mut self, cx: &mut Context<Self>) {
        let name = self.name(cx);
        if name.is_empty() {
            return;
        }
        self.state.update(cx, |state, cx| {
            state.create_project(name, cx);
            cx.notify();
        });
    }
}

impl Render for NewProjectView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *Theme::global(cx);
        let can_submit = !self.name(cx).is_empty();
        let close_state = self.state.clone();
        let close = Rc::new(move |_: &mut Window, cx: &mut App| {
            close_state.update(cx, |state, cx| {
                state.close_new_project();
                cx.notify();
            });
        });
        let view = cx.entity().downgrade();
        let create: AnyElement = Button::new("new-project-create", ButtonKind::Primary)
            .size(ButtonSize::Sm)
            .label("Create project")
            .disabled(!can_submit)
            .on_click(move |_, cx| {
                if let Some(view) = view.upgrade() {
                    view.update(cx, |this, cx| this.submit(cx));
                }
            })
            .into_any_element();
        let card = composer_card("New project", 1.0, 0.0, &theme)
            .root_scaled()
            .child(self.title.clone())
            .footer(composer_footer([], [create], &theme));
        let card = if self.state.read(cx).can_close_new_project() {
            card.on_close(close)
        } else {
            card
        };
        // Cmd+Enter creates from anywhere in the card.
        let view = cx.entity().downgrade();
        div()
            .capture_key_down(move |event: &KeyDownEvent, _, cx| {
                let keys = &event.keystroke;
                if keys.key == "enter" && (keys.modifiers.platform || keys.modifiers.control) {
                    cx.stop_propagation();
                    if let Some(view) = view.upgrade() {
                        view.update(cx, |this, cx| this.submit(cx));
                    }
                }
            })
            .child(card)
    }
}
