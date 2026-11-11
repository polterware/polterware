//! Port of `backend/providers/sqlite/polter_facts.go`.
//!
//! Facts as Polter writes them: the canon fact with its slug and the paths of
//! the Structure nodes it is about, written in the same transaction as the
//! change ops.

use rusqlite::{Connection, params};
use serde_json::Value;

use crate::domains::canon;
use crate::domains::work;

use super::canon_facts::{scan_canon_fact, upsert_canon_fact};
use super::scope::set_context_paths;
use super::{Error, Result, parse_rfc3339};

impl super::Store {
    /// Returns the project's facts with their Polter slug and context paths.
    pub fn load_polter_facts(&self, project_id: &str) -> Result<Vec<work::Fact>> {
        let facts = self.list_canon_facts(project_id, "")?;
        let mut result: Vec<work::Fact> = Vec::with_capacity(facts.len());
        let mut index = std::collections::HashMap::new();
        for fact in facts {
            index.insert(fact.id.clone(), result.len());
            result.push(work::Fact {
                fact,
                ..work::Fact::default()
            });
        }
        let contexts = self.context_paths(project_id)?;
        for item in &mut result {
            item.scopes = contexts.get(&item.fact.id).cloned().unwrap_or_default();
        }
        self.each_row(
            r#"SELECT id,COALESCE(slug,'') FROM canon_facts WHERE project_id=?"#,
            params![project_id],
            |row| {
                let id: String = row.get(0)?;
                let slug: String = row.get(1)?;
                if let Some(&i) = index.get(&id) {
                    result[i].slug = slug;
                } // A concurrent insert belongs to the next snapshot.
                Ok(())
            },
        )?;
        Ok(result)
    }
}

/// Writes the canon store in the same transaction as change_ops. The Work apply
/// in `work.rs`, its own run, calls it.
#[allow(dead_code)]
pub(crate) fn apply_fact_op(
    tx: &Connection,
    project_id: &str,
    op: &work::Op,
    now: &str,
) -> Result<()> {
    let timestamp = parse_rfc3339(now);
    let mut fact = canon::Fact {
        id: op.entity_id.clone(),
        project_id: project_id.to_string(),
        truth_class: canon::TRUTH_DECLARED.to_string(),
        source: "polter".to_string(),
        actor: "agent".to_string(),
        observed_at: timestamp,
        created_at: timestamp,
        ..canon::Fact::default()
    };
    if op.kind == work::OP_UPDATE {
        fact = tx
            .query_row(
                r#"SELECT id,project_id,statement,truth_class,source,actor,observed_at,entity_id,created_at,updated_at FROM canon_facts WHERE id=? AND project_id=?"#,
                params![op.entity_id, project_id],
                scan_canon_fact,
            )
            .map_err(|err| Error::message(format!("fact {} not found: {err}", op.entity_id)))?;
    } else if op.kind != work::OP_CREATE {
        return Err(Error::message("facts cannot be removed through Polter"));
    }
    if let Some(value) = op.fields.get("statement").and_then(Value::as_str) {
        fact.statement = value.to_string();
    }
    if let Some(value) = op.fields.get("kind").and_then(Value::as_str) {
        fact.truth_class = value.to_string();
    }
    if let Some(value) = op.fields.get("actor").and_then(Value::as_str) {
        fact.actor = value.to_string();
    }
    fact.updated_at = timestamp;
    canon::validate_fact(&fact)?;
    upsert_canon_fact(tx, &fact)?;
    if let Some(slug) = op.fields.get("slug").and_then(Value::as_str) {
        tx.execute(
            r#"UPDATE canon_facts SET slug=? WHERE id=? AND project_id=?"#,
            params![slug, op.entity_id, project_id],
        )?;
    }
    if let Some(paths) = op.fields.get("scope").and_then(paths_of) {
        set_context_paths(tx, project_id, &op.entity_id, &paths)?;
    }
    Ok(())
}

