//! Project settings.
//!
//! Reads and writes project settings.

use std::sync::Arc;

use crate::domains::projects;
use crate::local_v1;

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::{Core, internal, invalid};

pub fn routes() -> Vec<Route> {
    vec![
        Route::unary(
            "/polterware.local.v1.ProjectService/GetProjectSettings",
            get_project_settings,
        ),
        Route::unary(
            "/polterware.local.v1.ProjectService/SaveProjectSettings",
            save_project_settings,
        ),
    ]
}

pub fn get_project_settings(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::GetProjectSettingsRequest>,
) -> Result<local_v1::GetProjectSettingsResponse, ConnectError> {
    let item = match core
        .store()
        .get_project_settings(&request.message.project_id)
    {
        Ok(item) => item,
        Err(err) if err.is_no_rows() => projects::Settings {
            project_id: request.message.project_id.clone(),
            ..projects::Settings::default()
        },
        Err(err) => return Err(internal(err)),
    };
    Ok(local_v1::GetProjectSettingsResponse {
        settings: Some(settings_proto(&item)),
    })
}

pub fn save_project_settings(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::SaveProjectSettingsRequest>,
) -> Result<local_v1::SaveProjectSettingsResponse, ConnectError> {
    let Some(item) = request.message.settings else {
        return Err(invalid("project settings are required"));
    };
    if item.project_id.is_empty() {
        return Err(invalid("project settings are required"));
    }
    let settings = projects::Settings {
        project_id: item.project_id.clone(),
        description: item.description.clone(),
        repository: item.repository.clone(),
        agents_sync: false,
    };
    let store = core.store();
    store
        .save_project_settings(settings.clone())
        .map_err(internal)?;
    let saved = store
        .get_project_settings(&settings.project_id)
        .map_err(internal)?;
    Ok(local_v1::SaveProjectSettingsResponse {
        settings: Some(settings_proto(&saved)),
    })
}

pub fn settings_proto(item: &projects::Settings) -> local_v1::ProjectSettings {
    local_v1::ProjectSettings {
        project_id: item.project_id.clone(),
        description: item.description.clone(),
        repository: item.repository.clone(),
        agents_sync: item.agents_sync,
    }
}
