//! Change tracking for the change bus (Decision 0007).
//!
//! Each connection opened by [`super::open`] registers SQLite's pre-update,
//! commit, and rollback hooks ([`register_hooks`]). The pre-update hook
//! accumulates, per connection, the tables a transaction wrote and the project
//! ids found in the written rows; the commit hook calls a [`ChangeRecorder`]
//! once per commit, before the commit is visible to other connections; the
//! rollback hook drops what the rolled-back transaction recorded.
//!
//! [`PendingTx::pre_update`] and [`load_project_columns`] keep the Go behavior:
//! only the `main` database counts, `schema_migrations` is ignored, and the
//! project id column is `project_id` (or `id` on `projects`).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use rusqlite::Connection;
use rusqlite::hooks::PreUpdateCase;

use super::Result;

/// Receives, once per committed transaction, the tables it wrote and the
/// project ids found in the written rows (Decision 0007). It runs with the
/// commit, before the commit is visible to other connections, so it must only
/// record and return.
pub type ChangeRecorder = Arc<dyn Fn(&[String], &[String]) + Send + Sync>;

fn recorder_slot() -> &'static Mutex<Option<ChangeRecorder>> {
    static SLOT: OnceLock<Mutex<Option<ChangeRecorder>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn columns_slot() -> &'static Mutex<Option<BTreeMap<String, usize>>> {
    static SLOT: OnceLock<Mutex<Option<BTreeMap<String, usize>>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

fn guard<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// Installs `recorder` for the process; `None` turns recording off.
pub fn set_change_recorder(recorder: Option<ChangeRecorder>) {
    *guard(recorder_slot()) = recorder;
}

/// The installed recorder, when recording is on.
pub fn change_recorder() -> Option<ChangeRecorder> {
    guard(recorder_slot()).clone()
}

/// The SQLite operation code for DELETE (`sqlite3.SQLITE_DELETE`).
pub const SQLITE_DELETE: i32 = 9;

/// Records, for each table, which column holds its project id, so change
/// events can name the projects a transaction touched.
pub fn load_project_columns(conn: &Connection) -> Result<()> {
    let tables: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    let mut columns = BTreeMap::new();
    for table in tables {
        let want = if table == "projects" {
            "id"
        } else {
            "project_id"
        };
        let mut info = conn.prepare("SELECT cid, name FROM pragma_table_info(?)")?;
        let rows = info.query_map([&table], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (cid, name) = row?;
            if name == want {
                columns.insert(table.clone(), cid as usize);
            }
        }
    }
    *guard(columns_slot()) = Some(columns);
    Ok(())
}

/// The column index that holds a table's project id, when the table has one.
pub fn project_column_index(table: &str) -> Option<usize> {
    guard(columns_slot())
        .as_ref()
        .and_then(|columns| columns.get(table).copied())
}

/// Reads the project id column of the written row, when the table has one
/// (`projects.id` counts as the project id of that row). `value_at` reads the
/// row's value at the column index.
pub fn row_project_id<F: FnOnce(usize) -> Option<String>>(table: &str, value_at: F) -> String {
    let Some(index) = project_column_index(table) else {
        return String::new();
    };
    value_at(index).unwrap_or_default()
}

/// Accumulates what one connection's open transaction wrote.
#[derive(Debug, Default)]
pub struct PendingTx {
    tables: BTreeSet<String>,
    projects: BTreeSet<String>,
}

impl PendingTx {
    pub fn new() -> PendingTx {
        PendingTx::default()
    }

    /// Records one written row: its table and, when the table has the column,
    /// its project id.
    pub fn pre_update(&mut self, database_name: &str, table_name: &str, project_id: Option<&str>) {
        if change_recorder().is_none()
            || database_name != "main"
            || table_name == "schema_migrations"
        {
            return;
        }
        self.tables.insert(table_name.to_string());
        if let Some(project_id) = project_id.filter(|id| !id.is_empty()) {
            self.projects.insert(project_id.to_string());
        }
    }

    /// Calls the recorder when the transaction wrote a tracked table and starts
    /// a new accumulation. Reports whether a recorder was called.
    pub fn commit(&mut self) -> bool {
        let tables: Vec<String> = self.tables.iter().cloned().collect();
        let projects: Vec<String> = self.projects.iter().cloned().collect();
        self.tables.clear();
        self.projects.clear();
        if tables.is_empty() {
            return false;
        }
        if let Some(recorder) = change_recorder() {
            recorder(&tables, &projects);
        }
        true
    }

    /// Drops what the rolled-back transaction recorded.
    pub fn reset(&mut self) {
        self.tables.clear();
        self.projects.clear();
    }
}

/// Registers the change-tracking hooks on one connection, as the Go package's
/// connection hook does for every connection of the process.
pub fn register_hooks(conn: &Connection) {
    let pending = Arc::new(Mutex::new(PendingTx::new()));

    let pre_update = Arc::clone(&pending);
    conn.preupdate_hook(Some(
        move |_action, database_name: &str, table_name: &str, case: &PreUpdateCase| {
            let project_id = case_project_id(table_name, case);
            guard(&pre_update).pre_update(database_name, table_name, project_id.as_deref());
        },
    ));

    let commit = Arc::clone(&pending);
    conn.commit_hook(Some(move || {
        guard(&commit).commit();
        // The recorder only records; it never rolls the commit back.
        false
    }));

    let rollback = Arc::clone(&pending);
    conn.rollback_hook(Some(move || guard(&rollback).reset()));
}

/// Reads the project id column of the written row, when the table has one
/// (`projects.id` counts as the project id of that row). Delete reads the old
/// row; an insert or an update reads the new one.
fn case_project_id(table: &str, case: &PreUpdateCase) -> Option<String> {
    let index = i32::try_from(project_column_index(table)?).ok()?;
    let value = match case {
        PreUpdateCase::Insert(accessor) => accessor.get_new_column_value(index).ok()?,
        PreUpdateCase::Delete(accessor) => accessor.get_old_column_value(index).ok()?,
        PreUpdateCase::Update {
            new_value_accessor, ..
        } => new_value_accessor.get_new_column_value(index).ok()?,
        PreUpdateCase::Unknown => return None,
    };
    value.as_str().ok().map(str::to_owned)
}
