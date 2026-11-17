//! Project settings: General and Danger zone.

use gpui::{
    AnyElement, App, ClipboardItem, Context, IntoElement, SharedString, Window, div, prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::patterns::{settings_group, settings_row};
use polterware_ui::skeleton::text_skeleton;
use polterware_ui::theme::{Theme, tw};

use super::{Ask, SettingsView};
use crate::state::{Load, ProjectSettingsData};

pub(super) const CHIP_RADIUS: f32 = 6.0;

/// General's fields (`w-72`).
const FIELD_WIDTH: f32 = 72.0;

/// A secondary text in the page's muted 12px.
pub(super) fn muted_line(text: impl Into<SharedString>, cx: &App) -> AnyElement {
    div()
        .text_size(px(12.0))
        .text_color(Theme::global(cx).color_text_muted)
        .child(text.into())
        .into_any_element()
}

/// The page while its reads are on their way, or why they failed.
pub(super) fn loading_or_failed<T>(load: &Load<T>, window: &mut Window, cx: &App) -> AnyElement {
    match load {
        Load::Failed(error) => muted_line(error.clone(), cx),
        _ => text_skeleton(3, 12.0, window, cx).into_any_element(),
    }
}

impl SettingsView {
    fn project_data(&self, cx: &App) -> Option<ProjectSettingsData> {
        match self.state.read(cx).project_settings() {
            Load::Ready(data) => Some(data.clone()),
            _ => None,
        }
    }

    /// Fills General's fields from the project's settings when they change.
    fn fill_general(&mut self, data: &ProjectSettingsData, cx: &mut Context<Self>) {
        let app = self.state.read(cx);
        let id = app.open_project_id().unwrap_or_default();
        let name = app
            .project()
            .map(|project| project.name.clone())
            .unwrap_or_default();
        let key = (id, name.clone(), data.settings.description.clone());
        if self.general_for.as_ref() == Some(&key) {
            return;
        }
        self.reset_general(&key, cx);
        self.general_for = Some(key);
    }

    fn reset_general(&mut self, key: &(String, String, String), cx: &mut Context<Self>) {
        let (_, name, description) = key.clone();
        self.name.update(cx, |field, cx| field.set_text(name, cx));
        self.description
            .update(cx, |field, cx| field.set_text(description, cx));
    }

    pub(super) fn general_page(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(data) = self.project_data(cx) else {
            let load = self.state.read(cx).project_settings().clone();
            return loading_or_failed(&load, window, cx);
        };
        self.fill_general(&data, cx);
        let theme = *Theme::global(cx);
        let view = cx.entity().downgrade();
        let project_id = self.state.read(cx).open_project_id().unwrap_or_default();
        let copy_state = self.state.clone();
        let copy_id = project_id.clone();
        let cancel_view = view.clone();
        let save_view = view.clone();
        let sync_on = data.settings.agents_sync;
        let busy = self.state.read(cx).agents_sync_busy();
        self.agents_sync.update(cx, |switch, _| {
            if switch.is_on() != sync_on {
                switch.set_on(sync_on);
            }
        });
        let project = settings_group("Project")
            .description("Local project details. Changes persist in the local SQLite store.")
            .child(
                settings_row("Name").child(div().w(px(tw(FIELD_WIDTH))).child(self.name.clone())),
            )
            .child(
                settings_row("Description")
                    .child(div().w(px(tw(FIELD_WIDTH))).child(self.description.clone())),
            )
            .child(
                settings_row("Project ID")
                    .child(
                        div()
                            .font_family(theme.font_mono)
                            .text_size(px(12.0))
                            .text_color(theme.color_text_primary)
                            .child(SharedString::from(if project_id.is_empty() {
                                "\u{2014}".to_string()
                            } else {
                                project_id.clone()
                            })),
                    )
                    .child(
                        Button::new("settings-copy-project-id", ButtonKind::Ghost)
                            .size(ButtonSize::Sm)
                            .label("Copy")
                            .on_click(move |_, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(copy_id.clone()));
                                copy_state.update(cx, |state, cx| {
                                    state.push_toast(
                                        "Project ID copied.",
                                        polterware_ui::toast::ToastTone::Success,
                                    );
                                    cx.notify();
                                });
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(tw(2.0)))
                    .child(
                        Button::new("settings-general-cancel", ButtonKind::Secondary)
                            .size(ButtonSize::Sm)
                            .label("Cancel")
                            .on_click(move |_, cx| {
                                let _ = cancel_view.update(cx, |this, cx| {
                                    if let Some(key) = this.general_for.clone() {
                                        this.reset_general(&key, cx);
                                    }
                                    cx.notify();
                                });
                            }),
                    )
                    .child(
                        Button::new("settings-general-save", ButtonKind::Secondary)
                            .size(ButtonSize::Sm)
                            .label("Save changes")
                            .on_click(move |_, cx| {
                                let _ = save_view.update(cx, |this, cx| this.save_general(cx));
                            }),
                    ),
            );
        let agents = settings_group("AGENTS.md")
            .description(
                "The generated Polterware block in the AGENTS.md of each repository clone: the Rules agents must follow and the Structure. The rest of the file is left alone.",
            )
            .child(
                settings_row("Keep in sync")
                    .description(
                        "Rewrite the block a moment after project state changes, only when it differs. Off, it changes only on polterware export.",
                    )
                    .child(
                        div()
                            .when(busy, |switch| switch.opacity(0.5).occlude())
                            .child(self.agents_sync.clone()),
                    ),
            );
        div()
            .flex()
            .flex_col()
            .gap(px(polterware_ui::patterns::settings_group_gap()))
            .child(project)
            .child(agents)
            .into_any_element()
    }

    fn save_general(&mut self, cx: &mut Context<Self>) {
        let name = self.name.read(cx).text().to_string();
        let description = self.description.read(cx).text().to_string();
        self.state.update(cx, |state, cx| {
            state.save_project_general(name, description, cx);
            cx.notify();
        });
    }

    pub(super) fn project_danger_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let view = cx.entity().downgrade();
        settings_group("Danger zone")
            .description(
                "Hide this project from the switcher. Local data stays in SQLite until you reset the store.",
            )
            .child(
                settings_row("Archive project")
                    .description("Hide this project from the switcher.")
                    .child(
                        Button::new("settings-archive-project", ButtonKind::Danger)
                            .size(ButtonSize::Sm)
                            .label("Archive project")
                            .on_click(move |_, cx| {
                                let _ = view.update(cx, |this, cx| this.ask(Ask::ArchiveProject, cx));
                            }),
                    ),
            )
            .into_any_element()
    }
}
