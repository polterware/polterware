//! Settings' state (`SettingsView` and its host in `App.tsx`, the writes in
//! `workspace/persist.ts`, the Electron main process's MCP, core, and preference controllers):
//! the one settings dialog over the page, open on a tab of either scope, Project settings
//! (from the sidebar's footer) or App settings (beside Commander in the title bar), and what
//! its pages read and write.
//!
//! General reads project settings; Agent setup and Hooks read the setup doctor. The MCP helper
//! is a process this app
//! starts ([`crate::mcp`]); the local core is the one [`AppState`] connects to, stopped through
//! `MaintenanceService.Shutdown`; this computer's preferences live in the files the Electron
//! Desktop keeps ([`crate::prefs`]).

use std::path::PathBuf;
use std::process::Child;

use gpui::{App, AppContext as _, Context};
use polterware_ui::icons::Icon;
use polterware_ui::toast::ToastTone;
use polterware_ui::window::app_icon;

use super::{AppState, CoreState, Load, Route, Section};
use crate::client::{self, Core, CoreError};
use crate::contracts::{
    AgentSetup, ArchiveProjectRequest, GetAgentSetupRequest, GetProjectSettingsRequest, Project,
    ProjectSettings, ResetLocalStoreRequest, SaveProjectRequest, SaveProjectSettingsRequest,
    SetAgentsSyncRequest, SetHookSwitchRequest,
};
use crate::mcp::{self, McpClient, McpStatus};
use crate::prefs::{self, AppIcon, AppearancePrefs, CorePrefs, Idle, QuitMode};
use crate::shortcuts::{self, Overrides, OwnedChord};

/// The demonstration project a reset store reseeds (`refreshWorkspace("proj-polter")`).
const DEMO_PROJECT: &str = "proj-polter";

/// Which settings a tab belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsScope {
    Project,
    App,
}

/// The settings pages (`SETTINGS_TABS`), in the rail's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SettingsTab {
    General,
    ProjectDanger,
    AgentsMcp,
    AgentsSetup,
    AgentsHooks,
    AgentsSkill,
    CliReference,
    Shortcuts,
    Appearance,
    LocalCore,
    AppDanger,
}

impl SettingsTab {
    pub const ALL: [SettingsTab; 11] = [
        SettingsTab::General,
        SettingsTab::ProjectDanger,
        SettingsTab::AgentsMcp,
        SettingsTab::AgentsSetup,
        SettingsTab::AgentsHooks,
        SettingsTab::AgentsSkill,
        SettingsTab::CliReference,
        SettingsTab::Shortcuts,
        SettingsTab::Appearance,
        SettingsTab::LocalCore,
        SettingsTab::AppDanger,
    ];

