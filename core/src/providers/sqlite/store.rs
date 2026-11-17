//! The `Store`: one connection per database, and the writes over it.
//!
//! The [`Store`] itself, its projects, knowledge items, and
//! connections, and the health row the local core exposes.

use std::path::Path;

use rusqlite::{Connection, Row, params};
use time::OffsetDateTime;

use crate::domains::knowledge;
use crate::domains::projects;

use super::{
    Error, Result, format_optional_rfc3339, format_rfc3339, parse_optional_rfc3339, parse_rfc3339,
};

/// The project's local store. The core is the only process that opens it.
pub struct Store {
    pub db: Connection,
}

impl Store {
    /// Opens the store at `path`, applies the migrations, and reads which
    /// column holds each table's project id for the change bus.
    pub fn new(path: impl AsRef<Path>) -> Result<Store> {
        let mut db = crate::platform::db::sqlite::open(path)?;
        crate::platform::db::sqlite::migrate(&mut db)?;
        crate::platform::db::sqlite::load_project_columns(&db)?;
        let store = Store { db };
        store.backfill_canvas_sources()?;
        Ok(store)
    }

    /// Closes the store.
    pub fn close(self) -> Result<()> {
        self.db.close().map_err(|(_, err)| Error::Sqlite(err))
    }

    pub fn save_project(&self, mut project: projects::Project) -> Result<()> {
        if project.status.is_empty() {
            project.status = projects::STATUS_ACTIVE.to_string();
        }
        self.db.execute(
            r#"INSERT INTO projects(id,name,created_at,updated_at,status,archived_at) VALUES(?,?,?,?,?,?)
		ON CONFLICT(id) DO UPDATE SET name=excluded.name,updated_at=excluded.updated_at,status=excluded.status,archived_at=excluded.archived_at"#,
            params![
                project.id,
                project.name,
                format_rfc3339(project.created_at),
                format_rfc3339(project.updated_at),
                project.status,
                format_optional_rfc3339(project.archived_at),
            ],
        )?;
        Ok(())
    }

    pub fn get_project(&self, id: &str) -> Result<projects::Project> {
        self.db.query_row(
            r#"SELECT id,name,created_at,updated_at,status,archived_at FROM projects WHERE id=?"#,
            params![id],
            scan_project,
        )
        .map_err(Into::into)
    }

