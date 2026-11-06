//! The root view: the title bar over the sidebar and the routed page on the main surface, with
//! the settings dialog, Commander, the shared Composer, and toasts over them. The Projects
//! screen has no sidebar. C (`shell.create`) creates from anywhere in a project, outside a
//! field, a menu, or a dialog: what the open tab creates on Milestones and Knowledge, else a
//! card. Its chords and Commander's follow the remaps Settings › Shortcuts saved.
//!
//! Quitting does what Local core says: asks (Quit, Quit & Stop Core, or Cancel, as the
//! Electron Desktop's question), keeps the core running, or stops it first. The question is
//! drawn in the window, not as gpui's native prompt, since that prompt has no checkbox for
//! Electron's Don't ask again; checked, the answer becomes Local core's When quitting.

use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, FocusHandle, IntoElement, KeyBinding,
    KeyDownEvent, NoAction, Render, Window, actions, div, prelude::*, px,
};
use polterware_ui::list_popover::ListPopover;
use polterware_ui::patterns::{
    AnchorSlot, ConfirmDialog, main_surface, modal_layer, window_anchor,
};
use polterware_ui::text_input::TextInput;
use polterware_ui::theme::Theme;
use polterware_ui::toast::toast_stack_with_actions;

use crate::client;
use crate::prefs::QuitMode;
use crate::shortcuts;
use crate::state::{AppState, ComposeRequest, Load, Route, Section, SettingsTab};
use crate::views::activity::ActivityView;
use crate::views::canvas::CanvasView;
use crate::views::commander::CommanderView;
use crate::views::compose::ComposeView;
use crate::views::inbox::InboxView;
use crate::views::knowledge::KnowledgeView;
use crate::views::milestones::MilestonesView;
use crate::views::new_project::NewProjectView;
use crate::views::projects::ProjectsView;
use crate::views::settings::SettingsView;
use crate::views::sidebar::HelpParts;
use crate::views::structure::StructureView;
use crate::views::titlebar::TitlebarParts;
use crate::views::{sidebar, titlebar};

actions!(
    polterware_gpui,
    [
        Quit,
        CheckForUpdates,
        About,
        Hide,
        HideOthers,
        ShowAll,
        ToggleFullScreen,
        Minimize,
        Zoom,
        CloseWindow,
        ToggleCommander,
        OpenAppSettings,
        CloseOverlay
    ]
);

pub fn register_key_bindings(cx: &mut App) {
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.on_action(|_: &About, _| crate::updater::show_about());
    cx.on_action(|_: &Hide, cx| cx.hide());
    cx.on_action(|_: &HideOthers, cx| cx.hide_other_apps());
    cx.on_action(|_: &ShowAll, cx| cx.unhide_other_apps());
    cx.bind_keys([
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("cmd-h", Hide, None),
        KeyBinding::new("cmd-alt-h", HideOthers, None),
        KeyBinding::new("cmd-ctrl-f", ToggleFullScreen, None),
        KeyBinding::new("cmd-m", Minimize, None),
        KeyBinding::new("cmd-w", CloseWindow, None),
        KeyBinding::new("cmd-k", ToggleCommander, None),
        KeyBinding::new("cmd-,", OpenAppSettings, None),
        KeyBinding::new("escape", CloseOverlay, None),
    ]);
}

pub struct AppView {
    focus_handle: FocusHandle,
    state: Entity<AppState>,
    commander: Entity<CommanderView>,
    commander_was_open: bool,
    projects: Entity<ProjectsView>,
    inbox: Entity<InboxView>,
    activity: Entity<ActivityView>,
    knowledge: Entity<KnowledgeView>,
    milestones: Entity<MilestonesView>,
    structure: Entity<StructureView>,
    canvas: Entity<CanvasView>,
    settings: Entity<SettingsView>,
    compose: Entity<ComposeView>,
    new_project: Entity<NewProjectView>,
    new_project_was_open: bool,
    titlebar: TitlebarParts,
    help: HelpParts,
    /// `POLTERWARE_BENCH`: the file `projects-ready` goes to, until it has been written.
    bench: Option<std::path::PathBuf>,
    /// The keystroke Commander answers, as last bound.
    commander_binding: String,
    /// A quit is asking or stopping the core, so another one waits.
    quitting: bool,
    /// The quit question while it is open: whether Don't ask again is checked.
    quit_ask: Option<bool>,
    quit_focus: FocusHandle,
    /// `[` folded the sidebar away (`shell.toggle_sidebar`).
    sidebar_hidden: bool,
}