    pub fn id(self) -> &'static str {
        match self {
            SettingsTab::General => "general",
            SettingsTab::ProjectDanger => "project-danger",
            SettingsTab::AgentsMcp => "agents-mcp",
            SettingsTab::AgentsSetup => "agents-setup",
            SettingsTab::AgentsHooks => "agents-hooks",
            SettingsTab::AgentsSkill => "agents-skill",
            SettingsTab::CliReference => "cli-reference",
            SettingsTab::Shortcuts => "shortcuts",
            SettingsTab::Appearance => "appearance",
            SettingsTab::LocalCore => "local-core",
            SettingsTab::AppDanger => "app-danger",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tab| tab.id() == id)
    }

    pub fn label(self) -> &'static str {
        match self {
            SettingsTab::General => "General",
            SettingsTab::ProjectDanger | SettingsTab::AppDanger => "Danger zone",
            SettingsTab::AgentsMcp => "MCP server",
            SettingsTab::AgentsSetup => "Agent setup",
            SettingsTab::AgentsHooks => "Hooks",
            SettingsTab::AgentsSkill => "AI skill",
            SettingsTab::CliReference => "CLI reference",
            SettingsTab::Shortcuts => "Shortcuts",
            SettingsTab::Appearance => "Appearance",
            SettingsTab::LocalCore => "Local core",
        }
    }

    pub fn group(self) -> &'static str {
        match self {
            SettingsTab::General | SettingsTab::ProjectDanger => "Project",
            SettingsTab::AgentsMcp
            | SettingsTab::AgentsSetup
            | SettingsTab::AgentsHooks
            | SettingsTab::AgentsSkill
            | SettingsTab::CliReference => "Agents",
            _ => "App",
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            SettingsTab::General => Icon::Folder,
            SettingsTab::ProjectDanger => Icon::Archive,
            SettingsTab::AgentsMcp => Icon::Plug,
            SettingsTab::AgentsSetup => Icon::Bot,
            SettingsTab::AgentsHooks => Icon::Webhook,
            SettingsTab::AgentsSkill => Icon::BookOpen,
            SettingsTab::CliReference => Icon::Terminal,
            SettingsTab::Shortcuts => Icon::Zap,
            SettingsTab::Appearance => Icon::Sun,
            SettingsTab::LocalCore => Icon::Cpu,
            SettingsTab::AppDanger => Icon::Database,
        }
    }

    /// `settingsTabScope`: the Project group belongs to the open project; the rest apply to
    /// this computer.
    pub fn scope(self) -> SettingsScope {
        if self.group() == "Project" {
            SettingsScope::Project
        } else {
            SettingsScope::App
        }
    }

    /// `settingsTabsFor`.
    pub fn of(scope: SettingsScope) -> impl Iterator<Item = SettingsTab> {
        Self::ALL
            .into_iter()
            .filter(move |tab| tab.scope() == scope)
    }

    /// The panel's heading: `Group / Label`, except for the App group.
    pub fn heading(self) -> String {
        match self.group() {
            "App" => self.label().to_string(),
            group => format!("{group} / {}", self.label()),
        }
    }
}

/// General's reads.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectSettingsData {
    pub settings: ProjectSettings,
}

/// The local core as Local core shows it (`CoreState` in `core-card.tsx`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreRun {
    Running,
    Stopped,
    Starting,
    Stopping,
    Error,
}

impl CoreRun {
    pub fn label(self) -> &'static str {
        match self {
            CoreRun::Running => "Running",
            CoreRun::Stopped => "Stopped",
            CoreRun::Starting => "Starting",
            CoreRun::Stopping => "Stopping",
            CoreRun::Error => "Error",
        }
    }
}

/// Everything settings keeps on [`AppState`].
pub struct SettingsState {
    tab: Option<SettingsTab>,
    project: Load<ProjectSettingsData>,
    setup: Load<AgentSetup>,
    agents_sync_busy: bool,
    mcp_child: Option<Child>,
    mcp_status: Option<McpStatus>,
    mcp_clients: Option<Vec<McpClient>>,
    mcp_busy: bool,
    user_data: PathBuf,
    core_prefs: CorePrefs,
    /// The idle timeout the running core started with.
    applied_idle: Idle,
    quit_mode: QuitMode,
    appearance: AppearancePrefs,
    shortcuts: Overrides,
    stopping: bool,
    core_error: Option<String>,
    /// The app's first connection has reported, so no later one starts a core.
    auto_start_tried: bool,
}

impl SettingsState {
    pub fn new() -> Self {
        let user_data = prefs::user_data_dir();
        let core_prefs = prefs::core_prefs(&user_data);
        Self {
            tab: None,
            project: Load::Idle,
            setup: Load::Idle,
            agents_sync_busy: false,
            mcp_child: None,
            mcp_status: None,
            mcp_clients: None,
            mcp_busy: false,
            applied_idle: core_prefs.idle,
            quit_mode: prefs::quit_mode(&user_data),
            appearance: prefs::appearance_prefs(&user_data),
            shortcuts: shortcuts::decode(&prefs::shortcut_overrides(&user_data)),
            core_prefs,
            user_data,
            stopping: false,
            core_error: None,
            auto_start_tried: false,
        }
    }

    /// The MCP helper this app started, which quitting stops.
    pub fn take_mcp(&mut self) -> Option<Child> {
        self.mcp_child.take()
    }
}

