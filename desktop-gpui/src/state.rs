//! The app's state: the route, the core connection, and what has been read from it. Every
//! read runs on the background executor and lands here.

use std::path::PathBuf;

use gpui::{AppContext as _, Context, SharedString};
use polterware_ui::icons::Icon;
use polterware_ui::toast::{Toast, ToastTone};

use crate::client::{self, Core, CoreError, Inbox, Project};

/// The project surfaces, in the Electron sidebar's order and groups.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Section {
    Inbox,
    Activity,
    Milestones,
    Canvas,
    Structure,
    Knowledge,
    ProjectSettings,
}

impl Section {
    /// The sidebar's sections; Project settings sits in its footer.
    pub const NOW: [Section; 2] = [Section::Inbox, Section::Activity];
    pub const PROJECT: [Section; 4] = [
        Section::Milestones,
        Section::Canvas,
        Section::Structure,
        Section::Knowledge,
    ];
    pub const ALL: [Section; 7] = [
        Section::Inbox,
        Section::Activity,
        Section::Milestones,
        Section::Canvas,
        Section::Structure,
        Section::Knowledge,
        Section::ProjectSettings,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Section::Inbox => "Inbox",
            Section::Activity => "Activity",
            Section::Milestones => "Milestones",
            Section::Canvas => "Canvas",
            Section::Structure => "Structure",
            Section::Knowledge => "Knowledge",
            Section::ProjectSettings => "Project settings",
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
            Section::ProjectSettings => Icon::Settings,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    /// The Projects screen, reached through Home.
    Projects,
    Project {
        id: String,
        section: Section,
    },
    /// App settings; `back` is the route it was opened from.
    AppSettings {
        back: Box<Route>,
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

pub struct AppState {
    db: PathBuf,
    core: CoreState,
    route: Route,
    projects: Load<Vec<Project>>,
    inbox: Load<Inbox>,
    /// The Inbox item whose detail is open.
    inbox_open: Option<String>,
    commander_open: bool,
    toasts: Vec<Toast>,
    next_toast: u64,
}

impl AppState {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut state = Self {
            db: client::database_path(),
            core: CoreState::Connecting,
            route: Route::Projects,
            projects: Load::Idle,
            inbox: Load::Idle,
            inbox_open: None,
            commander_open: false,
            toasts: Vec::new(),
            next_toast: 1,
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

    pub fn projects(&self) -> &Load<Vec<Project>> {
        &self.projects
    }

    pub fn inbox(&self) -> &Load<Inbox> {
        &self.inbox
    }

    pub fn inbox_open(&self) -> Option<&str> {
        self.inbox_open.as_deref()
    }

    pub fn commander_open(&self) -> bool {
        self.commander_open
    }

    pub fn toasts(&self) -> &[Toast] {
        &self.toasts
    }

    /// The open project, when the route is in one.
    pub fn project(&self) -> Option<&Project> {
        let id = match &self.route {
            Route::Project { id, .. } => id,
            Route::AppSettings { back } => match back.as_ref() {
                Route::Project { id, .. } => id,
                _ => return None,
            },
            Route::Projects => return None,
        };
        match &self.projects {
            Load::Ready(projects) => projects.iter().find(|project| &project.id == id),
            _ => None,
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
        cx.spawn(async move |this, cx| {
            let started = cx.background_spawn(async move { client::start(&db) }).await;
            let _ = this.update(cx, |state, cx| {
                state.set_core(started, cx);
                cx.notify();
            });
        })
        .detach();
    }

    fn set_core(&mut self, found: Result<Core, CoreError>, cx: &mut Context<Self>) {
        match found {
            Ok(core) => {
                self.core = CoreState::Ready(core);
                self.reload(cx);
            }
            Err(CoreError::NotRunning) => self.core = CoreState::Missing { starting: false },
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

    /// Reads the projects again, and the open page's data.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.load_projects(cx);
        if matches!(self.route, Route::Project { .. }) {
            self.load_inbox(cx);
        }
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
                .background_spawn(async move { client::list_projects(&core) })
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
        if !matches!(self.inbox, Load::Ready(_)) {
            self.inbox = Load::Loading;
        }
        cx.spawn(async move |this, cx| {
            let wanted = project.clone();
            let result = cx
                .background_spawn(async move { client::list_inbox(&core, &project) })
                .await;
            let _ = this.update(cx, |state, cx| {
                // A read for a project that is no longer open is dropped.
                if !matches!(&state.route, Route::Project { id, .. } if *id == wanted) {
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

    /// The first item starts active; an item that left the Inbox hands over to the first.
    fn keep_inbox_open(&mut self, inbox: &Inbox) {
        let still = self
            .inbox_open
            .as_ref()
            .is_some_and(|id| inbox.items.iter().any(|item| &item.id == id));
        if !still {
            self.inbox_open = inbox.items.first().map(|item| item.id.clone());
        }
    }

    pub fn open_inbox_item(&mut self, id: Option<String>) {
        self.inbox_open = id;
    }

    pub fn go_home(&mut self) {
        self.route = Route::Projects;
        self.commander_open = false;
    }

    pub fn open_project(&mut self, id: String, cx: &mut Context<Self>) {
        let same = matches!(&self.route, Route::Project { id: open, .. } if *open == id);
        if !same {
            self.inbox = Load::Idle;
            self.inbox_open = None;
        }
        self.route = Route::Project {
            id,
            section: Section::Inbox,
        };
        self.commander_open = false;
        self.load_inbox(cx);
    }

    /// Opens a section of the open project. Choosing the section you are in returns to it.
    pub fn go_section(&mut self, section: Section, cx: &mut Context<Self>) {
        let id = match &self.route {
            Route::Project { id, .. } => id.clone(),
            Route::AppSettings { back } => match back.as_ref() {
                Route::Project { id, .. } => id.clone(),
                _ => return,
            },
            Route::Projects => return,
        };
        self.route = Route::Project { id, section };
        self.commander_open = false;
        if section == Section::Inbox {
            self.load_inbox(cx);
        }
    }

    pub fn open_app_settings(&mut self) {
        if !matches!(self.route, Route::AppSettings { .. }) {
            let back = Box::new(self.route.clone());
            self.route = Route::AppSettings { back };
        }
        self.commander_open = false;
    }

    pub fn close_app_settings(&mut self) {
        if let Route::AppSettings { back } = &self.route {
            self.route = back.as_ref().clone();
        }
    }

    pub fn set_commander(&mut self, open: bool) {
        self.commander_open = open;
    }

    pub fn toast(&mut self, message: impl Into<SharedString>, danger: bool) {
        let id = self.next_toast;
        self.next_toast += 1;
        self.toasts.push(Toast {
            id,
            message: message.into(),
            tone: if danger {
                ToastTone::Danger
            } else {
                ToastTone::Info
            },
        });
    }

    pub fn dismiss_toast(&mut self, id: u64) {
        self.toasts.retain(|toast| toast.id != id);
    }
}
