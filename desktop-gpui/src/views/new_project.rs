//! New project (`create-project-dialog.tsx`): the shared Composer with the name as its title
//! field (`Project name`) and the initial environments as tokens, each kind its own pill with
//! its status dot (Development muted, Staging syncing, Production warn), all three to start.
//! A project keeps at least one: removing the last does nothing. Create project waits for a
//! name; Enter in the name or Cmd+Enter creates it, as `persistCreatedProject` does.

use std::rc::Rc;

use gpui::{
    AnyElement, App, Context, Entity, Focusable, IntoElement, KeyDownEvent, Render, SharedString,
    Window, div, prelude::*, px,
};
use polterware_ui::patterns::{
    Property, PropertyOption, PropertyPills, PropertyPillsEvent, composer_card, composer_footer,
};
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::{StatusTone, Theme, tone_color, tw};
use polterware_ui::{Button, ButtonKind, ButtonSize};

use crate::state::AppState;

/// `ENV_OPTIONS`: each kind, its label, and its dot's tone, in the order a project keeps them.
const KINDS: [(&str, &str, StatusTone); 3] = [
    ("development", "Development", StatusTone::Muted),
    ("staging", "Staging", StatusTone::Syncing),
    ("production", "Production", StatusTone::Warn),
];

pub struct NewProjectView {
    state: Entity<AppState>,
    title: Entity<TextInput>,
    pills: Entity<PropertyPills>,
    environments: Vec<SharedString>,
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
        let pills = cx.new(PropertyPills::new);
        cx.observe(&pills, |_, _, cx| cx.notify()).detach();
        cx.subscribe(&pills, |this, _, event: &PropertyPillsEvent, cx| {
            let PropertyPillsEvent::Changed { values, .. } = event;
            // A project starts with at least one environment, kept in the kinds' order.
            let next: Vec<SharedString> = KINDS
                .iter()
                .map(|(kind, _, _)| SharedString::from(*kind))
                .filter(|kind| values.contains(kind))
                .collect();
            if !next.is_empty() {
                this.environments = next;
            }
            this.sync_pills(cx);
        })
        .detach();
        let mut view = Self {
            state,
            title,
            pills,
            environments: Vec::new(),
        };
        view.reset(cx);
        view
    }

    /// Starts again: no name and every kind, with the name focused.
    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset(cx);
        let focus = self.title.focus_handle(cx);
        focus.focus(window);
    }

    fn reset(&mut self, cx: &mut Context<Self>) {
        self.environments = KINDS
            .iter()
            .map(|(kind, _, _)| SharedString::from(*kind))
            .collect();
        self.title.update(cx, |title, cx| title.set_text("", cx));
        self.sync_pills(cx);
    }

    fn sync_pills(&mut self, cx: &mut Context<Self>) {
        let options = KINDS
            .iter()
            .map(|(kind, label, tone)| {
                let tone = *tone;
                PropertyOption::new(*kind, *label).mark(move |size, _, cx| {
                    let color = tone_color(Theme::global(cx), tone);
                    // `StatusDot` without its label: the `h-1.5 w-1.5` dot on its own on a
                    // token, centred in the picker's tile.
                    let dot = div().size(px(tw(1.5))).rounded_full().bg(color);
                    if size > 14.0 {
                        div()
                            .size(px(size))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(dot)
                            .into_any_element()
                    } else {
                        dot.into_any_element()
                    }
                })
            })
            .collect();
        let property = Property::tokens(
            "environments",
            "Environments",
            options,
            self.environments.clone(),
            "No environments",
        )
        .loose();
        self.pills
            .update(cx, |pills, cx| pills.set_properties(vec![property], cx));
        cx.notify();
    }

    fn name(&self, cx: &App) -> String {
        self.title.read(cx).text().trim().to_string()
    }

    fn submit(&mut self, cx: &mut Context<Self>) {
        let name = self.name(cx);
        if name.is_empty() || self.environments.is_empty() {
            return;
        }
        let environments = self
            .environments
            .iter()
            .map(|kind| kind.to_string())
            .collect();
        self.state.update(cx, |state, cx| {
            state.create_project(name, environments, cx);
            cx.notify();
        });
    }
}

impl Render for NewProjectView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *Theme::global(cx);
        let can_submit = !self.name(cx).is_empty() && !self.environments.is_empty();
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
            .radius(8.0)
            .disabled(!can_submit)
            .on_click(move |_, cx| {
                if let Some(view) = view.upgrade() {
                    view.update(cx, |this, cx| this.submit(cx));
                }
            })
            .into_any_element();
        let overlay = self.pills.read(cx).overlay(cx);
        let card = composer_card("New project", 1.0, 0.0, &theme)
            .root_scaled()
            .child(self.title.clone())
            .child(div().pt(px(4.0)).child(self.pills.clone()))
            .footer(composer_footer([], [create], &theme))
            .overlay(overlay);
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
