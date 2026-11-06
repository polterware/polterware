//! Settings' state (`SettingsView` and its host in `App.tsx`, the writes in
//! `workspace/persist.ts`, the Electron main process's MCP, core, and preference controllers):
//! the one settings dialog over the page, open on a tab of either scope, Project settings
//! (from the sidebar's footer) or App settings (beside Commander in the title bar), and what
//! its pages read and write.
//!
//! A project's General and Environments read its settings and environments; Platforms reads
//! the platform CLIs, the project's connections, their imported resources, and their sync
//! runs; Agent setup and Hooks read the setup doctor. The MCP helper is a process this app
//! starts ([`crate::mcp`]); the local core is the one [`AppState`] connects to, stopped through
//! `MaintenanceService.Shutdown`; this computer's preferences live in the files the Electron
//! Desktop keeps ([`crate::prefs`]).

use std::path::PathBuf;
use std::process::Child;

use gpui::{App, AppContext as _, Context};
use polterware_ui::icons::Icon;
use polterware_ui::toast::ToastTone;

use super::{AppState, CoreState, Load, Route, Section, short_id};
use crate::client::{self, Core, CoreError};
use crate::contracts::{
    AgentSetup, ArchiveProjectRequest, CliProvider, DeleteEnvironmentRequest, Environment,
    GetAgentSetupRequest, GetProjectSettingsRequest, IntegrationConnection, IntegrationResource,
    IntegrationSyncRun, ListCliProvidersRequest, ListEnvironmentsRequest,
    ListIntegrationConnectionsRequest, ListIntegrationResourcesRequest,
    ListIntegrationSyncRunsRequest, Project, ProjectSettings, ResetLocalStoreRequest,
    SaveEnvironmentRequest, SaveIntegrationConnectionRequest, SaveProjectRequest,
    SaveProjectSettingsRequest, SetAgentsSyncRequest, SetHookSwitchRequest,
    SyncIntegrationConnectionRequest,
};
use crate::mcp::{self, McpClient, McpStatus};
use crate::prefs::{self, CorePrefs, Idle, QuitMode};
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
    Environments,
    Platforms,
    ProjectDanger,
    AgentsMcp,
    AgentsSetup,
    AgentsHooks,
    AgentsSkill,
    CliReference,
    Shortcuts,
    Appearance,
    LocalCore,
    Privacy,
    AppDanger,
}

impl SettingsTab {
    pub const ALL: [SettingsTab; 14] = [
        SettingsTab::General,
        SettingsTab::Environments,
        SettingsTab::Platforms,
        SettingsTab::ProjectDanger,
        SettingsTab::AgentsMcp,
        SettingsTab::AgentsSetup,
        SettingsTab::AgentsHooks,
        SettingsTab::AgentsSkill,
        SettingsTab::CliReference,
        SettingsTab::Shortcuts,
        SettingsTab::Appearance,
        SettingsTab::LocalCore,
        SettingsTab::Privacy,
        SettingsTab::AppDanger,
    ];

    pub fn id(self) -> &'static str {
        match self {
            SettingsTab::General => "general",
            SettingsTab::Environments => "environments",
            SettingsTab::Platforms => "platforms",
            SettingsTab::ProjectDanger => "project-danger",
            SettingsTab::AgentsMcp => "agents-mcp",
            SettingsTab::AgentsSetup => "agents-setup",
            SettingsTab::AgentsHooks => "agents-hooks",
            SettingsTab::AgentsSkill => "agents-skill",
            SettingsTab::CliReference => "cli-reference",
            SettingsTab::Shortcuts => "shortcuts",
            SettingsTab::Appearance => "appearance",
            SettingsTab::LocalCore => "local-core",
            SettingsTab::Privacy => "privacy",
            SettingsTab::AppDanger => "app-danger",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tab| tab.id() == id)
    }

    pub fn label(self) -> &'static str {
        match self {
            SettingsTab::General => "General",
            SettingsTab::Environments => "Environments",
            SettingsTab::Platforms => "Platforms",
            SettingsTab::ProjectDanger | SettingsTab::AppDanger => "Danger zone",
            SettingsTab::AgentsMcp => "MCP server",
            SettingsTab::AgentsSetup => "Agent setup",
            SettingsTab::AgentsHooks => "Hooks",
            SettingsTab::AgentsSkill => "AI skill",
            SettingsTab::CliReference => "CLI reference",
            SettingsTab::Shortcuts => "Shortcuts",
            SettingsTab::Appearance => "Appearance",
            SettingsTab::LocalCore => "Local core",
            SettingsTab::Privacy => "Privacy",
        }
    }

    pub fn group(self) -> &'static str {
        match self {
            SettingsTab::General
            | SettingsTab::Environments
            | SettingsTab::Platforms
            | SettingsTab::ProjectDanger => "Project",
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
            SettingsTab::Environments => Icon::Globe,
            SettingsTab::Platforms => Icon::ShieldCheck,
            SettingsTab::ProjectDanger => Icon::Archive,
            SettingsTab::AgentsMcp => Icon::Plug,
            SettingsTab::AgentsSetup => Icon::Bot,
            SettingsTab::AgentsHooks => Icon::Webhook,
            SettingsTab::AgentsSkill => Icon::BookOpen,
            SettingsTab::CliReference => Icon::Terminal,
            SettingsTab::Shortcuts => Icon::Zap,
            SettingsTab::Appearance => Icon::Sun,
            SettingsTab::LocalCore => Icon::Cpu,
            SettingsTab::Privacy => Icon::Shield,
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

/// General's and Environments' reads.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectSettingsData {
    pub settings: ProjectSettings,
    pub environments: Vec<Environment>,
}

