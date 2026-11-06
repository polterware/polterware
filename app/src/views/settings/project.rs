//! Project settings' own pages (`project-settings-dialog.tsx`): General (the name, the
//! description, the default environment, and the project's id, saved together; then the
//! AGENTS.md sync switch), Environments (each one a row that opens to its summary, protection
//! rules, id, and actions; Add environment in the shared Composer, or a template when there is
//! none yet), and Danger zone (Archive project, after a confirmation).

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClipboardItem, Context, Entity, IntoElement, SharedString, Window, div,
    prelude::*, px,
};
use polterware_ui::button::{Button, ButtonKind, ButtonSize};
use polterware_ui::chip::chip_rounded;
use polterware_ui::icon_tile::{IconTile, TileTone};
use polterware_ui::icons::Icon;
use polterware_ui::menu::MenuItem;
use polterware_ui::patterns::{
    Composer, ComposerEvent, ComposerSpec, MascotMood, PromptCard, Property, PropertyOption,
    page_empty_with, settings_group, settings_row, template_card_grid,
};
use polterware_ui::skeleton::text_skeleton;
use polterware_ui::theme::{StatusTone, Theme, tone_color, tw};

use super::{Ask, SettingsView};
use crate::contracts::Environment;
use crate::state::{Load, ProjectSettingsData, default_protection};

/// `.pw-status-chip`'s corners.
pub(super) const CHIP_RADIUS: f32 = 6.0;
/// General's fields (`w-72`).
const FIELD_WIDTH: f32 = 72.0;

/// The kinds an environment takes (`KIND_OPTIONS`), in the Composer's order.
const KINDS: [(&str, &str); 4] = [
    ("development", "Development"),
    ("staging", "Staging"),
    ("production", "Production"),
    ("local", "Local"),
];

/// `PROTECTION_OPTIONS`.
const PROTECTIONS: [(&str, &str); 3] = [
    ("relaxed", "Relaxed"),
    ("standard", "Standard"),
    ("strict", "Strict"),
];

/// `ENVIRONMENT_CREATE_TEMPLATES`: a kind, its description, and its tile.
const TEMPLATES: [(&str, &str, TileTone); 3] = [
    (
        "development",
        "Relaxed local development environment.",
        TileTone::Execution,
    ),
    (
        "staging",
        "Standard pre-production environment.",
        TileTone::Infrastructure,
    ),
    (
        "production",
        "Strict production environment.",
        TileTone::Security,
    ),
];

/// `environmentKindLabel`.
pub(super) fn kind_label(kind: &str) -> String {
    KINDS
        .iter()
        .find(|(value, _)| value.eq_ignore_ascii_case(kind))
        .map(|(_, label)| label.to_string())
        .unwrap_or_else(|| kind.to_string())
}

/// `environmentKindTone`.
fn kind_tone(kind: &str) -> StatusTone {
    match kind.to_lowercase().as_str() {
        "production" => StatusTone::Danger,
        "staging" => StatusTone::Warn,
        "development" => StatusTone::Healthy,
        "local" => StatusTone::Muted,
        _ => StatusTone::Unknown,
    }
}

/// `protectionLabel`.
fn protection_label(protection: &str) -> String {
    PROTECTIONS
        .iter()
        .find(|(value, _)| value.eq_ignore_ascii_case(protection))
        .map(|(_, label)| label.to_string())
        .unwrap_or_else(|| protection.to_string())
}

/// `protectionTone`.
fn protection_tone(protection: &str) -> StatusTone {
    match protection.to_lowercase().as_str() {
        "strict" => StatusTone::Danger,
        "standard" => StatusTone::Warn,
        "relaxed" => StatusTone::Muted,
        _ => StatusTone::Unknown,
    }
}

/// `protectionRules`: what each protection level promises.
fn protection_rules(protection: &str) -> &'static [&'static str] {
    match protection {
        "strict" => &[
            "Changes persist locally",
            "Freeze window: Fri 18:00 \u{2013} Mon 08:00 UTC",
            "Destructive commands are not executed",
        ],
        "relaxed" => &[
            "Local changes are allowed",
            "No remote commands are executed",
        ],
        _ => &[
            "Changes persist locally",
            "Validation results stay in the local store",
        ],
    }
}

