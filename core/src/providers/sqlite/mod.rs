//! Port of `backend/providers/sqlite`.
//!
//! The local SQLite store: one [`Store`] per database, opened through
//! `platform::db::sqlite` with the migrations applied and the change hooks
//! registered. Every package file adds its own `impl Store` block here, one
//! module per Go file; this module holds the store itself, the shared helpers,
//! and the error type they return.
//!
//! Methods take `&self`, like the Go store's pool: a write that needs a
//! transaction uses `unchecked_transaction`, and the SQLite connection is held
//! behind the caller's lock.

mod agent_access;
mod agent_run;
mod canon;
mod canon_facts;
mod canvas;
mod commit_links;
mod conflicts;
mod context;
mod decision;
mod entities;
mod health;
mod hook_context;
mod hook_switches;
mod inbox;
mod milestones;
mod polter_facts;
mod repos;
mod rules;
mod saved_filters;
mod scope;
mod store;
mod structure;
mod work;

pub use agent_run::{ChangeRow, change_limit};
pub use scope::*;
pub use store::*;

use time::OffsetDateTime;
use time::UtcOffset;
use time::format_description::well_known::Rfc3339;

/// The errors the store returns: SQLite's own, the platform database's, and
/// the domain errors the Go package returns as sentinels, plus the plain
/// messages its validation raises. A variant is here when a Go caller compares
/// the error against that domain's sentinel with `errors.Is`.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Db(#[from] crate::platform::db::sqlite::Error),
    #[error(transparent)]
    Knowledge(#[from] crate::domains::knowledge::KnowledgeError),
    #[error(transparent)]
    Canon(#[from] crate::domains::canon::Error),
    #[error(transparent)]
    Canvas(#[from] crate::domains::canvas::Error),
    #[error(transparent)]
    Filters(#[from] crate::domains::filters::SavedFilterError),
    #[error(transparent)]
    AgentRun(#[from] crate::domains::agentrun::Error),
    #[error(transparent)]
    Work(#[from] crate::domains::work::Error),
    #[error("{0}")]
    Message(String),
}

impl Error {
    /// A plain message, the port of the package's `fmt.Errorf` and
    /// `errors.New` sites.
    pub fn message(text: impl Into<String>) -> Error {
        Error::Message(text.into())
    }

    /// Reports whether this is SQLite's "no rows" error, the port's
    /// `errors.Is(err, sql.ErrNoRows)`.
    pub fn is_no_rows(&self) -> bool {
        matches!(self, Error::Sqlite(rusqlite::Error::QueryReturnedNoRows))
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Formats a time the way Go's `time.RFC3339` layout does: UTC, second
/// precision, with no fractional seconds.
pub fn format_rfc3339(value: OffsetDateTime) -> String {
    format_rfc3339_seconds(value.to_offset(UtcOffset::UTC))
}

/// Formats a time the way Go's `time.RFC3339` layout does at a call site that
/// keeps the time's own zone: second precision, and no conversion, because
/// Go's `Time.Format` does not convert to UTC.
pub fn format_rfc3339_zone(value: OffsetDateTime) -> String {
    format_rfc3339_seconds(value)
}

/// Formats a time the way Go's `time.RFC3339Nano` layout does: UTC, with the
/// fraction trimmed of trailing zeros (absent when there is none). A step's
/// order depends on it, so the fraction is never dropped.
pub fn format_rfc3339_nano(value: OffsetDateTime) -> String {
    value
        .to_offset(UtcOffset::UTC)
        .format(&Rfc3339)
        .expect("RFC3339 formatting cannot fail")
}

/// A stamp at second precision, in the value's own zone.
fn format_rfc3339_seconds(value: OffsetDateTime) -> String {
    value
        .replace_nanosecond(0)
        .expect("a nanosecond is always in range")
        .format(&Rfc3339)
        .expect("RFC3339 formatting cannot fail")
}

/// Reads a stored RFC3339 stamp. The Go package ignores parse errors and keeps
/// the zero time; the port keeps the Unix epoch, which is its zero value.
pub fn parse_rfc3339(value: &str) -> OffsetDateTime {
    OffsetDateTime::parse(value, &Rfc3339).unwrap_or(OffsetDateTime::UNIX_EPOCH)
}

/// A stored stamp that is empty when absent, the port of writing `time.Time`
/// through `Format(time.RFC3339)`.
pub fn format_optional_rfc3339(value: Option<OffsetDateTime>) -> String {
    value.map(format_rfc3339).unwrap_or_default()
}

/// Reads an optional stamp: an empty column is `None`.
pub fn parse_optional_rfc3339(value: &str) -> Option<OffsetDateTime> {
    if value.is_empty() {
        None
    } else {
        Some(parse_rfc3339(value))
    }
}

/// A nullable text column: an empty value is stored as `NULL`.
pub fn nullable(value: &str) -> Option<&str> {
    if value.is_empty() { None } else { Some(value) }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_package_exports_what_other_packages_use() {
        // The Go package exports these, so the port re-exports them at the
        // package root: compositions read the change history rows, resolve a
        // scope, and name the scope kinds without reaching into a private
        // module.
        use crate::providers::sqlite::{
            ChangeRow, NodeScope, SCOPE_DECISION, SCOPE_FACT, SCOPE_ITEM, SCOPE_KNOWLEDGE,
            SCOPE_RULE, change_limit,
        };

        let row = ChangeRow {
            step_id: "step".to_string(),
            agent_run_id: "run".to_string(),
            project_id: "project".to_string(),
            actor_kind: "agent".to_string(),
            agent_label: "codex".to_string(),
            run_status: "open".to_string(),
            run_task: "task".to_string(),
            summary: "summary".to_string(),
            payload_json: "{}".to_string(),
            created_at: time::OffsetDateTime::UNIX_EPOCH,
        };
        assert_eq!(row.step_id, "step");
        assert_eq!(change_limit(0), 50);
        assert_eq!(change_limit(201), 50);
        assert_eq!(change_limit(10), 10);
        let _scope = NodeScope::default();
        assert_eq!(
            [
                SCOPE_DECISION,
                SCOPE_FACT,
                SCOPE_RULE,
                SCOPE_KNOWLEDGE,
                SCOPE_ITEM
            ],
            ["decision", "fact", "rule", "knowledge", "item"]
        );
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    /// A store in its own temporary database, and the directory that holds it.
    pub struct TempStore {
        pub store: Store,
        #[allow(dead_code)]
        pub dir: tempfile::TempDir,
    }

    impl TempStore {
        pub fn new() -> TempStore {
            TempStore::named("store.db")
        }

        pub fn named(file: &str) -> TempStore {
            let dir = tempfile::tempdir().expect("temp dir");
            let path = dir.path().join(file);
            let store = Store::new(&path).expect("open store");
            TempStore { store, dir }
        }
    }

    /// A project row, the fixture every other row hangs from.
    pub fn save_project(store: &Store, id: &str, now: OffsetDateTime) {
        store
            .save_project(crate::domains::projects::Project {
                id: id.to_string(),
                name: id.to_string(),
                created_at: now,
                updated_at: now,
                status: crate::domains::projects::STATUS_ACTIVE.to_string(),
                archived_at: None,
            })
            .expect("save project");
    }
}
