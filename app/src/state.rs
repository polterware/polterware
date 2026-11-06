//! The app's state: the route and its history, the core connection, and what has been read
//! from it. Every read runs on the background executor and lands here; the core's change
//! stream reads again what another client (the CLI, an agent, the Electron Desktop) wrote.

use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;
use std::time::Duration;

use gpui::{App, AppContext as _, Context, SharedString};
use polterware_ui::icons::Icon;
use polterware_ui::toast::{Toast, ToastTone};

use crate::client::{self, Core, CoreError};
use crate::contracts::{
    ApplyPolterRequest, ChangeEnvelope, DeleteKnowledgeRequest, Environment, GetAgentRunRequest,
    GetAgentRunResponse, GetProjectSettingsRequest, InboxItem, KnowledgeItem, ListChangesRequest,
    ListEnvironmentsRequest, ListInboxRequest, ListInboxResponse, ListKnowledgeRequest,
    ListProjectsRequest, ListRulesRequest, ListStructureRequest, Project, ProjectChange,
    ProjectSettings, ResolveInboxItemRequest, Rule, SaveEnvironmentRequest, SaveKnowledgeRequest,
    SaveProjectRequest, SaveProjectSettingsRequest, SetKnowledgeArchivedRequest, SetScopeRequest,
    StructureNode, WatchChangesRequest, WatchChangesResponse,
};
use crate::dispatch::{
    self,
    contracts::{DispatchRunEvent, ListHistoryRequest},
};
use crate::format::format_instant;

mod canvas;
mod commander;
pub use commander::{CommanderSelection, CommanderSources, default_tier, load_commander_sources};
mod compose;
mod milestones;
mod scope;
mod settings;
mod structure;
pub use canvas::{CanvasList, CanvasPatch, OpenCanvas};
pub use compose::ComposeRequest;
pub use milestones::{BoardRoute, BoardTab, CardReads, MilestoneAction, Work};
pub use scope::{ScopeCatalog, ScopeKey, ScopeToken};
pub use settings::{
    CoreRun, Platforms, ProjectSettingsData, SettingsScope, SettingsTab, default_protection,
};
pub use structure::{
    DeployInput, DeployTargets, NodeDraft, NodePatch, RecordKind, Structure, list_values,
};

/// How long a burst of change events waits before the reads they ask for.
const CHANGE_DEBOUNCE: Duration = Duration::from_millis(300);
/// How long a dropped change stream waits before it opens again.
const STREAM_RETRY: Duration = Duration::from_secs(2);
/// The name this app's own writes carry in a change event's origins.
const OWN_ORIGIN: &str = "desktop";
/// Activity's tiers before Dispatch names any (`DEFAULT_TIERS`).
pub const DEFAULT_TIERS: [&str; 3] = ["easy", "normal", "hard"];
/// The Dispatch entries Activity lists (`isDispatchHistoryKind`).
const DISPATCH_KINDS: [&str; 3] = ["started", "merged", "failed"];
/// How many Dispatch entries Activity reads (`listHistory`'s limit).
const DISPATCH_LIMIT: i32 = 200;
/// How many changes a page's Activity reads (`DocumentActivity`'s `LIMIT`).
const DOCUMENT_ACTIVITY_LIMIT: i32 = 20;
/// The step id of a Dispatch entry's row starts with this (`dispatchEventRow`).
pub const DISPATCH_ROW: &str = "dispatch:";

/// The project surfaces, in the Electron sidebar's order and groups.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Section {
    Inbox,
    Activity,
    Milestones,
    Canvas,
    Structure,
    Knowledge,
}

impl Section {
    /// The sidebar's sections; Project settings, a dialog, sits in its footer.
    pub const NOW: [Section; 2] = [Section::Inbox, Section::Activity];
    pub const PROJECT: [Section; 4] = [
        Section::Milestones,
        Section::Canvas,
        Section::Structure,
        Section::Knowledge,
    ];
    pub const ALL: [Section; 6] = [
        Section::Inbox,
        Section::Activity,
        Section::Milestones,
        Section::Canvas,
        Section::Structure,
        Section::Knowledge,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Section::Inbox => "Inbox",
            Section::Activity => "Activity",
            Section::Milestones => "Milestones",
            Section::Canvas => "Canvas",
            Section::Structure => "Structure",
            Section::Knowledge => "Knowledge",
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Section::Inbox => Icon::Inbox,
            Section::Activity => Icon::History,
            Section::Milestones => Icon::Briefcase06Icon,
            Section::Canvas => Icon::Pencil,
            Section::Structure => Icon::ChartRelationshipIcon,
            Section::Knowledge => Icon::Layers,
        }
    }
}

/// Knowledge's tabs (`SECTION_TABS.truth`, route tabs `decisions`, `facts`, and `docs`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum KnowledgeTab {
    #[default]
    Decisions,
    Rules,
    Docs,
}

impl KnowledgeTab {
    pub const ALL: [KnowledgeTab; 3] = [
        KnowledgeTab::Decisions,
        KnowledgeTab::Rules,
        KnowledgeTab::Docs,
    ];

    pub fn label(self) -> &'static str {
        match self {
            KnowledgeTab::Decisions => "Decisions",
            KnowledgeTab::Rules => "Rules",
            KnowledgeTab::Docs => "Docs",
        }
    }
}

