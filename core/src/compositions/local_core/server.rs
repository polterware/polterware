//! Port of `backend/compositions/local-core/server.go`.
//!
//! The [`Core`] the services read and write through, the token and listener it
//! starts with, the router that mounts every service route, and the project,
//! environment, knowledge, connection, and maintenance handlers.
//!
//! The one open store of this database lives inside the MCP session table
//! (`providers::mcp::Sessions` owns it, as `mcp.NewSessions` does in Go); every
//! handler borrows it through [`Core::store`].

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use time::OffsetDateTime;
use tokio::sync::{oneshot, watch};

use crate::compositions::wedge;
use crate::domains::agentrun;
use crate::domains::integrations;
use crate::domains::knowledge;
use crate::domains::projects;
use crate::local_v1;
use crate::platform::db::sqlite::set_change_recorder;
use crate::platform::events::Bus;
use crate::providers::mcp;
use crate::providers::seed;
use crate::providers::sqlite::{Store, format_rfc3339, format_rfc3339_zone};

use super::connect::{self, Code, ConnectError, ConnectRequest, Route};

/// The version the health check reports.
pub const VERSION: &str = "0.1.0";

/// The core: the store, the session token, the change bus, and the client calls
/// in flight.
pub struct Core {
    /// Serves MCP sessions over HTTP at /mcp, and owns the store.
    pub mcp: Arc<mcp::Sessions>,
    /// The bearer token every call but the health check must carry.
    pub token: String,
    pub started: OffsetDateTime,
    /// Turns committed SQLite writes into change events (Decision 0007).
    pub bus: Bus,
    /// One set of open agent runs per agent session id.
    pub sessions: AgentSessions,
    build: Mutex<String>,
    db_dir: Mutex<String>,
    life: Lifecycle,
    pub(crate) calls: Inflight,
}

impl Core {
    /// Wires the core around the only open store of this database. It installs
    /// the change recorder, so every committed write becomes an event.
    pub fn new(store: Store, token: &str) -> Arc<Core> {
        let _ = seed::defaults(&store);
        let core = Arc::new(Core {
            mcp: Arc::new(mcp::Sessions::new(store)),
            token: token.to_string(),
            started: OffsetDateTime::now_utc(),
            bus: Bus::new(),
            sessions: AgentSessions::default(),
            build: Mutex::new(String::new()),
            db_dir: Mutex::new(String::new()),
            life: Lifecycle::new(),
            calls: Inflight::default(),
        });
        core.touch();
        let weak = Arc::downgrade(&core);
        set_change_recorder(Some(Arc::new(move |tables, project_ids| {
            if let Some(core) = weak.upgrade() {
                core.bus.record(tables, project_ids, &core.calls.clients());
            }
        })));
        core
    }

    /// Borrows the store. Every handler reads and writes through it.
    pub fn store(&self) -> MutexGuard<'_, Store> {
        self.mcp.store()
    }

    /// The build of the core binary (`coreclient::build_id`).
    pub fn build(&self) -> String {
        self.build
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .clone()
    }

    pub fn set_build(&self, build: &str) {
        *self
            .build
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = build.to_string();
    }

    /// The database directory; agent setup keeps its local marketplace there.
    pub fn db_dir(&self) -> String {
        self.db_dir
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .clone()
    }

    pub fn set_db_dir(&self, dir: &str) {
        *self
            .db_dir
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = dir.to_string();
    }

    /// Marks the core used now, so the idle exit waits again.
    pub fn touch(&self) {
        self.life.touch();
    }

    /// Reports whether nothing used the core for at least `after`.
    pub fn idle(&self, after: Duration) -> bool {
        self.life.idle(after)
    }

    /// A receiver that answers `true` once the core was asked to stop.
    pub fn done(&self) -> watch::Receiver<bool> {
        self.life.done()
    }

    /// Asks the core to shut down; the runtime owns the actual exit. Stopping
    /// twice does nothing.
    pub fn stop(&self) {
        self.life.stop();
    }

    /// Counts a stream as open while it runs.
    pub fn begin_stream(&self) {
        self.life.begin_stream();
    }

    pub fn end_stream(&self) {
        self.life.end_stream();
    }
}

/// Mints a session token: 32 random bytes as lowercase hex.
pub fn mint_token() -> String {
    use rand::RngCore;
    let mut value = [0u8; 32];
    rand::rng().fill_bytes(&mut value);
    hex::encode(value)
}

