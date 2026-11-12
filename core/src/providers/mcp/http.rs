//! MCP over HTTP inside the core
//! (Decision 0007).
//!
//! POST one JSON-RPC message and get its JSON response (202 for
//! notifications); `initialize` opens a session and returns its id in
//! `Mcp-Session-Id`; DELETE closes it. Callers authenticate before reaching this
//! handler.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use time::{Duration, OffsetDateTime};

use crate::platform::ids;
use crate::providers::sqlite::Store;

use super::Server;

/// Carries the MCP session id (streamable HTTP transport).
pub const SESSION_HEADER: &str = "Mcp-Session-Id";

/// Closes sessions whose client stopped calling without a DELETE (for example a
/// killed bridge).
const SESSION_IDLE: Duration = Duration::minutes(30);

/// The body a POST may carry. Go reads at most this many bytes and parses the
/// truncation; here a body that long is refused before it is parsed, because
/// axum hands the bytes over at once.
const MAX_BODY: usize = 4 << 20;

/// One live session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    pub id: String,
    pub client_name: String,
    pub client_version: String,
    pub started_at: OffsetDateTime,
    pub last_seen: OffsetDateTime,
}

/// What the HTTP layer answers: the status, the session header, and the body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reply {
    pub status: u16,
    pub session_id: String,
    pub content_type: &'static str,
    pub allow: &'static str,
    pub body: Vec<u8>,
}

impl Reply {
    /// A JSON body with the session header.
    fn json(session_id: &str, body: Vec<u8>) -> Reply {
        Reply {
            status: 200,
            session_id: session_id.to_string(),
            content_type: "application/json",
            allow: "",
            body,
        }
    }

    /// Go's `http.Error`: a plain-text body that ends in a newline, and no
    /// session header.
    fn error(status: u16, message: &str) -> Reply {
        Reply {
            status,
            session_id: String::new(),
            content_type: "text/plain; charset=utf-8",
            allow: "",
            body: format!("{message}\n").into_bytes(),
        }
    }
}

/// Serves MCP over HTTP: the session table, the store they write through, and
/// the idle sweep.
///
/// The store is locked for one message at a time, before the session table, so
/// two handlers never hold them in the other order.
pub struct Sessions {
    store: Mutex<Store>,
    sessions: Mutex<HashMap<String, Server>>,
}

