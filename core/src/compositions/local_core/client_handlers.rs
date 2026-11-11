//! Port of `backend/compositions/local-core/client_handlers.go`.
//!
//! What a CLI client asks the core about itself: which project a directory
//! belongs to, registering a project, and the MCP sessions connected here.

use std::sync::Arc;

use crate::compositions::wedge;
use crate::local_v1;
use crate::providers::sqlite::format_rfc3339;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::{Core, invalid, project_proto};

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary(
            "/polterware.local.v1.ProjectService/ResolveProject",
            resolve_project,
        ),
        Route::unary(
            "/polterware.local.v1.ProjectService/InitProject",
            init_project,
        ),
        Route::unary(
            "/polterware.local.v1.AgentRunService/ListMcpSessions",
            list_mcp_sessions,
        ),
    ]
}

/// Serves CLI project resolution: an explicit id, or the registered project
/// whose root contains the caller's directory.
pub fn resolve_project(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ResolveProjectRequest>,
) -> Result<local_v1::ResolveProjectResponse, ConnectError> {
    let project = wedge::resolve_project(&core.store(), &request.message.id, &request.message.dir)
        .map_err(|err| ConnectError::not_found(err.to_string()))?;
    Ok(local_v1::ResolveProjectResponse {
        project: Some(project_proto(&project)),
    })
}

/// Registers a project; `dir` is optional and must be absolute (the client
/// resolves it against its own working directory).
pub fn init_project(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::InitProjectRequest>,
) -> Result<local_v1::InitProjectResponse, ConnectError> {
    let project = wedge::init_project(
        &core.store(),
        wedge::InitInput {
            id: request.message.id.clone(),
            name: request.message.name.clone(),
            dir: request.message.dir.clone(),
        },
    )
    .map_err(|err| invalid(err.to_string()))?;
    Ok(local_v1::InitProjectResponse {
        project: Some(project_proto(&project)),
    })
}

/// Lists MCP clients connected to this core.
pub fn list_mcp_sessions(
    core: Arc<Core>,
    _request: ConnectRequest<local_v1::ListMcpSessionsRequest>,
) -> Result<local_v1::ListMcpSessionsResponse, ConnectError> {
    let sessions = core.mcp.list();
    Ok(local_v1::ListMcpSessionsResponse {
        sessions: sessions
            .iter()
            .map(|session| local_v1::McpSession {
                id: session.id.clone(),
                client_name: session.client_name.clone(),
                client_version: session.client_version.clone(),
                started_at: format_rfc3339(session.started_at),
                last_seen_at: format_rfc3339(session.last_seen),
            })
            .collect(),
    })
}