/// Binds a loopback listener and answers its endpoint, as Go's `net.Listen`
/// plus `"http://" + Addr` does.
pub fn listen(address: &str) -> std::io::Result<(std::net::TcpListener, String)> {
    let listener = std::net::TcpListener::bind(address)?;
    let endpoint = format!("http://{}", listener.local_addr()?);
    Ok((listener, endpoint))
}

/// A served core. Dropping it leaves the server task running, as Go's
/// `http.Server` does; [`Server::close`] stops it.
pub struct Server {
    shutdown: Option<oneshot::Sender<()>>,
    handle: tokio::task::JoinHandle<()>,
}

impl Server {
    /// Stops the server, letting a request in flight finish.
    pub fn close(mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        self.handle.abort();
    }
}

/// Serves every route on `listener`, and starts the background deploy check.
/// Runs on the caller's tokio runtime.
pub fn serve_connect(listener: std::net::TcpListener, core: Arc<Core>) -> Server {
    super::deploy_checks::watch_deploys(&core);
    listener
        .set_nonblocking(true)
        .expect("the core listener turns non-blocking");
    let router = router(core);
    let (shutdown, stopped) = oneshot::channel();
    let handle = tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::from_std(listener) {
            Ok(listener) => listener,
            Err(_) => return,
        };
        let _ = axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                let _ = stopped.await;
            })
            .await;
    });
    Server {
        shutdown: Some(shutdown),
        handle,
    }
}

/// Every route of every service in `polterware/local/v1`, plus `/mcp`.
pub fn router(core: Arc<Core>) -> axum::Router {
    let mut router = connect::router(Arc::clone(&core), super::routes());
    let sessions = Arc::clone(&core.mcp);
    router = router.route(
        "/mcp",
        axum::routing::any(move |request: Request| {
            let core = Arc::clone(&core);
            let sessions = Arc::clone(&sessions);
            async move { serve_mcp(core, sessions, request).await }
        }),
    );
    router
}

/// The MCP endpoint: the same token, then the MCP transport, which a plain
/// text 401 refuses when the token is missing.
async fn serve_mcp(core: Arc<Core>, sessions: Arc<mcp::Sessions>, request: Request) -> Response {
    if connect::authorize(&core, request.headers()).is_err() {
        let mut response = (
            axum::http::StatusCode::UNAUTHORIZED,
            "invalid or missing session token\n",
        )
            .into_response();
        response.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        return response;
    }
    let _call = core.calls.begin("mcp");
    mcp::handle_request(&sessions, request).await
}

/// Every route this file's Go counterpart serves.
pub fn routes() -> Vec<Route> {
    vec![
        Route::unary(
            "/polterware.local.v1.ProjectService/ListProjects",
            list_projects,
        ),
        Route::unary(
            "/polterware.local.v1.ProjectService/SaveProject",
            save_project,
        ),
        Route::unary(
            "/polterware.local.v1.ProjectService/ArchiveProject",
            archive_project,
        ),
        Route::unary(
            "/polterware.local.v1.ProjectService/ListEnvironments",
            list_environments,
        ),
        Route::unary(
            "/polterware.local.v1.ProjectService/SaveEnvironment",
            save_environment,
        ),
        Route::unary(
            "/polterware.local.v1.ProjectService/DeleteEnvironment",
            delete_environment,
        ),
        Route::unary(
            "/polterware.local.v1.KnowledgeService/SaveKnowledge",
            save_knowledge,
        ),
        Route::unary(
            "/polterware.local.v1.KnowledgeService/ListKnowledge",
            list_knowledge,
        ),
        Route::unary(
            "/polterware.local.v1.KnowledgeService/DeleteKnowledge",
            delete_knowledge,
        ),
        Route::unary(
            "/polterware.local.v1.IntegrationService/ListCliProviders",
            list_cli_providers,
        ),
        Route::unary(
            "/polterware.local.v1.IntegrationService/ListIntegrationConnections",
            list_integration_connections,
        ),
        Route::unary(
            "/polterware.local.v1.IntegrationService/SaveIntegrationConnection",
            save_integration_connection,
        ),
        Route::unary(
            "/polterware.local.v1.MaintenanceService/ResetLocalStore",
            reset_local_store,
        ),
        Route::unary(
            "/polterware.local.v1.MaintenanceService/GetStoreHealth",
            get_store_health,
        ),
    ]
}

