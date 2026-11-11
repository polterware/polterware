//! Port of `backend/providers/sqlite/conflicts.go`.
//!
//! A Conflict is a request that goes against a Rule or a Decision: an agent
//! raises one instead of acting against the law, and a person decides it.

use rusqlite::{Connection, params};
use serde_json::{Map, Value};

use crate::domains::work;

use super::work::{ColumnValue, column_value};
use super::{Error, Result};

impl super::Store {
    /// Returns every Conflict of the project, newest first.
    pub(crate) fn load_conflicts(&self, project_id: &str) -> Result<Vec<work::Conflict>> {
        let mut conflicts = Vec::new();
        self.each_row(
            r#"SELECT id,slug,request,rule_id,decision_id,card_slug,options_json,state,answer,created_at,decided_at
		FROM conflicts WHERE project_id=? ORDER BY created_at DESC, id DESC"#,
            params![project_id],
            |row| {
                let options: String = row.get(6)?;
                conflicts.push(work::Conflict {
                    id: row.get(0)?,
                    slug: row.get(1)?,
                    request: row.get(2)?,
                    rule_id: row.get(3)?,
                    decision: row.get(4)?,
                    card_slug: row.get(5)?,
                    options: serde_json::from_str::<Vec<String>>(&options).unwrap_or_default(),
                    state: row.get(7)?,
                    answer: row.get(8)?,
                    created_at: row.get(9)?,
                    decided_at: row.get(10)?,
                });
                Ok(())
            },
        )?;
        Ok(conflicts)
    }

    /// Returns every Conflict of the project, newest first.
    pub fn list_conflicts(&self, project_id: &str) -> Result<Vec<work::Conflict>> {
        self.load_conflicts(project_id)
    }
}

/// Writes a Conflict in the same transaction as `change_ops`.
pub(super) fn apply_conflict_op(
    tx: &Connection,
    project_id: &str,
    op: &work::Op,
    now: &str,
) -> Result<()> {
    match op.kind.as_str() {
        work::OP_CREATE => {
            let slug = field_str(&op.fields, "slug");
            let request = field_str(&op.fields, "request");
            let rule = field_str(&op.fields, "rule_id");
            let decision_id = field_str(&op.fields, "decision_id");
            let card = field_str(&op.fields, "card");
            let mut options = column_value(op.fields.get("options").unwrap_or(&Value::Null))?;
            if options == ColumnValue::Null {
                options = ColumnValue::Text("[]".to_string());
            }
            tx.execute(
                r#"INSERT INTO conflicts(id,project_id,slug,request,rule_id,decision_id,card_slug,options_json,state,created_at,updated_at)
			VALUES(?,?,?,?,?,?,?,?,?,?,?)"#,
                params![
                    op.entity_id,
                    project_id,
                    slug,
                    request,
                    rule,
                    decision_id,
                    card,
                    options,
                    work::CONFLICT_OPEN,
                    now,
                    now
                ],
            )?;
            Ok(())
        }
        work::OP_UPDATE | work::OP_REMOVE => {
            for (field, value) in &op.fields {
                let Some(column) = conflict_column(field) else {
                    return Err(Error::message(format!("unknown conflict field {field:?}")));
                };
                let converted = column_value(value)?;
                tx.execute(
                    &format!(
                        r#"UPDATE conflicts SET {column}=?, updated_at=? WHERE id=? AND project_id=?"#
                    ),
                    params![converted, now, op.entity_id, project_id],
                )?;
            }
            if let Some(state) = field_text(&op.fields, "state") {
                let decided = if state == work::CONFLICT_OPEN {
                    String::new()
                } else {
                    now.to_string()
                };
                tx.execute(
                    r#"UPDATE conflicts SET decided_at=? WHERE id=?"#,
                    params![decided, op.entity_id],
                )?;
            }
            Ok(())
        }
        _ => Err(Error::message(format!("unknown conflict op {:?}", op.kind))),
    }
}

/// The column each Conflict field writes.
fn conflict_column(field: &str) -> Option<&'static str> {
    match field {
        "request" => Some("request"),
        "rule_id" => Some("rule_id"),
        "decision_id" => Some("decision_id"),
        "card" => Some("card_slug"),
        "options" => Some("options_json"),
        "state" => Some("state"),
        "answer" => Some("answer"),
        _ => None,
    }
}

fn field_str(fields: &Map<String, Value>, key: &str) -> String {
    field_text(fields, key).unwrap_or_default()
}

fn field_text(fields: &Map<String, Value>, key: &str) -> Option<String> {
    match fields.get(key) {
        Some(Value::String(text)) => Some(text.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, json};

    use crate::domains::work;
    use crate::providers::sqlite::test_support::{TempStore, save_project};
    use time::OffsetDateTime;

    use super::*;

    fn fields(entries: &[(&str, Value)]) -> Map<String, Value> {
        let mut fields = Map::new();
        for (key, value) in entries {
            fields.insert((*key).to_string(), value.clone());
        }
        fields
    }

    #[test]
    fn conflicts_are_raised_and_decided() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", OffsetDateTime::UNIX_EPOCH);
        let raise = work::Op {
            entity: work::ENTITY_CONFLICT.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: "conflict-1".to_string(),
            card_slug: "a".to_string(),
            fields: fields(&[
                ("slug", json!("skip-tests")),
                ("request", json!("Ship without the new tests")),
                ("rule_id", json!("rule-1")),
                ("card", json!("a")),
                ("options", json!(["wait", "ship"])),
            ]),
            ..work::Op::default()
        };
        store
            .apply_work_ops("proj", "", "batch-1", &[raise])
            .expect("raise");
        let conflicts = store.list_conflicts("proj").expect("conflicts");
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].slug, "skip-tests");
        assert_eq!(conflicts[0].rule_id, "rule-1");
        assert_eq!(conflicts[0].options, vec!["wait", "ship"]);
        assert_eq!(conflicts[0].state, work::CONFLICT_OPEN);
        assert!(conflicts[0].decided_at.is_empty());

        let decide = work::Op {
            entity: work::ENTITY_CONFLICT.to_string(),
            kind: work::OP_UPDATE.to_string(),
            entity_id: "conflict-1".to_string(),
            card_slug: "a".to_string(),
            fields: fields(&[
                ("state", json!(work::CONFLICT_DECIDED)),
                ("answer", json!("wait")),
            ]),
            ..work::Op::default()
        };
        store
            .apply_work_ops("proj", "", "batch-2", &[decide])
            .expect("decide");
        let conflicts = store.list_conflicts("proj").expect("conflicts");
        assert_eq!(conflicts[0].state, work::CONFLICT_DECIDED);
        assert_eq!(conflicts[0].answer, "wait");
        assert!(
            !conflicts[0].decided_at.is_empty(),
            "a decided Conflict keeps its stamp"
        );
    }

    #[test]
    fn an_unknown_conflict_field_is_rejected() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", OffsetDateTime::UNIX_EPOCH);
        let update = work::Op {
            entity: work::ENTITY_CONFLICT.to_string(),
            kind: work::OP_UPDATE.to_string(),
            entity_id: "conflict-1".to_string(),
            card_slug: "a".to_string(),
            fields: fields(&[("nope", json!("x"))]),
            ..work::Op::default()
        };
        let err = store
            .apply_work_ops("proj", "", "batch-1", &[update])
            .expect_err("unknown field");
        assert_eq!(
            err.to_string(),
            "update conflict a: unknown conflict field \"nope\""
        );
    }
}