    pub fn list_projects(&self, include_archived: bool) -> Result<Vec<projects::Project>> {
        let mut query = String::from(
            r#"SELECT id,name,created_at,updated_at,status,archived_at FROM projects"#,
        );
        if !include_archived {
            query.push_str(r#" WHERE status != 'archived'"#);
        }
        query.push_str(r#" ORDER BY created_at, rowid"#);
        let mut stmt = self.db.prepare(&query)?;
        let rows = stmt.query_map([], scan_project)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn archive_project(&self, id: &str) -> Result<projects::Project> {
        let mut project = self.get_project(id)?;
        let now = OffsetDateTime::now_utc();
        project.status = projects::STATUS_ARCHIVED.to_string();
        project.archived_at = Some(now);
        project.updated_at = now;
        self.save_project(project.clone())?;
        Ok(project)
    }

    pub fn save_knowledge(&self, mut item: knowledge::Item) -> Result<()> {
        let tags = serde_json::to_string(&item.tags).unwrap_or_default();
        if item.extra_json.is_empty() {
            item.extra_json = "{}".to_string();
        }
        item.doc_type = knowledge::doc_type_for(&item.kind).to_string();
        self.db.execute(
            r#"INSERT INTO knowledge_items(id,project_id,title,body,kind,doc_type,tags_json,summary,owner,status,extra_json,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)
		ON CONFLICT(id) DO UPDATE SET project_id=excluded.project_id,title=excluded.title,body=excluded.body,kind=excluded.kind,doc_type=excluded.doc_type,tags_json=excluded.tags_json,summary=excluded.summary,owner=excluded.owner,status=excluded.status,extra_json=excluded.extra_json,updated_at=excluded.updated_at"#,
            params![
                item.id,
                item.project_id,
                item.title,
                item.body,
                item.kind,
                item.doc_type,
                tags,
                item.summary,
                item.owner,
                item.status,
                item.extra_json,
                format_rfc3339(item.created_at),
                format_rfc3339(item.updated_at),
            ],
        )?;
        // An archived row stays out of search even when an import rewrites it.
        let archived_at: String = self.db.query_row(
            r#"SELECT archived_at FROM knowledge_items WHERE id=?"#,
            params![item.id],
            |row| row.get(0),
        )?;
        if !archived_at.is_empty() {
            return Ok(());
        }
        self.index_knowledge_fts(&item)
    }

    /// Returns one item, archived or not.
    pub fn get_knowledge(&self, id: &str) -> Result<knowledge::Item> {
        self.db
            .query_row(
                &format!("SELECT {KNOWLEDGE_COLUMNS} FROM knowledge_items WHERE id=?"),
                params![id],
                scan_knowledge,
            )
            .map_err(Into::into)
    }

    /// Returns the project's active items, newest first.
    pub fn list_knowledge(&self, project_id: &str, kind: &str) -> Result<Vec<knowledge::Item>> {
        self.list_knowledge_by(project_id, kind, false)
    }

    /// Returns the project's archived items, newest first.
    pub fn list_archived_knowledge(
        &self,
        project_id: &str,
        kind: &str,
    ) -> Result<Vec<knowledge::Item>> {
        self.list_knowledge_by(project_id, kind, true)
    }

    fn list_knowledge_by(
        &self,
        project_id: &str,
        kind: &str,
        archived: bool,
    ) -> Result<Vec<knowledge::Item>> {
        let archived_filter = if archived {
            "archived_at<>''"
        } else {
            "archived_at=''"
        };
        let mut query = format!(
            "SELECT {KNOWLEDGE_COLUMNS} FROM knowledge_items WHERE project_id=? AND {archived_filter}"
        );
        if !kind.is_empty() {
            query.push_str(r#" AND kind=?"#);
        }
        query.push_str(r#" ORDER BY updated_at DESC"#);
        let mut stmt = self.db.prepare(&query)?;
        let mut result = Vec::new();
        if kind.is_empty() {
            let rows = stmt.query_map(params![project_id], scan_knowledge)?;
            for row in rows {
                result.push(row?);
            }
        } else {
            let rows = stmt.query_map(params![project_id, kind], scan_knowledge)?;
            for row in rows {
                result.push(row?);
            }
        }
        Ok(result)
    }

    /// Moves an item into the archive or back out of it. An archived item
    /// leaves the search index; a restored one is indexed again. Decisions are
    /// never archived.
    pub fn set_knowledge_archived(
        &self,
        id: &str,
        archived: bool,
        at: OffsetDateTime,
    ) -> Result<knowledge::Item> {
        let item = self.get_knowledge(id)?;
        let stamp = if archived {
            format_rfc3339(at)
        } else {
            String::new()
        };
        self.db.execute(
            r#"UPDATE knowledge_items SET archived_at=? WHERE id=?"#,
            params![stamp, id],
        )?;
        if archived {
            self.db.execute(
                r#"DELETE FROM knowledge_fts WHERE knowledge_id=?"#,
                params![id],
            )?;
        } else {
            self.index_knowledge_fts(&item)?;
        }
        self.get_knowledge(id)
    }

    /// Removes an archived item for good. Active items are refused.
    pub fn delete_knowledge(&self, id: &str) -> Result<()> {
        let item = self.get_knowledge(id)?;
        if !item.archived() {
            return Err(knowledge::KnowledgeError::NotArchived.into());
        }
        self.db.execute(
            r#"DELETE FROM knowledge_fts WHERE knowledge_id=?"#,
            params![id],
        )?;
        self.db
            .execute(r#"DELETE FROM knowledge_items WHERE id=?"#, params![id])?;
        Ok(())
    }

    pub fn store_health(&self) -> Result<(bool, String)> {
        let (quarantined, message): (i64, String) = self.db.query_row(
            r#"SELECT quarantined,message FROM store_health WHERE id=1"#,
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        Ok((quarantined != 0, message))
    }

    pub fn reset(&self) -> Result<()> {
        const TABLES: [&str; 4] = [
            "knowledge_items",
            "canvases",
            "project_settings",
            "projects",
        ];
        let tx = self.db.unchecked_transaction()?;
        for table in TABLES {
            tx.execute(&format!("DELETE FROM {table}"), [])?;
        }
        tx.commit()?;
        Ok(())
    }
}

pub(crate) const KNOWLEDGE_COLUMNS: &str = "id,project_id,title,body,kind,doc_type,tags_json,summary,owner,status,extra_json,created_at,updated_at,archived_at";

fn scan_project(row: &Row<'_>) -> rusqlite::Result<projects::Project> {
    let status: String = row.get(4)?;
    let archived: String = row.get(5)?;
    Ok(projects::Project {
        id: row.get(0)?,
        name: row.get(1)?,
        created_at: parse_rfc3339(&row.get::<_, String>(2)?),
        updated_at: parse_rfc3339(&row.get::<_, String>(3)?),
        status: if status.is_empty() {
            projects::STATUS_ACTIVE.to_string()
        } else {
            status
        },
        archived_at: parse_optional_rfc3339(&archived),
    })
}

pub(crate) fn scan_knowledge(row: &Row<'_>) -> rusqlite::Result<knowledge::Item> {
    let tags: String = row.get(6)?;
    let created: String = row.get(11)?;
    let updated: String = row.get(12)?;
    let archived: String = row.get(13)?;
    let tags = serde_json::from_str::<Vec<String>>(&tags).unwrap_or_default();
    Ok(knowledge::Item {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        body: row.get(3)?,
        kind: row.get(4)?,
        doc_type: row.get(5)?,
        tags,
        summary: row.get(7)?,
        owner: row.get(8)?,
        status: row.get(9)?,
        extra_json: row.get(10)?,
        created_at: parse_rfc3339(&created),
        updated_at: parse_rfc3339(&updated),
        archived_at: parse_optional_rfc3339(&archived),
    })
}

#[cfg(test)]
mod tests {
    use rusqlite::params;
    use time::OffsetDateTime;

    use crate::domains::knowledge::{self, KnowledgeError};
    use crate::domains::projects;

    use super::super::test_support::{TempStore, save_project};
    use super::*;

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    fn project(id: &str, at: OffsetDateTime) -> projects::Project {
        projects::Project {
            id: id.to_string(),
            name: id.to_string(),
            created_at: at,
            updated_at: at,
            status: String::new(),
            archived_at: None,
        }
    }

    fn item(id: &str, project_id: &str, kind: &str, at: OffsetDateTime) -> knowledge::Item {
        knowledge::Item {
            id: id.to_string(),
            project_id: project_id.to_string(),
            title: id.to_string(),
            kind: kind.to_string(),
            doc_type: knowledge::doc_type_for(kind).to_string(),
            created_at: at,
            updated_at: at,
            ..knowledge::Item::default()
        }
    }

    fn fts_rows(store: &Store, id: &str) -> i64 {
        store
            .db
            .query_row(
                "SELECT COUNT(1) FROM knowledge_fts WHERE knowledge_id=?",
                params![id],
                |row| row.get(0),
            )
            .expect("fts count")
    }

    #[test]
    fn store_persists_across_reopen() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("reopen.db");
        let first = Store::new(&path).expect("open");
        let at = now();
        first.save_project(project("proj-a", at)).expect("save");
        first
            .set_hook_switch("session-start", false)
            .expect("switch");
        first.close().expect("close");

        let second = Store::new(&path).expect("reopen");
        let got = second.get_project("proj-a").expect("project");
        assert_eq!(got.name, "proj-a");
        assert_eq!(got.status, projects::STATUS_ACTIVE);
        let switches = second.hook_switches().expect("switches");
        assert_eq!(switches.get("session-start"), Some(&false));
    }

    #[test]
    fn project_archive_hides_from_default_list() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        for id in ["proj-a", "proj-b"] {
            store.save_project(project(id, at)).expect("save");
        }
        let archived = store.archive_project("proj-a").expect("archive");
        assert_eq!(archived.status, projects::STATUS_ARCHIVED);
        assert!(archived.archived_at.is_some());

        let active = store.list_projects(false).expect("active");
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "proj-b");
        let all = store.list_projects(true).expect("all");
        assert_eq!(all.len(), 2);
        assert!(
            store.archive_project("missing").is_err(),
            "archiving an unknown project must fail"
        );
    }

    #[test]
    fn get_project_missing_returns_no_rows() {
        let holder = TempStore::new();
        let err = holder.store.get_project("nope").expect_err("no rows");
        assert!(err.is_no_rows(), "err = {err:?}");
    }

    fn project_save(store: &Store, at: OffsetDateTime) {
        save_project(store, "proj-a", at);
    }

    #[test]
    fn save_and_list_knowledge() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        project_save(store, at);
        store
            .save_knowledge(item("doc-1", "proj-a", knowledge::KIND_DOC, at))
            .expect("save doc");

        let docs = store
            .list_knowledge("proj-a", knowledge::KIND_DOC)
            .expect("docs");
        assert_eq!(docs.len(), 1, "docs = {docs:?}");
        let first = &docs[0];
        assert_eq!(first.kind, knowledge::KIND_DOC);
        assert_eq!(first.project_id, "proj-a");
        let got = store.get_knowledge(&first.id).expect("get");
        assert_eq!(got.title, first.title);

        store
            .delete_knowledge(&first.id)
            .expect_err("an active item is refused");
    }

    #[test]
    fn knowledge_archive_restore_and_delete() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        project_save(store, at);
        store
            .save_knowledge(item("doc-1", "proj-a", knowledge::KIND_DOC, at))
            .expect("save doc");
        let item_id = "doc-1";

        let err = store.delete_knowledge(item_id).expect_err("active");
        assert!(
            matches!(err, Error::Knowledge(KnowledgeError::NotArchived)),
            "err = {err:?}"
        );
        let archived = store
            .set_knowledge_archived(item_id, true, at)
            .expect("archive");
        assert!(archived.archived());
        let active = store
            .list_knowledge("proj-a", knowledge::KIND_DOC)
            .expect("active");
        assert!(
            active.iter().all(|other| other.id != item_id),
            "archived item stayed in the active list: {active:?}"
        );
        let listed = store
            .list_archived_knowledge("proj-a", "")
            .expect("archived");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, item_id);
        assert_eq!(fts_rows(store, item_id), 0, "archived item stayed indexed");
        let hits = store
            .search_knowledge("proj-a", "doc", "", 20)
            .expect("search");
        assert!(
            hits.iter().all(|hit| hit.item.id != item_id),
            "archived item stayed in search: {hits:?}"
        );

        // An import that rewrites an archived row keeps it out of search.
        store.save_knowledge(archived).expect("rewrite");
        assert_eq!(fts_rows(store, item_id), 0);

        let restored = store
            .set_knowledge_archived(item_id, false, at)
            .expect("restore");
        assert!(!restored.archived());
        assert_eq!(fts_rows(store, item_id), 1, "restored item is not indexed");

        store
            .set_knowledge_archived(item_id, true, at)
            .expect("archive again");
        store.delete_knowledge(item_id).expect("delete");
        let err = store.get_knowledge(item_id).expect_err("deleted");
        assert!(err.is_no_rows(), "err = {err:?}");
    }

    #[test]
    fn store_health_starts_healthy() {
        let holder = TempStore::new();
        let (quarantined, message) = holder.store.store_health().expect("health");
        assert!(!quarantined, "quarantined = {quarantined}");
        assert!(message.is_empty());
    }

    #[test]
    fn reset_clears_projects() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        project_save(store, at);
        store.reset().expect("reset");
        let all = store.list_projects(true).expect("projects");
        assert!(all.is_empty(), "projects after reset: {all:?}");
    }
}