impl Default for SettingsState {
    fn default() -> Self {
        Self::new()
    }
}

/// The message of an error, for a toast.
fn message(error: &CoreError, fallback: &str) -> String {
    match error {
        CoreError::Rpc(text) if !text.trim().is_empty() => text.clone(),
        _ => fallback.to_string(),
    }
}

impl AppState {
    pub fn settings_tab(&self) -> Option<SettingsTab> {
        self.settings.tab
    }

    /// Opens the dialog on `tab`, over the page; a project tab needs an open project.
    pub fn open_settings(&mut self, tab: SettingsTab, cx: &mut Context<Self>) {
        if tab.scope() == SettingsScope::Project && self.project_id().is_none() {
            return;
        }
        self.commander_open = false;
        self.settings.tab = Some(tab);
        self.load_settings_tab(tab, cx);
    }

    pub fn close_settings(&mut self) {
        self.settings.tab = None;
    }

    /// The Projects screen's Project settings: the project, then its settings.
    pub fn open_project_settings(&mut self, id: String, cx: &mut Context<Self>) {
        self.open_project(id, cx);
        self.open_settings(SettingsTab::General, cx);
    }

    fn load_settings_tab(&mut self, tab: SettingsTab, cx: &mut Context<Self>) {
        match tab {
            SettingsTab::General => self.load_project_settings(cx),
            SettingsTab::AgentsSetup | SettingsTab::AgentsHooks => self.load_agent_setup(cx),
            SettingsTab::AgentsMcp => self.refresh_mcp(cx),
            _ => {}
        }
    }

    /// Reads the open tab again, after a change or a reconnect.
    pub(super) fn reload_settings(&mut self, cx: &mut Context<Self>) {
        if let Some(tab) = self.settings.tab
            && tab != SettingsTab::AgentsMcp
        {
            self.load_settings_tab(tab, cx);
        }
    }

    /// Another project opened: what the last one read is dropped.
    pub(super) fn forget_project_settings(&mut self) {
        self.settings.project = Load::Idle;
        self.settings.setup = Load::Idle;
    }

    pub fn project_settings(&self) -> &Load<ProjectSettingsData> {
        &self.settings.project
    }

    pub fn agent_setup(&self) -> &Load<AgentSetup> {
        &self.settings.setup
    }

    pub fn agents_sync_busy(&self) -> bool {
        self.settings.agents_sync_busy
    }

    pub fn open_project_id(&self) -> Option<String> {
        self.project_id().map(str::to_string)
    }