/// Platforms' reads (`workspace().providers`, `integrations`, `integrationResources`,
/// `syncRuns`), with the environments their connections name.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Platforms {
    pub providers: Vec<CliProvider>,
    pub connections: Vec<IntegrationConnection>,
    pub resources: Vec<IntegrationResource>,
    pub sync_runs: Vec<IntegrationSyncRun>,
    pub environments: Vec<Environment>,
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
    /// Commander's Connect integration: Platforms opens with its connect dialog.
    connect_requested: bool,
    project: Load<ProjectSettingsData>,
    platforms: Load<Platforms>,
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
    reduce_transparency: bool,
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
            connect_requested: false,
            project: Load::Idle,
            platforms: Load::Idle,
            setup: Load::Idle,
            agents_sync_busy: false,
            mcp_child: None,
            mcp_status: None,
            mcp_clients: None,
            mcp_busy: false,
            applied_idle: core_prefs.idle,
            quit_mode: prefs::quit_mode(&user_data),
            reduce_transparency: prefs::reduce_transparency(&user_data),
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
        self.settings.connect_requested = false;
    }

    /// `action.connect-integration`: Platforms, then its connect dialog once it has read.
    pub fn open_connect_integration(&mut self, cx: &mut Context<Self>) {
        self.settings.connect_requested = true;
        self.open_settings(SettingsTab::Platforms, cx);
    }

    /// Whether a connect dialog was asked for, which the Settings view answers once.
    pub fn take_connect_request(&mut self) -> bool {
        std::mem::take(&mut self.settings.connect_requested)
    }

    /// The Projects screen's Project settings: the project, then its settings.
    pub fn open_project_settings(&mut self, id: String, cx: &mut Context<Self>) {
        self.open_project(id, cx);
        self.open_settings(SettingsTab::General, cx);
    }

    fn load_settings_tab(&mut self, tab: SettingsTab, cx: &mut Context<Self>) {
        match tab {
            SettingsTab::General | SettingsTab::Environments => self.load_project_settings(cx),
            SettingsTab::Platforms => self.load_platforms(cx),
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
        self.settings.platforms = Load::Idle;
        self.settings.setup = Load::Idle;
    }

    pub fn project_settings(&self) -> &Load<ProjectSettingsData> {
        &self.settings.project
    }

    pub fn platforms(&self) -> &Load<Platforms> {
        &self.settings.platforms
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

    fn load_platforms(&mut self, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.open_project_id()) else {
            return;
        };
        if !matches!(self.settings.platforms, Load::Ready(_)) {
            self.settings.platforms = Load::Loading;
        }
        let wanted = project.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { read_platforms(&core, &project) })
                .await;
            let _ = this.update(cx, |state, cx| {
                if state.project_id() != Some(wanted.as_str()) {
                    return;
                }
                state.settings.platforms = match result {
                    Ok(platforms) => Load::Ready(platforms),
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

    /// Writes General (`persistProjectSettings`): the name, then the description and default
    /// environment, then each environment's default flag.
    pub fn save_project_general(
        &mut self,
        name: String,
        description: String,
        default_environment: String,
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
                    save_settings(
                        &core,
                        &project,
                        &current.settings,
                        &description,
                        &default_environment,
                    )?;
                    sync_default_flags(&core, &project, &current.environments, &default_environment)
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

    /// Adds an environment (`persistEnvironment`); the first one becomes the default.
    pub fn create_environment(
        &mut self,
        name: String,
        kind: String,
        protection: String,
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
        let label = name.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let id = format!("env-{}", short_id());
                    let is_default = current.environments.is_empty();
                    core.project().save_environment(&SaveEnvironmentRequest {
                        environment: Some(Environment {
                            id: id.clone(),
                            project_id: project.clone(),
                            name,
                            description: format!("{kind} environment"),
                            kind,
                            protection,
                            is_default,
                        }),
                    })?;
                    if is_default {
                        let settings = &current.settings;
                        save_settings(&core, &project, settings, &settings.description, &id)?;
                    }
                    Ok::<(), CoreError>(())
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                match result {
                    Ok(()) => state.push_toast(
                        format!("Environment \"{label}\" saved locally."),
                        ToastTone::Success,
                    ),
                    Err(error) => state.toast(message(&error, "Could not save environment."), true),
                }
                state.settings_written(cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// Makes `id` the default (`persistEnvironmentDefault`).
    pub fn set_default_environment(&mut self, id: String, cx: &mut Context<Self>) {
        let (Some(core), Some(project), Load::Ready(data)) = (
            self.ready_core(),
            self.open_project_id(),
            &self.settings.project,
        ) else {
            return;
        };
        let Some(name) = data
            .environments
            .iter()
            .find(|environment| environment.id == id)
            .map(|environment| environment.name.clone())
        else {
            return;
        };
        let current = data.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    sync_default_flags(&core, &project, &current.environments, &id)?;
                    let settings = &current.settings;
                    save_settings(&core, &project, settings, &settings.description, &id)
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                match result {
                    Ok(()) => state.push_toast(
                        format!("{name} is now the default environment."),
                        ToastTone::Success,
                    ),
                    Err(error) => state.toast(
                        message(&error, "Could not update default environment."),
                        true,
                    ),
                }
                state.settings_written(cx);
                cx.notify();
            });
        })
        .detach();
    }

    pub fn delete_environment(&mut self, id: String, cx: &mut Context<Self>) {
        let (Some(core), Some(project_id)) = (self.ready_core(), self.open_project_id()) else {
            return;
        };
        let name = match &self.settings.project {
            Load::Ready(data) => data
                .environments
                .iter()
                .find(|environment| environment.id == id)
                .map(|environment| environment.name.clone()),
            _ => None,
        };
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    core.project()
                        .delete_environment(&DeleteEnvironmentRequest { id, project_id })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                match result {
                    Ok(_) => state.push_toast(
                        match name {
                            Some(name) => format!("Environment \"{name}\" deleted."),
                            None => "Environment deleted.".into(),
                        },
                        ToastTone::Success,
                    ),
                    Err(error) => {
                        state.toast(message(&error, "Could not delete environment."), true)
                    }
                }
                state.settings_written(cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// Hides the project from the switcher, then opens another one, or the Projects screen
    /// and New project when none is left.
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

    /// Saves this project's connection to `provider` once its CLI session is found
    /// (`persistIntegrationConnection`).
    pub fn connect_platform(
        &mut self,
        provider: CliProvider,
        label: String,
        cx: &mut Context<Self>,
    ) {
        let (Some(core), Some(project_id)) = (self.ready_core(), self.open_project_id()) else {
            return;
        };
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    core.integration().save_integration_connection(
                        &SaveIntegrationConnectionRequest {
                            connection: Some(IntegrationConnection {
                                id: format!("int-{}", short_id()),
                                project_id,
                                provider_id: provider.id.clone(),
                                status: "connected".into(),
                                label: if label.is_empty() {
                                    provider.name.clone()
                                } else {
                                    label
                                },
                                summary: format!("Local {} connection", provider.name),
                                cli_command: provider.binary.clone(),
                                freshness: "unknown".into(),
                                ..IntegrationConnection::default()
                            }),
                        },
                    )
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                if let Err(error) = result {
                    state.toast(message(&error, "Could not save connection."), true);
                }
                state.settings_written(cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// Refreshes a connection's metadata through its CLI (`persistIntegrationSync`).
    pub fn sync_platform(&mut self, connection_id: String, cx: &mut Context<Self>) {
        let Some(core) = self.ready_core() else {
            return;
        };
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    core.integration().sync_integration_connection(
                        &SyncIntegrationConnectionRequest { connection_id },
                    )
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                match result {
                    Ok(response) => {
                        let run = response.run.unwrap_or_default();
                        let ok = run.result == "success";
                        let text = if run.message.is_empty() {
                            if ok {
                                "Integration inventory synced.".to_string()
                            } else {
                                "Integration inventory sync failed.".to_string()
                            }
                        } else {
                            run.message
                        };
                        state.push_toast(
                            text,
                            if ok {
                                ToastTone::Success
                            } else {
                                ToastTone::Danger
                            },
                        );
                    }
                    Err(error) => state.toast(
                        message(&error, "Could not refresh integration metadata."),
                        true,
                    ),
                }
                state.settings_written(cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// After a settings write: the open tab, the projects, and the scope's catalog again.
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
        self.settings.reduce_transparency
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
        self.settings.reduce_transparency = reduce;
        if let Err(error) = prefs::save_reduce_transparency(&self.settings.user_data, reduce) {
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
    let environments = core
        .project()
        .list_environments(&ListEnvironmentsRequest {
            project_id: project_id.to_string(),
        })?
        .environments;
    Ok(ProjectSettingsData {
        settings,
        environments,
    })
}

fn read_platforms(core: &Core, project_id: &str) -> Result<Platforms, CoreError> {
    let providers = core
        .integration()
        .list_cli_providers(&ListCliProvidersRequest::default())?
        .providers;
    let connections = core
        .integration()
        .list_integration_connections(&ListIntegrationConnectionsRequest {
            project_id: project_id.to_string(),
        })?
        .connections;
    // The inventory and its history are extra: a failed read shows none.
    let resources = core
        .integration()
        .list_integration_resources(&ListIntegrationResourcesRequest {
            project_id: project_id.to_string(),
        })
        .map(|response| response.resources)
        .unwrap_or_default();
    let sync_runs = core
        .integration()
        .list_integration_sync_runs(&ListIntegrationSyncRunsRequest {
            project_id: project_id.to_string(),
        })
        .map(|response| response.runs)
        .unwrap_or_default();
    let environments = core
        .project()
        .list_environments(&ListEnvironmentsRequest {
            project_id: project_id.to_string(),
        })
        .map(|response| response.environments)
        .unwrap_or_default();
    Ok(Platforms {
        providers,
        connections,
        resources,
        sync_runs,
        environments,
    })
}

/// `saveSettings` with the project's repository kept as it is.
fn save_settings(
    core: &Core,
    project_id: &str,
    current: &ProjectSettings,
    description: &str,
    default_environment: &str,
) -> Result<(), CoreError> {
    core.project()
        .save_project_settings(&SaveProjectSettingsRequest {
            settings: Some(ProjectSettings {
                project_id: project_id.to_string(),
                description: description.to_string(),
                default_environment_id: default_environment.to_string(),
                repository: current.repository.clone(),
                ..ProjectSettings::default()
            }),
        })
        .map(|_| ())
}

/// `syncEnvironmentDefaultFlags`: only `default` is the default.
fn sync_default_flags(
    core: &Core,
    project_id: &str,
    environments: &[Environment],
    default: &str,
) -> Result<(), CoreError> {
    if default.is_empty() {
        return Ok(());
    }
    for environment in environments {
        let is_default = environment.id == default;
        if environment.is_default == is_default {
            continue;
        }
        core.project().save_environment(&SaveEnvironmentRequest {
            environment: Some(Environment {
                project_id: project_id.to_string(),
                is_default,
                ..environment.clone()
            }),
        })?;
    }
    Ok(())
}

/// `defaultProtectionForKind`.
pub fn default_protection(kind: &str) -> &'static str {
    match kind {
        "production" => "strict",
        "staging" => "standard",
        _ => "relaxed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabs_split_into_their_scopes_in_the_rail_order() {
        let project: Vec<&str> = SettingsTab::of(SettingsScope::Project)
            .map(SettingsTab::id)
            .collect();
        assert_eq!(
            project,
            ["general", "environments", "platforms", "project-danger"]
        );
        assert_eq!(SettingsTab::of(SettingsScope::App).count(), 10);
        assert_eq!(SettingsTab::Platforms.heading(), "Project / Platforms");
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
