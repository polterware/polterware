//! Settings (`SettingsView` in `@polterware/feature-settings`): the one settings dialog over
//! the page, polterware-ui's [`settings_dialog`], titled by its scope. Project settings (the
//! sidebar's footer) has General, Environments, Platforms, and Danger zone under the Project
//! group; App settings (the title bar's gear, beside Commander) has the Agents group (MCP
//! server, Agent setup, Hooks, AI skill, CLI reference) and the App group (Shortcuts,
//! Appearance, Local core, Privacy, Danger zone). The panel's heading is `Group / Label`, the
//! App group's only its label. The rail's search finds the settings of the open scope, the
//! shortcuts, and the CLI's commands, and a result opens its page.
//!
//! It opens and closes on the dialog's motion ([`DialogPresence`]), drawing the page it showed
//! while it leaves. A press on the scrim, Close, or Escape closes it. Archive project and Reset
//! local store ask first, in a confirmation over it; the menus its selects open hang over the
//! whole dialog ([`SettingsDialog::overlay`]).

mod agents;
mod app_pages;
mod platforms;
mod project;

use std::collections::BTreeMap;
use std::rc::Rc;
use std::time::Instant;

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, FocusHandle, IntoElement, Render,
    SharedString, Subscription, Window, div, prelude::*, px,
};
use polterware_ui::patterns::{
    Composer, ConfirmDialog, DialogPresence, SettingsNavItem, SettingsSearchEntry, modal_layer,
    settings_dialog, settings_group_gap, window_layer,
};
use polterware_ui::switch::Switch;
use polterware_ui::text_input::{FieldSize, TextInput};
use polterware_ui::theme::Theme;

use super::picker::Picker;
use crate::cli_reference;
use crate::shortcuts;
use crate::state::{AppState, SettingsScope, SettingsTab};

use platforms::Connect;

/// A question the dialog asks before a destructive action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ask {
    ArchiveProject,
    ResetStore,
}

pub struct SettingsView {
    state: Entity<AppState>,
    presence: DialogPresence,
    /// The tab drawn: the open one, or the last one while the dialog leaves.
    shown: SettingsTab,
    search: Entity<TextInput>,
    // General.
    name: Entity<TextInput>,
    description: Entity<TextInput>,
    default_environment: String,
    /// The project and settings General's fields were filled from.
    general_for: Option<(String, String, String, String)>,
    environment_picker: Picker,
    agents_sync: Entity<Switch>,
    // Environments.
    expanded_environment: Option<String>,
    environment_composer: Entity<Composer>,
    environment_kind: String,
    environment_protection: String,
    // Danger zones.
    ask: Option<Ask>,
    ask_focus: FocusHandle,
    ask_focus_due: bool,
    // Platforms.
    platform_search: Entity<TextInput>,
    platform_detail: Option<String>,
    connect: Option<Connect>,
    // Agents.
    hooks: BTreeMap<String, Entity<Switch>>,
    // App.
    transparency: Entity<Switch>,
    auto_start: Entity<Switch>,
    idle_picker: Picker,
    quit_picker: Picker,
    /// The shortcut being recorded, and the interceptor that hears its keys.
    recording: Option<(&'static str, Subscription)>,
    /// The CLI reference's copy button that shows its check, and since when.
    copied: Option<(String, Instant)>,
}

impl SettingsView {
    pub fn new(state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let view = cx.entity().downgrade();
        let search = cx.new(|cx| {
            TextInput::new(cx)
                .placeholder("Search settings")
                .size(FieldSize::Sm)
                .pill()
        });
        cx.observe(&search, |_, _, cx| cx.notify()).detach();
        let name = cx.new(|cx| TextInput::new(cx).size(FieldSize::Md));
        let description = cx.new(|cx| TextInput::new(cx).multiline());
        cx.observe(&name, |_, _, cx| cx.notify()).detach();
        cx.observe(&description, |_, _, cx| cx.notify()).detach();
        let sync_view = view.clone();
        let agents_sync = cx.new(|cx| {
            Switch::new(false, cx).on_change(move |on, _, cx| {
                let _ = sync_view.update(cx, |this, cx| {
                    this.state
                        .update(cx, |state, cx| state.set_agents_sync(on, cx));
                });
            })
        });
        let environment_composer = cx.new(Composer::new);
        cx.observe(&environment_composer, |_, _, cx| cx.notify())
            .detach();
        cx.subscribe(&environment_composer, Self::environment_composed)
            .detach();
        let platform_search = cx.new(|cx| {
            TextInput::new(cx)
                .placeholder("Search platforms\u{2026}")
                .size(FieldSize::Sm)
                .pill()
        });
        cx.observe(&platform_search, |_, _, cx| cx.notify())
            .detach();
        let app = state.read(cx);
        let (reduce, auto) = (app.reduce_transparency(), app.core_prefs().auto_start);
        let transparency_state = state.clone();
        let transparency = cx.new(|cx| {
            Switch::new(reduce, cx).on_change(move |on, _, cx| {
                transparency_state.update(cx, |state, cx| {
                    state.set_reduce_transparency(on);
                    cx.notify();
                });
            })
        });
        let start_state = state.clone();
        let auto_start = cx.new(|cx| {
            Switch::new(auto, cx).on_change(move |on, _, cx| {
                start_state.update(cx, |state, cx| {
                    state.set_core_auto_start(on);
                    cx.notify();
                });
            })
        });
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        let _ = window;
        Self {
            state,
            presence: DialogPresence::new(),
            shown: SettingsTab::General,
            search,
            name,
            description,
            default_environment: String::new(),
            general_for: None,
            environment_picker: Picker::new(cx),
            agents_sync,
            expanded_environment: None,
            environment_composer,
            environment_kind: "development".into(),
            environment_protection: "relaxed".into(),
            ask: None,
            ask_focus: cx.focus_handle(),
            ask_focus_due: false,
            platform_search,
            platform_detail: None,
            connect: None,
            hooks: BTreeMap::new(),
            transparency,
            auto_start,
            idle_picker: Picker::new(cx),
            quit_picker: Picker::new(cx),
            recording: None,
            copied: None,
        }
    }

