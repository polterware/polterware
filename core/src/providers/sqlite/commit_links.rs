//! The commits people link to a card's tasks, stored.
//!
//! The links an agent or a person made between one commit and one task. Links
//! cited in commit subjects are derived on read, so only manual ones are
//! stored; the file also keeps the card search index up to date.

use rusqlite::{Connection, params};
use time::OffsetDateTime;

use crate::domains::work;
use crate::platform::ids;

use super::{Result, format_rfc3339, parse_rfc3339};

impl super::Store {
    /// Stores a link an agent or a person made between one commit and one task.
    pub fn insert_commit_link(&self, link: work::CommitLink, now: OffsetDateTime) -> Result<()> {
        let id = if link.id.is_empty() {
            ids::new_ulid()
        } else {
            link.id
        };
        let committed = if link.committed_at == OffsetDateTime::UNIX_EPOCH {
            String::new()
        } else {
            format_rfc3339(link.committed_at)
        };
        self.db.execute(
            r#"INSERT OR IGNORE INTO work_commit_links(id,project_id,repo,sha,task_id,subject,url,committed_at,created_at) VALUES(?,?,?,?,?,?,?,?,?)"#,
            params![
                id,
                link.project_id,
                link.repo,
                link.sha,
                link.task_id,
                link.subject,
                link.url,
                committed,
                format_rfc3339(now),
            ],
        )?;
        Ok(())
    }

    /// Removes the stored link between a commit and a task. It reports false
    /// when there was none.
    pub fn delete_commit_link(&self, project_id: &str, task_id: &str, sha: &str) -> Result<bool> {
        let removed = self.db.execute(
            r#"DELETE FROM work_commit_links WHERE project_id=? AND task_id=? AND sha=?"#,
            params![project_id, task_id, sha],
        )?;
        Ok(removed > 0)
    }

    /// Returns every stored commit-to-task link for a project.
    pub fn list_commit_links(&self, project_id: &str) -> Result<Vec<work::CommitLink>> {
        let mut stmt = self.db.prepare(
            r#"SELECT id, project_id, repo, sha, task_id, subject, url, committed_at, created_at
		FROM work_commit_links WHERE project_id=? ORDER BY created_at, id"#,
        )?;
        let mut rows = stmt.query(params![project_id])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let committed: String = row.get(7)?;
            let created: String = row.get(8)?;
            out.push(work::CommitLink {
                id: row.get(0)?,
                project_id: row.get(1)?,
                repo: row.get(2)?,
                sha: row.get(3)?,
                task_id: row.get(4)?,
                subject: row.get(5)?,
                url: row.get(6)?,
                committed_at: parse_rfc3339(&committed),
                created_at: parse_rfc3339(&created),
            });
        }
        Ok(out)
    }
}

/// Rewrites a card's rows in the Work search index, one per task, so search
/// reads the card's current title, intent, and task titles.
pub(super) fn reindex_work_card_fts(tx: &Connection, card_id: &str) -> Result<()> {
    let card = tx.query_row(
        r#"SELECT project_id, slug, title, intent, status FROM work_cards WHERE id=?"#,
        params![card_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        },
    );
    let (project_id, slug, title, intent, status) = match card {
        Ok(found) => found,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(()),
        Err(err) => return Err(err.into()),
    };
    let mut tasks: Vec<(String, String, String)> = Vec::new();
    {
        let mut stmt = tx.prepare(r#"SELECT id, title, state FROM work_tasks WHERE card_id=?"#)?;
        let mut rows = stmt.query(params![card_id])?;
        while let Some(row) = rows.next()? {
            tasks.push((row.get(0)?, row.get(1)?, row.get(2)?));
        }
    }
    for (task_id, task_title, task_state) in tasks {
        tx.execute(r#"DELETE FROM work_fts WHERE task_id=?"#, params![task_id])?;
        if status != work::CARD_ACTIVE || task_state == work::TASK_DROPPED {
            continue;
        }
        let document = format!("{task_title} {title} {intent}").trim().to_string();
        tx.execute(
            r#"INSERT INTO work_fts(task_id, project_id, card_slug, title) VALUES(?,?,?,?)"#,
            params![task_id, project_id, slug, document],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, json};
    use time::OffsetDateTime;

    use crate::domains::work;
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    use super::*;

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    fn fields(entries: &[(&str, serde_json::Value)]) -> Map<String, serde_json::Value> {
        let mut fields = Map::new();
        for (key, value) in entries {
            fields.insert((*key).to_string(), value.clone());
        }
        fields
    }

    /// The Go test reads a task from the demo seed; the seed is another run's
    /// port, so this one writes a card and a task through the same ops.
    fn seeded_task(store: &super::super::Store) -> String {
        let card = work::Op {
            entity: work::ENTITY_CARD.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: "card-a".to_string(),
            card_id: "card-a".to_string(),
            card_slug: "a".to_string(),
            fields: fields(&[
                ("slug", json!("a")),
                ("title", json!("A")),
                ("stage", json!(work::STAGE_BUILD)),
                ("status", json!(work::CARD_ACTIVE)),
            ]),
            ..work::Op::default()
        };
        let task = work::Op {
            entity: work::ENTITY_TASK.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: "task-1".to_string(),
            card_id: "card-a".to_string(),
            card_slug: "a".to_string(),
            fields: fields(&[
                ("number", json!(1)),
                ("title", json!("Link the commit")),
                ("state", json!(work::TASK_OPEN)),
            ]),
            ..work::Op::default()
        };
        store
            .apply_work_ops("proj", "", "batch-1", &[card, task])
            .expect("seed a task");
        "task-1".to_string()
    }

    #[test]
    fn commit_links_are_unique_per_commit_and_task() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", now());
        let task_id = seeded_task(store);
        let at = now();
        let link = work::CommitLink {
            project_id: "proj".to_string(),
            repo: "api".to_string(),
            sha: "abc123".to_string(),
            task_id: task_id.clone(),
            ..work::CommitLink::default()
        };
        for _ in 0..2 {
            store
                .insert_commit_link(link.clone(), at)
                .expect("insert commit link");
        }
        let stored = store.list_commit_links("proj").expect("links");
        assert_eq!(stored.len(), 1, "links = {stored:?}");
        assert_eq!(stored[0].repo, "api");
        assert_eq!(stored[0].sha, "abc123");
        assert_eq!(stored[0].task_id, task_id);

        assert!(
            store
                .delete_commit_link("proj", &task_id, "abc123")
                .expect("delete"),
            "the stored link must be removed"
        );
        assert!(
            !store
                .delete_commit_link("proj", &task_id, "abc123")
                .expect("delete twice"),
            "a second delete removes nothing"
        );
    }

    #[test]
    fn the_card_search_index_follows_the_card() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", now());
        let task_id = seeded_task(store);
        let document: String = store
            .db
            .query_row(
                "SELECT title FROM work_fts WHERE task_id=?",
                params![task_id],
                |row| row.get(0),
            )
            .expect("indexed task");
        assert_eq!(document, "Link the commit A", "the document is trimmed");
        let slug: String = store
            .db
            .query_row(
                "SELECT card_slug FROM work_fts WHERE task_id=?",
                params![task_id],
                |row| row.get(0),
            )
            .expect("card slug");
        assert_eq!(slug, "a");
    }
}