impl Sessions {
    /// Opens the session table over `store`.
    pub fn new(store: Store) -> Sessions {
        Sessions {
            store: Mutex::new(store),
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Borrows the store the sessions write through.
    pub fn store(&self) -> MutexGuard<'_, Store> {
        self.store
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    /// Returns live sessions, oldest first.
    pub fn list(&self) -> Vec<Info> {
        let store = self.store();
        let mut sessions = self.sessions();
        expire(&store, &mut sessions);
        let mut out: Vec<Info> = sessions
            .values()
            .map(|session| Info {
                id: session.id.clone(),
                client_name: session.client_name.clone(),
                client_version: session.client_version.clone(),
                started_at: session.started_at,
                last_seen: session.last_seen,
            })
            .collect();
        // Oldest first. A session's ULID is minted with it, so the id breaks
        // the tie two initializes in the same instant would leave.
        out.sort_by_key(|info| (info.started_at, info.id.clone()));
        out
    }

    /// Completes every session's runs (core shutdown).
    pub fn close_all(&self) {
        let store = self.store();
        let mut sessions = self.sessions();
        for session in sessions.values_mut() {
            session.close(&store);
        }
        sessions.clear();
    }

    /// Answers one HTTP request the way Go's handler does.
    pub fn serve(&self, method: &str, session_id: &str, body: &[u8]) -> Reply {
        match method {
            "POST" => self.post(session_id, body),
            "DELETE" => {
                self.delete(session_id);
                Reply {
                    status: 204,
                    ..Reply::default()
                }
            }
            _ => Reply {
                status: 405,
                allow: "POST, DELETE",
                ..Reply::default()
            },
        }
    }

    fn post(&self, session_id: &str, body: &[u8]) -> Reply {
        let store = self.store();
        let mut sessions = self.sessions();
        expire(&store, &mut sessions);
        let id = if probe_method(body) == "initialize" {
            let now = OffsetDateTime::now_utc();
            let session = Server {
                id: ids::new_ulid(),
                started_at: now,
                last_seen: now,
                ..Server::default()
            };
            let id = session.id.clone();
            sessions.insert(id.clone(), session);
            id
        } else {
            session_id.to_string()
        };
        // Unknown or expired session: the client must initialize again.
        let Some(session) = sessions.get_mut(&id) else {
            return Reply::error(404, "unknown MCP session: initialize again");
        };
        session.last_seen = OffsetDateTime::now_utc();
        // Writes are serialized by SQLite; tool calls of one session run in
        // order because the bridge sends one message at a time.
        match session.handle(&store, body) {
            Some(response) => Reply::json(&id, response),
            None => Reply {
                status: 202,
                session_id: id,
                ..Reply::default()
            },
        }
    }

    fn delete(&self, session_id: &str) {
        let store = self.store();
        let mut sessions = self.sessions();
        if let Some(mut session) = sessions.remove(session_id) {
            session.close(&store);
        }
    }

    /// Locks the session table; a poisoned lock still answers, because every
    /// write is one message and the store is left consistent.
    fn sessions(&self) -> MutexGuard<'_, HashMap<String, Server>> {
        self.sessions
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

/// Drops the sessions whose client stopped calling.
fn expire(store: &Store, sessions: &mut HashMap<String, Server>) {
    let cutoff = OffsetDateTime::now_utc() - SESSION_IDLE;
    let expired: Vec<String> = sessions
        .iter()
        .filter(|(_, session)| session.last_seen < cutoff)
        .map(|(id, _)| id.clone())
        .collect();
    for id in expired {
        if let Some(mut session) = sessions.remove(&id) {
            session.close(store);
        }
    }
}

/// Reads the `method` of a message without trusting the rest, as Go's probe
/// does.
fn probe_method(body: &[u8]) -> String {
    let Ok(serde_json::Value::Object(map)) = serde_json::from_slice::<serde_json::Value>(body)
    else {
        return String::new();
    };
    match map.get("method") {
        Some(serde_json::Value::String(method)) => method.clone(),
        _ => String::new(),
    }
}

/// The route the core mounts for MCP, carrying its own sessions.
///
/// The core adds it as `router.route(path, method_router(sessions))`. A router
/// with its own state can instead capture the sessions and call
/// [`handle_request`], which is the same handler without the route.
pub fn method_router(sessions: Arc<Sessions>) -> axum::routing::MethodRouter {
    axum::routing::any(move |request: axum::extract::Request| {
        let sessions = Arc::clone(&sessions);
        async move { handle_request(&sessions, request).await }
    })
}

/// One HTTP request as an axum response.
pub async fn handle_request(
    sessions: &Sessions,
    request: axum::extract::Request,
) -> axum::response::Response {
    let method = request.method().as_str().to_string();
    let session_id = request
        .headers()
        .get(SESSION_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let body = match axum::body::to_bytes(request.into_body(), MAX_BODY).await {
        Ok(body) => body.to_vec(),
        Err(_) => {
            return response(Reply::error(400, "read body"));
        }
    };
    response(sessions.serve(&method, &session_id, &body))
}

/// Turns a [`Reply`] into the axum response the client reads.
fn response(reply: Reply) -> axum::response::Response {
    use axum::body::Body;
    use axum::http::{HeaderValue, StatusCode, header};

    let mut out = axum::response::Response::new(Body::from(reply.body));
    *out.status_mut() =
        StatusCode::from_u16(reply.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    if !reply.content_type.is_empty()
        && let Ok(value) = HeaderValue::from_str(reply.content_type)
    {
        out.headers_mut().insert(header::CONTENT_TYPE, value);
    }
    if !reply.allow.is_empty()
        && let Ok(value) = HeaderValue::from_str(reply.allow)
    {
        out.headers_mut().insert(header::ALLOW, value);
    }
    if !reply.session_id.is_empty()
        && let Ok(name) = axum::http::HeaderName::from_bytes(SESSION_HEADER.as_bytes())
        && let Ok(value) = HeaderValue::from_str(&reply.session_id)
    {
        out.headers_mut().insert(name, value);
    }
    out
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use serde_json::json;

    use super::super::TOOL_POLTER_APPLY;
    use super::super::test_support::{ROADMAP, TestStore};
    use super::*;
    use crate::providers::sqlite::test_support::TempStore;

    /// The sessions over the fixture store, which the temporary directory keeps
    /// alive.
    struct TestSessions {
        sessions: Sessions,
        project_id: String,
        #[allow(dead_code)]
        dir: tempfile::TempDir,
    }

    impl TestSessions {
        fn new() -> TestSessions {
            let TestStore { temp, project_id } = TestStore::new();
            let TempStore { store, dir } = temp;
            TestSessions {
                sessions: Sessions::new(store),
                project_id,
                dir,
            }
        }

        fn post(&self, session: &str, body: &str) -> Reply {
            self.sessions.serve("POST", session, body.as_bytes())
        }

        /// Opens a session the way a client does and returns its id.
        fn initialize(&self, client: &str) -> String {
            let body = format!(
                r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"clientInfo":{{"name":"{client}","version":"1.2.3"}}}}}}"#
            );
            let reply = self.post("", &body);
            assert_eq!(reply.status, 200, "initialize: {reply:?}");
            assert!(!reply.session_id.is_empty(), "initialize: {reply:?}");
            reply.session_id
        }
    }

    /// The JSON-RPC reply a body carries.
    fn reply(body: &[u8]) -> Value {
        serde_json::from_slice(body).unwrap_or_else(|error| {
            panic!(
                "unexpected reply {}: {error}",
                String::from_utf8_lossy(body)
            )
        })
    }

    /// The text and `isError` of a `tools/call` reply.
    fn tool_text(body: &[u8]) -> (String, bool) {
        let reply = reply(body);
        assert!(reply.get("error").is_none(), "unexpected error: {reply}");
        let text = reply
            .get("result")
            .and_then(|result| result.get("content"))
            .and_then(Value::as_array)
            .and_then(|content| content.first())
            .and_then(|item| item.get("text"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let is_error = reply
            .get("result")
            .and_then(|result| result.get("isError"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        assert!(!text.is_empty(), "unexpected tool result: {reply}");
        (text, is_error)
    }

    #[test]
    fn http_initialize_opens_a_session_and_lists_tools() {
        let test = TestSessions::new();
        let id = test.initialize("cursor");

        let reply = test.post(&id, r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#);
        assert_eq!(reply.status, 200);
        assert_eq!(reply.content_type, "application/json");
        let body = String::from_utf8(reply.body).expect("utf-8");
        assert!(
            body.contains(TOOL_POLTER_APPLY),
            "tools/list is missing {TOOL_POLTER_APPLY}: {body}"
        );

        let live = test.sessions.list();
        assert_eq!(live.len(), 1, "sessions = {live:?}");
        assert_eq!(live[0].id, id);
        assert_eq!(live[0].client_name, "cursor");
        assert_eq!(live[0].client_version, "1.2.3");
    }

    #[test]
    fn http_requires_a_known_session() {
        let test = TestSessions::new();
        for (name, session) in [("missing header", ""), ("unknown id", "01NOSUCHSESSION")] {
            let reply = test.post(session, r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#);
            assert_eq!(
                reply.status, 404,
                "{name}: status {}; want 404 so the client initializes again",
                reply.status
            );
        }
    }

    #[test]
    fn http_notifications_return_accepted() {
        let test = TestSessions::new();
        let id = test.initialize("codex");
        let reply = test.post(
            &id,
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        );
        assert_eq!(reply.status, 202, "status {}", reply.status);
        assert!(reply.body.is_empty(), "body {:?}", reply.body);
    }

    #[test]
    fn http_malformed_json_is_a_parse_error() {
        let test = TestSessions::new();
        let id = test.initialize("codex");
        let body = test.post(&id, "{not json");
        assert_eq!(body.status, 200);
        let reply = reply(&body.body);
        assert_eq!(
            reply.get("error").and_then(|error| error.get("code")),
            Some(&json!(-32700)),
            "reply: {reply}"
        );
    }

    #[test]
    fn http_unknown_method_is_an_error() {
        let test = TestSessions::new();
        let id = test.initialize("codex");
        let reply = reply(
            &test
                .post(&id, r#"{"jsonrpc":"2.0","id":5,"method":"no/such"}"#)
                .body,
        );
        let message = reply
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        assert!(message.contains("method not found"), "reply: {reply}");
    }

    #[test]
    fn http_tool_errors_are_reported_in_band_with_is_error() {
        let test = TestSessions::new();
        let id = test.initialize("codex");
        let reply = test.post(
            &id,
            r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"no.such.tool","arguments":{}}}"#,
        );
        let (text, is_error) = tool_text(&reply.body);
        assert!(
            is_error && !text.is_empty(),
            "an unknown tool must set isError, got {text:?} (isError={is_error})"
        );
    }

    #[test]
    fn http_rejects_unsupported_methods() {
        let test = TestSessions::new();
        let reply = test.sessions.serve("GET", "", &[]);
        assert_eq!(reply.status, 405, "status {}", reply.status);
        assert_eq!(reply.allow, "POST, DELETE");
    }

    #[test]
    fn http_delete_closes_the_session() {
        let test = TestSessions::new();
        let id = test.initialize("cursor");

        let reply = test.sessions.serve("DELETE", &id, &[]);
        assert_eq!(reply.status, 204);
        assert_eq!(test.sessions.list().len(), 0, "session still live");

        let again = test.post(&id, r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#);
        assert_eq!(
            again.status, 404,
            "a closed session must 404, got {}",
            again.status
        );
    }

    #[test]
    fn http_sessions_are_isolated() {
        let test = TestSessions::new();
        let first = test.initialize("claude-code");
        let second = test.initialize("codex");
        assert_ne!(first, second, "sessions must get distinct ids");
        let live = test.sessions.list();
        assert_eq!(live.len(), 2, "sessions = {live:?}");
        assert_eq!(live[0].client_name, "claude-code");
        assert_eq!(live[1].client_name, "codex");
    }

    #[test]
    fn http_write_joins_the_session_run_and_close_all_completes_it() {
        let test = TestSessions::new();
        let id = test.initialize("claude-code");
        let seeded = test
            .sessions
            .store()
            .list_agent_runs(&test.project_id, 50)
            .expect("seeded runs");

        let params = json!({
            "name": TOOL_POLTER_APPLY,
            "arguments": {"project_id": test.project_id, "text": ROADMAP},
        });
        let body = format!(r#"{{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{params}}}"#);
        let reply = test.post(&id, &body);
        let (text, is_error) = tool_text(&reply.body);
        assert!(!is_error, "apply failed: {text}");

        let known: Vec<String> = seeded.iter().map(|run| run.id.clone()).collect();
        let opened: Vec<_> = test
            .sessions
            .store()
            .list_agent_runs(&test.project_id, 50)
            .expect("runs")
            .into_iter()
            .filter(|run| !known.contains(&run.id))
            .collect();
        assert_eq!(opened.len(), 1, "session run = {opened:?}");
        assert_eq!(opened[0].status, crate::domains::agentrun::STATUS_OPEN);
        assert_eq!(opened[0].agent_label, "claude-code");

        let run_id = opened[0].id.clone();
        test.sessions.close_all();
        let run = test.sessions.store().get_agent_run(&run_id).expect("run");
        assert_eq!(run.status, crate::domains::agentrun::STATUS_COMPLETED);
        assert!(
            test.sessions.list().is_empty(),
            "CloseAll must drop every session"
        );
    }

    #[test]
    fn http_ping_without_a_session_is_not_found() {
        let test = TestSessions::new();
        let reply = test.post("", r#"{"jsonrpc":"2.0","id":9,"method":"ping"}"#);
        assert_eq!(reply.status, 404);
    }
}
