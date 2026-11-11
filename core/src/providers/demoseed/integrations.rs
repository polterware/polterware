//! Port of `backend/providers/demoseed/integrations.go`.

use crate::domains::integrations;
use crate::providers::sqlite::{self, Store};

use super::seed::{ENV_DEVELOPMENT_ID, ENV_PRODUCTION_ID, PROJECT_ID};

pub(super) fn seed_integrations(store: &Store) -> sqlite::Result<()> {
    let connections = [
        integrations::Connection {
            id: "int-github".to_string(),
            project_id: PROJECT_ID.to_string(),
            provider_id: "github".to_string(),
            status: "connected".to_string(),
            label: "local-demo".to_string(),
            summary: "Local GitHub connection. Refresh metadata to import repositories via gh."
                .to_string(),
            cli_command: "gh auth login".to_string(),
            sync_interval: "Manual".to_string(),
            freshness: "unknown".to_string(),
            ..integrations::Connection::default()
        },
        integrations::Connection {
            id: "int-aws".to_string(),
            project_id: PROJECT_ID.to_string(),
            provider_id: "aws".to_string(),
            status: "connected".to_string(),
            label: "demo-profile".to_string(),
            environment_id: ENV_PRODUCTION_ID.to_string(),
            summary: "Local AWS connection. Refresh metadata to import account inventory via aws."
                .to_string(),
            cli_command: "aws configure sso".to_string(),
            sync_interval: "Manual".to_string(),
            freshness: "unknown".to_string(),
            ..integrations::Connection::default()
        },
        integrations::Connection {
            id: "int-vercel".to_string(),
            project_id: PROJECT_ID.to_string(),
            provider_id: "vercel".to_string(),
            status: "connected".to_string(),
            label: "demo-workspace".to_string(),
            environment_id: ENV_PRODUCTION_ID.to_string(),
            summary: "Local Vercel connection. Refresh metadata to import projects via vercel."
                .to_string(),
            cli_command: "vercel login".to_string(),
            sync_interval: "Manual".to_string(),
            freshness: "unknown".to_string(),
            ..integrations::Connection::default()
        },
        integrations::Connection {
            id: "int-supabase".to_string(),
            project_id: PROJECT_ID.to_string(),
            provider_id: "supabase".to_string(),
            status: "connected".to_string(),
            label: "demo-project".to_string(),
            environment_id: ENV_DEVELOPMENT_ID.to_string(),
            summary: "Local Supabase connection. Refresh metadata to import projects via supabase."
                .to_string(),
            cli_command: "supabase login".to_string(),
            sync_interval: "Manual".to_string(),
            freshness: "unknown".to_string(),
            ..integrations::Connection::default()
        },
    ];
    for connection in connections {
        store.save_connection(connection)?;
    }
    Ok(())
}