/// The port of the package's `invalid`.
pub fn invalid(message: impl Into<String>) -> ConnectError {
    ConnectError::invalid(message)
}

/// The port of the package's `internal`.
pub fn internal(err: impl std::fmt::Display) -> ConnectError {
    ConnectError::internal(err.to_string())
}

pub fn list_projects(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListProjectsRequest>,
) -> Result<local_v1::ListProjectsResponse, ConnectError> {
    let items = core
        .store()
        .list_projects(request.message.include_archived)
        .map_err(internal)?;
    Ok(local_v1::ListProjectsResponse {
        projects: items.iter().map(project_proto).collect(),
    })
}

pub fn save_project(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::SaveProjectRequest>,
) -> Result<local_v1::SaveProjectResponse, ConnectError> {
    let Some(item) = request.message.project else {
        return Err(invalid("project is required"));
    };
    let project =
        projects::new_project(&item.id, &item.name).map_err(|err| invalid(err.to_string()))?;
    {
        let store = core.store();
        store.save_project(project.clone()).map_err(internal)?;
        if store.get_project_settings(&project.id).is_err() {
            store
                .save_project_settings(projects::Settings {
                    project_id: project.id.clone(),
                    ..projects::Settings::default()
                })
                .map_err(internal)?;
        }
    }
    Ok(local_v1::SaveProjectResponse {
        project: Some(project_proto(&project)),
    })
}

pub fn archive_project(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ArchiveProjectRequest>,
) -> Result<local_v1::ArchiveProjectResponse, ConnectError> {
    if request.message.id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let project = core
        .store()
        .archive_project(&request.message.id)
        .map_err(internal)?;
    Ok(local_v1::ArchiveProjectResponse {
        project: Some(project_proto(&project)),
    })
}

pub fn list_environments(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListEnvironmentsRequest>,
) -> Result<local_v1::ListEnvironmentsResponse, ConnectError> {
    let items = core
        .store()
        .list_environments(&request.message.project_id)
        .map_err(internal)?;
    Ok(local_v1::ListEnvironmentsResponse {
        environments: items.iter().map(environment_proto).collect(),
    })
}

pub fn save_environment(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::SaveEnvironmentRequest>,
) -> Result<local_v1::SaveEnvironmentResponse, ConnectError> {
    let Some(item) = request.message.environment else {
        return Err(invalid("environment is required"));
    };
    let mut environment =
        projects::new_environment(&item.id, &item.project_id, &item.name, &item.kind)
            .map_err(|err| invalid(err.to_string()))?;
    environment.protection = item.protection;
    environment.is_default = item.is_default;
    environment.description = item.description;
    core.store()
        .save_environment(environment.clone())
        .map_err(internal)?;
    Ok(local_v1::SaveEnvironmentResponse {
        environment: Some(environment_proto(&environment)),
    })
}

pub fn delete_environment(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::DeleteEnvironmentRequest>,
) -> Result<local_v1::DeleteEnvironmentResponse, ConnectError> {
    if request.message.id.is_empty() || request.message.project_id.is_empty() {
        return Err(invalid("environment id and project id are required"));
    }
    core.store()
        .delete_environment(&request.message.id, &request.message.project_id)
        .map_err(|err| invalid(err.to_string()))?;
    Ok(local_v1::DeleteEnvironmentResponse {})
}

pub fn save_knowledge(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::SaveKnowledgeRequest>,
) -> Result<local_v1::SaveKnowledgeResponse, ConnectError> {
    let Some(item) = request.message.item else {
        return Err(invalid("knowledge item is required"));
    };
    let mut env = super::audited_write::change_envelope_from_proto(request.message.change.as_ref())
        .map_err(super::audited_write::map_audited_err)?;
    env.project_id = item.project_id.clone();
    let mut kind = item.kind.clone();
    if kind == knowledge::KIND_DOC && !item.doc_type.is_empty() {
        kind = item.doc_type.clone();
    }
    let mut created = knowledge::new_item(
        &item.id,
        &item.project_id,
        &item.title,
        &item.body,
        &kind,
        item.tags.clone(),
    )
    .map_err(|err| invalid(err.to_string()))?;
    created.summary = item.summary.clone();
    created.owner = item.owner.clone();
    if created.owner.is_empty() {
        created.owner = agentrun::ACTOR_KIND_HUMAN.to_string();
    }
    created.status = item.status.clone();
    created.extra_json = item.extra_json.clone();
    {
        let store = core.store();
        if let Ok(existing) = store.get_knowledge(&created.id)
            && existing.archived()
        {
            return Err(super::audited_write::map_archive_err(
                knowledge::KnowledgeError::Archived,
            ));
        }
    }
    let write = {
        let store = core.store();
        let created = created.clone();
        wedge::execute_audited_write(
            &store,
            env,
            agentrun::ACTOR_KIND_HUMAN,
            "desktop.knowledge.save",
            |_run| {
                store.save_knowledge(created.clone())?;
                Ok((vec![created.id.clone()], created.clone()))
            },
        )
    };
    write.map_err(super::audited_write::map_audited_err)?;
    Ok(local_v1::SaveKnowledgeResponse {
        item: Some(knowledge_proto(&created)),
    })
}

