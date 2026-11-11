//! Port of `backend/providers/sqlite/saved_filters.go`.
//!
//! Saved filters: one surface's named filter combinations, upserted by name.
//! They are UI preferences, so no change ops are written.

use rusqlite::Connection;
use rusqlite::params;
use time::OffsetDateTime;

use crate::domains::filters;
use crate::platform::ids;

use super::*;

impl super::Store {
    /// Returns one surface's saved filters in position order. An empty
    /// `project_id` lists the app-level surface.
    pub fn list_saved_filters(
        &self,
        project_id: &str,
        surface: &str,
    ) -> Result<Vec<filters::SavedFilter>> {
        query_saved_filters(&self.db, project_id.trim(), surface.trim())
    }

    /// Upserts by (project, surface, name). A new name is appended after the
    /// surface's filters; an existing one keeps its id and position and takes
    /// the new payload. Saved filters are UI preferences, so no change ops are
    /// written.
    pub fn save_saved_filter(&self, filter: filters::SavedFilter) -> Result<filters::SavedFilter> {
        let mut prepared = filters::prepare_save(filter).map_err(filter_error)?;
        let tx = self.db.unchecked_transaction()?;

        let existing = query_saved_filters(&tx, &prepared.project_id, &prepared.surface)?;
        let stamp = format_rfc3339(OffsetDateTime::now_utc());
        for current in &existing {
            if current.name != prepared.name {
                continue;
            }
            if current.payload_json == prepared.payload_json {
                return Ok(current.clone());
            }
            tx.execute(
                r#"UPDATE saved_filters SET payload_json=?, updated_at=? WHERE id=?"#,
                params![&prepared.payload_json, &stamp, &current.id],
            )?;
            tx.commit()?;
            let mut updated = current.clone();
            updated.payload_json = prepared.payload_json.clone();
            updated.updated_at = parse_rfc3339(&stamp);
            return Ok(updated);
        }
        prepared.id = ids::new_ulid();
        prepared.position = filters::next_position(&existing);
        let inserted = tx.execute(
            r#"INSERT INTO saved_filters(id,project_id,surface,name,payload_json,position,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?)"#,
            params![
                &prepared.id,
                nullable_project_id(&prepared.project_id),
                &prepared.surface,
                &prepared.name,
                &prepared.payload_json,
                prepared.position,
                &stamp,
                &stamp,
            ],
        );
        if let Err(err) = inserted {
            if err.to_string().contains("FOREIGN KEY constraint failed") {
                return Err(filter_error(filters::SavedFilterError::ProjectNotFound));
            }
            return Err(err.into());
        }
        tx.commit()?;
        prepared.created_at = parse_rfc3339(&stamp);
        prepared.updated_at = prepared.created_at;
        Ok(prepared)
    }

    /// Removes one saved filter. Positions of the rest are left as they are;
    /// list order only depends on their relative order.
    pub fn delete_saved_filter(&self, id: &str) -> Result<()> {
        let removed = self
            .db
            .execute(r#"DELETE FROM saved_filters WHERE id=?"#, params![id])?;
        if removed == 0 {
            return Err(filter_error(filters::SavedFilterError::NotFound));
        }
        Ok(())
    }
}

fn query_saved_filters(
    q: &Connection,
    project_id: &str,
    surface: &str,
) -> Result<Vec<filters::SavedFilter>> {
    let mut stmt = q.prepare(
        r#"SELECT id,COALESCE(project_id,''),surface,name,payload_json,position,created_at,updated_at
		FROM saved_filters WHERE COALESCE(project_id,'')=? AND surface=? ORDER BY position, name"#,
    )?;
    let mut rows = stmt.query(params![project_id, surface])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        let created: String = row.get(6)?;
        let updated: String = row.get(7)?;
        out.push(filters::SavedFilter {
            id: row.get(0)?,
            project_id: row.get(1)?,
            surface: row.get(2)?,
            name: row.get(3)?,
            payload_json: row.get(4)?,
            position: row.get(5)?,
            created_at: parse_rfc3339(&created),
            updated_at: parse_rfc3339(&updated),
        });
    }
    Ok(out)
}

/// A nullable project id: the app-level surface stores no project.
fn nullable_project_id(project_id: &str) -> Option<&str> {
    if project_id.is_empty() {
        None
    } else {
        Some(project_id)
    }
}

