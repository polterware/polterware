//! The repositories a project's Structure components link by name.
//!
//! The repositories a project's Structure components link by name.

use std::sync::Arc;

use crate::domains::projects;
use crate::local_v1;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::{Core, internal, invalid};

pub fn routes() -> Vec<Route> {
    vec![Route::unary(
        "/polterware.local.v1.ProjectService/ListProjectRepos",
        list_project_repos,
    )]
}

pub fn list_project_repos(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListProjectReposRequest>,
) -> Result<local_v1::ListProjectReposResponse, ConnectError> {
    if request.message.project_id.is_empty() {
        return Err(invalid("project id is required"));
    }
    let items = core
        .store()
        .list_repositories(&request.message.project_id)
        .map_err(internal)?;
    Ok(local_v1::ListProjectReposResponse {
        repos: items.iter().map(repo_proto).collect(),
    })
}

pub fn repo_proto(item: &projects::Repository) -> local_v1::ProjectRepo {
    local_v1::ProjectRepo {
        id: item.id.clone(),
        project_id: item.project_id.clone(),
        name: item.name.clone(),
        local_path: item.local_path.clone(),
        remote: item.remote.clone(),
    }
}