pub fn list_knowledge(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListKnowledgeRequest>,
) -> Result<local_v1::ListKnowledgeResponse, ConnectError> {
    let store = core.store();
    let items = if request.message.archived {
        store.list_archived_knowledge(&request.message.project_id, &request.message.kind)
    } else {
        store.list_knowledge(&request.message.project_id, &request.message.kind)
    }
    .map_err(internal)?;
    let refs = wedge::scope_refs(&request.message.scope_node, &request.message.scope_nodes);
    let resolved = wedge::load_node_scopes(
        &store,
        &request.message.project_id,
        &refs,
        request.message.scope_exact,
    )
    .map_err(|err| invalid(err.to_string()))?;
    let scopes = store
        .scope_map(
            &request.message.project_id,
            crate::providers::sqlite::SCOPE_DECISION,
        )
        .map_err(internal)?;
    let mut result = Vec::with_capacity(items.len());
    for item in &items {
        let node_ids = scopes.get(&item.id).cloned().unwrap_or_default();
        if !resolved.matches_any(&node_ids) {
            continue;
        }
        let mut message = knowledge_proto(item);
        message.scope_node_ids = node_ids;
        result.push(message);
    }
    Ok(local_v1::ListKnowledgeResponse { items: result })
}

pub fn delete_knowledge(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::DeleteKnowledgeRequest>,
) -> Result<local_v1::DeleteKnowledgeResponse, ConnectError> {
    if request.message.id.is_empty() {
        return Err(invalid("knowledge id is required"));
    }
    let mut env = super::audited_write::change_envelope_from_proto(request.message.change.as_ref())
        .map_err(super::audited_write::map_audited_err)?;
    let existing = {
        let store = core.store();
        match store.get_knowledge(&request.message.id) {
            Ok(existing) => existing,
            Err(err) if err.is_no_rows() => {
                return Err(ConnectError::new(Code::NotFound, "unknown knowledge item"));
            }
            Err(err) => return Err(internal(err)),
        }
    };
    if existing.kind == knowledge::KIND_DECISION {
        return Err(super::audited_write::map_archive_err(
            knowledge::KnowledgeError::DecisionArchive,
        ));
    }
    if !existing.archived() {
        return Err(super::audited_write::map_archive_err(
            knowledge::KnowledgeError::NotArchived,
        ));
    }
    env.project_id = existing.project_id.clone();
    let id = request.message.id.clone();
    let write = {
        let store = core.store();
        wedge::execute_audited_write(
            &store,
            env,
            agentrun::ACTOR_KIND_HUMAN,
            "desktop.knowledge.delete",
            |_run| {
                store.delete_knowledge(&id)?;
                Ok((vec![id.clone()], ()))
            },
        )
    };
    write.map_err(super::audited_write::map_audited_err)?;
    Ok(local_v1::DeleteKnowledgeResponse {})
}

pub fn list_cli_providers(
    _core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListCliProvidersRequest>,
) -> Result<local_v1::ListCliProvidersResponse, ConnectError> {
    Ok(local_v1::ListCliProvidersResponse {
        providers: integrations::CLI_PROVIDERS
            .iter()
            .map(|item| local_v1::CliProvider {
                id: item.id.to_string(),
                name: item.name.to_string(),
                binary: item.binary.to_string(),
                auth_commands: item
                    .auth_commands
                    .iter()
                    .map(|command| command.to_string())
                    .collect(),
                description: item.description.to_string(),
                category: item.category.to_string(),
            })
            .collect(),
    })
}