    fn load_project_settings(&mut self, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.open_project_id()) else {
            return;
        };
        if !matches!(self.settings.project, Load::Ready(_)) {
            self.settings.project = Load::Loading;
        }
        let wanted = project.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { read_project_settings(&core, &project) })
                .await;
            let _ = this.update(cx, |state, cx| {
                if state.project_id() != Some(wanted.as_str()) {
                    return;
                }
                state.settings.project = match result {
                    Ok(data) => Load::Ready(data),
                    Err(error) => Load::Failed(state.lost(error, cx)),
                };
                cx.notify();
            });
        })
        .detach();
    }

    pub fn load_agent_setup(&mut self, cx: &mut Context<Self>) {
        let Some(core) = self.ready_core() else {
            return;
        };
        let project_id = self.open_project_id().unwrap_or_default();
        if !matches!(self.settings.setup, Load::Ready(_)) {
            self.settings.setup = Load::Loading;
        }
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    core.agent_run()
                        .get_agent_setup(&GetAgentSetupRequest { project_id })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.settings.setup = match result {
                    Ok(response) => Load::Ready(response.setup.unwrap_or_default()),
                    Err(error) => {
                        let text = message(&error, "Could not read agent setup.");
                        state.lost(error, cx);
                        state.toast(text.clone(), true);
                        Load::Failed(text)
                    }
                };
                cx.notify();
            });
        })
        .detach();
    }

    /// Turns a plugin hook on or off; the switches come back from the core.
    pub fn set_hook(&mut self, name: String, enabled: bool, cx: &mut Context<Self>) {
        let Some(core) = self.ready_core() else {
            return;
        };
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    core.agent_run()
                        .set_hook_switch(&SetHookSwitchRequest { name, enabled })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                match result {
                    Ok(response) => {
                        if let Load::Ready(setup) = &mut state.settings.setup {
                            setup.hooks = response.hooks;
                        }
                    }
                    Err(error) => {
                        state.toast(message(&error, "Could not save the hook switch."), true)
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Writes the project name and description.
    pub fn save_project_general(
        &mut self,
        name: String,
        description: String,
        cx: &mut Context<Self>,
    ) {
        let (Some(core), Some(project), Load::Ready(data)) = (
            self.ready_core(),
            self.open_project_id(),
            &self.settings.project,
        ) else {
            return;
        };
        let current = data.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    core.project().save_project(&SaveProjectRequest {
                        project: Some(Project {
                            id: project.clone(),
                            name,
                            ..Project::default()
                        }),
                    })?;
                    save_settings(&core, &project, &current.settings, &description)
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                match result {
                    Ok(()) => state.push_toast("Project settings saved.", ToastTone::Success),
                    Err(error) => state.toast(message(&error, "Could not save settings."), true),
                }
                state.settings_written(cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// Turns AGENTS.md sync on or off; on writes each checkout's block at once.
    pub fn set_agents_sync(&mut self, enabled: bool, cx: &mut Context<Self>) {
        let (Some(core), Some(project_id)) = (self.ready_core(), self.open_project_id()) else {
            return;
        };
        self.settings.agents_sync_busy = true;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    core.work().set_agents_sync(&SetAgentsSyncRequest {
                        project_id,
                        enabled,
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.settings.agents_sync_busy = false;
                match result {
                    Ok(response) => {
                        let written = response
                            .files
                            .iter()
                            .filter(|file| file.action == "write")
                            .count();
                        state.push_toast(agents_sync_toast(enabled, written), ToastTone::Success);
                    }
                    Err(error) => {
                        state.toast(message(&error, "Could not change AGENTS.md sync."), true)
                    }
                }
                state.settings_written(cx);
                cx.notify();
            });
        })
        .detach();
    }

    pub fn archive_open_project(&mut self, cx: &mut Context<Self>) {
        let (Some(core), Some(id)) = (self.ready_core(), self.open_project_id()) else {
            return;
        };
        let name = self
            .project()
            .map(|project| project.name.clone())
            .unwrap_or_default();
        self.settings.tab = None;
        cx.spawn(async move |this, cx| {
            let archived = id.clone();
            let result = cx
                .background_spawn(async move {
                    core.project().archive_project(&ArchiveProjectRequest {
                        id: archived.clone(),
                    })?;
                    super::read_projects(&core)
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                match result {
                    Ok(projects) => {
                        state.push_toast(
                            format!("Project \"{name}\" archived."),
                            ToastTone::Success,
                        );
                        let next = projects
                            .iter()
                            .find(|project| project.id != id)
                            .map(|project| project.id.clone());
                        state.projects = Load::Ready(projects);
                        match next {
                            Some(next) => state.open_project(next, cx),
                            None => {
                                state.go_home(cx);
                                state.set_new_project(true);
                            }
                        }
                    }
                    Err(error) => state.toast(message(&error, "Could not archive project."), true),
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Deletes the local data and reseeds the demonstration project, then opens it.
    pub fn reset_local_store(&mut self, cx: &mut Context<Self>) {
        let Some(core) = self.ready_core() else {
            return;
        };
        self.settings.tab = None;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    core.maintenance()
                        .reset_local_store(&ResetLocalStoreRequest {})
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                match result {
                    Ok(_) => {
                        state.push_toast(
                            "Local store reset to the demonstration project.",
                            ToastTone::Success,
                        );
                        state.open_project(DEMO_PROJECT.into(), cx);
                        state.reload(cx);
                    }
                    Err(error) => {
                        state.toast(message(&error, "Could not reset the local store."), true)
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn settings_written(&mut self, cx: &mut Context<Self>) {
        self.reload_settings(cx);
        self.load_projects(cx);
        if matches!(self.route, Route::Project { .. }) {
            self.load_catalog(cx);
        }
        if self.section() == Some(Section::Structure) {
            self.load_structure(cx);
        }
    }

    pub fn mcp_status(&self) -> Option<&McpStatus> {
        self.settings.mcp_status.as_ref()
    }

    pub fn mcp_clients(&self) -> Option<&[McpClient]> {
        self.settings.mcp_clients.as_deref()
    }

    pub fn mcp_busy(&self) -> bool {
        self.settings.mcp_busy
    }

    /// Reads the helper's state and the MCP clients this computer has.
    pub fn refresh_mcp(&mut self, cx: &mut Context<Self>) {
        let db = self.db.clone();
        self.settings.mcp_status = Some(mcp::status(self.settings.mcp_child.as_mut(), &db));
        self.settings.mcp_busy = true;
        cx.spawn(async move |this, cx| {
            let clients = cx
                .background_spawn(async move { mcp::list_clients(&db) })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.settings.mcp_busy = false;
                match clients {
                    Ok(clients) => state.settings.mcp_clients = Some(clients),
                    Err(error) => {
                        state.settings.mcp_clients = Some(Vec::new());
                        state.toast(
                            if error.is_empty() {
                                "Could not read the MCP clients.".into()
                            } else {
                                error
                            },
                            true,
                        );
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn start_mcp(&mut self, cx: &mut Context<Self>) {
        if self.settings.mcp_busy {
            return;
        }
        let db = self.db.clone();
        self.settings.mcp_busy = true;
        cx.spawn(async move |this, cx| {
            let start_db = db.clone();
            let started = cx
                .background_spawn(async move { mcp::start(&start_db) })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.settings.mcp_busy = false;
                match started {
                    Ok(child) => {
                        state.settings.mcp_child = Some(child);
                        state.push_toast("MCP helper started.", ToastTone::Success);
                    }
                    Err(error) => state.toast(error, true),
                }
                state.settings.mcp_status =
                    Some(mcp::status(state.settings.mcp_child.as_mut(), &db));
                cx.notify();
            });
        })
        .detach();
    }

    pub fn stop_mcp(&mut self, cx: &mut Context<Self>) {
        let Some(child) = self.settings.mcp_child.take() else {
            self.settings.mcp_status = Some(mcp::status(None, &self.db));
            return;
        };
        let db = self.db.clone();
        self.settings.mcp_busy = true;
        cx.spawn(async move |this, cx| {
            cx.background_spawn(async move { mcp::stop(child) }).await;
            let _ = this.update(cx, |state, cx| {
                state.settings.mcp_busy = false;
                state.settings.mcp_status = Some(mcp::status(None, &db));
                state.push_toast("MCP helper stopped.", ToastTone::Success);
                cx.notify();
            });
        })
        .detach();
    }

    /// Sets this Polterware up in MCP client `id`, or removes it.
    pub fn set_mcp_client(&mut self, id: String, enabled: bool, cx: &mut Context<Self>) {
        if self.settings.mcp_busy {
            return;
        }
        let db = self.db.clone();
        self.settings.mcp_busy = true;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { mcp::set_client(&db, &id, enabled) })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.settings.mcp_busy = false;
                match result {
                    Ok(clients) => {
                        state.settings.mcp_clients = Some(clients);
                        state.push_toast(
                            if enabled {
                                "MCP client set up."
                            } else {
                                "MCP client removed."
                            },
                            ToastTone::Success,
                        );
                    }
                    Err(error) => state.toast(
                        if error.is_empty() {
                            "Could not change the MCP client.".into()
                        } else {
                            error
                        },
                        true,
                    ),
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// The local core as Local core shows it.
    pub fn core_run(&self) -> CoreRun {
        if self.settings.stopping {
            return CoreRun::Stopping;
        }
        match &self.core {
            CoreState::Ready(_) => CoreRun::Running,
            CoreState::Connecting | CoreState::Missing { starting: true } => CoreRun::Starting,
            CoreState::Missing { starting: false } => CoreRun::Stopped,
            CoreState::Failed(_) => CoreRun::Error,
        }
    }

    /// What the last lifecycle action, or the core, reported.
    pub fn core_error(&self) -> Option<String> {
        self.settings.core_error.clone().or(match &self.core {
            CoreState::Failed(text) => Some(text.clone()),
            _ => None,
        })
    }

    pub fn core_prefs(&self) -> CorePrefs {
        self.settings.core_prefs
    }

    /// The idle timeout the running core started with, which a change applies to only on the
    /// next start.
    pub fn applied_idle(&self) -> Idle {
        self.settings.applied_idle
    }

    pub fn quit_mode(&self) -> QuitMode {
        self.settings.quit_mode
    }

    pub fn reduce_transparency(&self) -> bool {
        self.settings.appearance.reduce_transparency
    }

    pub fn app_icon(&self) -> AppIcon {
        self.settings.appearance.app_icon
    }

    pub(super) fn idle_flag(&self) -> &'static str {
        self.settings.core_prefs.idle.flag()
    }

    /// The core started with the saved idle timeout.
    pub(super) fn core_started(&mut self) {
        self.settings.applied_idle = self.settings.core_prefs.idle;
        self.settings.core_error = None;
    }

    /// Whether this is the app's first connection result; later ones never start a core.
    pub(super) fn first_connection(&mut self) -> bool {
        !std::mem::replace(&mut self.settings.auto_start_tried, true)
    }

    /// Start the core when Polterware opens (`autoStart`), when there is a binary to start.
    pub(super) fn auto_start_wanted(&self) -> bool {
        self.settings.core_prefs.auto_start && client::cli_binary().is_some()
    }

    /// Asks the core to shut down (`stopCore`) and waits for it to exit.
    pub fn stop_core(&mut self, cx: &mut Context<Self>) {
        self.shut_down_core(false, cx);
    }

    /// Stops the core, then starts it again with the saved idle timeout.
    pub fn restart_core(&mut self, cx: &mut Context<Self>) {
        self.shut_down_core(true, cx);
    }

    fn shut_down_core(&mut self, restart: bool, cx: &mut Context<Self>) {
        if self.settings.stopping {
            return;
        }
        self.settings.stopping = true;
        self.settings.core_error = None;
        let db = self.db.clone();
        cx.spawn(async move |this, cx| {
            let stopped = cx.background_spawn(async move { client::stop(&db) }).await;
            let _ = this.update(cx, |state, cx| {
                state.settings.stopping = false;
                state.stream_generation += 1;
                match stopped {
                    Ok(()) => {
                        state.core = CoreState::Missing { starting: false };
                        if restart {
                            state.start_core(cx);
                        }
                    }
                    Err(error) => state.settings.core_error = Some(error.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Shows the core's log in Finder (`revealCoreLogs`).
    pub fn reveal_core_logs(&self, cx: &App) {
        cx.reveal_path(&client::core_log(&self.db));
    }

    pub fn set_core_idle(&mut self, idle: Idle) {
        self.settings.core_prefs.idle = idle;
        self.save_core_prefs();
    }

    pub fn set_core_auto_start(&mut self, auto_start: bool) {
        self.settings.core_prefs.auto_start = auto_start;
        self.save_core_prefs();
    }

    fn save_core_prefs(&mut self) {
        if let Err(error) =
            prefs::save_core_prefs(&self.settings.user_data, self.settings.core_prefs)
        {
            self.toast(format!("Could not save the core settings: {error}"), true);
        }
    }

    pub fn set_quit_mode(&mut self, mode: QuitMode) {
        self.settings.quit_mode = mode;
        if let Err(error) = prefs::save_quit_mode(&self.settings.user_data, mode) {
            self.toast(format!("Could not save the quit setting: {error}"), true);
        }
    }

    pub fn set_reduce_transparency(&mut self, reduce: bool) {
        self.settings.appearance.reduce_transparency = reduce;
        self.save_appearance();
    }

    pub fn set_app_icon(&mut self, icon: AppIcon) {
        self.settings.appearance.app_icon = icon;
        self.save_appearance();
        app_icon::apply(icon.override_png(), AppIcon::Default.png());
    }

    fn save_appearance(&mut self) {
        if let Err(error) =
            prefs::save_appearance_prefs(&self.settings.user_data, self.settings.appearance)
        {
            self.toast(format!("Could not save the appearance: {error}"), true);
        }
    }

    pub fn shortcut_overrides(&self) -> &Overrides {
        &self.settings.shortcuts
    }

    /// Remaps a shortcut (`applyChord`); another command of its group with the chord refuses it.
    pub fn set_shortcut(&mut self, id: &str, chord: OwnedChord) {
        if let Err(other) = shortcuts::set_override(&mut self.settings.shortcuts, id, chord) {
            self.toast(
                format!("Conflicts with \u{201C}{}\u{201D}.", other.label),
                true,
            );
            return;
        }
        self.save_shortcuts("Shortcut saved.");
    }

    pub fn reset_shortcut(&mut self, id: &str) {
        self.settings.shortcuts.remove(id);
        self.save_shortcuts("Default shortcut restored.");
    }

    fn save_shortcuts(&mut self, done: &str) {
        let stored = shortcuts::encode(&self.settings.shortcuts);
        match prefs::save_shortcut_overrides(&self.settings.user_data, &stored) {
            Ok(()) => self.push_toast(done.to_string(), ToastTone::Success),
            Err(error) => self.toast(format!("Could not save shortcut: {error}"), true),
        }
    }

    pub fn take_mcp(&mut self) -> Option<Child> {
        self.settings.take_mcp()
    }
}

/// `AGENTS.md sync on: 2 files updated.`
fn agents_sync_toast(enabled: bool, written: usize) -> String {
    if !enabled {
        return "AGENTS.md sync off.".into();
    }
    match written {
        0 => "AGENTS.md sync on.".into(),
        1 => "AGENTS.md sync on: 1 file updated.".into(),
        count => format!("AGENTS.md sync on: {count} files updated."),
    }
}

fn read_project_settings(core: &Core, project_id: &str) -> Result<ProjectSettingsData, CoreError> {
    let settings = core
        .project()
        .get_project_settings(&GetProjectSettingsRequest {
            project_id: project_id.to_string(),
        })?
        .settings
        .unwrap_or_default();
    Ok(ProjectSettingsData { settings })
}

/// `saveSettings` with the project's repository kept as it is.
fn save_settings(
    core: &Core,
    project_id: &str,
    current: &ProjectSettings,
    description: &str,
) -> Result<(), CoreError> {
    core.project()
        .save_project_settings(&SaveProjectSettingsRequest {
            settings: Some(ProjectSettings {
                project_id: project_id.to_string(),
                description: description.to_string(),
                ..current.clone()
            }),
        })
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabs_split_into_their_scopes_in_the_rail_order() {
        let project: Vec<&str> = SettingsTab::of(SettingsScope::Project)
            .map(SettingsTab::id)
            .collect();
        assert_eq!(project, ["general", "project-danger"]);
        assert_eq!(SettingsTab::of(SettingsScope::App).count(), 9);
        assert_eq!(SettingsTab::LocalCore.heading(), "Local core");
        for tab in SettingsTab::ALL {
            assert_eq!(SettingsTab::from_id(tab.id()), Some(tab));
        }
    }

    #[test]
    fn the_sync_toast_counts_the_files_written() {
        assert_eq!(agents_sync_toast(false, 3), "AGENTS.md sync off.");
        assert_eq!(agents_sync_toast(true, 0), "AGENTS.md sync on.");
        assert_eq!(
            agents_sync_toast(true, 1),
            "AGENTS.md sync on: 1 file updated."
        );
        assert_eq!(
            agents_sync_toast(true, 2),
            "AGENTS.md sync on: 2 files updated."
        );
    }
}
