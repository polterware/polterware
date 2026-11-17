//! Structure nodes, stored.
//!
//! The Structure tree reader `scope.go` resolves a scope against, and the write
//! ops a node takes: a create, an update, and a removal.

use rusqlite::{Connection, params, params_from_iter};

use crate::domains::structure;
use crate::domains::work;

use super::work::{ColumnValue, column_value, field_column, field_str};
use super::{Error, Result};

impl super::Store {
    /// Returns every Structure node of the project, removed ones included, in
    /// position order.
    pub fn load_structure_nodes(&self, project_id: &str) -> Result<Vec<structure::Node>> {
        let mut nodes: Vec<structure::Node> = Vec::new();
        {
            let mut stmt = self.db.prepare(
                r#"SELECT id,project_id,COALESCE(parent_id,''),level,slug,path,title,intent,status,decisions_json,position,repo,folder,kind,tech,migration_from,migration_to
		FROM structure_nodes WHERE project_id=? ORDER BY position, created_at"#,
            )?;
            let mut rows = stmt.query(params![project_id])?;
            while let Some(row) = rows.next()? {
                let decisions: String = row.get(9)?;
                let node = structure::Node {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    parent_id: row.get(2)?,
                    level: row.get(3)?,
                    slug: row.get(4)?,
                    path: row.get(5)?,
                    title: row.get(6)?,
                    intent: row.get(7)?,
                    status: row.get(8)?,
                    decisions: serde_json::from_str::<Vec<String>>(&decisions).unwrap_or_default(),
                    position: row.get(10)?,
                    repo: row.get(11)?,
                    folder: row.get(12)?,
                    kind: row.get(13)?,
                    tech: row.get(14)?,
                    migration_from: row.get(15)?,
                    migration_to: row.get(16)?,
                };
                nodes.push(node);
            }
        }
        Ok(nodes)
    }
}