pub fn list_integration_connections(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListIntegrationConnectionsRequest>,
) -> Result<local_v1::ListIntegrationConnectionsResponse, ConnectError> {
    let items = core
        .store()
        .list_connections(&request.message.project_id)
        .map_err(internal)?;
    Ok(local_v1::ListIntegrationConnectionsResponse {
        connections: items.iter().map(connection_proto).collect(),
    })
}

pub fn save_integration_connection(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::SaveIntegrationConnectionRequest>,
) -> Result<local_v1::SaveIntegrationConnectionResponse, ConnectError> {
    let Some(item) = request.message.connection else {
        return Err(invalid("connection is required"));
    };
    let connection = integrations::Connection {
        id: item.id.clone(),
        project_id: item.project_id.clone(),
        provider_id: item.provider_id.clone(),
        status: item.status.clone(),
        label: item.label.clone(),
        environment_id: item.environment_id.clone(),
        last_sync_at: item.last_sync_at.clone(),
        summary: item.summary.clone(),
        cli_command: item.cli_command.clone(),
        sync_interval: item.sync_interval.clone(),
        freshness: item.freshness.clone(),
        error_message: item.error_message.clone(),
    };
    if connection.id.is_empty()
        || connection.project_id.is_empty()
        || connection.provider_id.is_empty()
    {
        return Err(invalid(
            "connection id, project id, and provider id are required",
        ));
    }
    core.store()
        .save_connection(connection.clone())
        .map_err(internal)?;
    Ok(local_v1::SaveIntegrationConnectionResponse {
        connection: Some(connection_proto(&connection)),
    })
}

pub fn reset_local_store(
    core: Arc<Core>,
    _request: ConnectRequest<local_v1::ResetLocalStoreRequest>,
) -> Result<local_v1::ResetLocalStoreResponse, ConnectError> {
    let store = core.store();
    store.reset().map_err(internal)?;
    seed::defaults(&store).map_err(internal)?;
    Ok(local_v1::ResetLocalStoreResponse {
        status: "reset".to_string(),
    })
}

pub fn get_store_health(
    core: Arc<Core>,
    _request: ConnectRequest<local_v1::GetStoreHealthRequest>,
) -> Result<local_v1::GetStoreHealthResponse, ConnectError> {
    let (quarantined, message) = core.store().store_health().map_err(internal)?;
    Ok(local_v1::GetStoreHealthResponse {
        quarantined,
        message,
    })
}

pub fn project_proto(item: &projects::Project) -> local_v1::Project {
    let archived_at = item
        .archived_at
        .map(format_rfc3339_zone)
        .unwrap_or_default();
    let status = if item.status.is_empty() {
        projects::STATUS_ACTIVE.to_string()
    } else {
        item.status.clone()
    };
    local_v1::Project {
        id: item.id.clone(),
        name: item.name.clone(),
        created_at: format_rfc3339_zone(item.created_at),
        updated_at: format_rfc3339_zone(item.updated_at),
        status,
        archived_at,
    }
}

pub fn environment_proto(item: &projects::Environment) -> local_v1::Environment {
    local_v1::Environment {
        id: item.id.clone(),
        project_id: item.project_id.clone(),
        name: item.name.clone(),
        kind: item.kind.clone(),
        protection: item.protection.clone(),
        is_default: item.is_default,
        description: item.description.clone(),
    }
}

pub fn knowledge_proto(item: &knowledge::Item) -> local_v1::KnowledgeItem {
    local_v1::KnowledgeItem {
        id: item.id.clone(),
        project_id: item.project_id.clone(),
        title: item.title.clone(),
        body: item.body.clone(),
        kind: item.kind.clone(),
        doc_type: item.doc_type.clone(),
        tags: item.tags.clone(),
        summary: item.summary.clone(),
        owner: item.owner.clone(),
        status: item.status.clone(),
        extra_json: item.extra_json.clone(),
        created_at: format_rfc3339_zone(item.created_at),
        updated_at: format_rfc3339_zone(item.updated_at),
        archived_at: archived_stamp(item.archived_at),
        scope_node_ids: Vec::new(),
    }
}

