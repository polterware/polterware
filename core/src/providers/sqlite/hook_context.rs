//! Port of `backend/providers/sqlite/hook_context.go`.
//!
//! What each agent session already received from the context hook: one row per
//! session and Structure node, holding the hash of the brief it sent.

use rusqlite::params;
use time::{Duration, OffsetDateTime};

use super::{Result, format_rfc3339};

/// How long a session's sent briefs are remembered.
const HOOK_CONTEXT_KEEP: Duration = Duration::days(7);

impl super::Store {
    /// Records that a session received a node's brief with `hash`. `fresh` is
    /// false when the session already has that same brief. Rows older than a
    /// week are dropped on the way.
    pub fn mark_hook_context_seen(
        &self,
        session_id: &str,
        project_id: &str,
        node_id: &str,
        hash: &str,
        at: OffsetDateTime,
    ) -> Result<bool> {
        let current = self.db.query_row(
            r#"SELECT brief_hash FROM hook_context_seen WHERE session_id=? AND project_id=? AND node_id=?"#,
            params![session_id, project_id, node_id],
            |row| row.get::<_, String>(0),
        );
        match current {
            Ok(current) if current == hash => return Ok(false),
            Ok(_) => {}
            Err(rusqlite::Error::QueryReturnedNoRows) => {}
            Err(err) => return Err(err.into()),
        }
        let stamp = format_rfc3339(at);
        self.db.execute(
            r#"INSERT INTO hook_context_seen(session_id,project_id,node_id,brief_hash,seen_at) VALUES(?,?,?,?,?)
		ON CONFLICT(session_id,project_id,node_id) DO UPDATE SET brief_hash=excluded.brief_hash, seen_at=excluded.seen_at"#,
            params![session_id, project_id, node_id, hash, stamp],
        )?;
        self.db.execute(
            r#"DELETE FROM hook_context_seen WHERE seen_at < ?"#,
            params![format_rfc3339(at - HOOK_CONTEXT_KEEP)],
        )?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::params;
    use time::OffsetDateTime;

    use crate::providers::sqlite::test_support::{TempStore, save_project};

    use super::*;

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    #[test]
    fn hook_context_seen_is_fresh_once_per_brief() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);

        assert!(
            store
                .mark_hook_context_seen("s", "p", "n", "h1", at)
                .expect("first brief")
        );
        assert!(
            !store
                .mark_hook_context_seen("s", "p", "n", "h1", at)
                .expect("same brief"),
            "the same brief must not be fresh twice"
        );
        assert!(
            store
                .mark_hook_context_seen("s", "p", "n", "h2", at)
                .expect("changed brief"),
            "a changed brief is fresh again"
        );

        let old = at - Duration::days(8);
        store
            .db
            .execute(
                "INSERT INTO hook_context_seen(session_id,project_id,node_id,brief_hash,seen_at) VALUES('old','p','n2','h',?)",
                params![format_rfc3339(old)],
            )
            .expect("old row");
        store
            .mark_hook_context_seen("s", "p", "n", "h3", at)
            .expect("mark");
        let left: i64 = store
            .db
            .query_row(
                "SELECT COUNT(*) FROM hook_context_seen WHERE session_id='old'",
                [],
                |row| row.get(0),
            )
            .expect("count");
        assert_eq!(left, 0, "rows older than a week stay");
    }
}
