//! Port of `backend/providers/sqlite/decision.go`.
//!
//! The store side of the Decision lifecycle: reading a project's Decisions with
//! their state and contexts, and writing one in the same transaction as its
//! `change_ops` row.

use rusqlite::{Connection, params};
use serde_json::Value;

use crate::domains::decision;
use crate::domains::work;

use super::work::{field_str, field_text};
use super::{Error, Result};

impl super::Store {
    /// Returns the project's Decisions with lifecycle state, oldest first.
    pub(crate) fn load_decisions(&self, project_id: &str) -> Result<Vec<decision::Decision>> {
        let contexts = self.context_paths(project_id)?;
        let mut out = Vec::new();
        self.each_row(
            r#"SELECT id,title,status,body,extra_json FROM knowledge_items
		WHERE project_id=? AND kind='decision' ORDER BY created_at, id"#,
            params![project_id],
            |row| {
                let extra: String = row.get(4)?;
                let status: String = row.get(2)?;
                let id: String = row.get(0)?;
                out.push(decision::Decision {
                    id: id.clone(),
                    title: row.get(1)?,
                    status: decision::normalize(&status).to_string(),
                    body: row.get(3)?,
                    scopes: contexts.get(&id).cloned().unwrap_or_default(),
                    meta: decision::parse_meta(&extra),
                });
                Ok(())
            },
        )?;
        Ok(out)
    }
}

