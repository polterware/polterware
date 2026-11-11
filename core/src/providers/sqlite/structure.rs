//! Port of `backend/providers/sqlite/structure.go`.
//!
//! The Structure tree reader `scope.go` resolves a scope against, and the write
//! ops a node takes: a create, an update, a removal, and the deploy links it
//! holds per environment.

use std::collections::HashMap;

use rusqlite::types::Value as SqlValue;
use rusqlite::{Connection, params};
use serde_json::Value as Json;

use crate::domains::structure;
use crate::domains::work;
use crate::platform::ids;

use super::{Error, Result};

impl super::Store {
    /// Returns every Structure node of the project, removed ones included, in
    /// position order, each with its deploy links and their latest check.
    pub fn load_structure_nodes(&self, project_id: &str) -> Result<Vec<structure::Node>> {
        let mut nodes: Vec<structure::Node> = Vec::new();
        let mut index = std::collections::HashMap::new();
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
                    deploys: Vec::new(),
                    position: row.get(10)?,
                    repo: row.get(11)?,
                    folder: row.get(12)?,
                    kind: row.get(13)?,
                    tech: row.get(14)?,
                    migration_from: row.get(15)?,
                    migration_to: row.get(16)?,
                };
                index.insert(node.id.clone(), nodes.len());
                nodes.push(node);
            }
        }
        if nodes.is_empty() {
            return Ok(nodes);
        }
        self.each_row(
            r#"SELECT d.id,d.node_id,d.environment_id,COALESCE(e.name,''),d.provider,d.resource,d.target_id,d.target_url,
			COALESCE(c.state,''),COALESCE(c.summary,''),COALESCE(c.url,''),COALESCE(c.checked_at,'')
		FROM node_deployments d LEFT JOIN environments e ON e.id=d.environment_id LEFT JOIN deploy_checks c ON c.deployment_id=d.id
		WHERE d.project_id=? ORDER BY e.name, d.environment_id"#,
            params![project_id],
            |row| {
                let node_id: String = row.get(1)?;
                let mut deploy = structure::Deployment {
                    id: row.get(0)?,
                    environment_id: row.get(2)?,
                    environment: row.get(3)?,
                    provider: row.get(4)?,
                    resource: row.get(5)?,
                    target_id: row.get(6)?,
                    url: row.get(7)?,
                    check: None,
                };
                let check = structure::DeployCheck {
                    state: row.get(8)?,
                    summary: row.get(9)?,
                    url: row.get(10)?,
                    checked_at: row.get(11)?,
                };
                if !check.state.is_empty() {
                    deploy.check = Some(check);
                }
                if let Some(&position) = index.get(&node_id) {
                    nodes[position].deploys.push(deploy);
                }
                Ok(())
            },
        )?;
        Ok(nodes)
    }
}

// The Work apply in `work.rs`, its own run, calls the entry points below.
#[allow(dead_code)]
pub(crate) fn apply_structure_op(
    tx: &Connection,
    project_id: &str,
    op: &work::Op,
    now: &str,
) -> Result<()> {
    let deploys = op.fields.get("deploys").cloned();
    let has_deploys = deploys.is_some();
    let parent = op
        .fields
        .get("parent_id")
        .and_then(Json::as_str)
        .unwrap_or("");
    match op.kind.as_str() {
        work::OP_CREATE => {
            let decisions = column_value(op.fields.get("decisions"))?;
            let repo = op.fields.get("repo").and_then(Json::as_str).unwrap_or("");
            let folder = op.fields.get("folder").and_then(Json::as_str).unwrap_or("");
            let kind = op.fields.get("kind").and_then(Json::as_str).unwrap_or("");
            let tech = op.fields.get("tech").and_then(Json::as_str).unwrap_or("");
            let migration_from = op
                .fields
                .get("migration_from")
                .and_then(Json::as_str)
                .unwrap_or("");
            let migration_to = op
                .fields
                .get("migration_to")
                .and_then(Json::as_str)
                .unwrap_or("");
            let values = vec![
                SqlValue::Text(op.entity_id.clone()),
                SqlValue::Text(project_id.to_string()),
                SqlValue::Text(parent.to_string()),
                column_value(op.fields.get("level"))?,
                column_value(op.fields.get("slug"))?,
                column_value(op.fields.get("path"))?,
                column_value(op.fields.get("title"))?,
                column_value(op.fields.get("intent"))?,
                column_value(op.fields.get("status"))?,
                decisions,
                column_value(op.fields.get("position"))?,
                SqlValue::Text(repo.to_string()),
                SqlValue::Text(folder.to_string()),
                SqlValue::Text(kind.to_string()),
                SqlValue::Text(tech.to_string()),
                SqlValue::Text(migration_from.to_string()),
                SqlValue::Text(migration_to.to_string()),
                SqlValue::Text(now.to_string()),
                SqlValue::Text(now.to_string()),
            ];
            tx.execute(
                r#"INSERT INTO structure_nodes(id,project_id,parent_id,level,slug,path,title,intent,status,decisions_json,position,repo,folder,kind,tech,migration_from,migration_to,created_at,updated_at)
			VALUES(?,?,NULLIF(?,''),?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)"#,
                rusqlite::params_from_iter(values),
            )?;
        }
        work::OP_UPDATE | work::OP_REMOVE => {
            let column_op = work::Op {
                fields: without_field(&op.fields, "deploys"),
                ..op.clone()
            };
            update_structure_node(tx, &column_op, now)?;
        }
        other => return Err(Error::message(format!("unknown op {other:?}"))),
    }
    if !has_deploys {
        return Ok(());
    }
    let value = deploys.unwrap_or(Json::Null);
    apply_node_deploys(tx, project_id, &op.entity_id, &value, now)
}