pub fn connection_proto(item: &integrations::Connection) -> local_v1::IntegrationConnection {
    local_v1::IntegrationConnection {
        id: item.id.clone(),
        project_id: item.project_id.clone(),
        provider_id: item.provider_id.clone(),
        status: item.status.clone(),
        label: item.label.clone(),
        environment_id: item.environment_id.clone(),
        last_sync_at: item.last_sync_at.clone(),
        summary: item.summary.clone(),
        cli_command: item.cli_command.clone(),
        sync_interval: item.sync_interval.clone(),
        freshness: item.freshness.clone(),
        error_message: item.error_message.clone(),
    }
}

/// A stamp that is empty while absent, as `archive_handlers.go` writes it.
pub fn archived_stamp(at: Option<OffsetDateTime>) -> String {
    at.map(format_rfc3339).unwrap_or_default()
}

/// The lifecycle the core keeps: recent activity, open streams, and the stop
/// signal.
pub struct Lifecycle {
    last_active: std::sync::atomic::AtomicI64,
    streams: std::sync::atomic::AtomicI32,
    done: watch::Sender<bool>,
}

impl Lifecycle {
    fn new() -> Lifecycle {
        let (done, _receiver) = watch::channel(false);
        Lifecycle {
            last_active: std::sync::atomic::AtomicI64::new(0),
            streams: std::sync::atomic::AtomicI32::new(0),
            done,
        }
    }

    fn now() -> i64 {
        OffsetDateTime::now_utc().unix_timestamp_nanos() as i64
    }

    fn touch(&self) {
        self.last_active
            .store(Self::now(), std::sync::atomic::Ordering::SeqCst);
    }

    fn done(&self) -> watch::Receiver<bool> {
        self.done.subscribe()
    }

    fn stop(&self) {
        if *self.done.borrow() {
            return;
        }
        self.done.send_replace(true);
    }

    fn begin_stream(&self) {
        self.streams
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    fn end_stream(&self) {
        self.streams
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        self.touch();
    }

    fn idle(&self, after: Duration) -> bool {
        if self.streams.load(std::sync::atomic::Ordering::SeqCst) > 0 {
            return false;
        }
        let elapsed = Self::now() - self.last_active.load(std::sync::atomic::Ordering::SeqCst);
        elapsed >= after.as_nanos() as i64
    }
}

/// Counts calls in progress per client, so a commit can be traced to the client
/// that caused it (change event origins).
#[derive(Default)]
pub struct Inflight {
    counts: Mutex<std::collections::HashMap<String, i32>>,
}

impl Inflight {
    /// Counts a call for `client` until the returned guard drops.
    pub fn begin(&self, client: &str) -> CallGuard<'_> {
        {
            let mut counts = self
                .counts
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            *counts.entry(client.to_string()).or_insert(0) += 1;
        }
        CallGuard {
            calls: self,
            client: client.to_string(),
        }
    }

    /// Lists the clients with calls in flight, or "core" for background work.
    pub fn clients(&self) -> Vec<String> {
        let counts = self
            .counts
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if counts.is_empty() {
            return vec!["core".to_string()];
        }
        counts.keys().cloned().collect()
    }
}

/// One call in flight; dropping it leaves the count.
pub struct CallGuard<'a> {
    calls: &'a Inflight,
    client: String,
}

impl Drop for CallGuard<'_> {
    fn drop(&mut self) {
        let mut counts = self
            .calls
            .counts
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if let Some(count) = counts.get_mut(&self.client) {
            *count -= 1;
            if *count <= 0 {
                counts.remove(&self.client);
            }
        }
    }
}

/// One set of open agent runs per agent session id (a Claude Code session),
/// replacing the per-session state files the CLI used to write.
#[derive(Default)]
pub struct AgentSessions {
    runs: Mutex<std::collections::HashMap<String, wedge::SessionRuns>>,
}

impl AgentSessions {
    /// Runs `apply` with the session's runs; applies are serialized per core.
    pub fn apply<T>(
        &self,
        id: &str,
        label: &str,
        apply: impl FnOnce(&mut wedge::SessionRuns) -> crate::providers::sqlite::Result<T>,
    ) -> crate::providers::sqlite::Result<T> {
        let mut runs = self
            .runs
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let session = runs.entry(id.to_string()).or_default();
        if !label.is_empty() {
            session.label = label.to_string();
        }
        apply(session)
    }

    /// Removes the session and returns it for closing.
    pub fn end(&self, id: &str) -> Option<wedge::SessionRuns> {
        self.runs
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .remove(id)
    }
}