impl AppView {
    /// Knowledge's view, which the tour drives.
    pub(crate) fn knowledge(&self) -> &Entity<KnowledgeView> {
        &self.knowledge
    }

    /// The shared Composer, which the tour opens and closes.
    pub(crate) fn compose(&self) -> &Entity<ComposeView> {
        &self.compose
    }

    /// The shell's own chords, which Electron's window listeners take first: ⌘/ shows the
    /// shortcuts (Settings › Shortcuts, as Commander's Keyboard shortcuts opens them), ⌘⇧C
    /// opens Canvas, `[` folds the sidebar (`shell.toggle_sidebar`), and ⌘J closes the open
    /// side panel (`shell.toggle_detail`: Inbox's detail, Milestones' Peek). A bare key is a
    /// field's own text.
    fn shell_shortcut(
        &mut self,
        keys: &gpui::Keystroke,
        overrides: &shortcuts::Overrides,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let typing = window
            .context_stack()
            .iter()
            .any(|context| context.contains("TextInput") || context.contains("Terminal"));
        let modded = keys.modifiers.platform || keys.modifiers.control;
        if typing && !modded {
            return false;
        }
        let app = self.state.read(cx);
        let in_project = matches!(app.route(), Route::Project { .. });
        if shortcuts::matches("shell.show_shortcuts", keys, overrides) {
            let open = app.settings_tab() == Some(SettingsTab::Shortcuts);
            self.state.update(cx, |state, cx| {
                if open {
                    state.close_settings();
                } else {
                    state.open_settings(SettingsTab::Shortcuts, cx);
                }
                cx.notify();
            });
            return true;
        }
        // Over Commander, a dialog, or the settings, the rest are theirs.
        if app.commander_open()
            || app.help_topic().is_some()
            || app.new_project_open()
            || app.settings_tab().is_some()
            || self.quit_ask.is_some()
        {
            return false;
        }
        if shortcuts::matches("shell.open_canvas", keys, overrides) {
            if !in_project {
                return false;
            }
            self.state.update(cx, |state, cx| {
                state.go_section(Section::Canvas, cx);
                cx.notify();
            });
            return true;
        }
        if shortcuts::matches("shell.toggle_sidebar", keys, overrides) {
            if !in_project {
                return false;
            }
            self.sidebar_hidden = !self.sidebar_hidden;
            cx.notify();
            return true;
        }
        if shortcuts::matches("shell.toggle_detail", keys, overrides) {
            let section = match app.route() {
                Route::Project { section, .. } => Some(*section),
                _ => None,
            };
            return match section {
                Some(Section::Inbox) if app.inbox_detail_open() => {
                    self.state.update(cx, |state, cx| {
                        state.close_inbox_detail();
                        cx.notify();
                    });
                    true
                }
                Some(Section::Milestones) => self
                    .milestones
                    .update(cx, |milestones, cx| milestones.close_peek(cx)),
                _ => false,
            };
        }
        false
    }