/// Reads a `scope` field: a list of Structure node paths, or nothing when the
/// value is not a list.
#[allow(dead_code)]
fn paths_of(value: &Value) -> Option<Vec<String>> {
    match value {
        Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(item.as_str()?.to_string());
            }
            Some(out)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use time::OffsetDateTime;

    use crate::domains::canon;
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    use super::*;

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    fn fact_op(kind: &str, id: &str, fields: work::Fields) -> work::Op {
        work::Op {
            entity: work::ENTITY_FACT.to_string(),
            kind: kind.to_string(),
            entity_id: id.to_string(),
            fields,
            ..work::Op::default()
        }
    }

    #[test]
    fn apply_fact_op_creates_and_updates() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        store
            .db
            .execute(
                "INSERT INTO structure_nodes(id,project_id,level,slug,path,title,created_at,updated_at) VALUES('n','p','component','api','app/api','API',?,?)",
                params![crate::providers::sqlite::format_rfc3339(at), crate::providers::sqlite::format_rfc3339(at)],
            )
            .expect("node");
        let stamp = crate::providers::sqlite::format_rfc3339(at);

        let mut fields = work::Fields::new();
        fields.insert("statement".into(), json!("Uses Cargo"));
        fields.insert("slug".into(), json!("cargo"));
        fields.insert("scope".into(), json!(["app/api"]));
        let tx = store.db.unchecked_transaction().expect("tx");
        apply_fact_op(
            &tx,
            "p",
            &fact_op(work::OP_CREATE, "fact-1", fields),
            &stamp,
        )
        .expect("create");
        tx.commit().expect("commit");

        let facts = store.load_polter_facts("p").expect("facts");
        assert_eq!(facts.len(), 1, "facts = {facts:?}");
        assert_eq!(facts[0].fact.statement, "Uses Cargo");
        assert_eq!(facts[0].fact.truth_class, canon::TRUTH_DECLARED);
        assert_eq!(facts[0].fact.source, "polter");
        assert_eq!(facts[0].fact.actor, "agent");
        assert_eq!(facts[0].slug, "cargo");
        assert_eq!(facts[0].scopes, vec!["app/api".to_string()]);

        let mut fields = work::Fields::new();
        fields.insert("statement".into(), json!("Uses Cargo workspaces"));
        fields.insert("kind".into(), json!(canon::TRUTH_INFERRED));
        let tx = store.db.unchecked_transaction().expect("tx");
        apply_fact_op(
            &tx,
            "p",
            &fact_op(work::OP_UPDATE, "fact-1", fields),
            &stamp,
        )
        .expect("update");
        tx.commit().expect("commit");
        let facts = store.load_polter_facts("p").expect("facts");
        assert_eq!(facts.len(), 1, "an update must not add a fact");
        assert_eq!(facts[0].fact.statement, "Uses Cargo workspaces");
        assert_eq!(facts[0].fact.truth_class, canon::TRUTH_INFERRED);
        assert_eq!(facts[0].slug, "cargo", "an update keeps the slug");
        assert_eq!(
            facts[0].scopes,
            vec!["app/api".to_string()],
            "an update keeps the contexts"
        );
    }

    #[test]
    fn apply_fact_op_refuses_unknown_and_removed() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        let stamp = crate::providers::sqlite::format_rfc3339(at);
        let mut fields = work::Fields::new();
        fields.insert("statement".into(), json!("One core"));
        let tx = store.db.unchecked_transaction().expect("tx");
        let err = apply_fact_op(
            &tx,
            "p",
            &fact_op(work::OP_UPDATE, "missing", fields.clone()),
            &stamp,
        )
        .expect_err("unknown fact");
        assert!(
            err.to_string().starts_with("fact missing not found: "),
            "err = {err}"
        );
        let err = apply_fact_op(
            &tx,
            "p",
            &fact_op(work::OP_REMOVE, "fact-1", fields),
            &stamp,
        )
        .expect_err("remove");
        assert_eq!(err.to_string(), "facts cannot be removed through Polter");
        tx.rollback().expect("rollback");
    }

    #[test]
    fn apply_fact_op_validates_the_fact() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        let stamp = crate::providers::sqlite::format_rfc3339(at);
        let mut fields = work::Fields::new();
        fields.insert("statement".into(), json!("One core"));
        fields.insert("kind".into(), json!(canon::TRUTH_VERIFIED));
        fields.insert("actor".into(), json!("system"));
        let tx = store.db.unchecked_transaction().expect("tx");
        let err = apply_fact_op(
            &tx,
            "p",
            &fact_op(work::OP_CREATE, "fact-1", fields),
            &stamp,
        )
        .expect_err("verified needs an actor");
        assert_eq!(
            err.to_string(),
            "verified facts require a non-system actor",
            "err = {err}"
        );
        tx.rollback().expect("rollback");
    }
}