#[allow(dead_code)]
fn without_field(fields: &work::Fields, key: &str) -> work::Fields {
    let mut out = work::Fields::new();
    for (name, value) in fields {
        if name == key {
            continue;
        }
        out.insert(name.clone(), value.clone());
    }
    out
}

/// Replaces the node's links with `value`. An empty list clears them. The
/// provider is checked again so a hand-built op cannot store an unknown catalog
/// id or a credential.
#[allow(dead_code)]
fn apply_node_deploys(
    tx: &Connection,
    project_id: &str,
    node_id: &str,
    value: &Json,
    now: &str,
) -> Result<()> {
    let Some(deploys) = structure::deployments_from(value) else {
        return Err(Error::message("invalid deploys"));
    };
    let mut existing: HashMap<String, structure::Deployment> = HashMap::new();
    {
        let mut stmt = tx.prepare(
            r#"SELECT id,environment_id,provider,resource,target_id FROM node_deployments WHERE node_id=?"#,
        )?;
        let mut rows = stmt.query(params![node_id])?;
        while let Some(row) = rows.next()? {
            let deploy = structure::Deployment {
                id: row.get(0)?,
                environment_id: row.get(1)?,
                provider: row.get(2)?,
                resource: row.get(3)?,
                target_id: row.get(4)?,
                ..structure::Deployment::default()
            };
            existing.insert(deploy.environment_id.clone(), deploy);
        }
    }
    let mut seen: HashMap<String, bool> = HashMap::new();
    for deploy in &deploys {
        if !crate::domains::integrations::known_provider(&deploy.provider) {
            return Err(Error::message(format!(
                "unknown provider {:?}",
                deploy.provider
            )));
        }
        if deploy.environment_id.is_empty()
            || deploy.resource.trim().is_empty()
            || deploy.resource.contains(['\r', '\n'])
        {
            return Err(Error::message(
                "deployment needs an environment and a resource",
            ));
        }
        let lowered = deploy.resource.to_lowercase();
        if lowered.contains("begin private") || lowered.contains("begin rsa") {
            return Err(Error::message(
                "deployment resource must not be a credential",
            ));
        }
        seen.insert(deploy.environment_id.clone(), true);
        let current = existing.get(&deploy.environment_id);
        if let Some(current) = current
            && current.provider == deploy.provider
            && current.resource == deploy.resource
            && current.target_id == deploy.target_id
        {
            continue;
        }
        if let Some(current) = current {
            // Another resource is another target: its dashboard link and last
            // check go until the next check.
            tx.execute(
                r#"UPDATE node_deployments SET provider=?, resource=?, target_id=?, target_url='', updated_at=? WHERE id=?"#,
                params![
                    deploy.provider,
                    deploy.resource,
                    deploy.target_id,
                    now,
                    current.id
                ],
            )?;
            tx.execute(
                r#"DELETE FROM deploy_checks WHERE deployment_id=?"#,
                params![current.id],
            )?;
            continue;
        }
        let id = if deploy.id.is_empty() {
            ids::new_ulid()
        } else {
            deploy.id.clone()
        };
        tx.execute(
            r#"INSERT INTO node_deployments(id,project_id,node_id,environment_id,provider,resource,target_id,created_at,updated_at)
			VALUES(?,?,?,?,?,?,?,?,?)"#,
            params![
                id,
                project_id,
                node_id,
                deploy.environment_id,
                deploy.provider,
                deploy.resource,
                deploy.target_id,
                now,
                now,
            ],
        )?;
    }
    for (env, deploy) in &existing {
        if seen.contains_key(env) {
            continue;
        }
        tx.execute(
            r#"DELETE FROM deploy_checks WHERE deployment_id=?"#,
            params![deploy.id],
        )?;
        tx.execute(
            r#"DELETE FROM node_deployments WHERE id=?"#,
            params![deploy.id],
        )?;
    }
    Ok(())
}