/// Maps a saved filter error to the store's error, keeping Go's message.
fn filter_error(err: filters::SavedFilterError) -> Error {
    Error::message(err.to_string())
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

    use crate::providers::sqlite::test_support::{TempStore, save_project};

    use super::*;

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    fn filter(
        project_id: &str,
        surface: &str,
        name: &str,
        payload_json: &str,
    ) -> filters::SavedFilter {
        filters::SavedFilter {
            project_id: project_id.to_string(),
            surface: surface.to_string(),
            name: name.to_string(),
            payload_json: payload_json.to_string(),
            ..filters::SavedFilter::default()
        }
    }

    #[test]
    fn saved_filter_store_round_trip() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        for id in ["proj-a", "proj-b"] {
            save_project(store, id, at);
        }
        let items = store.list_saved_filters("proj-a", "work").expect("empty");
        assert!(items.is_empty(), "empty surface: {items:?}");

        let mine = store
            .save_saved_filter(filter(
                "proj-a",
                " work ",
                " Mine ",
                r#"{"version":1,"clauses":[]}"#,
            ))
            .expect("mine");
        assert!(!mine.id.is_empty(), "mine = {mine:?}");
        assert_eq!(mine.surface, "work");
        assert_eq!(mine.name, "Mine");
        assert_eq!(mine.position, 0);
        assert_ne!(mine.created_at, OffsetDateTime::UNIX_EPOCH);

        let blocked = store
            .save_saved_filter(filter("proj-a", "work", "Blocked", r#"{"version":1}"#))
            .expect("blocked");
        assert_eq!(blocked.position, 1, "blocked = {blocked:?}");

        // Saving an existing name replaces the payload in place.
        let updated = store
            .save_saved_filter(filter(
                "proj-a",
                "work",
                "Mine",
                r#"{"version":1,"clauses":[{"field":"owner"}]}"#,
            ))
            .expect("upsert");
        assert_eq!(updated.id, mine.id);
        assert_eq!(updated.position, 0);
        assert_eq!(
            updated.payload_json,
            r#"{"version":1,"clauses":[{"field":"owner"}]}"#
        );

        // The same name is free on another surface, another project, and app
        // level.
        for (project_id, surface) in [("proj-a", "truth.facts"), ("proj-b", "work"), ("", "work")] {
            let saved = store
                .save_saved_filter(filter(project_id, surface, "Mine", r#"{"version":1}"#))
                .expect("scoped save");
            assert_ne!(saved.id, mine.id, "saved = {saved:?}");
            assert_eq!(saved.position, 0, "saved = {saved:?}");
        }
        let app_level = store
            .save_saved_filter(filter("", "work", "Mine", r#"{"version":2}"#))
            .expect("app-level");
        let listed = store
            .list_saved_filters("", "work")
            .expect("app-level list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, app_level.id);
        assert_eq!(listed[0].project_id, "");
        assert_eq!(
            listed[0].payload_json, r#"{"version":2}"#,
            "app-level upsert must reuse the NULL-project row: {listed:?}"
        );

        let listed = store.list_saved_filters("proj-a", "work").expect("list");
        assert_eq!(listed.len(), 2, "list = {listed:?}");
        assert_eq!(listed[0].id, mine.id);
        assert_eq!(listed[1].id, blocked.id);

        let err = store
            .save_saved_filter(filter("proj-a", "work", "Bad", "{}"))
            .expect_err("bad payload");
        assert_eq!(
            err.to_string(),
            "saved filter payload must be a JSON object with an integer version"
        );
        let err = store
            .save_saved_filter(filter("missing", "work", "Ghost", r#"{"version":1}"#))
            .expect_err("unknown project");
        assert_eq!(err.to_string(), "saved filter project not found");

        store.delete_saved_filter(&mine.id).expect("delete");
        let err = store
            .delete_saved_filter(&mine.id)
            .expect_err("second delete");
        assert_eq!(err.to_string(), "saved filter not found");
        let listed = store.list_saved_filters("proj-a", "work").expect("after");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, blocked.id);

        let change_ops: i64 = store
            .db
            .query_row(r#"SELECT COUNT(*) FROM change_ops"#, [], |row| row.get(0))
            .expect("change ops");
        assert_eq!(
            change_ops, 0,
            "saved filters must not write change_ops: {change_ops}"
        );

        store
            .db
            .execute(r#"DELETE FROM projects WHERE id='proj-b'"#, [])
            .expect("delete project");
        let listed = store.list_saved_filters("proj-b", "work").expect("cascade");
        assert!(listed.is_empty(), "project delete must cascade: {listed:?}");
    }
}