/// A status dot as an option's mark, centered in the picker's tile.
fn dot_option(value: &str, label: &str, tone: StatusTone) -> PropertyOption {
    PropertyOption::new(value.to_string(), label.to_string()).mark(move |size, _, cx| {
        let dot = div()
            .size(px(tw(1.5)))
            .rounded_full()
            .bg(tone_color(Theme::global(cx), tone));
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
}

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
        let key = (
            id,
            name.clone(),
            data.settings.description.clone(),
            data.settings.default_environment_id.clone(),
        );
        if self.general_for.as_ref() == Some(&key) {
            return;
        }
        self.reset_general(&key, cx);
        self.general_for = Some(key);
    }

    fn reset_general(&mut self, key: &(String, String, String, String), cx: &mut Context<Self>) {
        let (_, name, description, default) = key.clone();
        self.name.update(cx, |field, cx| field.set_text(name, cx));
        self.description
            .update(cx, |field, cx| field.set_text(description, cx));
        self.default_environment = default;
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
        let environment_label = data
            .environments
            .iter()
            .find(|environment| environment.id == self.default_environment)
            .map(|environment| environment.name.clone())
            .unwrap_or_default();
        let items = data
            .environments
            .iter()
            .map(|environment| {
                let id = environment.id.clone();
                let view = view.clone();
                MenuItem::radio(
                    environment.name.clone(),
                    environment.id == self.default_environment,
                    Rc::new(move |_, cx: &mut App| {
                        let id = id.clone();
                        let _ = view.update(cx, |this, cx| {
                            this.default_environment = id;
                            cx.notify();
                        });
                    }),
                )
            })
            .collect();
        let select = self.environment_picker.trigger(
            "settings-default-environment",
            environment_label,
            tw(FIELD_WIDTH),
            items,
            data.environments.is_empty(),
            cx,
        );
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
            .child(settings_row("Default environment").child(select))
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
                "The generated Polterware block in the AGENTS.md of each repository clone: the Rules agents must follow, principles, Structure, and decisions. The rest of the file is left alone.",
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
        let default = self.default_environment.clone();
        self.state.update(cx, |state, cx| {
            state.save_project_general(name, description, default, cx);
            cx.notify();
        });
    }

    pub(super) fn environments_page(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(data) = self.project_data(cx) else {
            let load = self.state.read(cx).project_settings().clone();
            return loading_or_failed(&load, window, cx);
        };
        let view = cx.entity().downgrade();
        let add_view = view.clone();
        let add = div().flex().justify_end().child(
            Button::new("settings-add-environment", ButtonKind::Secondary)
                .size(ButtonSize::Sm)
                .icon(Icon::Plus)
                .label("Add environment")
                .on_click(move |window, cx| {
                    let _ =
                        add_view.update(cx, |this, cx| this.open_environment_composer(window, cx));
                }),
        );
        let body = if data.environments.is_empty() {
            let pick_view = view.clone();
            let cards = TEMPLATES
                .iter()
                .map(|(kind, description, tone)| {
                    PromptCard::new(kind_label(kind), *description, Icon::Server, *tone)
                })
                .collect();
            let grid = template_card_grid(
                "settings-environment-templates",
                None,
                cards,
                Rc::new(move |index, _, cx| {
                    let Some((kind, _, _)) = TEMPLATES.get(index) else {
                        return;
                    };
                    let _ = pick_view.update(cx, |this, cx| {
                        this.state.update(cx, |state, cx| {
                            state.create_environment(
                                kind_label(kind),
                                kind.to_string(),
                                default_protection(kind).to_string(),
                                cx,
                            )
                        });
                    });
                }),
                window,
                cx,
            );
            div()
                .w_full()
                .min_h(px(200.0))
                .p(px(tw(8.0)))
                .flex()
                .items_center()
                .justify_center()
                .child(page_empty_with(
                    Some("No environments"),
                    MascotMood::Sleeping,
                    Some(grid),
                    cx,
                ))
                .into_any_element()
        } else {
            let can_delete = data.environments.len() > 1;
            div()
                .flex()
                .flex_col()
                .gap(px(tw(3.0)))
                .children(
                    data.environments
                        .iter()
                        .map(|environment| self.environment_row(environment, can_delete, cx)),
                )
                .into_any_element()
        };
        settings_group("Environments")
            .description("Manage local environments and choose the project default.")
            .child(add)
            .child(body)
            .into_any_element()
    }

    /// `EnvironmentRows`' row: the button that opens it, and what it shows open.
    fn environment_row(
        &self,
        environment: &Environment,
        can_delete: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = *Theme::global(cx);
        let open = self.expanded_environment.as_deref() == Some(environment.id.as_str());
        let view = cx.entity().downgrade();
        let toggle_view = view.clone();
        let id = environment.id.clone();
        let mut row = div()
            .id(SharedString::from(format!(
                "settings-environment-{}",
                environment.id
            )))
            .flex()
            .items_center()
            .gap(px(tw(3.0)))
            .cursor_pointer()
            .text_size(px(12.5))
            .text_color(theme.color_text_primary)
            .on_click(move |_, _, cx| {
                let id = id.clone();
                let _ = toggle_view.update(cx, |this, cx| {
                    this.expanded_environment =
                        (this.expanded_environment.as_deref() != Some(id.as_str())).then_some(id);
                    cx.notify();
                });
            })
            .child(
                div().flex_none().child(
                    IconTile::new(Icon::Server)
                        .size(24.0, 14.0)
                        .tone(TileTone::Infrastructure),
                ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(SharedString::from(environment.name.clone())),
            );
        if environment.is_default {
            row = row.child(chip_rounded("Default", StatusTone::Muted, CHIP_RADIUS, cx));
        }
        row = row
            .child(chip_rounded(
                kind_label(&environment.kind),
                kind_tone(&environment.kind),
                CHIP_RADIUS,
                cx,
            ))
            .child(chip_rounded(
                protection_label(&environment.protection),
                protection_tone(&environment.protection),
                CHIP_RADIUS,
                cx,
            ));
        let mut entry = div().flex().flex_col().gap(px(tw(3.0))).child(row);
        if open {
            let rules = protection_rules(&environment.protection);
            let default_view = view.clone();
            let delete_view = view.clone();
            let (default_id, delete_id) = (environment.id.clone(), environment.id.clone());
            let mut actions = div().flex().gap(px(tw(2.0)));
            if !environment.is_default {
                actions = actions.child(
                    Button::new(
                        SharedString::from(format!("settings-default-{}", environment.id)),
                        ButtonKind::Secondary,
                    )
                    .size(ButtonSize::Sm)
                    .label("Set as default")
                    .on_click(move |_, cx| {
                        let id = default_id.clone();
                        let _ = default_view.update(cx, |this, cx| {
                            this.state
                                .update(cx, |state, cx| state.set_default_environment(id, cx));
                        });
                    }),
                );
            }
            if can_delete {
                actions = actions.child(
                    Button::new(
                        SharedString::from(format!("settings-delete-{}", environment.id)),
                        ButtonKind::Danger,
                    )
                    .size(ButtonSize::Sm)
                    .label("Delete environment")
                    .on_click(move |_, cx| {
                        let id = delete_id.clone();
                        let _ = delete_view.update(cx, |this, cx| {
                            this.expanded_environment = None;
                            this.state
                                .update(cx, |state, cx| state.delete_environment(id, cx));
                        });
                    }),
                );
            }
            entry = entry.child(
                div()
                    .pl(px(tw(9.0)))
                    .flex()
                    .flex_col()
                    .gap(px(tw(2.0)))
                    .text_size(px(12.0))
                    .text_color(theme.color_text_secondary)
                    .child(SharedString::from(if environment.description.is_empty() {
                        "No summary yet.".to_string()
                    } else {
                        environment.description.clone()
                    }))
                    .child(
                        div()
                            .pl(px(tw(4.0)))
                            .flex()
                            .flex_col()
                            .gap(px(tw(1.0)))
                            .children(rules.iter().map(|rule| {
                                div()
                                    .flex()
                                    .gap(px(6.0))
                                    .child("\u{2022}")
                                    .child(SharedString::from(*rule))
                            })),
                    )
                    .child(
                        div()
                            .font_family(theme.font_mono)
                            .text_size(px(11.0))
                            .text_color(theme.color_text_muted)
                            .child(SharedString::from(environment.id.clone())),
                    )
                    .child(actions),
            );
        }
        entry.into_any_element()
    }

    /// Add environment (`CreateEnvironmentDialog`): the name as the title, then Kind and
    /// Protection, which follows the kind until it is picked.
    fn open_environment_composer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.environment_kind = "development".into();
        self.environment_protection = "relaxed".into();
        let spec = ComposerSpec::new("Environment name", "Create environment")
            .crumbs(["Environments", "Add environment"])
            .edit();
        let properties = self.environment_properties();
        let view = cx.entity().downgrade();
        self.environment_composer.update(cx, |composer, cx| {
            composer.open(spec, properties, window, cx);
            composer.set_body(Some(Rc::new(move |window, cx| {
                let view = view.clone();
                template_card_grid(
                    "settings-add-environment-templates",
                    Some("Start from a template".into()),
                    TEMPLATES
                        .iter()
                        .map(|(kind, description, tone)| {
                            PromptCard::new(kind_label(kind), *description, Icon::Server, *tone)
                        })
                        .collect(),
                    Rc::new(move |index, _, cx| {
                        let Some((kind, _, _)) = TEMPLATES.get(index) else {
                            return;
                        };
                        let _ = view.update(cx, |this, cx| {
                            this.environment_composer
                                .update(cx, |composer, cx| composer.close(cx));
                            this.state.update(cx, |state, cx| {
                                state.create_environment(
                                    kind_label(kind),
                                    kind.to_string(),
                                    default_protection(kind).to_string(),
                                    cx,
                                )
                            });
                        });
                    }),
                    window,
                    cx,
                )
            })));
        });
    }

    fn environment_properties(&self) -> Vec<Property> {
        let kinds = KINDS
            .iter()
            .map(|(value, label)| dot_option(value, label, kind_tone(value)))
            .collect();
        let protections = PROTECTIONS
            .iter()
            .map(|(value, label)| dot_option(value, label, protection_tone(value)))
            .collect();
        vec![
            Property::single("kind", "Kind", kinds, self.environment_kind.clone())
                .icon(Icon::Layers),
            Property::single(
                "protection",
                "Protection",
                protections,
                self.environment_protection.clone(),
            )
            .icon(Icon::ShieldCheck),
        ]
    }

    pub(super) fn environment_composed(
        &mut self,
        composer: Entity<Composer>,
        event: &ComposerEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            ComposerEvent::Submit(draft) => {
                let name = draft.title.trim().to_string();
                if name.is_empty() {
                    composer.update(cx, |composer, cx| composer.finish(false, cx));
                    return;
                }
                let (kind, protection) = (
                    self.environment_kind.clone(),
                    self.environment_protection.clone(),
                );
                self.state.update(cx, |state, cx| {
                    state.create_environment(name, kind, protection, cx)
                });
                composer.update(cx, |composer, cx| composer.finish(true, cx));
            }
            ComposerEvent::Property { id, values } => {
                let value = values
                    .first()
                    .map(|value| value.to_string())
                    .unwrap_or_default();
                match id.as_ref() {
                    "kind" if !value.is_empty() => {
                        self.environment_protection = default_protection(&value).to_string();
                        self.environment_kind = value;
                    }
                    "protection" if !value.is_empty() => self.environment_protection = value,
                    _ => {}
                }
                let properties = self.environment_properties();
                composer.update(cx, |composer, cx| composer.set_properties(properties, cx));
            }
            ComposerEvent::Action | ComposerEvent::Closed => {}
        }
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