#[allow(dead_code)]
fn update_structure_node(tx: &Connection, op: &work::Op, now: &str) -> Result<()> {
    let mut sets: Vec<String> = Vec::new();
    let mut values: Vec<SqlValue> = Vec::new();
    for (field, value) in &op.fields {
        match field.as_str() {
            "parent_id" => {
                sets.push("parent_id=NULLIF(?,'')".to_string());
                values.push(column_value(Some(value))?);
            }
            "decisions" => {
                sets.push("decisions_json=?".to_string());
                values.push(column_value(Some(value))?);
            }
            "slug" | "path" | "title" | "intent" | "level" | "status" | "position" | "repo"
            | "folder" | "kind" | "tech" | "migration_from" | "migration_to" => {
                sets.push(format!("{field}=?"));
                values.push(column_value(Some(value))?);
            }
            other => return Err(Error::message(format!("unknown field {other:?}"))),
        }
    }
    if sets.is_empty() {
        return Ok(());
    }
    sets.push("updated_at=?".to_string());
    values.push(SqlValue::Text(now.to_string()));
    values.push(SqlValue::Text(op.entity_id.clone()));
    let query = format!("UPDATE structure_nodes SET {} WHERE id=?", sets.join(","));
    tx.execute(&query, rusqlite::params_from_iter(values))?;
    Ok(())
}

