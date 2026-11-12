//! Seeding the demo database.
//!
//! The demo database itself: the Polter project with its environments and
//! settings, then every seed step in order, and the empty project that starts
//! the way a user-registered one does.

use time::{Month, OffsetDateTime};

use crate::compositions::wedge;
use crate::domains::projects;
use crate::providers::sqlite::{self, Store};

use super::integrations::seed_integrations;
use super::knowledge::{seed_knowledge, seed_knowledge_links};
use super::work::seed_work;

pub const PROJECT_ID: &str = "proj-polter";
pub const ENV_PRODUCTION_ID: &str = "env-polter-prod";
pub const ENV_STAGING_ID: &str = "env-polter-staging";
pub const ENV_DEVELOPMENT_ID: &str = "env-polter-dev";

/// A project with no content, to show every empty state.
pub const EMPTY_PROJECT_ID: &str = "proj-empty";

/// Seeds the demo projects when the database has no project yet.
pub fn defaults(store: &Store) -> sqlite::Result<()> {
    let count: i64 = store
        .db
        .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))?;
    if count != 0 {
        return Ok(());
    }
    seed_project(store)?;
    seed_knowledge(store)?;
    let _ = store.sync_project_model(PROJECT_ID)?;
    seed_knowledge_links(store)?;
    seed_work(store)?;
    seed_integrations(store)?;
    seed_empty_project(store)
}

/// Creates a project the way a user-registered one starts: empty layout,
/// default settings, and one local environment.
fn seed_empty_project(store: &Store) -> sqlite::Result<()> {
    wedge::init_project(
        store,
        wedge::InitInput {
            id: EMPTY_PROJECT_ID.to_string(),
            name: "Empty project".to_string(),
            dir: String::new(),
        },
    )
    .map_err(|err| sqlite::Error::message(err.to_string()))?;
    Ok(())
}

fn seed_project(store: &Store) -> sqlite::Result<()> {
    let created = fixed_utc(2026, Month::March, 12, 9, 14, 2);
    store.save_project(projects::Project {
        id: PROJECT_ID.to_string(),
        name: "Polter".to_string(),
        created_at: created,
        updated_at: created,
        ..projects::Project::default()
    })?;
    let environments = [
        projects::Environment {
            id: ENV_PRODUCTION_ID.to_string(),
            project_id: PROJECT_ID.to_string(),
            name: "main".to_string(),
            kind: "production".to_string(),
            protection: "strict".to_string(),
            is_default: true,
            description: "Customer-facing production stage.".to_string(),
        },
        projects::Environment {
            id: ENV_STAGING_ID.to_string(),
            project_id: PROJECT_ID.to_string(),
            name: "staging".to_string(),
            kind: "staging".to_string(),
            protection: "standard".to_string(),
            is_default: false,
            description: "Pre-production validation stage.".to_string(),
        },
        projects::Environment {
            id: ENV_DEVELOPMENT_ID.to_string(),
            project_id: PROJECT_ID.to_string(),
            name: "dev".to_string(),
            kind: "development".to_string(),
            protection: "relaxed".to_string(),
            is_default: false,
            description: "Day-to-day engineering stage.".to_string(),
        },
    ];
    for environment in environments {
        store.save_environment(environment)?;
    }
    store.save_project_settings(projects::Settings {
        project_id: PROJECT_ID.to_string(),
        description: "Local project context for AI coding".to_string(),
        default_environment_id: ENV_PRODUCTION_ID.to_string(),
        repository: "polterware/polter".to_string(),
        ..projects::Settings::default()
    })
}

/// A fixed UTC instant, as the seed writes every stamp.
pub(super) fn fixed_utc(
    year: i32,
    month: Month,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> OffsetDateTime {
    time::Date::from_calendar_date(year, month, day)
        .expect("a valid seed date")
        .with_hms(hour, minute, second)
        .expect("a valid seed time")
        .assume_utc()
}
