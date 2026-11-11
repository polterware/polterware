//! Port of `backend/compositions/local-core/lifecycle.go`.
//!
//! What keeps the daemon alive (Decision 0007): recent requests (MCP calls
//! included) and open change streams. The MCP bridge re-initializes
//! transparently when an idle core exited, so MCP sessions do not pin it.
//!
//! Go's `interceptor` authenticates every call and records activity; the port's
//! router does that around each route (`connect::authorize` plus
//! [`Inflight::begin`]), so what is left here is the lifecycle itself, the
//! client header, and the health, shutdown, and change-stream handlers.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use time::OffsetDateTime;
use tokio::sync::watch;

use crate::compositions::wedge;
use crate::local_v1;
use crate::platform::coreclient::CLIENT_HEADER;
use crate::platform::events;
use crate::providers::sqlite::format_rfc3339_nano;

use super::connect::{Code, ConnectError, ConnectRequest, Route, StreamSender};
use super::server::{Core, VERSION};

/// Tracks what keeps the core alive: recent activity and open streams.
pub struct Lifecycle {
    last_active: AtomicI64,
    streams: AtomicI32,
    done: watch::Sender<bool>,
}

impl Default for Lifecycle {
    fn default() -> Lifecycle {
        Lifecycle::new()
    }
}

impl Lifecycle {
    pub fn new() -> Lifecycle {
        let (done, _receiver) = watch::channel(false);
        Lifecycle {
            last_active: AtomicI64::new(0),
            streams: AtomicI32::new(0),
            done,
        }
    }

    /// The clock in nanoseconds, as Go's `time.Now().UnixNano()`.
    fn now() -> i64 {
        OffsetDateTime::now_utc().unix_timestamp_nanos() as i64
    }

    pub fn touch(&self) {
        self.last_active.store(Self::now(), Ordering::SeqCst);
    }

    /// Returns a receiver that answers `true` once the core stops.
    pub fn done(&self) -> watch::Receiver<bool> {
        self.done.subscribe()
    }

    /// Asks the core to stop; the runtime owns the actual exit. Stopping twice
    /// does nothing.
    pub fn stop(&self) {
        if *self.done.borrow() {
            return;
        }
        self.done.send_replace(true);
    }

    pub fn begin_stream(&self) {
        self.streams.fetch_add(1, Ordering::SeqCst);
    }

    pub fn end_stream(&self) {
        self.streams.fetch_sub(1, Ordering::SeqCst);
        self.touch();
    }

    /// Reports whether nothing used the core for at least `after`.
    pub fn idle(&self, after: Duration) -> bool {
        if self.streams.load(Ordering::SeqCst) > 0 {
            return false;
        }
        Self::now() - self.last_active.load(Ordering::SeqCst) >= after.as_nanos() as i64
    }
}

/// Counts calls in progress per client, so a commit can be traced to the client
/// that caused it (change event origins).
#[derive(Default)]
pub struct Inflight {
    counts: Mutex<HashMap<String, i32>>,
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

/// Names the calling client from the `Polterware-Client` header. The Desktop
/// predates the header, so it is the default.
pub fn client_of(headers: &axum::http::HeaderMap) -> String {
    let client = headers
        .get(CLIENT_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .trim();
    if client.is_empty() {
        "desktop".to_string()
    } else {
        client.to_string()
    }
}

/// One set of open agent runs per agent session id (a Claude Code session),
/// replacing the per-session state files the CLI used to write.
#[derive(Default)]
pub struct AgentSessions {
    runs: Mutex<HashMap<String, wedge::SessionRuns>>,
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

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary("/polterware.local.v1.HealthService/Check", check).public(),
        Route::unary("/polterware.local.v1.MaintenanceService/Shutdown", shutdown),
        Route::server_stream(
            "/polterware.local.v1.ChangeService/WatchChanges",
            watch_changes,
        ),
    ]
}

pub fn check(
    core: Arc<Core>,
    _request: ConnectRequest<local_v1::CheckRequest>,
) -> Result<local_v1::CheckResponse, ConnectError> {
    Ok(local_v1::CheckResponse {
        status: "ok".to_string(),
        version: VERSION.to_string(),
        uptime_sec: (OffsetDateTime::now_utc() - core.started).as_seconds_f64(),
        build: core.build(),
        pid: std::process::id() as i32,
    })
}

pub fn shutdown(
    core: Arc<Core>,
    _request: ConnectRequest<local_v1::ShutdownRequest>,
) -> Result<local_v1::ShutdownResponse, ConnectError> {
    // Let the response reach the caller before the server stops.
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        core.stop();
    });
    Ok(local_v1::ShutdownResponse {})
}

/// Streams change events until the client leaves or the core stops.
///
/// A stream answers the current sequence, the backlog after `after_seq`, and
/// then its changes. Until `connect.rs` can deliver a response while it runs
/// (it needs a `futures_core` stream), the handler answers the first two and
/// the stream ends, so the client reconnects and picks the changes up from the
/// backlog.
pub fn watch_changes(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::WatchChangesRequest>,
    out: StreamSender<local_v1::WatchChangesResponse>,
) -> Result<(), ConnectError> {
    core.begin_stream();
    let mut subscription = core.bus.subscribe(request.message.after_seq);
    let send = |message: local_v1::WatchChangesResponse| -> Result<(), ConnectError> {
        out.try_send(message).map_err(|err| {
            ConnectError::new(Code::ResourceExhausted, format!("stream closed: {err}"))
        })
    };
    let result = (|| {
        send(local_v1::WatchChangesResponse {
            seq: subscription.current,
            resync: subscription.resync,
            at: now_stamp(),
            ..local_v1::WatchChangesResponse::default()
        })?;
        for change in &subscription.backlog {
            send(change_proto(change))?;
        }
        Ok(())
    })();
    subscription.cancel();
    core.end_stream();
    result
}

pub fn change_proto(change: &events::Change) -> local_v1::WatchChangesResponse {
    local_v1::WatchChangesResponse {
        seq: change.seq,
        project_ids: change.project_ids.clone(),
        kinds: change.kinds.clone(),
        resync: change.resync,
        at: format_rfc3339_nano(change.at),
        origins: change.origins.clone(),
    }
}

/// A stamp in Go's `time.RFC3339Nano` layout.
fn now_stamp() -> String {
    format_rfc3339_nano(OffsetDateTime::now_utc())
}
