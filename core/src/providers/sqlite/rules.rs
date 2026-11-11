//! Port of `backend/providers/sqlite/rules.go`.
//!
//! Rules are the law agents read: a short imperative statement with any number
//! of Structure node contexts (none is the whole project).

use rusqlite::{Connection, params};
use serde_json::Value;

use crate::domains::work;

use super::work::{field_str, field_text, scope_paths};
use super::{Error, Result};

impl super::Store {
    /// Returns the project's active Rules in position order, each with the
    /// paths of its context nodes.
    pub(crate) fn load_rules(&self, project_id: &str) -> Result<Vec<work::Rule>> {
        let mut rules = Vec::new();
        self.each_row(
            r#"SELECT id,COALESCE(slug,''),statement,status,position FROM rules
		WHERE project_id=? AND status='active' ORDER BY position, created_at, id"#,
            params![project_id],
            |row| {
                rules.push(work::Rule {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    statement: row.get(2)?,
                    status: row.get(3)?,
                    position: row.get(4)?,
                    scopes: Vec::new(),
                });
                Ok(())
            },
        )?;
        let contexts = self.context_paths(project_id)?;
        for rule in &mut rules {
            rule.scopes = contexts.get(&rule.id).cloned().unwrap_or_default();
        }
        Ok(rules)
    }

    /// Returns the project's active Rules.
    pub fn list_rules(&self, project_id: &str) -> Result<Vec<work::Rule>> {
        self.load_rules(project_id)
    }
}

/// Writes a Rule in the same transaction as `change_ops`.
pub(super) fn apply_rule_op(
    tx: &Connection,
    project_id: &str,
    op: &work::Op,
    now: &str,
) -> Result<()> {
    match op.kind.as_str() {
        work::OP_CREATE => {
            let statement = field_str(&op.fields, "statement");
            let slug = match op.fields.get("slug") {
                Some(Value::String(value)) if !value.is_empty() => Some(value.clone()),
                _ => None,
            };
            let position: i64 = tx.query_row(
                r#"SELECT COALESCE(MAX(position),-1)+1 FROM rules WHERE project_id=?"#,
                params![project_id],
                |row| row.get(0),
            )?;
            tx.execute(
                r#"INSERT INTO rules(id,project_id,slug,statement,status,position,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?)"#,
                params![
                    op.entity_id,
                    project_id,
                    slug,
                    statement,
                    work::RULE_ACTIVE,
                    position,
                    now,
                    now
                ],
            )?;
        }
        work::OP_UPDATE | work::OP_REMOVE => {
            let found = tx.query_row(
                r#"SELECT id FROM rules WHERE id=? AND project_id=?"#,
                params![op.entity_id, project_id],
                |row| row.get::<_, String>(0),
            );
            if let Err(err) = found {
                return Err(Error::message(format!(
                    "rule {} not found: {err}",
                    op.entity_id
                )));
            }
            if let Some(value) = field_text(&op.fields, "statement") {
                tx.execute(
                    r#"UPDATE rules SET statement=?, updated_at=? WHERE id=?"#,
                    params![value, now, op.entity_id],
                )?;
            }
            let mut status = String::new();
            if op.kind == work::OP_REMOVE {
                status = work::RULE_REMOVED.to_string();
            } else if let Some(value) = field_text(&op.fields, "status") {
                status = value;
            }
            if !status.is_empty() {
                // A removed Rule gives its slug back, so a new Rule may take it.
                tx.execute(
                    r#"UPDATE rules SET status=?, slug=CASE WHEN ?='removed' THEN NULL ELSE slug END, updated_at=? WHERE id=?"#,
                    params![status, status, now, op.entity_id],
                )?;
            }
        }
        _ => {
            return Err(Error::message(format!("unknown rule op {:?}", op.kind)));
        }
    }
    if let Some(paths) = op.fields.get("scope").and_then(scope_paths) {
        super::set_context_paths(tx, project_id, &op.entity_id, &paths)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::params;
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

    fn node(store: &super::super::Store, id: &str, path: &str) {
        store
            .db
            .execute(
                "INSERT INTO structure_nodes(id,project_id,parent_id,level,slug,path,title,intent,status,decisions_json,position,repo,folder,kind,tech,migration_from,migration_to,created_at,updated_at) VALUES(?, 'proj', NULL, 'component', ?, ?, ?, '', 'active', '[]', 0, '', '', '', '', '', '', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                params![id, id, path, id],
            )
            .expect("insert node");
    }

    #[test]
    fn rules_are_created_scoped_and_removed() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", OffsetDateTime::UNIX_EPOCH);
        node(store, "api", "app/api");

        let create = work::Op {
            entity: work::ENTITY_RULE.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: "rule-1".to_string(),
            card_slug: "api".to_string(),
            fields: fields(&[
                ("slug", json!("no-globals")),
                ("statement", json!("Never use globals")),
                ("scope", json!(["app/api"])),
            ]),
            ..work::Op::default()
        };
        store
            .apply_work_ops("proj", "", "batch-1", &[create])
            .expect("create rule");

        let rules = store.list_rules("proj").expect("rules");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].slug, "no-globals");
        assert_eq!(rules[0].statement, "Never use globals");
        assert_eq!(rules[0].scopes, vec!["app/api".to_string()]);

        let update = work::Op {
            entity: work::ENTITY_RULE.to_string(),
            kind: work::OP_UPDATE.to_string(),
            entity_id: "rule-1".to_string(),
            card_slug: "api".to_string(),
            fields: fields(&[("statement", json!("Never use mutable globals"))]),
            ..work::Op::default()
        };
        store
            .apply_work_ops("proj", "", "batch-2", &[update])
            .expect("update rule");
        let rules = store.list_rules("proj").expect("rules");
        assert_eq!(rules[0].statement, "Never use mutable globals");

        let remove = work::Op {
            entity: work::ENTITY_RULE.to_string(),
            kind: work::OP_REMOVE.to_string(),
            entity_id: "rule-1".to_string(),
            card_slug: "api".to_string(),
            fields: Map::new(),
            ..work::Op::default()
        };
        store
            .apply_work_ops("proj", "", "batch-3", &[remove])
            .expect("remove rule");
        assert!(
            store.list_rules("proj").expect("rules").is_empty(),
            "a removed Rule leaves the list"
        );
        let slug: Option<String> = store
            .db
            .query_row("SELECT slug FROM rules WHERE id='rule-1'", [], |row| {
                row.get(0)
            })
            .expect("row");
        assert_eq!(slug, None, "a removed Rule gives its slug back");
    }

    #[test]
    fn a_missing_rule_is_reported() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", OffsetDateTime::UNIX_EPOCH);
        let update = work::Op {
            entity: work::ENTITY_RULE.to_string(),
            kind: work::OP_UPDATE.to_string(),
            entity_id: "missing".to_string(),
            card_slug: "api".to_string(),
            fields: fields(&[("statement", json!("Anything"))]),
            ..work::Op::default()
        };
        let err = store
            .apply_work_ops("proj", "", "batch-1", &[update])
            .expect_err("missing rule");
        assert!(
            err.to_string()
                .starts_with("update rule api: rule missing not found: "),
            "err = {err}"
        );
    }
}
