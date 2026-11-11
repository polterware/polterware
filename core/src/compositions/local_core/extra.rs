//! Port of `backend/compositions/local-core/extra.go`.
//!
//! Project settings, integration resources and sync runs, and the platform CLI
//! probe and sign-in plan.

use std::sync::Arc;

use crate::domains::integrations;
use crate::domains::projects;
use crate::local_v1;
use crate::providers::cli;

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
        Route::unary(
            "/polterware.local.v1.IntegrationService/ListIntegrationResources",
            list_integration_resources,
        ),
        Route::unary(
            "/polterware.local.v1.IntegrationService/ListIntegrationSyncRuns",
            list_integration_sync_runs,
        ),
        Route::unary(
            "/polterware.local.v1.IntegrationService/SaveIntegrationSyncRun",
            save_integration_sync_run,
        ),
        Route::unary(
            "/polterware.local.v1.IntegrationService/ProbeCliProvider",
            probe_cli_provider,
        ),
        Route::unary(
            "/polterware.local.v1.IntegrationService/BeginCliAuth",
            begin_cli_auth,
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
        default_environment_id: item.default_environment_id.clone(),
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

pub fn list_integration_resources(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListIntegrationResourcesRequest>,
) -> Result<local_v1::ListIntegrationResourcesResponse, ConnectError> {
    let items = core
        .store()
        .list_integration_resources(&request.message.project_id)
        .map_err(internal)?;
    Ok(local_v1::ListIntegrationResourcesResponse {
        resources: items.iter().map(integration_resource_proto).collect(),
    })
}

pub fn list_integration_sync_runs(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::ListIntegrationSyncRunsRequest>,
) -> Result<local_v1::ListIntegrationSyncRunsResponse, ConnectError> {
    let items = core
        .store()
        .list_integration_sync_runs(&request.message.project_id)
        .map_err(internal)?;
    Ok(local_v1::ListIntegrationSyncRunsResponse {
        runs: items.iter().map(sync_run_proto).collect(),
    })
}

pub fn save_integration_sync_run(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::SaveIntegrationSyncRunRequest>,
) -> Result<local_v1::SaveIntegrationSyncRunResponse, ConnectError> {
    let Some(item) = request.message.run else {
        return Err(invalid("sync run id and connection id are required"));
    };
    if item.id.is_empty() || item.connection_id.is_empty() {
        return Err(invalid("sync run id and connection id are required"));
    }
    let run = integrations::SyncRun {
        id: item.id.clone(),
        connection_id: item.connection_id.clone(),
        started_at: item.started_at.clone(),
        finished_at: item.finished_at.clone(),
        result: item.result.clone(),
        imported: item.imported,
        changed: item.changed,
        message: item.message.clone(),
    };
    core.store()
        .save_integration_sync_run(run.clone())
        .map_err(internal)?;
    Ok(local_v1::SaveIntegrationSyncRunResponse {
        run: Some(sync_run_proto(&run)),
    })
}

pub fn probe_cli_provider(
    _core: Arc<Core>,
    request: ConnectRequest<local_v1::ProbeCliProviderRequest>,
) -> Result<local_v1::ProbeCliProviderResponse, ConnectError> {
    if request.message.provider_id.is_empty() {
        return Err(invalid("provider id is required"));
    }
    let result = cli::probe(&request.message.provider_id);
    Ok(local_v1::ProbeCliProviderResponse {
        provider_id: result.provider_id,
        available: result.available,
        binary_path: result.binary_path,
        status: result.status,
        account_label: result.account_label,
        message: result.message,
    })
}

pub fn begin_cli_auth(
    _core: Arc<Core>,
    request: ConnectRequest<local_v1::BeginCliAuthRequest>,
) -> Result<local_v1::BeginCliAuthResponse, ConnectError> {
    if request.message.provider_id.is_empty() {
        return Err(invalid("provider id is required"));
    }
    let plan =
        cli::begin_auth(&request.message.provider_id).map_err(|_| invalid("unknown provider"))?;
    Ok(local_v1::BeginCliAuthResponse {
        provider_id: plan.provider_id,
        auth_command: plan.auth_command,
        docs_url: plan.docs_url,
        message: plan.message,
    })
}

pub fn settings_proto(item: &projects::Settings) -> local_v1::ProjectSettings {
    local_v1::ProjectSettings {
        project_id: item.project_id.clone(),
        description: item.description.clone(),
        default_environment_id: item.default_environment_id.clone(),
        repository: item.repository.clone(),
        agents_sync: item.agents_sync,
    }
}

pub fn integration_resource_proto(item: &integrations::Resource) -> local_v1::IntegrationResource {
    local_v1::IntegrationResource {
        id: item.id.clone(),
        connection_id: item.connection_id.clone(),
        name: item.name.clone(),
        kind: item.kind.clone(),
        external_id: item.external_id.clone(),
        environment_id: item.environment_id.clone(),
        status: item.status.clone(),
    }
}

pub fn sync_run_proto(item: &integrations::SyncRun) -> local_v1::IntegrationSyncRun {
    local_v1::IntegrationSyncRun {
        id: item.id.clone(),
        connection_id: item.connection_id.clone(),
        started_at: item.started_at.clone(),
        finished_at: item.finished_at.clone(),
        result: item.result.clone(),
        imported: item.imported,
        changed: item.changed,
        message: item.message.clone(),
    }
}