    /// C (`shell.create`), unless it is typed into a field or a menu, a dialog, or Commander
    /// is open.
    fn create_shortcut(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        /// The key contexts of fields, menus, and dialogs, where C is theirs.
        const TAKEN: [&str; 10] = [
            "TextInput",
            "Composer",
            "Menu",
            "Popover",
            "ListPopover",
            "Commander",
            "Confirm",
            "Morph",
            "PropertyPicker",
            "Terminal",
        ];
        let keys = &event.keystroke;
        let overrides = self.state.read(cx).shortcut_overrides().clone();
        let overrides = &overrides;
        if !event.is_held && self.shell_shortcut(keys, overrides, window, cx) {
            cx.stop_propagation();
            return;
        }
        let create = shortcuts::matches("shell.create", keys, overrides);
        // N adds a structure node on Structure (`structure.add_node`).
        let add_node = shortcuts::matches("structure.add_node", keys, overrides);
        // F opens the page's filters (`shell.open_filter`).
        let open_filter = shortcuts::matches("shell.open_filter", keys, overrides);
        // ⌘B and M belong to an open Milestones board (`work.toggle_layout`, `set_milestone`).
        let toggle_layout = shortcuts::matches("work.toggle_layout", keys, overrides);
        let set_milestone = shortcuts::matches("work.set_milestone", keys, overrides);
        if !(create || add_node || open_filter || toggle_layout || set_milestone) || event.is_held {
            return;
        }
        if window
            .context_stack()
            .iter()
            .any(|context| TAKEN.iter().any(|name| context.contains(name)))
        {
            return;
        }
        let app = self.state.read(cx);
        if app.commander_open()
            || app.help_topic().is_some()
            || app.new_project_open()
            || app.settings_tab().is_some()
        {
            return;
        }
        if toggle_layout || set_milestone {
            let on_milestones = matches!(
                app.route(),
                Route::Project {
                    section: Section::Milestones,
                    ..
                }
            );
            if on_milestones
                && self.milestones.update(cx, |milestones, cx| {
                    if toggle_layout {
                        milestones.key_toggle_layout(cx)
                    } else {
                        milestones.key_set_milestone(window, cx)
                    }
                })
            {
                cx.stop_propagation();
            }
            return;
        }
        if open_filter {
            let filters = match app.route().clone() {
                Route::Project { section, tab, .. } => match section {
                    Section::Inbox => Some(self.inbox.read(cx).filters().clone()),
                    Section::Activity => Some(self.activity.read(cx).filters().clone()),
                    Section::Milestones => Some(self.milestones.read(cx).filters().clone()),
                    Section::Structure => Some(self.structure.read(cx).filters().clone()),
                    Section::Knowledge => Some(self.knowledge.read(cx).filters_for(tab).clone()),
                    Section::Canvas => None,
                },
                _ => Some(self.projects.read(cx).filters().clone()),
            };
            if let Some(filters) = filters
                && filters.update(cx, |filters, cx| filters.open_from_shortcut(cx))
            {
                cx.stop_propagation();
            }
            return;
        }
        let Route::Project { section, tab, .. } = app.route().clone() else {
            return;
        };
        if add_node {
            if section == Section::Structure {
                cx.stop_propagation();
                self.structure
                    .update(cx, |structure, cx| structure.open_create(window, cx));
            }
            return;
        }
        cx.stop_propagation();
        match section {
            Section::Knowledge => self
                .knowledge
                .update(cx, |knowledge, cx| knowledge.compose_here(tab, window, cx)),
            Section::Milestones => self
                .milestones
                .update(cx, |milestones, cx| milestones.compose_card(None, cx)),
            _ => self.state.update(cx, |state, cx| {
                state.compose(
                    ComposeRequest::Card {
                        stage: None,
                        milestone: String::new(),
                    },
                    cx,
                )
            }),
        }
    }

    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let state = cx.new(AppState::new);
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        let commander = cx.new(|cx| CommanderView::new(state.clone(), window, cx));
        let projects = cx.new(|cx| ProjectsView::new(state.clone(), cx));
        let inbox = cx.new(|cx| InboxView::new(state.clone(), cx));
        let new_project = cx.new(|cx| NewProjectView::new(state.clone(), cx));
        let activity = cx.new(|cx| ActivityView::new(state.clone(), cx));
        let knowledge = cx.new(|cx| KnowledgeView::new(state.clone(), cx));
        let milestones = cx.new(|cx| MilestonesView::new(state.clone(), cx));
        let structure = cx.new(|cx| StructureView::new(state.clone(), window, cx));
        let canvas = cx.new(|cx| CanvasView::new(state.clone(), cx));
        let settings = cx.new(|cx| SettingsView::new(state.clone(), window, cx));
        let compose = cx.new(|cx| ComposeView::new(state.clone(), window, cx));
        let switcher = cx.new(ListPopover::new);
        cx.observe(&switcher, |_, _, cx| cx.notify()).detach();
        let switcher_search = cx.new(|cx| {
            TextInput::new(cx)
                .placeholder("Find project\u{2026}")
                .bare()
        });
        // The rows follow the search as it is typed.
        cx.observe(&switcher_search, {
            let state = state.clone();
            let switcher = switcher.clone();
            move |_, search, cx| {
                let query = search.read(cx).text().to_string();
                let rows = titlebar::switcher_rows(&state, &query, cx);
                switcher.update(cx, |popover, cx| {
                    popover.set_rows(rows);
                    cx.notify();
                });
            }
        })
        .detach();
        let help = cx.new(ListPopover::new);
        cx.observe(&help, |_, _, cx| cx.notify()).detach();
        focus_handle.focus(window);
        // The MCP helper this app started stops with it, as the Electron main process stops it.
        cx.on_app_quit({
            let state = state.clone();
            move |_, cx| {
                let child = state.update(cx, |state, _| state.take_mcp());
                async move {
                    if let Some(child) = child {
                        crate::mcp::stop(child);
                    }
                }
            }
        })
        .detach();
        if let Some(dir) = crate::shots::shots_dir() {
            crate::shots::start_tour(dir, state.clone(), cx.weak_entity(), window, cx);
        }
        Self {
            focus_handle,
            state,
            commander,
            commander_was_open: false,
            projects,
            inbox,
            activity,
            knowledge,
            milestones,
            structure,
            canvas,
            settings,
            compose,
            new_project,
            new_project_was_open: false,
            titlebar: TitlebarParts {
                switcher,
                switcher_search,
                switcher_anchor: AnchorSlot::default(),
            },
            help: HelpParts {
                popover: help,
                anchor: AnchorSlot::default(),
            },
            bench: crate::shots::bench_file(),
            commander_binding: "cmd-k".into(),
            quitting: false,
            quit_ask: None,
            quit_focus: cx.focus_handle(),
            sidebar_hidden: false,
        }
    }

    /// Binds Commander to its remapped chord, and turns the chord it had off.
    fn bind_commander(&mut self, cx: &mut Context<Self>) {
        let wanted = shortcuts::command("shell.toggle_commander")
            .map(|command| {
                shortcuts::resolve(command, self.state.read(cx).shortcut_overrides()).keystroke()
            })
            .unwrap_or_else(|| "cmd-k".into());
        if wanted == self.commander_binding {
            return;
        }
        // A later binding wins over an earlier one for the same keystroke.
        cx.bind_keys([
            KeyBinding::new(&self.commander_binding, NoAction, None),
            KeyBinding::new(&wanted, ToggleCommander, None),
        ]);
        self.commander_binding = wanted;
    }

    /// Quit (`before-quit`): as Local core's When quitting Polterware says.
    fn quit(&mut self, _: &Quit, window: &mut Window, cx: &mut Context<Self>) {
        if self.quitting {
            return;
        }
        match self.state.read(cx).quit_mode() {
            QuitMode::Keep => cx.quit(),
            QuitMode::Stop => self.stop_core_and_quit(cx),
            QuitMode::Ask => {
                self.quitting = true;
                self.quit_ask = Some(false);
                self.quit_focus.focus(window);
                cx.notify();
            }
        }
    }

    /// The quit question's answer: `None` cancels; with Don't ask again checked, a quit or a
    /// stop is kept as Local core's When quitting, as Electron's `confirmQuit` keeps it.
    fn answer_quit(&mut self, stop: Option<bool>, cx: &mut Context<Self>) {
        let remember = self.quit_ask.take().unwrap_or(false);
        self.quitting = false;
        cx.notify();
        let Some(stop) = stop else {
            return;
        };
        if remember {
            let mode = if stop { QuitMode::Stop } else { QuitMode::Keep };
            self.state.update(cx, |state, _| state.set_quit_mode(mode));
        }
        if stop {
            self.stop_core_and_quit(cx);
        } else {
            cx.quit();
        }
    }

    fn quit_dialog(&self, checked: bool, cx: &mut Context<Self>) -> AnyElement {
        let view = cx.entity().downgrade();
        let answer = move |stop: Option<bool>| -> polterware_ui::Handler {
            let view = view.clone();
            Rc::new(move |_, cx: &mut App| {
                let _ = view.update(cx, |this, cx| this.answer_quit(stop, cx));
            })
        };
        let toggle = {
            let view = cx.entity().downgrade();
            Rc::new(move |_: &mut Window, cx: &mut App| {
                let _ = view.update(cx, |this, cx| {
                    this.quit_ask = this.quit_ask.map(|checked| !checked);
                    cx.notify();
                });
            })
        };
        ConfirmDialog::new(
            "quit",
            "Quit Polterware?",
            "The core stays on for the CLI and MCP.",
        )
        .confirm_label("Quit")
        .alternate("Quit & Stop Core", answer(Some(true)))
        .check("Don't ask again", checked, toggle)
        .focus(&self.quit_focus)
        .on_cancel(answer(None))
        .on_confirm(answer(Some(false)))
        .into_any_element()
    }

    fn stop_core_and_quit(&mut self, cx: &mut Context<Self>) {
        self.quitting = true;
        let db = self.state.read(cx).database().clone();
        cx.spawn(async move |_, cx| {
            let _ = cx.background_spawn(async move { client::stop(&db) }).await;
            let _ = cx.update(|cx| cx.quit());
        })
        .detach();
    }

    pub fn state(&self) -> &Entity<AppState> {
        &self.state
    }

    /// Opens or closes Switch project, as the title bar's project crumb does.
    pub fn toggle_switcher(&self, window: &mut Window, cx: &mut App) {
        titlebar::open_switcher(
            &self.titlebar.switcher,
            &self.titlebar.switcher_search,
            &self.state,
            window,
            cx,
        );
    }

    /// Opens or closes Help's topics, as the sidebar's "?" does.
    pub fn toggle_help(&self, window: &mut Window, cx: &mut App) {
        sidebar::open_help(&self.help.popover, &self.state, window, cx);
    }

    fn page(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.state.read(cx).route() {
            Route::Projects => self.projects.clone().into_any_element(),
            Route::Project { section, .. } => match section {
                Section::Inbox => self.inbox.clone().into_any_element(),
                Section::Activity => self.activity.clone().into_any_element(),
                Section::Knowledge => self.knowledge.clone().into_any_element(),
                Section::Milestones => self.milestones.clone().into_any_element(),
                Section::Structure => self.structure.clone().into_any_element(),
                Section::Canvas => self.canvas.clone().into_any_element(),
            },
        }
    }

    fn update_state(&self, cx: &mut Context<Self>, f: impl FnOnce(&mut AppState)) {
        self.state.update(cx, |state, cx| {
            f(state);
            cx.notify();
        });
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *Theme::global(cx);
        self.bind_commander(cx);
        let page = self.page(cx);
        let app = self.state.read(cx);
        let toasts = app.toasts().to_vec();
        let toast_actions = app.toast_actions.clone();
        let in_project = matches!(app.route(), Route::Project { .. });
        let commander_open = app.commander_open();
        let new_project_open = app.new_project_open();
        // Reduce transparency: the app background in place of the frosted glass.
        let backdrop = if app.reduce_transparency() {
            theme.color_app_background
        } else {
            two_coats(theme.color_shell_glass)
        };
        // The frame after the one that first draws project rows has been presented.
        if matches!(app.route(), Route::Projects)
            && matches!(app.projects(), Load::Ready(projects) if !projects.is_empty())
            && let Some(file) = self.bench.take()
        {
            window.on_next_frame(move |_, _| crate::shots::mark(&file, "projects-ready"));
        }
        if commander_open && !self.commander_was_open {
            self.commander
                .update(cx, |commander, cx| commander.open(window, cx));
        } else if !commander_open && self.commander_was_open {
            self.commander
                .update(cx, |commander, cx| commander.close(window, cx));
            if !self.compose.read(cx).is_open() {
                self.focus_handle.focus(window);
            }
        }
        self.commander_was_open = commander_open;
        if new_project_open && !self.new_project_was_open {
            self.new_project
                .update(cx, |composer, cx| composer.open(window, cx));
        } else if !new_project_open && self.new_project_was_open {
            self.focus_handle.focus(window);
        }
        self.new_project_was_open = new_project_open;

        let dismiss_state = self.state.clone();
        let dismiss = Rc::new(move |id, _: &mut Window, cx: &mut App| {
            dismiss_state.update(cx, |state, cx| {
                state.dismiss_toast(id);
                cx.notify();
            });
        });

        // `.pw-shell-main-frame`: padding 0 8px 8px 0 under the title bar, 8px on the left
        // too when there is no sidebar.
        let frame = div()
            .flex_1()
            .h_full()
            .min_w(px(0.0))
            .pr(px(theme.space_2))
            .pb(px(theme.space_2))
            .when(!in_project || self.sidebar_hidden, |frame| {
                frame.pl(px(theme.space_2))
            })
            .child(main_surface(page, &theme));
        let mut body = div().flex_1().min_h(px(0.0)).w_full().flex().flex_row();
        if in_project && !self.sidebar_hidden {
            body = body.child(sidebar::sidebar(&self.state, &self.help, cx));
        }
        body = body.child(frame);

        let mut root = div()
            .id("polterware-root")
            .track_focus(&self.focus_handle)
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(backdrop)
            .text_color(theme.color_text_primary)
            .font_family(theme.font_sans)
            .text_size(px(theme.font_size_md))
            .on_key_down(cx.listener(Self::create_shortcut))
            .on_action(cx.listener(|_, _: &CloseWindow, window, _| window.remove_window()))
            .on_action(cx.listener(|this, _: &ToggleCommander, _, cx| {
                this.update_state(cx, |state| {
                    let open = state.commander_open();
                    state.set_commander(!open);
                });
            }))
            .on_action(cx.listener(Self::quit))
            .on_action(cx.listener(|_, _: &ToggleFullScreen, window, _| window.toggle_fullscreen()))
            .on_action(cx.listener(|_, _: &Minimize, window, _| window.minimize_window()))
            .on_action(cx.listener(|_, _: &Zoom, window, _| window.zoom_window()))
            .on_action(cx.listener(|this, _: &OpenAppSettings, _, cx| {
                this.state.update(cx, |state, cx| {
                    state.open_settings(SettingsTab::AgentsMcp, cx);
                    cx.notify();
                });
            }))
            .on_action(cx.listener(|this, _: &CloseOverlay, _, cx| {
                this.update_state(cx, |state| {
                    if state.commander_open() {
                        state.set_commander(false);
                    } else if state.help_topic().is_some() {
                        state.set_help_topic(None);
                    } else if state.new_project_open() {
                        state.close_new_project();
                    } else if state.settings_tab().is_some() {
                        state.close_settings();
                    } else if state.activity_open().is_some() {
                        state.close_change();
                    }
                });
            }))
            .child(titlebar::titlebar(&self.state, &self.titlebar, window, cx))
            .child(body);
        if let Some(anchor) = self.titlebar.switcher_anchor.get()
            && self.titlebar.switcher.read(cx).is_open()
        {
            root = root.child(window_anchor(anchor, self.titlebar.switcher.clone()));
        }
        if let Some(anchor) = self.help.anchor.get()
            && self.help.popover.read(cx).is_open()
        {
            root = root.child(window_anchor(anchor, self.help.popover.clone()));
        }
        if let Some(topic) = self.state.read(cx).help_topic() {
            let close = self.state.clone();
            root = root.child(modal_layer(
                1.0,
                true,
                Rc::new(move |_, cx: &mut App| {
                    close.update(cx, |state, cx| {
                        state.set_help_topic(None);
                        cx.notify();
                    });
                }),
                crate::views::help::help_dialog(&self.state, topic, cx),
                &theme,
            ));
        }
        if new_project_open {
            // A press on the scrim closes it, unless the store is empty.
            let close = self.state.clone();
            root = root.child(modal_layer(
                1.0,
                true,
                Rc::new(move |_, cx: &mut App| {
                    close.update(cx, |state, cx| {
                        state.close_new_project();
                        cx.notify();
                    });
                }),
                self.new_project.clone(),
                &theme,
            ));
        }
        if commander_open {
            let close = self.state.clone();
            root = root.child(modal_layer(
                1.0,
                true,
                Rc::new(move |_, cx: &mut App| {
                    close.update(cx, |state, cx| {
                        state.set_commander(false);
                        cx.notify();
                    });
                }),
                self.commander.clone(),
                &theme,
            ));
        }
        root = root
            .child(self.settings.clone())
            .child(self.compose.clone())
            .child(toast_stack_with_actions(
                &toasts,
                &toast_actions,
                dismiss,
                cx,
            ));
        if let Some(checked) = self.quit_ask {
            let view = cx.entity().downgrade();
            root = root.child(modal_layer(
                1.0,
                true,
                Rc::new(move |_, cx: &mut App| {
                    let _ = view.update(cx, |this, cx| this.answer_quit(None, cx));
                }),
                self.quit_dialog(checked, cx),
                &theme,
            ));
        }
        root
    }
}

/// `.pw-shell-titlebar`, `.pw-shell-sidebar`, and `.pw-shell-main-frame` each paint the shell
/// glass again over `.pw-shell-root`'s, so the material shows through two coats of it. One
/// fill with the alpha of both stands in for them: GPUI adds the alphas of fills drawn over
/// each other, which would leave the second coat opaque.
fn two_coats(glass: gpui::Rgba) -> gpui::Rgba {
    let clear = 1.0 - glass.a;
    gpui::Rgba {
        a: 1.0 - clear * clear,
        ..glass
    }
}
