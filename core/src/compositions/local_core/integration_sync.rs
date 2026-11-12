//! `SyncIntegrationConnection`: one provider's CLI inventory into a connection.
//!
//! `SyncIntegrationConnection` pulls one provider's bounded CLI inventory and
//! replaces the connection's resources, or records the run as failed and leaves
//! them as they are.

use std::sync::{Arc, LazyLock, Mutex};

use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use crate::domains::integrations;
use crate::local_v1;
use crate::providers::cli;
use crate::providers::sqlite::{self, format_rfc3339};

use super::connect::{ConnectError, ConnectRequest, Route};
use super::server::{Core, internal, invalid};

/// The inventory pull, replaced by the tests as Go's `listInventoryFn` is.
type ListInventoryHook = fn(&str) -> cli::InventoryResult;

static LIST_INVENTORY: LazyLock<Mutex<ListInventoryHook>> =
    LazyLock::new(|| Mutex::new(cli::list_inventory));

fn inventory_hook() -> ListInventoryHook {
    *LIST_INVENTORY
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

#[cfg(test)]
fn set_inventory_hook(hook: ListInventoryHook) {
    *LIST_INVENTORY
        .lock()
        .unwrap_or_else(|poison| poison.into_inner()) = hook;
}

#[cfg(test)]
fn reset_inventory_hook() {
    set_inventory_hook(cli::list_inventory);
}

pub fn routes() -> Vec<Route> {
    vec![Route::unary(
        "/polterware.local.v1.IntegrationService/SyncIntegrationConnection",
        sync_integration_connection,
    )]
}

pub fn sync_integration_connection(
    core: Arc<Core>,
    request: ConnectRequest<local_v1::SyncIntegrationConnectionRequest>,
) -> Result<local_v1::SyncIntegrationConnectionResponse, ConnectError> {
    let connection_id = request.message.connection_id.clone();
    if connection_id.is_empty() {
        return Err(invalid("connection id is required"));
    }
    let connection = match core.store().get_connection(&connection_id) {
        Ok(connection) => connection,
        Err(err) if err.is_no_rows() => {
            return Err(ConnectError::not_found("connection not found"));
        }
        Err(err) => return Err(internal(err)),
    };

    let started_at = OffsetDateTime::now_utc();
    let inventory = inventory_hook()(&connection.provider_id);
    let finished_at = OffsetDateTime::now_utc();

    if inventory.err.is_some() {
        let run = {
            let store = core.store();
            record_failed_sync(
                &store,
                connection,
                started_at,
                finished_at,
                &inventory.message,
            )
            .map_err(internal)?
        };
        return Ok(local_v1::SyncIntegrationConnectionResponse {
            run: Some(sync_run_proto(&run)),
        });
    }
    let run = {
        let store = core.store();
        replace_inventory(&store, connection, &inventory, started_at, finished_at)
            .map_err(internal)?
    };
    Ok(local_v1::SyncIntegrationConnectionResponse {
        run: Some(sync_run_proto(&run)),
    })
}

fn record_failed_sync(
    store: &sqlite::Store,
    mut connection: integrations::Connection,
    started_at: OffsetDateTime,
    finished_at: OffsetDateTime,
    message: &str,
) -> sqlite::Result<integrations::SyncRun> {
    let message = if message.is_empty() {
        "CLI inventory sync failed".to_string()
    } else {
        message.to_string()
    };
    let run = integrations::SyncRun {
        id: sync_run_id(&connection.id, finished_at),
        connection_id: connection.id.clone(),
        started_at: format_rfc3339(started_at),
        finished_at: format_rfc3339(finished_at),
        result: "failed".to_string(),
        imported: 0,
        changed: 0,
        message: message.clone(),
    };
    store.save_integration_sync_run(run.clone())?;
    connection.freshness = "stale".to_string();
    connection.error_message = message;
    connection.last_sync_at = format_rfc3339(finished_at);
    store.save_connection(connection)?;
    Ok(run)
}

fn replace_inventory(
    store: &sqlite::Store,
    mut connection: integrations::Connection,
    inventory: &cli::InventoryResult,
    started_at: OffsetDateTime,
    finished_at: OffsetDateTime,
) -> sqlite::Result<integrations::SyncRun> {
    let old_count = store.count_integration_resources_by_connection(&connection.id)?;
    store.delete_integration_resources_by_connection(&connection.id)?;
    for item in &inventory.items {
        store.save_integration_resource(integrations::Resource {
            id: integration_resource_id(&connection.id, &item.external_id),
            connection_id: connection.id.clone(),
            name: item.name.clone(),
            kind: item.kind.clone(),
            external_id: item.external_id.clone(),
            environment_id: connection.environment_id.clone(),
            status: "synced".to_string(),
        })?;
    }

    let imported = inventory.items.len();
    let changed = abs_int(imported as i64 - old_count as i64);
    let message = if inventory.message.is_empty() {
        format!("Imported {imported} items from {}.", connection.provider_id)
    } else {
        inventory.message.clone()
    };
    let run = integrations::SyncRun {
        id: sync_run_id(&connection.id, finished_at),
        connection_id: connection.id.clone(),
        started_at: format_rfc3339(started_at),
        finished_at: format_rfc3339(finished_at),
        result: "success".to_string(),
        imported: imported as i32,
        changed: changed as i32,
        message: message.clone(),
    };
    store.save_integration_sync_run(run.clone())?;

    connection.last_sync_at = format_rfc3339(finished_at);
    connection.freshness = "fresh".to_string();
    connection.error_message.clear();
    connection.summary = message;
    store.save_connection(connection)?;
    Ok(run)
}

fn integration_resource_id(connection_id: &str, external_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(connection_id.as_bytes());
    hasher.update([0u8]);
    hasher.update(external_id.as_bytes());
    let sum = hasher.finalize();
    format!("ires-{}-{}", connection_id, hex::encode(&sum[..8]))
}

fn sync_run_id(connection_id: &str, at: OffsetDateTime) -> String {
    format!(
        "sync-{}-{}",
        connection_id,
        at.unix_timestamp_nanos() as i64
    )
}

fn abs_int(value: i64) -> i64 {
    if value < 0 { -value } else { value }
}

fn sync_run_proto(item: &integrations::SyncRun) -> local_v1::IntegrationSyncRun {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compositions::local_core::test_support::Harness;
    use crate::domains::projects;
    use crate::providers::cli::Error as CliError;

    /// Restores the inventory pull when the test ends, even on a panic.
    struct HookGuard;

    impl Drop for HookGuard {
        fn drop(&mut self) {
            reset_inventory_hook();
        }
    }

    fn seed_project(harness: &Harness, project_id: &str) {
        let store = harness.core.store();
        store
            .save_project(projects::Project {
                id: project_id.to_string(),
                name: "Test".to_string(),
                created_at: OffsetDateTime::now_utc(),
                updated_at: OffsetDateTime::now_utc(),
                status: projects::STATUS_ACTIVE.to_string(),
                ..projects::Project::default()
            })
            .expect("project");
    }

    fn save_connection(
        harness: &Harness,
        id: &str,
        project_id: &str,
        provider: &str,
        freshness: &str,
    ) -> integrations::Connection {
        let connection = integrations::Connection {
            id: id.to_string(),
            project_id: project_id.to_string(),
            provider_id: provider.to_string(),
            status: "connected".to_string(),
            label: "demo".to_string(),
            freshness: freshness.to_string(),
            ..integrations::Connection::default()
        };
        harness
            .core
            .store()
            .save_connection(connection.clone())
            .expect("connection");
        connection
    }

    fn sync(
        harness: &Harness,
        connection_id: &str,
    ) -> Result<local_v1::SyncIntegrationConnectionResponse, ConnectError> {
        sync_integration_connection(
            Arc::clone(&harness.core),
            ConnectRequest {
                headers: axum::http::HeaderMap::new(),
                message: local_v1::SyncIntegrationConnectionRequest {
                    connection_id: connection_id.to_string(),
                },
            },
        )
    }

    #[test]
    fn a_failed_sync_leaves_the_resources_and_marks_the_connection_stale() {
        let _guard = HookGuard;
        let harness = Harness::start();
        seed_project(&harness, "proj-test");
        let connection = save_connection(&harness, "int-github", "proj-test", "github", "fresh");
        harness
            .core
            .store()
            .save_integration_resource(integrations::Resource {
                id: "ires-keep".to_string(),
                connection_id: connection.id.clone(),
                name: "keep-me".to_string(),
                kind: "Repository".to_string(),
                external_id: "keep-me".to_string(),
                status: "synced".to_string(),
                ..integrations::Resource::default()
            })
            .expect("resource");

        set_inventory_hook(|_provider| cli::InventoryResult {
            items: Vec::new(),
            message: "gh auth required".to_string(),
            err: Some(CliError::Message("auth required".to_string())),
        });

        let response = sync(&harness, &connection.id).expect("sync");
        let run = response.run.expect("run");
        assert_eq!(run.result, "failed");

        let store = harness.core.store();
        assert_eq!(
            store
                .count_integration_resources_by_connection(&connection.id)
                .expect("count"),
            1,
            "resources stay unchanged"
        );
        let updated = store.get_connection(&connection.id).expect("connection");
        assert_eq!(updated.freshness, "stale");
        assert_eq!(updated.error_message, "gh auth required");
    }

    #[test]
    fn a_successful_sync_replaces_the_resources() {
        let _guard = HookGuard;
        let harness = Harness::start();
        seed_project(&harness, "proj-test");
        let connection = save_connection(&harness, "int-docker", "proj-test", "docker", "stale");
        harness
            .core
            .store()
            .save_integration_resource(integrations::Resource {
                id: "ires-old".to_string(),
                connection_id: connection.id.clone(),
                name: "old".to_string(),
                kind: "Container".to_string(),
                external_id: "old".to_string(),
                status: "synced".to_string(),
                ..integrations::Resource::default()
            })
            .expect("resource");

        set_inventory_hook(|_provider| cli::InventoryResult {
            items: vec![cli::InventoryItem {
                name: "nginx".to_string(),
                kind: "Container".to_string(),
                external_id: "abc123".to_string(),
            }],
            message: "Imported 1 containers from docker.".to_string(),
            err: None,
        });

        let response = sync(&harness, &connection.id).expect("sync");
        let run = response.run.expect("run");
        assert_eq!(run.result, "success");
        assert_eq!(run.imported, 1);

        let store = harness.core.store();
        assert_eq!(
            store
                .count_integration_resources_by_connection(&connection.id)
                .expect("count"),
            1
        );
        let items = store
            .list_integration_resources(&connection.project_id)
            .expect("resources");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "nginx");
        assert_eq!(
            items[0].id,
            integration_resource_id(&connection.id, "abc123")
        );

        let updated = store.get_connection(&connection.id).expect("connection");
        assert_eq!(updated.freshness, "fresh");
        assert!(updated.error_message.is_empty());
        assert_eq!(updated.summary, "Imported 1 containers from docker.");
    }

    #[test]
    fn an_unknown_connection_is_not_found() {
        let harness = Harness::start();
        let error = sync(&harness, "missing").expect_err("not found");
        assert_eq!(error.code, super::super::connect::Code::NotFound);
        assert_eq!(error.message, "connection not found");
    }

    #[test]
    fn integration_resource_ids_are_stable_and_distinct() {
        let a = integration_resource_id("int-a", "repo/x");
        let b = integration_resource_id("int-a", "repo/x");
        let c = integration_resource_id("int-a", "repo/y");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a.starts_with("ires-int-a-"));
    }
}
