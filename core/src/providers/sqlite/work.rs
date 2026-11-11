//! Port of `backend/providers/sqlite/work.go`.
//!
//! Only the row helper the other readers share is here; the Work cards, their
//! ops, and the snapshot land with their own run.

use rusqlite::{Params, Row};

use super::Result;

impl super::Store {
    /// Runs `query` and calls `scan` for each row. The Go helper every reader
    /// shares takes one argument; the port takes the statement's parameters.
    pub fn each_row(
        &self,
        query: &str,
        params: impl Params,
        mut scan: impl FnMut(&Row<'_>) -> rusqlite::Result<()>,
    ) -> Result<()> {
        let mut stmt = self.db.prepare(query)?;
        let mut rows = stmt.query(params)?;
        while let Some(row) = rows.next()? {
            scan(row)?;
        }
        Ok(())
    }
}
