//! Port of `backend/platform/db/sqlite`.

mod hooks;
mod migrate;

use std::io;
use std::os::unix::fs::DirBuilderExt;
use std::path::Path;

use rusqlite::Connection;

pub use hooks::{
    ChangeRecorder, PendingTx, SQLITE_DELETE, change_recorder, load_project_columns,
    project_column_index, register_hooks, row_project_id, set_change_recorder,
};
pub use migrate::{applied_versions, migrate, migration_versions};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("{0}")]
    Other(String),
}

impl Error {
    pub(crate) fn other(message: impl Into<String>) -> Error {
        Error::Other(message.into())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Opens the SQLite store at `path` with the core's pragmas: foreign keys on,
/// WAL journaling, and a five-second busy timeout. `:memory:` opens a private
/// in-memory database. The connection tracks the writes committed on it for the
/// change bus (`register_hooks`).
pub fn open(path: impl AsRef<Path>) -> Result<Connection> {
    let path = path.as_ref();
    if path != Path::new(":memory:")
        && let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty())
    {
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(parent)?;
    }
    let conn = if path == Path::new(":memory:") {
        Connection::open_in_memory()?
    } else {
        Connection::open(path)?
    };
    conn.execute_batch(
        "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;",
    )?;
    register_hooks(&conn);
    Ok(conn)
}

#[cfg(test)]
mod tests;
