//! Port of `backend/compositions/local-core/terminal.go`.
//!
//! The Desktop terminal and Connect's provider sign-in flow: one manager per
//! core, one PTY session per shell. `Core` keeps no terminal field here, so the
//! managers live in this module keyed by the core they serve and are dropped
//! with it.

use std::sync::{Arc, Mutex, OnceLock, Weak};

use crate::local_v1;
use crate::platform::observability::new_correlation_id;
use crate::providers::sqlite::format_rfc3339;
use crate::providers::terminal::{Manager, Session};

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::{Core, internal, invalid};

struct ManagerEntry {
    core: Weak<Core>,
    manager: Arc<Manager>,
}

/// One manager per core, as Go's `core.terminalOnce` and `core.Terminals` keep.
static TERMINALS: OnceLock<Mutex<Vec<ManagerEntry>>> = OnceLock::new();

fn terminal_manager(core: &Arc<Core>) -> Arc<Manager> {
    let registry = TERMINALS.get_or_init(|| Mutex::new(Vec::new()));
    let mut entries = registry.lock().unwrap_or_else(|poison| poison.into_inner());
    // A dropped core takes its manager with it, so a later core at the same
    // address starts empty.
    entries.retain(|entry| entry.core.strong_count() > 0);
    let key = Arc::as_ptr(core);
    if let Some(entry) = entries.iter().find(|entry| entry.core.as_ptr() == key) {
        return Arc::clone(&entry.manager);
    }
    let manager = Arc::new(Manager::new());
    entries.push(ManagerEntry {
        core: Arc::downgrade(core),
        manager: Arc::clone(&manager),
    });
    manager
}

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary(
            "/polterware.local.v1.TerminalService/CreateTerminalSession",
            create_terminal_session,
        ),
        Route::unary(
            "/polterware.local.v1.TerminalService/WriteTerminalSession",
            write_terminal_session,
        ),
        Route::unary(
            "/polterware.local.v1.TerminalService/ResizeTerminalSession",
            resize_terminal_session,
        ),
        Route::unary(
            "/polterware.local.v1.TerminalService/CloseTerminalSession",
            close_terminal_session,
        ),
        Route::unary(
            "/polterware.local.v1.TerminalService/ReadTerminalOutput",
            read_terminal_output,
        ),
        Route::unary(
            "/polterware.local.v1.TerminalService/ListTerminalSessions",
            list_terminal_sessions,
        ),
    ]
}

pub fn create_terminal_session(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::CreateTerminalSessionRequest>,
) -> Result<local_v1::CreateTerminalSessionResponse, ConnectError> {
    let message = request.message;
    if message.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let id = format!("term-{}", &new_correlation_id()[..12]);
    let cols = message.cols as u16;
    let rows = message.rows as u16;
    let target_id = message.target_id.clone();
    let shell = message.shell.clone();
    let cwd = message.cwd.clone();

    if !is_synthetic_local_target(&target_id) {
        return Err(invalid("terminal target not found"));
    }

    let session = terminal_manager(&core)
        .create(
            &id,
            &message.project_id,
            &synthetic_target_label(&target_id),
            &shell,
            &cwd,
            cols,
            rows,
        )
        .map_err(|err| internal(format!("create terminal: {err}")))?;
    Ok(local_v1::CreateTerminalSessionResponse {
        session: Some(terminal_proto(&session)),
    })
}

/// Treats empty, "local", and Connect auth-* targets as local PTYs without
/// consulting a nodes table.
fn is_synthetic_local_target(target_id: &str) -> bool {
    let trimmed = target_id.trim();
    trimmed.is_empty() || trimmed == "local" || trimmed.starts_with("auth-")
}

fn synthetic_target_label(target_id: &str) -> String {
    let trimmed = target_id.trim();
    if trimmed.is_empty() {
        return "local".to_string();
    }
    trimmed.to_string()
}

pub fn write_terminal_session(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::WriteTerminalSessionRequest>,
) -> Result<local_v1::WriteTerminalSessionResponse, ConnectError> {
    if request.message.id.is_empty() {
        return Err(invalid("session id is required"));
    }
    terminal_manager(&core)
        .write(&request.message.id, &request.message.data)
        .map_err(internal)?;
    Ok(local_v1::WriteTerminalSessionResponse {})
}

pub fn resize_terminal_session(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ResizeTerminalSessionRequest>,
) -> Result<local_v1::ResizeTerminalSessionResponse, ConnectError> {
    let session = terminal_manager(&core)
        .resize(
            &request.message.id,
            request.message.cols as u16,
            request.message.rows as u16,
        )
        .map_err(internal)?;
    Ok(local_v1::ResizeTerminalSessionResponse {
        session: Some(terminal_proto(&session)),
    })
}

