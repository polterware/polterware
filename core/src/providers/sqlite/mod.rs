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

pub use scope::*;
pub use store::*;

use time::OffsetDateTime;
use time::UtcOffset;
use time::format_description::well_known::Rfc3339;

/// The errors the store returns: SQLite's own, the platform database's, and
/// the domain errors the Go package returns as sentinels, plus the plain
/// messages its validation raises.
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
    value
        .to_offset(UtcOffset::UTC)
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