    /// Whether a field, a menu, or a dialog of its own holds the keys: C and N stay its.
    pub fn is_open(&self) -> bool {
        self.presence.is_open()
    }

    /// Follows the state's open tab: opens, changes page, or leaves.
    fn sync_open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let tab = self.state.read(cx).settings_tab();
        match tab {
            Some(tab) => {
                if !self.presence.is_open() {
                    self.presence.set_open(true);
                    self.search.update(cx, |field, cx| field.set_text("", cx));
                }
                if tab != self.shown {
                    self.leave_page(cx);
                    self.shown = tab;
                }
            }
            None => {
                if self.presence.is_open() {
                    self.presence.set_open(false);
                    self.leave_page(cx);
                    self.ask = None;
                    window.blur();
                }
            }
        }
    }

    /// What a page leaves behind when another one opens: its menus, its recording, its peek.
    fn leave_page(&mut self, cx: &mut Context<Self>) {
        for picker in [
            &self.environment_picker,
            &self.idle_picker,
            &self.quit_picker,
        ] {
            picker.close(cx);
        }
        self.recording = None;
        self.platform_detail = None;
        self.expanded_environment = None;
    }

    fn select(&mut self, tab: SettingsTab, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.open_settings(tab, cx);
            cx.notify();
        });
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.close_settings();
            cx.notify();
        });
    }

    fn ask(&mut self, ask: Ask, cx: &mut Context<Self>) {
        self.ask = Some(ask);
        self.ask_focus_due = true;
        cx.notify();
    }

    fn answer(&mut self, cx: &mut Context<Self>) {
        let Some(ask) = self.ask.take() else {
            return;
        };
        self.state.update(cx, |state, cx| {
            match ask {
                Ask::ArchiveProject => state.archive_open_project(cx),
                Ask::ResetStore => state.reset_local_store(cx),
            }
            cx.notify();
        });
    }

    /// The settings the rail's search finds in `scope` (`SETTINGS_SEARCH_ENTRIES` and
    /// `dynamicSettingsEntries`).
    fn search_entries(scope: SettingsScope) -> Vec<SettingsSearchEntry> {
        let entry = |tab: SettingsTab, section: &str, label: &str| {
            SettingsSearchEntry::new(tab.id(), label.to_string()).section(section.to_string())
        };
        let mut entries = vec![
            entry(SettingsTab::General, "Project", "Project")
                .keywords("name description default environment project id"),
            entry(SettingsTab::General, "AGENTS.md", "AGENTS.md sync").keywords(
                "agents md sync generated block principles rules structure decisions export",
            ),
            entry(SettingsTab::Environments, "Environments", "Environments")
                .keywords("add environment default delete"),
            entry(SettingsTab::ProjectDanger, "Danger zone", "Archive project")
                .description("Hide this project from the switcher. Local data stays in SQLite.")
                .keywords("project archive"),
            entry(SettingsTab::Privacy, "Privacy", "Privacy")
                .description("Local-first data and credential handling.")
                .keywords("credentials tokens account"),
            entry(SettingsTab::AgentsMcp, "MCP server", "MCP helper")
                .description("Start or stop the local MCP server agents connect to.")
                .keywords("agents server cursor claude"),
            entry(SettingsTab::AgentsSetup, "Agent setup", "Agent setup")
                .description("Check Claude Code and Codex setup and copy repair commands.")
                .keywords("claude codex plugin install doctor path"),
            entry(SettingsTab::AgentsHooks, "Hooks", "Agent hooks")
                .description("Switch the Polterware plugin hooks on or off.")
                .keywords("claude codex plugin setup"),
            entry(SettingsTab::AgentsSkill, "AI skill", "AI skill")
                .description("Copy or download the Polterware instructions for coding agents.")
                .keywords("markdown download copy instructions"),
            entry(SettingsTab::Platforms, "Platforms", "Platforms")
                .description(
                    "Platform CLIs: installed here, connected to this project, and their last refresh.",
                )
                .keywords(
                    "integrations providers vercel github aws sync refresh connect install login catalog connected",
                ),
            entry(SettingsTab::CliReference, "CLI reference", "CLI reference")
                .description("Every polterware command, its options, and the interactive keys.")
                .keywords("command line terminal flags markdown"),
            entry(SettingsTab::Appearance, "Appearance", "Reduce transparency")
                .description("Use a solid fill instead of the macOS frosted title bar and sidebar.")
                .keywords("appearance glass vibrancy"),
            entry(SettingsTab::LocalCore, "Local core", "Local core")
                .description("Status, start on open, and quit behavior.")
                .keywords("server serve process quit start"),
            entry(SettingsTab::AppDanger, "Danger zone", "Reset local store")
                .description("Delete local data and reseed the demonstration project.")
                .keywords("database sqlite"),
        ];
        for group in shortcuts::Group::ALL {
            entries.extend(shortcuts::in_group(group).map(|command| {
                entry(SettingsTab::Shortcuts, "Shortcuts", command.label)
                    .keywords(format!("{} keyboard chord", group.label()))
            }));
        }
        entries.extend(cli_reference::reference().commands.iter().map(|command| {
            entry(SettingsTab::CliReference, "Commands", &command.name)
                .description(command.summary.clone())
                .keywords("cli command")
        }));
        entries
            .into_iter()
            .filter(|entry| {
                SettingsTab::from_id(&entry.item).is_some_and(|tab| tab.scope() == scope)
            })
            .collect()
    }

    /// The open page.
    fn page(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match self.shown {
            SettingsTab::General => self.general_page(window, cx),
            SettingsTab::Environments => self.environments_page(window, cx),
            SettingsTab::Platforms => self.platforms_page(cx),
            SettingsTab::ProjectDanger => self.project_danger_page(cx),
            SettingsTab::AgentsMcp => self.mcp_page(cx),
            SettingsTab::AgentsSetup => self.setup_page(window, cx),
            SettingsTab::AgentsHooks => self.hooks_page(window, cx),
            SettingsTab::AgentsSkill => self.skill_page(),
            SettingsTab::CliReference => self.cli_reference_page(window, cx),
            SettingsTab::Shortcuts => self.shortcuts_page(cx),
            SettingsTab::Appearance => self.appearance_page(window, cx),
            SettingsTab::LocalCore => self.core_page(window, cx),
            SettingsTab::Privacy => self.privacy_page(cx),
            SettingsTab::AppDanger => self.app_danger_page(cx),
        }
    }

    /// The confirmation over the dialog, while one is asked.
    fn ask_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let ask = self.ask?;
        if std::mem::take(&mut self.ask_focus_due) {
            self.ask_focus.focus(window);
        }
        let theme = *Theme::global(cx);
        let view = cx.entity().downgrade();
        let cancel: polterware_ui::Handler = {
            let view = view.clone();
            Rc::new(move |_, cx: &mut App| {
                let _ = view.update(cx, |this, cx| {
                    this.ask = None;
                    cx.notify();
                });
            })
        };
        let confirm: polterware_ui::Handler = Rc::new(move |_, cx: &mut App| {
            let _ = view.update(cx, |this, cx| this.answer(cx));
        });
        let dialog = match ask {
            Ask::ArchiveProject => ConfirmDialog::new(
                "settings-archive",
                "Archive project?",
                "The project will be hidden from the switcher. Local data stays in SQLite until you reset the store.",
            )
            .confirm_label("Archive"),
            Ask::ResetStore => ConfirmDialog::new(
                "settings-reset",
                "Reset local store?",
                "This deletes local project data and reseeds the demonstration project.",
            )
            .confirm_label("Reset store"),
        }
        .danger()
        .focus(&self.ask_focus)
        .on_cancel(cancel.clone())
        .on_confirm(confirm);
        Some(window_layer(modal_layer(1.0, true, cancel, dialog, &theme)))
    }
}