pub fn close_terminal_session(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::CloseTerminalSessionRequest>,
) -> Result<local_v1::CloseTerminalSessionResponse, ConnectError> {
    let session = terminal_manager(&core)
        .close(&request.message.id)
        .map_err(internal)?;
    Ok(local_v1::CloseTerminalSessionResponse {
        session: Some(terminal_proto(&session)),
    })
}

pub fn read_terminal_output(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ReadTerminalOutputRequest>,
) -> Result<local_v1::ReadTerminalOutputResponse, ConnectError> {
    let (data, next, closed) = terminal_manager(&core)
        .read(&request.message.id, request.message.offset)
        .map_err(|err| ConnectError::not_found(err.to_string()))?;
    Ok(local_v1::ReadTerminalOutputResponse {
        data,
        next_offset: next,
        closed,
    })
}

pub fn list_terminal_sessions(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListTerminalSessionsRequest>,
) -> Result<local_v1::ListTerminalSessionsResponse, ConnectError> {
    let items = terminal_manager(&core).list(&request.message.project_id);
    Ok(local_v1::ListTerminalSessionsResponse {
        sessions: items.iter().map(|item| terminal_proto(item)).collect(),
    })
}

fn terminal_proto(item: &Session) -> local_v1::TerminalSession {
    local_v1::TerminalSession {
        id: item.id.clone(),
        project_id: item.project_id.clone(),
        target_id: item.target_id.clone(),
        status: item.status(),
        shell: item.shell.clone(),
        cwd: item.cwd.clone(),
        started_at: format_rfc3339(item.started_at),
        cols: item.cols() as i32,
        rows: item.rows() as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositions::local_core::test_support::Harness;
    use axum::http::HeaderMap;

    fn call<Req, Res>(
        harness: &Harness,
        handler: fn(Arc<Core>, ConnectRequest<Req>) -> Result<Res, ConnectError>,
        message: Req,
    ) -> Result<Res, ConnectError> {
        handler(
            Arc::clone(&harness.core),
            ConnectRequest {
                headers: HeaderMap::new(),
                message,
            },
        )
    }

    #[test]
    fn a_synthetic_target_is_local() {
        assert!(is_synthetic_local_target(""));
        assert!(is_synthetic_local_target(" local "));
        assert!(is_synthetic_local_target("auth-github"));
        assert!(!is_synthetic_local_target("remote-1"));
        assert_eq!(synthetic_target_label(""), "local");
        assert_eq!(synthetic_target_label(" local "), "local");
    }

    #[test]
    fn creating_a_session_validates_the_project_and_the_target() {
        let harness = Harness::start();
        let error = call(
            &harness,
            create_terminal_session,
            local_v1::CreateTerminalSessionRequest::default(),
        )
        .expect_err("project is required");
        assert_eq!(error.code, super::super::connect::Code::InvalidArgument);
        assert_eq!(error.message, "project id is required");

        let error = call(
            &harness,
            create_terminal_session,
            local_v1::CreateTerminalSessionRequest {
                project_id: "proj-1".to_string(),
                target_id: "remote-1".to_string(),
                ..local_v1::CreateTerminalSessionRequest::default()
            },
        )
        .expect_err("unknown target");
        assert_eq!(error.message, "terminal target not found");
    }

    #[test]
    fn session_operations_report_a_missing_session() {
        let harness = Harness::start();
        let error = call(
            &harness,
            write_terminal_session,
            local_v1::WriteTerminalSessionRequest::default(),
        )
        .expect_err("session id is required");
        assert_eq!(error.message, "session id is required");

        let error = call(
            &harness,
            write_terminal_session,
            local_v1::WriteTerminalSessionRequest {
                id: "term-nope".to_string(),
                data: Vec::new(),
            },
        )
        .expect_err("unknown session");
        assert_eq!(error.code, super::super::connect::Code::Internal);
        assert_eq!(error.message, "terminal session not found");

        let error = call(
            &harness,
            read_terminal_output,
            local_v1::ReadTerminalOutputRequest {
                id: "term-nope".to_string(),
                ..local_v1::ReadTerminalOutputRequest::default()
            },
        )
        .expect_err("unknown session");
        assert_eq!(error.code, super::super::connect::Code::NotFound);
        assert_eq!(error.message, "terminal session not found");

        let error = call(
            &harness,
            resize_terminal_session,
            local_v1::ResizeTerminalSessionRequest {
                id: "term-nope".to_string(),
                cols: 80,
                rows: 24,
            },
        )
        .expect_err("unknown session");
        assert_eq!(error.code, super::super::connect::Code::Internal);

        let error = call(
            &harness,
            close_terminal_session,
            local_v1::CloseTerminalSessionRequest {
                id: "term-nope".to_string(),
            },
        )
        .expect_err("unknown session");
        assert_eq!(error.code, super::super::connect::Code::Internal);

        let listed = call(
            &harness,
            list_terminal_sessions,
            local_v1::ListTerminalSessionsRequest::default(),
        )
        .expect("list");
        assert!(listed.sessions.is_empty());
    }
}