/// What Knowledge lists: its decisions and docs (`ListKnowledge`), its Rules (`ListRules`),
/// and the Structure nodes their contexts name (`ListStructure`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Knowledge {
    pub items: Vec<KnowledgeItem>,
    pub rules: Vec<Rule>,
    pub nodes: Vec<StructureNode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    /// The Projects screen, reached through Home.
    Projects,
    Project {
        id: String,
        section: Section,
        /// Knowledge's tab; the other sections leave it at its default.
        tab: KnowledgeTab,
        /// Milestones' place: the board open and its layout.
        board: BoardRoute,
        /// The record open as a page in place of the list (a decision's or doc's id, a card's
        /// slug), so Back returns to the list.
        page: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Load<T> {
    Idle,
    Loading,
    Ready(T),
    Failed(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum CoreState {
    Connecting,
    /// No core runs for the database; `starting` while this app starts one.
    Missing {
        starting: bool,
    },
    Ready(Core),
    Failed(String),
}

/// A project as the Projects screen lists it (`mapProjects`): its default environment's
/// name, the description from its settings, and when it last changed, formatted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectRow {
    pub id: String,
    pub name: String,
    /// "active", "paused", or "archived"; a project without one is active.
    pub status: String,
    pub description: String,
    /// The production environment's name, else the first one's, else "No environments".
    pub environments: String,
    /// `updatedAt` as the Electron Desktop prints it.
    pub updated: String,
    /// `updatedAt` itself, which sorts.
    pub updated_at: String,
}

/// An Inbox action, as the core names it (`ResolveInboxItem`).
pub const ACTION_DECIDE: &str = "decide";
pub const ACTION_DELEGATE: &str = "delegate";
pub const ACTION_DEFER: &str = "defer";
pub const ACTION_DISMISS: &str = "ignore";

pub struct AppState {
    db: PathBuf,
    core: CoreState,
    route: Route,
    /// The routes visited, Back and Forward stepping through them (`applyRoute`).
    history: Vec<Route>,
    history_index: usize,
    projects: Load<Vec<ProjectRow>>,
    inbox: Load<ListInboxResponse>,
    /// The Inbox item whose detail is open.
    inbox_open: Option<String>,
    /// The Inbox items picked in the selection column.
    inbox_selected: BTreeSet<String>,
    /// The item an action is being written for, while it is.
    inbox_pending: Option<String>,
    /// The action whose Morph is open in the detail's footer.
    inbox_confirm: Option<String>,
    inbox_row_presses: u64,
    /// The detail shows the active item (`detailPanelStore.open`): it stays closed until a
    /// row is opened.
    inbox_detail: bool,
    /// Knowledge's lists, read when it opens.
    knowledge: Load<Knowledge>,
    /// The project's archived docs (`loadArchivedKnowledge(projectId, "doc")`), read when the
    /// archive opens and again after each restore or delete.
    archived_docs: Load<Vec<KnowledgeItem>>,
    /// The changes that named the open page's record, by its id (`DocumentActivity`).
    document_activity: Option<(String, Vec<ProjectChange>)>,
    /// A decision's move or a doc's archiving is being written.
    knowledge_pending: bool,
    /// Milestones' boards and cards, read when it opens.
    work: Load<Work>,
    /// The scope filter's selection, which every scoped page's filters share.
    scope: Vec<ScopeToken>,
    scope_generation: u64,
    /// Bumped when the scope's node ids change, so a read sent under the old ones is dropped.
    scope_reads: u64,
    /// The open project's Structure nodes, environments, and milestones.
    catalog: Load<ScopeCatalog>,
    catalog_generation: u64,
    /// A milestone or card write is on its way.
    work_pending: bool,
    /// Structure's nodes and what their documents show, read when it opens.
    structure: Load<Structure>,
    /// A component's deploys are being checked.
    deploys_checking: bool,
    /// The targets the deploy Composer's platform lists.
    deploy_targets: Option<DeployTargets>,
    /// Commander asked Structure to open its node Composer.
    add_node_request: bool,
    /// What the open card's page reads besides the card.
    card_reads: CardReads,
    /// Activity's feed: the latest changes for its actor filter.
    activity: Load<Vec<ProjectChange>>,
    activity_actor: ActorFilter,
    /// Polterware Dispatch's entries for the open project (started, merged, failed), which
    /// Activity merges into its feed; none without the Dispatch app.
    activity_dispatch: Vec<DispatchRunEvent>,
    /// The tiers and agents Dispatch's entries have named, for the filters (`knownTiers`,
    /// `knownAgents`): kept across projects, as the web page keeps them while it is mounted.
    dispatch_tiers: Vec<String>,
    dispatch_agents: Vec<String>,
    /// The change open in Activity's detail (its step id, `dispatch:<id>` for a Dispatch
    /// entry), which nothing opens on its own.
    activity_open: Option<String>,
    /// The open change's run, for its audit chain.
    activity_run: Option<GetAgentRunResponse>,
    activity_presses: u64,
    /// The sidebar groups collapsed by their label.
    collapsed_groups: HashSet<&'static str>,
    commander_open: bool,
    /// The Help topic whose dialog is open (an index into `HELP_TOPICS`).
    help_topic: Option<usize>,
    /// New project's Composer is open.
    new_project: bool,
    /// What the shared Composer is asked to open for next.
    compose_request: Option<ComposeRequest>,
    toasts: Vec<Toast>,
    next_toast: u64,
    pub toast_actions: std::collections::HashMap<u64, polterware_ui::toast::ToastAction>,
    /// A change asked for reads that wait out the debounce.
    refresh_due: bool,
    /// Bumped by each connection, so an old connection's stream stops.
    stream_generation: u64,
    /// The settings dialog and what its pages read.
    settings: settings::SettingsState,
    canvas: canvas::CanvasState,
}

impl AppState {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut state = Self {
            db: client::database_path(),
            core: CoreState::Connecting,
            route: Route::Projects,
            history: vec![Route::Projects],
            history_index: 0,
            projects: Load::Idle,
            inbox: Load::Idle,
            inbox_open: None,
            inbox_selected: BTreeSet::new(),
            inbox_pending: None,
            inbox_confirm: None,
            inbox_row_presses: 0,
            inbox_detail: false,
            knowledge: Load::Idle,
            archived_docs: Load::Idle,
            document_activity: None,
            knowledge_pending: false,
            work: Load::Idle,
            scope: Vec::new(),
            scope_generation: 0,
            scope_reads: 0,
            catalog: Load::Idle,
            catalog_generation: 0,
            work_pending: false,
            structure: Load::Idle,
            deploys_checking: false,
            deploy_targets: None,
            add_node_request: false,
            card_reads: CardReads::default(),
            activity: Load::Idle,
            activity_actor: ActorFilter::All,
            activity_dispatch: Vec::new(),
            dispatch_tiers: DEFAULT_TIERS.iter().map(|tier| tier.to_string()).collect(),
            dispatch_agents: Vec::new(),
            activity_open: None,
            activity_run: None,
            activity_presses: 0,
            collapsed_groups: HashSet::new(),
            commander_open: false,
            help_topic: None,
            new_project: false,
            compose_request: None,
            toasts: Vec::new(),
            next_toast: 1,
            toast_actions: Default::default(),
            refresh_due: false,
            stream_generation: 0,
            settings: settings::SettingsState::new(),
            canvas: canvas::CanvasState::default(),
        };
        state.connect(cx);
        state
    }

    pub fn route(&self) -> &Route {
        &self.route
    }

    pub fn core(&self) -> &CoreState {
        &self.core
    }

    pub fn database(&self) -> &PathBuf {
        &self.db
    }

    pub fn projects(&self) -> &Load<Vec<ProjectRow>> {
        &self.projects
    }

    pub fn inbox(&self) -> &Load<ListInboxResponse> {
        &self.inbox
    }

    pub fn inbox_open(&self) -> Option<&str> {
        self.inbox_open.as_deref()
    }

    pub fn inbox_selected(&self) -> &BTreeSet<String> {
        &self.inbox_selected
    }

    pub fn inbox_pending(&self) -> bool {
        self.inbox_pending.is_some()
    }

    pub fn inbox_confirm(&self) -> Option<&str> {
        self.inbox_confirm.as_deref()
    }

    pub fn group_collapsed(&self, label: &str) -> bool {
        self.collapsed_groups.contains(label)
    }

    pub fn commander_open(&self) -> bool {
        self.commander_open
    }

    pub fn toasts(&self) -> &[Toast] {
        &self.toasts
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.history.len()
    }

    /// The open project, when the route is in one.
    pub fn project(&self) -> Option<&ProjectRow> {
        let id = self.project_id()?;
        match &self.projects {
            Load::Ready(projects) => projects.iter().find(|project| project.id == id),
            _ => None,
        }
    }

    fn project_id(&self) -> Option<&str> {
        match &self.route {
            Route::Project { id, .. } => Some(id),
            Route::Projects => None,
        }
    }

    pub fn section(&self) -> Option<Section> {
        match &self.route {
            Route::Project { section, .. } => Some(*section),
            _ => None,
        }
    }

    /// Finds the core that owns the database, then reads the projects.
    pub fn connect(&mut self, cx: &mut Context<Self>) {
        self.core = CoreState::Connecting;
        let db = self.db.clone();
        cx.spawn(async move |this, cx| {
            let found = cx
                .background_spawn(async move { client::discover(&db) })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.set_core(found, cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// Starts `polterware serve` for the database, then reads the projects.
    pub fn start_core(&mut self, cx: &mut Context<Self>) {
        self.core = CoreState::Missing { starting: true };
        let db = self.db.clone();
        let idle = self.idle_flag();
        cx.spawn(async move |this, cx| {
            let started = cx
                .background_spawn(async move { client::start(&db, idle) })
                .await;
            let _ = this.update(cx, |state, cx| {
                if started.is_ok() {
                    state.core_started();
                }
                state.set_core(started, cx);
                cx.notify();
            });
        })
        .detach();
    }

    fn set_core(&mut self, found: Result<Core, CoreError>, cx: &mut Context<Self>) {
        let first = self.first_connection();
        match found {
            Ok(core) => {
                self.core = CoreState::Ready(core.clone());
                self.reload(cx);
                self.watch(core, cx);
            }
            Err(CoreError::NotRunning) => {
                self.core = CoreState::Missing { starting: false };
                // Start the core when Polterware opens, as Local core asks.
                if first && self.auto_start_wanted() {
                    self.start_core(cx);
                }
            }
            Err(CoreError::Rpc(message)) => self.core = CoreState::Failed(message),
        }
    }

    fn ready_core(&self) -> Option<Core> {
        match &self.core {
            CoreState::Ready(core) => Some(core.clone()),
            _ => None,
        }
    }

    /// A call failed because the core went away: look for it again.
    fn lost(&mut self, error: CoreError, cx: &mut Context<Self>) -> String {
        if error == CoreError::NotRunning {
            self.core = CoreState::Missing { starting: false };
            self.connect(cx);
        }
        error.to_string()
    }

    /// Follows the core's changes (`ChangeService.WatchChanges`) for as long as this
    /// connection lasts, opening the stream again when it drops.
    fn watch(&mut self, core: Core, cx: &mut Context<Self>) {
        self.stream_generation += 1;
        let generation = self.stream_generation;
        cx.spawn(async move |this, cx| {
            let mut after_seq = 0;
            loop {
                let watcher = core.clone();
                let opened = cx
                    .background_spawn(async move {
                        watcher
                            .change()
                            .watch_changes(&WatchChangesRequest { after_seq })
                    })
                    .await;
                if let Ok(mut stream) = opened {
                    loop {
                        let (back, next) = cx
                            .background_spawn(async move {
                                let next = stream.next();
                                (stream, next)
                            })
                            .await;
                        stream = back;
                        let Some(Ok(event)) = next else {
                            break;
                        };
                        after_seq = event.seq.max(after_seq);
                        let alive = this
                            .update(cx, |state, cx| {
                                if state.stream_generation != generation {
                                    return false;
                                }
                                state.changed(&event, cx);
                                true
                            })
                            .unwrap_or(false);
                        if !alive {
                            return;
                        }
                    }
                }
                cx.background_executor().timer(STREAM_RETRY).await;
                let current = this
                    .update(cx, |state, _| state.stream_generation == generation)
                    .unwrap_or(false);
                if !current {
                    return;
                }
            }
        })
        .detach();
    }

    /// A change event: a write by another client reads the projects and the open page again,
    /// once a burst of events has settled.
    fn changed(&mut self, event: &WatchChangesResponse, cx: &mut Context<Self>) {
        let own =
            !event.origins.is_empty() && event.origins.iter().all(|origin| origin == OWN_ORIGIN);
        if own && !event.resync {
            return;
        }
        if self.refresh_due {
            return;
        }
        self.refresh_due = true;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(CHANGE_DEBOUNCE).await;
            let _ = this.update(cx, |state, cx| {
                state.refresh_due = false;
                state.reload(cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// Reads the projects again, and the open page's data.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.load_projects(cx);
        if matches!(self.route, Route::Project { .. }) {
            self.load_inbox(cx);
            self.load_catalog(cx);
        }
        if self.section() == Some(Section::Activity) {
            self.load_activity(cx);
        }
        if self.section() == Some(Section::Knowledge) {
            self.load_knowledge(cx);
        }
        if self.section() == Some(Section::Milestones) {
            self.load_work(cx);
        }
        if self.section() == Some(Section::Structure) {
            self.load_structure(cx);
        }
        if self.section() == Some(Section::Canvas) {
            self.load_canvases(cx);
        }
        self.reload_settings(cx);
    }

    fn load_projects(&mut self, cx: &mut Context<Self>) {
        let Some(core) = self.ready_core() else {
            return;
        };
        if !matches!(self.projects, Load::Ready(_)) {
            self.projects = Load::Loading;
        }
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { read_projects(&core) })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.projects = match result {
                    Ok(projects) => Load::Ready(projects),
                    Err(error) => Load::Failed(state.lost(error, cx)),
                };
                cx.notify();
            });
        })
        .detach();
    }

    /// Reads the open project's Inbox; it also feeds the sidebar's count.
    pub fn load_inbox(&mut self, cx: &mut Context<Self>) {
        let (Some(core), Route::Project { id, .. }) = (self.ready_core(), &self.route) else {
            return;
        };
        let project = id.clone();
        let (scope_nodes, reads) = (self.scope_node_ids(), self.scope_reads);
        if !matches!(self.inbox, Load::Ready(_)) {
            self.inbox = Load::Loading;
        }
        cx.spawn(async move |this, cx| {
            let wanted = project.clone();
            let result = cx
                .background_spawn(async move {
                    core.work().list_inbox(&ListInboxRequest {
                        project_id: project,
                        scope_nodes,
                        ..ListInboxRequest::default()
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                // A read for a project that is no longer open, or another scope, is dropped.
                if !matches!(&state.route, Route::Project { id, .. } if *id == wanted)
                    || state.scope_reads != reads
                {
                    return;
                }
                state.inbox = match result {
                    Ok(inbox) => {
                        state.keep_inbox_open(&inbox);
                        Load::Ready(inbox)
                    }
                    Err(error) => Load::Failed(state.lost(error, cx)),
                };
                cx.notify();
            });
        })
        .detach();
    }

    /// The first item in triage order starts active, with the detail closed; an item that
    /// left the Inbox hands over to the one now at its place, which opens it (`markActive`).
    /// Picks of items that left are dropped.
    fn keep_inbox_open(&mut self, inbox: &ListInboxResponse) {
        let ordered = inbox_order(&inbox.items);
        let previous = match &self.inbox {
            Load::Ready(previous) => inbox_order(&previous.items)
                .iter()
                .map(|item| item.id.clone())
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };
        let still = self
            .inbox_open
            .as_ref()
            .is_some_and(|id| ordered.iter().any(|item| &item.id == id));
        if !still {
            let at = self
                .inbox_open
                .as_ref()
                .and_then(|id| previous.iter().position(|previous| previous == id))
                .unwrap_or(0);
            let handover = self.inbox_open.is_some();
            self.inbox_open = ordered
                .get(at.min(ordered.len().saturating_sub(1)))
                .map(|item| item.id.clone());
            self.inbox_confirm = None;
            if handover {
                self.inbox_detail = self.inbox_open.is_some();
            }
        }
        self.inbox_selected
            .retain(|id| ordered.iter().any(|item| &item.id == id));
    }

    /// Opens item `id` in the detail, as a click on its row does.
    pub fn open_inbox_item(&mut self, id: String) {
        if self.inbox_open.as_ref() != Some(&id) {
            self.inbox_confirm = None;
        }
        self.inbox_open = Some(id);
        self.inbox_detail = true;
        self.inbox_row_presses += 1;
    }

    /// Hides the detail; its item stays the active row.
    pub fn close_inbox_detail(&mut self) {
        self.inbox_detail = false;
        self.inbox_confirm = None;
    }

    /// Whether the detail shows the active item.
    pub fn inbox_detail_open(&self) -> bool {
        self.inbox_detail && self.inbox_open.is_some()
    }

    pub fn toggle_inbox_selected(&mut self, id: &str) {
        if !self.inbox_selected.remove(id) {
            self.inbox_selected.insert(id.to_string());
        }
        self.inbox_row_presses += 1;
    }

    /// Counts the presses on Inbox rows (opening an item or picking one), which keep the
    /// detail open where another click outside it closes it (`data-detail-opener`).
    pub fn inbox_row_presses(&self) -> u64 {
        self.inbox_row_presses
    }

    pub fn set_inbox_selected(&mut self, ids: impl IntoIterator<Item = String>) {
        self.inbox_selected = ids.into_iter().collect();
    }

    pub fn set_inbox_confirm(&mut self, action: Option<String>) {
        self.inbox_confirm = action;
    }

    /// Runs `action` on each of `items` (`ResolveInboxItem`, acting as "desktop"), then reads
    /// the Inbox again. A failure stays on a toast; the confirm stays open to retry.
    pub fn resolve_inbox(
        &mut self,
        items: Vec<String>,
        action: &str,
        answer: String,
        cx: &mut Context<Self>,
    ) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        if items.is_empty() || self.inbox_pending.is_some() {
            return;
        }
        let project = project.to_string();
        let action = action.to_string();
        self.inbox_pending = items.first().cloned();
        cx.spawn(async move |this, cx| {
            let many = items.len() > 1;
            let written = items.clone();
            let request_action = action.clone();
            let result = cx
                .background_spawn(async move {
                    for item in written {
                        core.work().resolve_inbox_item(&ResolveInboxItemRequest {
                            project_id: project.clone(),
                            item_id: item,
                            action: request_action.clone(),
                            answer: answer.clone(),
                            session_label: "desktop".into(),
                        })?;
                    }
                    Ok::<(), CoreError>(())
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.inbox_pending = None;
                match result {
                    Ok(()) => {
                        state.inbox_confirm = None;
                        if many {
                            state.inbox_selected.clear();
                        }
                        state.load_inbox(cx);
                    }
                    Err(error) => {
                        let message = state.lost(error, cx);
                        state.toast(message, true);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn activity(&self) -> &Load<Vec<ProjectChange>> {
        &self.activity
    }

    pub fn knowledge(&self) -> &Load<Knowledge> {
        &self.knowledge
    }

    /// Knowledge's tab, and the record open as a page, on a Knowledge route.
    pub fn knowledge_route(&self) -> Option<(KnowledgeTab, Option<&str>)> {
        match &self.route {
            Route::Project {
                section: Section::Knowledge,
                tab,
                page,
                ..
            } => Some((*tab, page.as_deref())),
            _ => None,
        }
    }

    /// Shows Knowledge's `tab`, a step of history (`setTab`).
    pub fn set_knowledge_tab(&mut self, tab: KnowledgeTab, cx: &mut Context<Self>) {
        let Some(id) = self.project_id().map(str::to_string) else {
            return;
        };
        self.navigate(
            Route::Project {
                id,
                section: Section::Knowledge,
                tab,
                board: BoardRoute::default(),
                page: None,
            },
            cx,
        );
    }

    /// Opens record `id` of Knowledge's tab as a page in place of the list (`navigateToDetail`).
    pub fn open_knowledge_page(&mut self, id: String, cx: &mut Context<Self>) {
        let Some((tab, _)) = self.knowledge_route() else {
            return;
        };
        let Some(project) = self.project_id().map(str::to_string) else {
            return;
        };
        self.navigate(
            Route::Project {
                id: project,
                section: Section::Knowledge,
                tab,
                board: BoardRoute::default(),
                page: Some(id),
            },
            cx,
        );
    }

    /// Reads Knowledge's decisions and docs, its Rules, and the Structure nodes they name.
    pub fn load_knowledge(&mut self, cx: &mut Context<Self>) {
        let (Some(core), Route::Project { id, .. }) = (self.ready_core(), &self.route) else {
            return;
        };
        let project = id.clone();
        let (scope_nodes, reads) = (self.scope_node_ids(), self.scope_reads);
        if !matches!(self.knowledge, Load::Ready(_)) {
            self.knowledge = Load::Loading;
        }
        cx.spawn(async move |this, cx| {
            let wanted = project.clone();
            let result = cx
                .background_spawn(async move { read_knowledge(&core, &project, scope_nodes) })
                .await;
            let _ = this.update(cx, |state, cx| {
                if !matches!(&state.route, Route::Project { id, .. } if *id == wanted)
                    || state.scope_reads != reads
                {
                    return;
                }
                state.knowledge = match result {
                    Ok(knowledge) => Load::Ready(knowledge),
                    Err(error) => Load::Failed(state.lost(error, cx)),
                };
                cx.notify();
            });
        })
        .detach();
    }

    /// The changes that named record `subject`, newest first; none until they are read.
    pub fn document_activity(&self, subject: &str) -> &[ProjectChange] {
        match &self.document_activity {
            Some((id, changes)) if id == subject => changes,
            _ => &[],
        }
    }

    pub fn knowledge_pending(&self) -> bool {
        self.knowledge_pending
    }

    /// Reads the last 20 changes that named `subject` (`DocumentActivity`); a failure leaves
    /// none, as the page hides the section then.
    pub fn load_document_activity(&mut self, subject: String, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let project = project.to_string();
        cx.spawn(async move |this, cx| {
            let wanted = subject.clone();
            let result = cx
                .background_spawn(async move {
                    core.agent_run().list_changes(&ListChangesRequest {
                        project_id: project,
                        subject_id: subject,
                        limit: DOCUMENT_ACTIVITY_LIMIT,
                        ..ListChangesRequest::default()
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                let changes = result.map(|response| response.changes).unwrap_or_default();
                state.document_activity = Some((wanted, changes));
                cx.notify();
            });
        })
        .detach();
    }

    /// Writes an edit of a decision or a doc (`persistKnowledgeTitle`, `persistKnowledgeBody`,
    /// `persistKnowledgeTags`) under `summary` with no reason. The lists show it at once, as
    /// the web keeps an override, without reading Knowledge again; the page's Activity is.
    pub fn save_knowledge_item(
        &mut self,
        item: KnowledgeItem,
        summary: &'static str,
        cx: &mut Context<Self>,
    ) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let project = project.to_string();
        if let Load::Ready(knowledge) = &mut self.knowledge
            && let Some(slot) = knowledge.items.iter_mut().find(|slot| slot.id == item.id)
        {
            *slot = item.clone();
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let subject = item.id.clone();
            let result = cx
                .background_spawn(async move {
                    core.knowledge().save_knowledge(&SaveKnowledgeRequest {
                        item: Some(KnowledgeItem {
                            project_id: project,
                            ..item
                        }),
                        change: Some(change(summary, "")),
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                match result {
                    Ok(_) => state.load_document_activity(subject, cx),
                    Err(error) => {
                        let message = state.lost(error, cx);
                        state.toast(message, true);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Sets the Structure nodes a decision or a doc is about (`persistKnowledgeScope`), none
    /// being the whole project. The record shows them at once; Knowledge is read again after
    /// the write (`refreshWorkspace`), which also undoes them when the core refuses, with a
    /// toast.
    pub fn set_knowledge_scope(&mut self, id: String, nodes: Vec<String>, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let project = project.to_string();
        if let Load::Ready(knowledge) = &mut self.knowledge
            && let Some(item) = knowledge.items.iter_mut().find(|item| item.id == id)
        {
            item.scope_node_ids = nodes.clone();
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { knowledge_scope(&core, project, id, nodes) })
                .await;
            let _ = this.update(cx, |state, cx| {
                if let Err(error) = result {
                    let message = state.lost(error, cx);
                    state.toast(message, true);
                }
                state.load_knowledge(cx);
            });
        })
        .detach();
    }

    pub fn archived_docs(&self) -> &Load<Vec<KnowledgeItem>> {
        &self.archived_docs
    }

    /// Reads the archived docs, newest first as the core lists them.
    pub fn load_archived_docs(&mut self, cx: &mut Context<Self>) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        let project = project.to_string();
        if !matches!(self.archived_docs, Load::Ready(_)) {
            self.archived_docs = Load::Loading;
        }
        cx.spawn(async move |this, cx| {
            let wanted = project.clone();
            let result = cx
                .background_spawn(async move {
                    core.knowledge().list_knowledge(&ListKnowledgeRequest {
                        project_id: project,
                        kind: "doc".into(),
                        archived: true,
                        ..ListKnowledgeRequest::default()
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                if state.project_id() != Some(wanted.as_str()) {
                    return;
                }
                state.archived_docs = match result {
                    Ok(response) => Load::Ready(response.items),
                    Err(error) => Load::Failed(state.lost(error, cx)),
                };
                cx.notify();
            });
        })
        .detach();
    }

    /// Restores an archived doc (`persistKnowledgeArchived(id, false)` under "Restore: …"),
    /// then reads Knowledge and the archive again.
    pub fn restore_doc(&mut self, id: String, title: String, cx: &mut Context<Self>) {
        self.archive_write(
            format!("Restored “{title}”"),
            move |core| {
                core.knowledge()
                    .set_knowledge_archived(&SetKnowledgeArchivedRequest {
                        id,
                        archived: false,
                        change: Some(change(&format!("Restore: {title}"), "")),
                    })
                    .map(|_| ())
            },
            cx,
        );
    }

    /// Deletes an archived doc for good (`persistKnowledgeDeletePermanently` under "Delete
    /// permanently: …"), then reads the archive again.
    pub fn delete_doc(&mut self, id: String, title: String, cx: &mut Context<Self>) {
        self.archive_write(
            format!("Deleted “{title}”"),
            move |core| {
                core.knowledge()
                    .delete_knowledge(&DeleteKnowledgeRequest {
                        id,
                        change: Some(change(&format!("Delete permanently: {title}"), "")),
                    })
                    .map(|_| ())
            },
            cx,
        );
    }

    /// One write from the archive (`runArchiveAction`): busy while it runs, then Knowledge and
    /// the archive read again and `done` as the toast, or the error.
    fn archive_write(
        &mut self,
        done: String,
        write: impl FnOnce(&Core) -> Result<(), CoreError> + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        let Some(core) = self.ready_core() else {
            return;
        };
        if self.knowledge_pending {
            return;
        }
        self.knowledge_pending = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx.background_spawn(async move { write(&core) }).await;
            let _ = this.update(cx, |state, cx| {
                state.knowledge_pending = false;
                match result {
                    Ok(()) => {
                        state.load_knowledge(cx);
                        state.push_toast(done, ToastTone::Info);
                    }
                    Err(error) => {
                        let message = state.lost(error, cx);
                        state.toast(message, true);
                    }
                }
                state.load_archived_docs(cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// Creates a decision or a doc from the Composer (`createDecision`, `createDoc`): the item
    /// under its summary with the Composer's reason, then its Context when it has one, and
    /// Knowledge read again. `done` hears whether it was written, so a draft that failed
    /// stays in the Composer.
    pub fn create_knowledge(
        &mut self,
        record: NewRecord,
        done: impl FnOnce(bool, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            done(false, cx);
            return;
        };
        let project = project.to_string();
        let title = record.title.clone();
        let tab = if record.decision {
            KnowledgeTab::Decisions
        } else {
            KnowledgeTab::Docs
        };
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let (item, summary) = record.item(&project);
                    let id = item.id.clone();
                    let created = id.clone();
                    core.knowledge().save_knowledge(&SaveKnowledgeRequest {
                        item: Some(item),
                        change: Some(change(&summary, &record.why)),
                    })?;
                    if !record.nodes.is_empty() {
                        knowledge_scope(&core, project.clone(), id, record.nodes)?;
                    }
                    Ok((project, created))
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                let written = match result {
                    Ok((project, id)) => {
                        state.load_knowledge(cx);
                        // Open shows the record's page on its tab (`navigateToDetail`).
                        let weak = cx.entity().downgrade();
                        state.push_action_toast(
                            format!("Created “{title}”"),
                            ToastTone::Success,
                            "Open",
                            std::rc::Rc::new(move |_, cx| {
                                let (project, id) = (project.clone(), id.clone());
                                let _ = weak.update(cx, |state, cx| {
                                    if state.project_id() != Some(project.as_str()) {
                                        return;
                                    }
                                    state.navigate(
                                        Route::Project {
                                            id: project,
                                            section: Section::Knowledge,
                                            tab,
                                            board: BoardRoute::default(),
                                            page: Some(id),
                                        },
                                        cx,
                                    );
                                    cx.notify();
                                });
                            }),
                            cx,
                        );
                        true
                    }
                    Err(error) => {
                        let message = state.lost(error, cx);
                        state.toast(message, true);
                        false
                    }
                };
                done(written, cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// Writes a Rule as Polter text (`rules-panel.tsx`'s `write`): `text` under `summary` with
    /// `rationale`, then the Rules read again and `written` as the toast. `done` hears whether
    /// it went through.
    pub fn write_rule(
        &mut self,
        text: String,
        summary: String,
        rationale: String,
        written: &'static str,
        done: impl FnOnce(bool, &mut App) + 'static,
        cx: &mut Context<Self>,
    ) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            done(false, cx);
            return;
        };
        let project = project.to_string();
        self.knowledge_pending = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    core.work().apply_polter(&ApplyPolterRequest {
                        project_id: project,
                        text,
                        change: Some(change(&summary, &rationale)),
                        session_label: "desktop".into(),
                        ..ApplyPolterRequest::default()
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.knowledge_pending = false;
                let ok = match result {
                    Ok(_) => {
                        state.load_knowledge(cx);
                        state.push_toast(written, ToastTone::Success);
                        true
                    }
                    Err(error) => {
                        let message = state.lost(error, cx);
                        state.toast(message, true);
                        false
                    }
                };
                done(ok, cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// Moves a decision through its lifecycle with a Polter decision block
    /// (`persistDecisionAction`); the core refuses an illegal move, which a toast shows.
    pub fn move_decision(
        &mut self,
        block: String,
        summary: String,
        rationale: String,
        cx: &mut Context<Self>,
    ) {
        let (Some(core), Some(project)) = (self.ready_core(), self.project_id()) else {
            return;
        };
        if self.knowledge_pending {
            return;
        }
        let project = project.to_string();
        self.knowledge_pending = true;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    core.work().apply_polter(&ApplyPolterRequest {
                        project_id: project,
                        text: format!("---\npolter: 1\n---\n\n{block}\n"),
                        change: Some(change(&summary, &rationale)),
                        session_label: "desktop".into(),
                        ..ApplyPolterRequest::default()
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.knowledge_pending = false;
                match result {
                    Ok(_) => {
                        state.load_knowledge(cx);
                        state.push_toast("Decision updated.", ToastTone::Success);
                    }
                    Err(error) => {
                        let message = state.lost(error, cx);
                        state.toast(message, true);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Archives a doc with its reason (`persistKnowledgeArchived`); the page goes back to the
    /// list it left, which no longer holds it.
    pub fn archive_knowledge(
        &mut self,
        id: String,
        title: String,
        rationale: String,
        cx: &mut Context<Self>,
    ) {
        let Some(core) = self.ready_core() else {
            return;
        };
        if self.knowledge_pending {
            return;
        }
        self.knowledge_pending = true;
        cx.spawn(async move |this, cx| {
            let summary = format!("Archive: {title}");
            let result = cx
                .background_spawn(async move {
                    core.knowledge()
                        .set_knowledge_archived(&SetKnowledgeArchivedRequest {
                            id,
                            archived: true,
                            change: Some(change(&summary, &rationale)),
                        })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                state.knowledge_pending = false;
                match result {
                    Ok(_) => {
                        if let Some((tab, Some(_))) = state.knowledge_route() {
                            state.set_knowledge_tab(tab, cx);
                        }
                        state.load_knowledge(cx);
                        state.push_toast(format!("Archived “{title}”"), ToastTone::Info);
                    }
                    Err(error) => {
                        let message = state.lost(error, cx);
                        state.toast(message, true);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn activity_actor(&self) -> ActorFilter {
        self.activity_actor
    }

    pub fn activity_open(&self) -> Option<&str> {
        self.activity_open.as_deref()
    }

    /// Dispatch's entries for the open project, as Dispatch lists them.
    pub fn activity_dispatch(&self) -> &[DispatchRunEvent] {
        &self.activity_dispatch
    }

    /// The tiers Activity's Tier filter offers: the defaults, then those Dispatch named.
    pub fn dispatch_tiers(&self) -> &[String] {
        &self.dispatch_tiers
    }

    /// The agents Activity's Agent filter offers, as Dispatch named them.
    pub fn dispatch_agents(&self) -> &[String] {
        &self.dispatch_agents
    }

    /// The open change's run, once it has been read.
    pub fn activity_run(&self) -> Option<&GetAgentRunResponse> {
        self.activity_run.as_ref()
    }

    /// Shows the changes `actor` made, read again.
    pub fn set_activity_actor(&mut self, actor: ActorFilter, cx: &mut Context<Self>) {
        if self.activity_actor != actor {
            self.activity_actor = actor;
            self.load_activity(cx);
        }
    }

    /// Reads the open project's latest 50 changes (`listChanges`), keeping the open one
    /// while it is still there, and Dispatch's entries beside them.
    pub fn load_activity(&mut self, cx: &mut Context<Self>) {
        let (Some(core), Route::Project { id, .. }) = (self.ready_core(), &self.route) else {
            return;
        };
        let project = id.clone();
        self.load_dispatch_history(project.clone(), cx);
        let actor = self.activity_actor;
        let (scope_nodes, reads) = (self.scope_node_ids(), self.scope_reads);
        if !matches!(self.activity, Load::Ready(_)) {
            self.activity = Load::Loading;
        }
        cx.spawn(async move |this, cx| {
            let wanted = project.clone();
            let result = cx
                .background_spawn(async move {
                    core.agent_run().list_changes(&ListChangesRequest {
                        project_id: project,
                        actor_kind: actor.kind().to_string(),
                        limit: 50,
                        scope_nodes,
                        ..ListChangesRequest::default()
                    })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                if !matches!(&state.route, Route::Project { id, .. } if *id == wanted)
                    || state.activity_actor != actor
                    || state.scope_reads != reads
                {
                    return;
                }
                state.activity = match result {
                    Ok(response) => {
                        // A Dispatch entry stays open while Dispatch still lists it.
                        let open = state
                            .activity_open
                            .clone()
                            .filter(|id| !id.starts_with(DISPATCH_ROW));
                        let still = open.as_ref().and_then(|id| {
                            response.changes.iter().find(|change| &change.step_id == id)
                        });
                        match (still, &open) {
                            (Some(change), _) => {
                                let run = change.agent_run_id.clone();
                                state.load_change_run(run, cx);
                            }
                            (None, Some(_)) => {
                                state.activity_open = None;
                                state.activity_run = None;
                            }
                            (None, None) => {}
                        }
                        Load::Ready(response.changes)
                    }
                    Err(error) => Load::Failed(state.lost(error, cx)),
                };
                cx.notify();
            });
        })
        .detach();
    }

    /// Reads Polterware Dispatch's latest entries for `project` (`ListHistory`), starting its
    /// core first when the app is installed and none runs. Without the app, or when its core
    /// does not answer, Activity lists none.
    fn load_dispatch_history(&mut self, project: String, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let wanted = project.clone();
            let events = cx
                .background_spawn(async move {
                    let Some(dispatch) = dispatch::connect() else {
                        return Vec::new();
                    };
                    dispatch
                        .dispatch()
                        .list_history(&ListHistoryRequest {
                            project_id: project,
                            limit: DISPATCH_LIMIT,
                            ..ListHistoryRequest::default()
                        })
                        .map(|response| history_entries(response.events))
                        .unwrap_or_default()
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                if !matches!(&state.route, Route::Project { id, .. } if *id == wanted) {
                    return;
                }
                for event in &events {
                    learn(&mut state.dispatch_tiers, &event.tier);
                    learn(&mut state.dispatch_agents, &event.agent);
                }
                state.activity_dispatch = events;
                cx.notify();
            });
        })
        .detach();
    }

    /// Opens change `step` in the detail, as a click on its row does, and reads its run.
    pub fn open_change(&mut self, step: String, cx: &mut Context<Self>) {
        let run = match &self.activity {
            Load::Ready(changes) => changes
                .iter()
                .find(|change| change.step_id == step)
                .map(|change| change.agent_run_id.clone()),
            _ => None,
        };
        if self.activity_open.as_ref() != Some(&step) {
            self.activity_run = None;
        }
        self.activity_open = Some(step);
        self.activity_presses += 1;
        if let Some(run) = run {
            self.load_change_run(run, cx);
        }
    }

    pub fn close_change(&mut self) {
        self.activity_open = None;
        self.activity_run = None;
    }

    /// Counts the presses on Activity's rows, which keep its detail open.
    pub fn activity_presses(&self) -> u64 {
        self.activity_presses
    }

    fn load_change_run(&mut self, run: String, cx: &mut Context<Self>) {
        let Some(core) = self.ready_core() else {
            return;
        };
        if run.is_empty() {
            return;
        }
        cx.spawn(async move |this, cx| {
            let wanted = run.clone();
            let result = cx
                .background_spawn(async move {
                    core.agent_run()
                        .get_agent_run(&GetAgentRunRequest { id: run })
                })
                .await;
            let _ = this.update(cx, |state, cx| {
                let open_run = match (&state.activity, &state.activity_open) {
                    (Load::Ready(changes), Some(open)) => changes
                        .iter()
                        .find(|change| &change.step_id == open)
                        .map(|change| change.agent_run_id.clone()),
                    _ => None,
                };
                if open_run.as_deref() != Some(wanted.as_str()) {
                    return;
                }
                match result {
                    Ok(response) => state.activity_run = Some(response),
                    Err(error) => {
                        state.activity_run = None;
                        let message = state.lost(error, cx);
                        state.toast(message, true);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn toggle_group(&mut self, label: &'static str) {
        if !self.collapsed_groups.remove(label) {
            self.collapsed_groups.insert(label);
        }
    }

    /// Moves to `route` and reads what it shows; a route other than the one in history pushes
    /// it, dropping the forward routes (`applyRoute`).
    fn navigate(&mut self, route: Route, cx: &mut Context<Self>) {
        self.enter(route.clone(), cx);
        let base = self.history_index + 1;
        self.history.truncate(base);
        if self.history.last() != Some(&route) {
            self.history.push(route);
        }
        self.history_index = self.history.len() - 1;
    }

    fn enter(&mut self, route: Route, cx: &mut Context<Self>) {
        let project = |route: &Route| match route {
            Route::Project { id, .. } => Some(id.clone()),
            _ => None,
        };
        if project(&route) != project(&self.route) {
            self.inbox = Load::Idle;
            self.inbox_open = None;
            self.inbox_detail = false;
            self.inbox_selected.clear();
            self.inbox_confirm = None;
            self.activity = Load::Idle;
            self.activity_dispatch.clear();
            self.activity_open = None;
            self.activity_run = None;
            self.knowledge = Load::Idle;
            self.archived_docs = Load::Idle;
            self.work = Load::Idle;
            self.card_reads = CardReads::default();
            self.structure = Load::Idle;
            self.deploy_targets = None;
            self.forget_project_settings();
            self.forget_canvases();
            self.clear_scope();
        }
        let was_knowledge = self.section() == Some(Section::Knowledge);
        let was_milestones = self.section() == Some(Section::Milestones);
        let was_structure = self.section() == Some(Section::Structure);
        let was_canvas = self.section() == Some(Section::Canvas);
        self.route = route;
        self.commander_open = false;
        if matches!(self.route, Route::Project { .. }) {
            self.load_inbox(cx);
            if matches!(self.catalog, Load::Idle) {
                self.load_catalog(cx);
            }
        }
        if self.section() == Some(Section::Activity) {
            self.load_activity(cx);
        }
        // Moving between Knowledge's tabs and pages reads nothing again.
        if self.section() == Some(Section::Knowledge)
            && (!was_knowledge || !matches!(self.knowledge, Load::Ready(_)))
        {
            self.load_knowledge(cx);
        }
        // Moving between boards and cards reads only the card page's own reads again.
        if self.section() == Some(Section::Milestones) {
            if !was_milestones || !matches!(self.work, Load::Ready(_)) {
                self.load_work(cx);
            } else {
                self.keep_card_reads(cx);
            }
        }
        // Moving between nodes reads nothing again.
        if self.section() == Some(Section::Structure)
            && (!was_structure || !matches!(self.structure, Load::Ready(_)))
        {
            self.load_structure(cx);
        }
        if self.section() == Some(Section::Canvas)
            && (!was_canvas || !matches!(self.canvas_list(), Load::Ready(_)))
        {
            self.load_canvases(cx);
        }
    }

    pub fn go_back(&mut self, cx: &mut Context<Self>) {
        if self.history_index == 0 {
            return;
        }
        self.history_index -= 1;
        let route = self.history[self.history_index].clone();
        self.enter(route, cx);
    }

    pub fn go_forward(&mut self, cx: &mut Context<Self>) {
        if !self.can_go_forward() {
            return;
        }
        self.history_index += 1;
        let route = self.history[self.history_index].clone();
        self.enter(route, cx);
    }

    pub fn go_home(&mut self, cx: &mut Context<Self>) {
        self.navigate(Route::Projects, cx);
    }

    pub fn open_project(&mut self, id: String, cx: &mut Context<Self>) {
        self.navigate(
            Route::Project {
                id,
                section: Section::Inbox,
                tab: KnowledgeTab::default(),
                board: BoardRoute::default(),
                page: None,
            },
            cx,
        );
    }

    /// Opens a section of the open project. Choosing the section you are in returns to it.
    pub fn go_section(&mut self, section: Section, cx: &mut Context<Self>) {
        let Some(id) = self.project_id().map(str::to_string) else {
            return;
        };
        // Choosing Canvas again closes the open canvas and shows the list (`homeKey`).
        if section == Section::Canvas && self.section() == Some(Section::Canvas) {
            self.close_canvas(cx);
        }
        self.navigate(
            Route::Project {
                id,
                section,
                tab: KnowledgeTab::default(),
                board: BoardRoute::default(),
                page: None,
            },
            cx,
        );
    }

    pub fn set_commander(&mut self, open: bool) {
        self.commander_open = open;
    }

    pub fn help_topic(&self) -> Option<usize> {
        self.help_topic
    }

    pub fn set_help_topic(&mut self, topic: Option<usize>) {
        self.help_topic = topic;
    }

    pub fn new_project_open(&self) -> bool {
        self.new_project
    }

    pub fn set_new_project(&mut self, open: bool) {
        self.new_project = open;
    }

    /// An empty store keeps New project open: there is nothing to go back to.
    pub fn can_close_new_project(&self) -> bool {
        !matches!(&self.projects, Load::Ready(projects) if projects.is_empty())
    }

    pub fn close_new_project(&mut self) {
        if self.can_close_new_project() {
            self.new_project = false;
        }
    }

    /// Creates the project named `name` with `environments` (kinds, the first the default),
    /// as `persistCreatedProject` does, then enters it with a toast. A failure stays on a
    /// toast and the Composer stays open to retry.
    pub fn create_project(
        &mut self,
        name: String,
        environments: Vec<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(core) = self.ready_core() else {
            return;
        };
        cx.spawn(async move |this, cx| {
            let written = name.clone();
            let result = cx
                .background_spawn(
                    async move { persist_created_project(&core, &written, &environments) },
                )
                .await;
            let _ = this.update(cx, |state, cx| {
                match result {
                    Ok(id) => {
                        state.new_project = false;
                        state.push_toast(format!("Created {name}."), ToastTone::Success);
                        state.reload(cx);
                        state.open_project(id, cx);
                    }
                    Err(error) => {
                        let message = state.lost(error, cx);
                        state.toast(message, true);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn toast(&mut self, message: impl Into<SharedString>, danger: bool) {
        let tone = if danger {
            ToastTone::Danger
        } else {
            ToastTone::Info
        };
        self.push_toast(message, tone);
    }

    pub fn push_toast(&mut self, message: impl Into<SharedString>, tone: ToastTone) {
        let id = self.next_toast;
        self.next_toast += 1;
        self.toasts.push(Toast {
            id,
            message: message.into(),
            tone,
        });
    }

    /// A toast with one action (Undo, Open canvas), gone after `TOAST_ACTION_LIFETIME` as
    /// Electron's `toast.tsx` drops it.
    pub fn push_action_toast(
        &mut self,
        message: impl Into<SharedString>,
        tone: ToastTone,
        label: impl Into<SharedString>,
        on_click: polterware_ui::Handler,
        cx: &mut Context<Self>,
    ) {
        self.push_toast(message, tone);
        let id = self.next_toast - 1;
        self.toast_actions.insert(
            id,
            polterware_ui::toast::ToastAction {
                label: label.into(),
                on_click,
            },
        );
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(polterware_ui::toast::TOAST_ACTION_LIFETIME)
                .await;
            let _ = this.update(cx, |state, cx| {
                if state.toasts.iter().any(|toast| toast.id == id) {
                    state.dismiss_toast(id);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub fn dismiss_toast(&mut self, id: u64) {
        self.toasts.retain(|toast| toast.id != id);
        self.toast_actions.remove(&id);
    }
}

/// A decision or a doc the Composer creates.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NewRecord {
    pub decision: bool,
    pub title: String,
    /// A decision's Context paragraph, or a doc's Markdown.
    pub body: String,
    pub nodes: Vec<String>,
    pub why: String,
}

impl NewRecord {
    /// The item `persistKnowledgeCreate` saves, and the summary its change goes under: a
    /// decision proposed with the ADR's sections and the Context typed, a doc owned by a
    /// person with the Markdown as its body.
    pub fn item(&self, project: &str) -> (KnowledgeItem, String) {
        let (kind, prefix) = if self.decision {
            ("decision", "decision")
        } else {
            ("doc", "doc")
        };
        let body = if self.decision {
            format!(
                "## Status\n\nProposed\n\n## Context\n\n{}\n\n## Decision\n\n\n\n## Consequences\n\n",
                self.body
            )
        } else {
            self.body.clone()
        };
        let summary = if self.decision {
            format!("Create decision: {}", self.title)
        } else {
            format!("Add doc: {}", self.title)
        };
        let item = KnowledgeItem {
            id: new_id(prefix),
            project_id: project.to_string(),
            title: self.title.clone(),
            body,
            kind: kind.into(),
            doc_type: if self.decision { "" } else { "doc" }.into(),
            owner: "human".into(),
            status: if self.decision { "proposed" } else { "" }.into(),
            extra_json: "{}".into(),
            ..KnowledgeItem::default()
        };
        (item, summary)
    }
}

/// `newId`: the prefix and eight hex digits.
fn new_id(prefix: &str) -> String {
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    hasher.write_u128(now);
    format!("{prefix}-{:08x}", hasher.finish() as u32)
}

/// `WorkService.SetScope` for a decision or a doc.
fn knowledge_scope(
    core: &Core,
    project_id: String,
    id: String,
    nodes: Vec<String>,
) -> Result<(), CoreError> {
    core.work()
        .set_scope(&SetScopeRequest {
            project_id,
            kind: "knowledge".into(),
            id,
            nodes,
        })
        .map(|_| ())
}

/// A person's change: its summary and reason, under no agent run.
fn change(summary: &str, rationale: &str) -> ChangeEnvelope {
    ChangeEnvelope {
        summary: summary.to_string(),
        rationale: rationale.to_string(),
        agent_run_id: String::new(),
    }
}

/// Knowledge's three reads, made together as the web workspace makes them.
/// Reads Knowledge under `scope_nodes` (none: the whole project); the Structure nodes it
/// names stay the project's.
fn read_knowledge(
    core: &Core,
    project: &str,
    scope_nodes: Vec<String>,
) -> Result<Knowledge, CoreError> {
    let items = core
        .knowledge()
        .list_knowledge(&ListKnowledgeRequest {
            project_id: project.to_string(),
            scope_nodes: scope_nodes.clone(),
            ..ListKnowledgeRequest::default()
        })?
        .items;
    let rules = core
        .work()
        .list_rules(&ListRulesRequest {
            project_id: project.to_string(),
            scope_nodes,
            ..ListRulesRequest::default()
        })?
        .rules;
    let nodes = core
        .work()
        .list_structure(&ListStructureRequest {
            project_id: project.to_string(),
            ..ListStructureRequest::default()
        })?
        .nodes;
    Ok(Knowledge {
        items,
        rules,
        nodes,
    })
}

/// The entries Activity lists from Dispatch's history (`mapDispatchHistory`): those with an id
/// that started, merged, or failed.
fn history_entries(events: Vec<DispatchRunEvent>) -> Vec<DispatchRunEvent> {
    events
        .into_iter()
        .filter(|event| !event.id.is_empty() && DISPATCH_KINDS.contains(&event.kind.as_str()))
        .collect()
}

/// Adds `value` to `known` once, after the others.
fn learn(known: &mut Vec<String>, value: &str) {
    if !value.is_empty() && !known.iter().any(|seen| seen == value) {
        known.push(value.to_string());
    }
}

/// Activity's tabs: whose changes it lists (`ChangeActorFilter`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorFilter {
    All,
    Human,
    Agent,
}

impl ActorFilter {
    pub const ALL: [ActorFilter; 3] = [ActorFilter::All, ActorFilter::Human, ActorFilter::Agent];

    pub fn label(self) -> &'static str {
        match self {
            ActorFilter::All => "All",
            ActorFilter::Human => "Human",
            ActorFilter::Agent => "Agent",
        }
    }

    /// `ListChanges.actor_kind`: empty for both.
    pub fn kind(self) -> &'static str {
        match self {
            ActorFilter::All => "",
            ActorFilter::Human => "human",
            ActorFilter::Agent => "agent",
        }
    }
}

/// The Inbox's sections in triage order (`KIND_GROUPS`): the kind, its section's label, its
/// row's label, and its icon.
pub const INBOX_KINDS: [(&str, &str, &str, Icon); 4] = [
    ("conflict", "Conflicts", "Conflict", Icon::AlertDiamond),
    ("deploy", "Deploys", "Deploy", Icon::Cloud),
    ("decision", "Questions", "Question", Icon::Scale),
    ("setup", "Setup", "Setup", Icon::ShieldCheck),
];

/// The Inbox in display order: grouped by kind in triage order, open before deferred within
/// each group, and otherwise as the core listed them.
pub fn inbox_order(items: &[InboxItem]) -> Vec<&InboxItem> {
    let mut ordered = Vec::with_capacity(items.len());
    for (kind, ..) in INBOX_KINDS {
        for deferred in [false, true] {
            ordered.extend(
                items
                    .iter()
                    .filter(|item| item.kind == kind && (item.state == "deferred") == deferred),
            );
        }
    }
    ordered
}

/// Every project with its default environment and its settings' description, as
/// `loadProjectMetadata` reads them.
/// `persistCreatedProject`: the project (`proj-` and 8 hex digits), an environment per kind
/// (`{project}-{kind}`, production named main, protection by kind, the first the default),
/// and the settings pointing at that default. Returns the project's id.
fn persist_created_project(
    core: &Core,
    name: &str,
    environments: &[String],
) -> Result<String, CoreError> {
    let project_id = format!("proj-{}", short_id());
    core.project().save_project(&SaveProjectRequest {
        project: Some(Project {
            id: project_id.clone(),
            name: name.to_string(),
            ..Project::default()
        }),
    })?;
    let mut default_environment = None;
    for (index, kind) in environments.iter().enumerate() {
        let environment_id = format!("{project_id}-{kind}");
        core.project().save_environment(&SaveEnvironmentRequest {
            environment: Some(Environment {
                id: environment_id.clone(),
                project_id: project_id.clone(),
                name: if kind == "production" {
                    "main".into()
                } else {
                    kind.clone()
                },
                kind: kind.clone(),
                protection: match kind.as_str() {
                    "production" => "strict",
                    "staging" => "standard",
                    _ => "relaxed",
                }
                .into(),
                is_default: index == 0,
                description: format!("{kind} environment"),
            }),
        })?;
        default_environment.get_or_insert(environment_id);
    }
    core.project()
        .save_project_settings(&SaveProjectSettingsRequest {
            settings: Some(ProjectSettings {
                project_id: project_id.clone(),
                default_environment_id: default_environment.unwrap_or_default(),
                ..ProjectSettings::default()
            }),
        })?;
    Ok(project_id)
}

/// Eight random hex digits (`crypto.randomUUID().slice(0, 8)`).
fn short_id() -> String {
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    hasher.write_u128(now);
    format!("{:08x}", hasher.finish() as u32)
}

fn read_projects(core: &Core) -> Result<Vec<ProjectRow>, CoreError> {
    let projects = core
        .project()
        .list_projects(&ListProjectsRequest::default())?
        .projects;
    let mut rows = Vec::with_capacity(projects.len());
    for project in projects {
        let environments = core
            .project()
            .list_environments(&ListEnvironmentsRequest {
                project_id: project.id.clone(),
            })
            .map(|response| response.environments)
            .unwrap_or_default();
        let summary = environments
            .iter()
            .find(|environment| environment.kind == "production")
            .or(environments.first())
            .map(|environment| environment.name.clone())
            .unwrap_or_else(|| "No environments".into());
        let description = core
            .project()
            .get_project_settings(&GetProjectSettingsRequest {
                project_id: project.id.clone(),
            })
            .ok()
            .and_then(|response| response.settings)
            .map(|settings| settings.description)
            .unwrap_or_default();
        rows.push(ProjectRow {
            name: if project.name.is_empty() {
                project.id.clone()
            } else {
                project.name.clone()
            },
            status: if project.status.is_empty() {
                "active".into()
            } else {
                project.status.clone()
            },
            description,
            environments: summary,
            updated: format_instant(&project.updated_at),
            updated_at: project.updated_at,
            id: project.id,
        });
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::{NewRecord, history_entries, inbox_order, learn};
    use crate::contracts::InboxItem;
    use crate::dispatch::contracts::DispatchRunEvent;

    #[test]
    fn activity_keeps_the_dispatch_entries_that_started_merged_or_failed() {
        let event = |id: &str, kind: &str| DispatchRunEvent {
            id: id.into(),
            kind: kind.into(),
            ..DispatchRunEvent::default()
        };
        let kept: Vec<String> = history_entries(vec![
            event("e1", "started"),
            event("e2", "output"),
            event("e3", "merged"),
            event("", "failed"),
            event("e5", "failed"),
            event("e6", "queued"),
        ])
        .into_iter()
        .map(|event| event.id)
        .collect();
        assert_eq!(kept, ["e1", "e3", "e5"]);
    }

    #[test]
    fn the_filters_learn_each_tier_and_agent_once_after_the_others() {
        let mut tiers: Vec<String> = super::DEFAULT_TIERS.iter().map(|t| t.to_string()).collect();
        for tier in ["hard", "auto", "", "auto", "easy"] {
            learn(&mut tiers, tier);
        }
        assert_eq!(tiers, ["easy", "normal", "hard", "auto"]);
    }

    fn item(id: &str, kind: &str, state: &str) -> InboxItem {
        InboxItem {
            id: id.into(),
            kind: kind.into(),
            state: state.into(),
            ..InboxItem::default()
        }
    }

    #[test]
    fn the_inbox_groups_by_kind_with_open_items_first() {
        let items = vec![
            item("q1", "decision", "deferred"),
            item("c1", "conflict", "open"),
            item("q2", "decision", "open"),
            item("s1", "setup", "open"),
        ];
        let order: Vec<&str> = inbox_order(&items)
            .iter()
            .map(|item| item.id.as_str())
            .collect();
        assert_eq!(order, ["c1", "q2", "q1", "s1"]);
    }

    #[test]
    fn a_new_decision_is_proposed_with_its_sections_and_a_doc_keeps_its_markdown() {
        let decision = NewRecord {
            decision: true,
            title: "One writer".into(),
            body: "Two writers raced.".into(),
            ..NewRecord::default()
        };
        let (item, summary) = decision.item("p1");
        assert_eq!(summary, "Create decision: One writer");
        assert!(item.id.starts_with("decision-") && item.id.len() == "decision-".len() + 8);
        assert_eq!(
            item.body,
            "## Status\n\nProposed\n\n## Context\n\nTwo writers raced.\n\n## Decision\n\n\n\n## Consequences\n\n"
        );
        assert_eq!(
            (
                item.kind.as_str(),
                item.status.as_str(),
                item.doc_type.as_str()
            ),
            ("decision", "proposed", "")
        );
        let doc = NewRecord {
            title: "Runbook".into(),
            body: "# Steps\n\n1. Drain".into(),
            ..NewRecord::default()
        };
        let (item, summary) = doc.item("p1");
        assert_eq!(summary, "Add doc: Runbook");
        assert_eq!(
            (
                item.kind.as_str(),
                item.doc_type.as_str(),
                item.owner.as_str()
            ),
            ("doc", "doc", "human")
        );
        assert_eq!(item.body, "# Steps\n\n1. Drain");
        assert_eq!(item.project_id, "p1");
    }
}