// The Work apply in `work.rs` calls the entry point below.
pub(crate) fn apply_structure_op(
    tx: &Connection,
    project_id: &str,
    op: &work::Op,
    now: &str,
) -> Result<()> {
    match op.kind.as_str() {
        work::OP_CREATE => {
            let values = vec![
                ColumnValue::text(op.entity_id.as_str()),
                ColumnValue::text(project_id),
                ColumnValue::text(field_str(&op.fields, "parent_id")),
                field_column(&op.fields, "level")?,
                field_column(&op.fields, "slug")?,
                field_column(&op.fields, "path")?,
                field_column(&op.fields, "title")?,
                field_column(&op.fields, "intent")?,
                field_column(&op.fields, "status")?,
                field_column(&op.fields, "decisions")?,
                field_column(&op.fields, "position")?,
                ColumnValue::text(field_str(&op.fields, "repo")),
                ColumnValue::text(field_str(&op.fields, "folder")),
                ColumnValue::text(field_str(&op.fields, "kind")),
                ColumnValue::text(field_str(&op.fields, "tech")),
                ColumnValue::text(field_str(&op.fields, "migration_from")),
                ColumnValue::text(field_str(&op.fields, "migration_to")),
                ColumnValue::text(now),
                ColumnValue::text(now),
            ];
            tx.execute(
                r#"INSERT INTO structure_nodes(id,project_id,parent_id,level,slug,path,title,intent,status,decisions_json,position,repo,folder,kind,tech,migration_from,migration_to,created_at,updated_at)
			VALUES(?,?,NULLIF(?,''),?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"#,
                params_from_iter(values.iter()),
            )?;
        }
        work::OP_UPDATE | work::OP_REMOVE => {
            update_structure_node(tx, op, now)?;
        }
        other => return Err(Error::message(format!("unknown op {other:?}"))),
    }
    Ok(())
}

fn update_structure_node(tx: &Connection, op: &work::Op, now: &str) -> Result<()> {
    let mut sets: Vec<String> = Vec::new();
    let mut values: Vec<ColumnValue> = Vec::new();
    for (field, value) in &op.fields {
        match field.as_str() {
            "parent_id" => {
                sets.push("parent_id=NULLIF(?,'')".to_string());
                values.push(column_value(value)?);
            }
            "decisions" => {
                sets.push("decisions_json=?".to_string());
                values.push(column_value(value)?);
            }
            "slug" | "path" | "title" | "intent" | "level" | "status" | "position" | "repo"
            | "folder" | "kind" | "tech" | "migration_from" | "migration_to" => {
                sets.push(format!("{field}=?"));
                values.push(column_value(value)?);
            }
            other => return Err(Error::message(format!("unknown field {other:?}"))),
        }
    }
    if sets.is_empty() {
        return Ok(());
    }
    sets.push("updated_at=?".to_string());
    values.push(ColumnValue::text(now.to_string()));
    values.push(ColumnValue::text(op.entity_id.clone()));
    let query = format!("UPDATE structure_nodes SET {} WHERE id=?", sets.join(","));
    tx.execute(&query, rusqlite::params_from_iter(values))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use time::OffsetDateTime;

    use crate::providers::sqlite::format_rfc3339;
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    use super::*;

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    fn node_op(kind: &str, id: &str, fields: work::Fields) -> work::Op {
        work::Op {
            entity: work::ENTITY_NODE.to_string(),
            kind: kind.to_string(),
            entity_id: id.to_string(),
            node_id: id.to_string(),
            fields,
            ..work::Op::default()
        }
    }

    fn create_fields() -> work::Fields {
        let mut fields = work::Fields::new();
        fields.insert("level".into(), json!("component"));
        fields.insert("slug".into(), json!("api"));
        fields.insert("path".into(), json!("app/api"));
        fields.insert("title".into(), json!("API"));
        fields.insert("status".into(), json!("active"));
        fields.insert("position".into(), json!(0));
        fields.insert("intent".into(), json!(""));
        fields.insert("decisions".into(), json!([]));
        fields
    }

    fn apply(store: &super::super::Store, op: &work::Op) -> Result<()> {
        let stamp = format_rfc3339(now());
        let tx = store.db.unchecked_transaction().expect("tx");
        let result = apply_structure_op(&tx, "p", op, &stamp);
        match &result {
            Ok(()) => tx.commit().expect("commit"),
            Err(_) => tx.rollback().expect("rollback"),
        }
        result
    }

    #[test]
    fn apply_structure_op_creates_updates_and_removes() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);

        apply(store, &node_op(work::OP_CREATE, "n-api", create_fields())).expect("create");
        let nodes = store.load_structure_nodes("p").expect("nodes");
        assert_eq!(nodes.len(), 1, "nodes = {nodes:?}");
        assert_eq!(nodes[0].path, "app/api");
        assert_eq!(nodes[0].parent_id, "", "an empty parent is NULL");
        assert!(nodes[0].decisions.is_empty());

        let mut fields = work::Fields::new();
        fields.insert("title".into(), json!("API service"));
        fields.insert("decisions".into(), json!(["Decision-0001"]));
        apply(store, &node_op(work::OP_UPDATE, "n-api", fields)).expect("update");
        let nodes = store.load_structure_nodes("p").expect("nodes");
        assert_eq!(nodes[0].title, "API service");
        assert_eq!(nodes[0].decisions, vec!["Decision-0001".to_string()]);
        assert_eq!(
            nodes[0].path, "app/api",
            "an update leaves the fields it does not name alone"
        );

        apply(
            store,
            &node_op(work::OP_REMOVE, "n-api", {
                let mut fields = work::Fields::new();
                fields.insert("status".into(), json!("removed"));
                fields
            }),
        )
        .expect("remove");
        let nodes = store.load_structure_nodes("p").expect("nodes");
        assert_eq!(nodes[0].status, "removed");
    }

    #[test]
    fn apply_structure_op_refuses_unknown_fields_and_ops() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        apply(store, &node_op(work::OP_CREATE, "n-api", create_fields())).expect("create");

        let mut fields = work::Fields::new();
        fields.insert("nope".into(), json!("x"));
        let err =
            apply(store, &node_op(work::OP_UPDATE, "n-api", fields)).expect_err("unknown field");
        assert_eq!(err.to_string(), "unknown field \"nope\"");

        let err =
            apply(store, &node_op("bogus", "n-api", work::Fields::new())).expect_err("unknown op");
        assert_eq!(err.to_string(), "unknown op \"bogus\"");
    }

    #[test]
    fn a_parent_link_survives_and_can_be_cleared() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        let mut fields = create_fields();
        fields.insert("level".into(), json!("domain"));
        fields.insert("path".into(), json!("app"));
        fields.insert("slug".into(), json!("app"));
        apply(store, &node_op(work::OP_CREATE, "n-app", fields)).expect("domain");
        apply(store, &node_op(work::OP_CREATE, "n-api", create_fields())).expect("component");
        let mut fields = work::Fields::new();
        fields.insert("parent_id".into(), json!("n-app"));
        apply(store, &node_op(work::OP_UPDATE, "n-api", fields)).expect("parent");
        let nodes = store.load_structure_nodes("p").expect("nodes");
        let api = nodes.iter().find(|node| node.id == "n-api").expect("api");
        assert_eq!(api.parent_id, "n-app");
        let mut fields = work::Fields::new();
        fields.insert("parent_id".into(), json!(""));
        apply(store, &node_op(work::OP_UPDATE, "n-api", fields)).expect("clear");
        let nodes = store.load_structure_nodes("p").expect("nodes");
        let api = nodes.iter().find(|node| node.id == "n-api").expect("api");
        assert_eq!(api.parent_id, "", "an empty parent clears the link");
    }
}