impl Render for SettingsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_open(window, cx);
        if !self.presence.is_present() {
            return div().into_any_element();
        }
        let theme = *Theme::global(cx);
        let pose = self.presence.frame(window);
        let tab = self.shown;
        let scope = tab.scope();
        let items = SettingsTab::of(scope)
            .map(|item| {
                SettingsNavItem::new(item.id(), item.label())
                    .icon(item.icon())
                    .group(item.group())
            })
            .collect();
        let view = cx.entity().downgrade();
        let select_view = view.clone();
        let close_view = view.clone();
        let open_view = view.clone();
        let query = self.search.read(cx).text().to_string();
        let page = self.page(window, cx);
        let mut dialog = settings_dialog(pose.opacity)
            .pose(pose)
            .items(items)
            .active(tab.id())
            .heading(tab.heading())
            .on_select(Rc::new(move |id: &SharedString, _, cx: &mut App| {
                let Some(tab) = SettingsTab::from_id(id) else {
                    return;
                };
                let _ = select_view.update(cx, |this, cx| this.select(tab, cx));
            }))
            .on_close(Rc::new(move |_, cx: &mut App| {
                let _ = close_view.update(cx, |this, cx| this.close(cx));
            }))
            .search(
                self.search.clone(),
                query,
                Self::search_entries(scope),
                Rc::new(move |entry: &SettingsSearchEntry, _, cx: &mut App| {
                    let Some(tab) = SettingsTab::from_id(&entry.item) else {
                        return;
                    };
                    let _ = open_view.update(cx, |this, cx| {
                        this.search.update(cx, |field, cx| field.set_text("", cx));
                        this.select(tab, cx);
                    });
                }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(settings_group_gap()))
                    .text_size(px(12.5))
                    .child(page),
            );
        for picker in [
            &self.environment_picker,
            &self.idle_picker,
            &self.quit_picker,
        ] {
            if let Some(overlay) = picker.overlay(cx) {
                dialog = dialog.overlay(overlay);
            }
        }
        let scrim_view = view.clone();
        let interactive = self.presence.is_open();
        div()
            .child(modal_layer(
                pose.opacity,
                interactive,
                Rc::new(move |_, cx: &mut App| {
                    let _ = scrim_view.update(cx, |this, cx| this.close(cx));
                }),
                dialog,
                &theme,
            ))
            .child(self.environment_composer.clone())
            .children(self.connect_dialog(window, cx))
            .children(self.ask_dialog(window, cx))
            .into_any_element()
    }
}