/// Reads a planned op's field into a SQLite value the way Go's `columnValue`
/// does: a string list becomes its JSON text, a bool becomes 0 or 1, and any
/// other value is stored as it comes.
// TODO(dedupe): `work.go`'s `columnValue`; `work.rs` owns the original.
#[allow(dead_code)]
fn column_value(value: Option<&Json>) -> Result<SqlValue> {
    match value {
        None | Some(Json::Null) => Ok(SqlValue::Null),
        Some(Json::Array(_)) | Some(Json::Object(_)) => {
            let body = serde_json::to_string(value.expect("the value is present"))
                .map_err(|err| Error::message(err.to_string()))?;
            Ok(SqlValue::Text(body))
        }
        Some(Json::Bool(flag)) => Ok(SqlValue::Integer(i64::from(*flag))),
        Some(Json::Number(number)) => {
            if let Some(value) = number.as_i64() {
                return Ok(SqlValue::Integer(value));
            }
            match number.as_f64() {
                Some(value) => Ok(SqlValue::Real(value)),
                None => Ok(SqlValue::Null),
            }
        }
        Some(Json::String(text)) => Ok(SqlValue::Text(text.clone())),
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::params;
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

    #[test]
    fn deploys_replace_per_environment() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        for id in ["local", "production"] {
            store
                .save_environment(crate::domains::projects::Environment {
                    id: id.to_string(),
                    project_id: "p".to_string(),
                    name: id.to_string(),
                    kind: "local".to_string(),
                    ..crate::domains::projects::Environment::default()
                })
                .expect("environment");
        }
        apply(store, &node_op(work::OP_CREATE, "n-api", create_fields())).expect("create");

        let mut fields = work::Fields::new();
        fields.insert(
            "deploys".into(),
            json!([
                {"environment_id": "production", "provider": "vercel", "resource": "web", "target_id": "prj_1"},
                {"environment_id": "local", "provider": "docker", "resource": "api"}
            ]),
        );
        apply(store, &node_op(work::OP_UPDATE, "n-api", fields)).expect("deploys");
        let nodes = store.load_structure_nodes("p").expect("nodes");
        let deploys = &nodes[0].deploys;
        assert_eq!(deploys.len(), 2, "deploys = {deploys:?}");
        let production = deploys
            .iter()
            .find(|deploy| deploy.environment_id == "production")
            .expect("production deploy");
        assert_eq!(production.provider, "vercel");
        assert_eq!(production.target_id, "prj_1");
        assert!(production.check.is_none(), "a new link has no check yet");

        // A link whose resource changed is rewritten and loses its check.
        let deploy_id: String = production.id.clone();
        store
            .db
            .execute(
                "UPDATE node_deployments SET target_url='https://example.test' WHERE id=?",
                params![deploy_id],
            )
            .expect("target url");
        store
            .db
            .execute(
                "INSERT INTO deploy_checks(deployment_id,state,summary,url,checked_at) VALUES(?,'ok','','','2026-01-01T00:00:00Z')",
                params![deploy_id],
            )
            .expect("check");

        let mut fields = work::Fields::new();
        fields.insert(
            "deploys".into(),
            json!([
                {"environment_id": "production", "provider": "vercel", "resource": "docs"},
                {"environment_id": "local", "provider": "docker", "resource": "api"}
            ]),
        );
        apply(store, &node_op(work::OP_UPDATE, "n-api", fields)).expect("rewrite");
        let nodes = store.load_structure_nodes("p").expect("nodes");
        let production = nodes[0]
            .deploys
            .iter()
            .find(|deploy| deploy.environment_id == "production")
            .expect("production deploy");
        assert_eq!(production.resource, "docs");
        assert_eq!(production.url, "", "the old dashboard page goes with it");
        assert!(production.check.is_none(), "the old check goes with it");

        // Dropping an environment's link deletes the row.
        let mut fields = work::Fields::new();
        fields.insert(
            "deploys".into(),
            json!([{"environment_id": "local", "provider": "docker", "resource": "api"}]),
        );
        apply(store, &node_op(work::OP_UPDATE, "n-api", fields)).expect("drop");
        let nodes = store.load_structure_nodes("p").expect("nodes");
        assert_eq!(nodes[0].deploys.len(), 1);
        assert_eq!(nodes[0].deploys[0].environment_id, "local");
    }

    #[test]
    fn deploys_validate_provider_environment_resource_and_secrets() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        store
            .save_environment(crate::domains::projects::Environment {
                id: "local".to_string(),
                project_id: "p".to_string(),
                name: "local".to_string(),
                kind: "local".to_string(),
                ..crate::domains::projects::Environment::default()
            })
            .expect("environment");
        apply(store, &node_op(work::OP_CREATE, "n-api", create_fields())).expect("create");

        let deploy_error = |value: Json| {
            let mut fields = work::Fields::new();
            fields.insert("deploys".into(), value);
            apply(store, &node_op(work::OP_UPDATE, "n-api", fields))
                .expect_err("invalid deploy")
                .to_string()
        };
        assert_eq!(
            deploy_error(
                json!([{"environment_id": "local", "provider": "nope", "resource": "web"}])
            ),
            "unknown provider \"nope\""
        );
        assert_eq!(
            deploy_error(json!([{"environment_id": "", "provider": "vercel", "resource": "web"}])),
            "deployment needs an environment and a resource"
        );
        assert_eq!(
            deploy_error(
                json!([{"environment_id": "local", "provider": "vercel", "resource": "  "}])
            ),
            "deployment needs an environment and a resource"
        );
        assert_eq!(
            deploy_error(
                json!([{"environment_id": "local", "provider": "vercel", "resource": "a\nb"}])
            ),
            "deployment needs an environment and a resource"
        );
        assert_eq!(
            deploy_error(
                json!([{"environment_id": "local", "provider": "vercel", "resource": "-----BEGIN RSA PRIVATE KEY-----"}])
            ),
            "deployment resource must not be a credential"
        );
        assert_eq!(deploy_error(json!("not a list")), "invalid deploys");
        assert!(
            store.load_structure_nodes("p").expect("nodes")[0]
                .deploys
                .is_empty(),
            "a refused op stores nothing"
        );

        // An empty list clears the links.
        let mut fields = work::Fields::new();
        fields.insert("deploys".into(), json!([]));
        apply(store, &node_op(work::OP_UPDATE, "n-api", fields)).expect("clear");
    }

    #[test]
    fn structure_nodes_load_with_their_deploy_checks() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        store
            .save_environment(crate::domains::projects::Environment {
                id: "production".to_string(),
                project_id: "p".to_string(),
                name: "production".to_string(),
                kind: "cloud".to_string(),
                ..crate::domains::projects::Environment::default()
            })
            .expect("environment");
        apply(store, &node_op(work::OP_CREATE, "n-api", create_fields())).expect("create");
        let mut fields = work::Fields::new();
        fields.insert(
            "deploys".into(),
            json!([{"environment_id": "production", "provider": "vercel", "resource": "web", "id": "d-1"}]),
        );
        apply(store, &node_op(work::OP_UPDATE, "n-api", fields)).expect("deploy");
        store
            .db
            .execute(
                "INSERT INTO deploy_checks(deployment_id,state,summary,url,checked_at) VALUES('d-1','failed','Build failed','https://example.test','2026-01-01T00:00:00Z')",
                [],
            )
            .expect("check");

        let nodes = store.load_structure_nodes("p").expect("nodes");
        let deploy = &nodes[0].deploys[0];
        assert_eq!(deploy.environment, "production");
        assert_eq!(deploy.id, "d-1");
        let check = deploy.check.as_ref().expect("check");
        assert_eq!(check.state, "failed");
        assert_eq!(check.summary, "Build failed");
        assert_eq!(check.checked_at, "2026-01-01T00:00:00Z");

        // A node with no deploy keeps an empty list, and so does a project with
        // no nodes.
        assert!(
            store
                .load_structure_nodes("other")
                .expect("none")
                .is_empty()
        );
    }
}