/// Creates or updates one Decision knowledge item. `fields` holds title, body,
/// status, `superseded_by`, `review_trigger`, and `status_reason`; only the keys
/// present change.
pub(super) fn apply_decision_op(
    tx: &Connection,
    project_id: &str,
    op: &work::Op,
    now: &str,
) -> Result<()> {
    let (title, body);
    match op.kind.as_str() {
        work::OP_CREATE => {
            let create_title = field_str(&op.fields, "title");
            let create_body = field_str(&op.fields, "body");
            let declared = field_str(&op.fields, "status");
            let create_status = if declared.is_empty() {
                decision::STATE_PROPOSED.to_string()
            } else {
                declared
            };
            let meta = decision::Meta {
                superseded_by: field_str(&op.fields, "superseded_by"),
                review_trigger: field_str(&op.fields, "review_trigger"),
                status_reason: field_str(&op.fields, "status_reason"),
            };
            let create_extra = decision::merge_meta("{}", &meta);
            tx.execute(
                r#"INSERT INTO knowledge_items(id,project_id,title,body,kind,tags_json,summary,owner,status,extra_json,created_at,updated_at)
			VALUES(?,?,?,?,'decision','[]','','',?,?,?,?)"#,
                params![
                    op.entity_id,
                    project_id,
                    create_title,
                    create_body,
                    create_status,
                    create_extra,
                    now,
                    now
                ],
            )?;
            title = create_title;
            body = create_body;
        }
        work::OP_UPDATE => {
            let current = tx.query_row(
                r#"SELECT title,body,status,extra_json FROM knowledge_items WHERE id=? AND kind='decision'"#,
                params![op.entity_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            );
            let (mut current_title, mut current_body, mut current_status, current_extra) =
                match current {
                    Ok(found) => found,
                    Err(_) => {
                        return Err(Error::message(format!(
                            "decision {} not found",
                            op.entity_id
                        )));
                    }
                };
            if let Some(value) = field_text(&op.fields, "title") {
                current_title = value;
            }
            if let Some(value) = field_text(&op.fields, "body") {
                current_body = value;
            }
            if let Some(value) = field_text(&op.fields, "status") {
                current_status = value;
            }
            let mut meta = decision::parse_meta(&current_extra);
            if let Some(value) = field_text(&op.fields, "superseded_by") {
                meta.superseded_by = value;
            }
            if let Some(value) = field_text(&op.fields, "review_trigger") {
                meta.review_trigger = value;
            }
            if let Some(value) = field_text(&op.fields, "status_reason") {
                meta.status_reason = value;
            }
            let extra = decision::merge_meta(&current_extra, &meta);
            tx.execute(
                r#"UPDATE knowledge_items SET title=?,body=?,status=?,extra_json=?,updated_at=? WHERE id=?"#,
                params![current_title, current_body, current_status, extra, now, op.entity_id],
            )?;
            title = current_title;
            body = current_body;
        }
        _ => {
            return Err(Error::message(
                "decisions are never removed: supersede them",
            ));
        }
    }
    if let Some(Value::Array(paths)) = op.fields.get("scope") {
        let paths: Vec<String> = paths
            .iter()
            .filter_map(|path| path.as_str().map(str::to_string))
            .collect();
        super::set_context_paths(tx, project_id, &op.entity_id, &paths)?;
    }
    tx.execute(
        r#"DELETE FROM knowledge_fts WHERE knowledge_id=?"#,
        params![op.entity_id],
    )?;
    tx.execute(
        r#"INSERT INTO knowledge_fts(knowledge_id, project_id, kind, title, body, summary, tags) VALUES(?,?,?,?,?,?,?)"#,
        params![op.entity_id, project_id, "decision", title, body, "", ""],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, json};
    use time::OffsetDateTime;

    use crate::domains::work;
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    use super::*;

    fn fields(entries: &[(&str, Value)]) -> Map<String, Value> {
        let mut fields = Map::new();
        for (key, value) in entries {
            fields.insert((*key).to_string(), value.clone());
        }
        fields
    }

    fn create_op(id: &str, title: &str) -> work::Op {
        work::Op {
            entity: work::ENTITY_DECISION.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: id.to_string(),
            card_slug: "decision".to_string(),
            fields: fields(&[("title", json!(title)), ("body", json!("Why it holds"))]),
            ..work::Op::default()
        }
    }

    #[test]
    fn decisions_are_created_updated_and_never_removed() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", OffsetDateTime::UNIX_EPOCH);
        store
            .apply_work_ops(
                "proj",
                "",
                "batch-1",
                &[create_op("decision-1", "One store")],
            )
            .expect("create decision");
        let decisions = store.load_decisions("proj").expect("decisions");
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].title, "One store");
        assert_eq!(decisions[0].status, decision::STATE_PROPOSED);

        let update = work::Op {
            entity: work::ENTITY_DECISION.to_string(),
            kind: work::OP_UPDATE.to_string(),
            entity_id: "decision-1".to_string(),
            card_slug: "decision".to_string(),
            fields: fields(&[
                ("status", json!(decision::STATE_ACTIVE)),
                ("status_reason", json!("agreed in review")),
            ]),
            ..work::Op::default()
        };
        store
            .apply_work_ops("proj", "", "batch-2", &[update])
            .expect("update decision");
        let decisions = store.load_decisions("proj").expect("decisions");
        assert_eq!(decisions[0].status, decision::STATE_ACTIVE);
        assert_eq!(decisions[0].meta.status_reason, "agreed in review");

        let remove = work::Op {
            entity: work::ENTITY_DECISION.to_string(),
            kind: work::OP_REMOVE.to_string(),
            entity_id: "decision-1".to_string(),
            card_slug: "decision".to_string(),
            fields: Map::new(),
            ..work::Op::default()
        };
        let err = store
            .apply_work_ops("proj", "", "batch-3", &[remove])
            .expect_err("decisions are never removed");
        assert_eq!(
            err.to_string(),
            "remove decision decision: decisions are never removed: supersede them"
        );

        let missing = work::Op {
            entity: work::ENTITY_DECISION.to_string(),
            kind: work::OP_UPDATE.to_string(),
            entity_id: "missing".to_string(),
            card_slug: "decision".to_string(),
            fields: fields(&[("title", json!("Title"))]),
            ..work::Op::default()
        };
        let err = store
            .apply_work_ops("proj", "", "batch-4", &[missing])
            .expect_err("missing decision");
        assert_eq!(
            err.to_string(),
            "update decision decision: decision missing not found"
        );
    }

    #[test]
    fn a_decision_is_indexed_for_search() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", OffsetDateTime::UNIX_EPOCH);
        store
            .apply_work_ops(
                "proj",
                "",
                "batch-1",
                &[create_op("decision-1", "One store")],
            )
            .expect("create decision");
        let indexed: i64 = store
            .db
            .query_row(
                "SELECT COUNT(*) FROM knowledge_fts WHERE knowledge_id='decision-1'",
                [],
                |row| row.get(0),
            )
            .expect("fts count");
        assert_eq!(indexed, 1, "a written Decision joins the search index");
    }
}
