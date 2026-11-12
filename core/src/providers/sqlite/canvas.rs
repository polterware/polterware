//! Canvases and their archives, stored.
//!
//! Canvases: the semantic graph, the opaque layout beside it, the Structure
//! node it is scoped to, and the change op each write appends.

use rusqlite::Connection;
use rusqlite::Row;
use rusqlite::params;
use serde_json::{Map, Value};
use time::OffsetDateTime;

use crate::domains::canvas;
use crate::domains::structure;
use crate::platform::ids;

use super::*;

const CANVAS_COLUMNS: &str = "id,project_id,name,type,agent_visible,graph_json,layout_json,COALESCE(scope_node_id,''),created_at,updated_at,archived_at";

impl super::Store {
    /// Reports whether `node_id` belongs to the project. An empty `node_id` is
    /// the whole project and is allowed.
    pub fn require_structure_node(&self, project_id: &str, node_id: &str) -> Result<()> {
        require_structure_node(&self.db, project_id, node_id)
    }

    /// Returns the project's active canvases ordered by name.
    pub fn list_canvases(&self, project_id: &str) -> Result<Vec<canvas::Canvas>> {
        self.list_canvases_by(project_id, false)
    }

    /// Returns the project's archived canvases ordered by name.
    pub fn list_archived_canvases(&self, project_id: &str) -> Result<Vec<canvas::Canvas>> {
        self.list_canvases_by(project_id, true)
    }

    fn list_canvases_by(&self, project_id: &str, archived: bool) -> Result<Vec<canvas::Canvas>> {
        let filter = if archived {
            "archived_at<>''"
        } else {
            "archived_at=''"
        };
        let mut stmt = self.db.prepare(&format!(
            "SELECT {CANVAS_COLUMNS} FROM canvases WHERE project_id=? AND {filter} ORDER BY name, id"
        ))?;
        let mut rows = stmt.query(params![project_id])?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            items.push(scan_canvas(row)?);
        }
        Ok(items)
    }

    /// Returns one canvas in the project, archived or not.
    pub fn get_canvas(&self, project_id: &str, id: &str) -> Result<canvas::Canvas> {
        let mut stmt = self.db.prepare(&format!(
            "SELECT {CANVAS_COLUMNS} FROM canvases WHERE project_id=? AND id=?"
        ))?;
        let mut rows = stmt.query(params![project_id, id])?;
        match rows.next()? {
            Some(row) => scan_canvas(row),
            None => Err(rusqlite::Error::QueryReturnedNoRows.into()),
        }
    }

    /// Inserts a canvas and one change op. The graph is validated.
    pub fn create_canvas(
        &self,
        item: canvas::Canvas,
        agent_run_id: &str,
        batch_id: &str,
    ) -> Result<canvas::Canvas> {
        self.write_canvas(item, agent_run_id, batch_id, canvas::OP_CREATE, true)
    }

    /// Replaces the mutable columns and appends one change op.
    pub fn update_canvas(
        &self,
        item: canvas::Canvas,
        agent_run_id: &str,
        batch_id: &str,
    ) -> Result<canvas::Canvas> {
        self.write_canvas(item, agent_run_id, batch_id, canvas::OP_UPDATE, false)
    }

    /// Moves a canvas into the archive or back out of it and appends an update
    /// op that records the move.
    pub fn set_canvas_archived(
        &self,
        project_id: &str,
        id: &str,
        archived: bool,
        at: OffsetDateTime,
        agent_run_id: &str,
        batch_id: &str,
    ) -> Result<canvas::Canvas> {
        let existing = self.get_canvas(project_id, id)?;
        let batch_id = if batch_id.is_empty() {
            ids::new_ulid()
        } else {
            batch_id.to_string()
        };
        let stamp = if archived {
            format_rfc3339(at)
        } else {
            String::new()
        };
        let tx = self.db.unchecked_transaction()?;
        let now = format_rfc3339(OffsetDateTime::now_utc());
        tx.execute(
            r#"UPDATE canvases SET archived_at=?, updated_at=? WHERE project_id=? AND id=?"#,
            params![&stamp, &now, project_id, id],
        )?;
        let mut extra = Map::new();
        extra.insert("archived".to_string(), Value::Bool(archived));
        insert_canvas_op_data(
            &tx,
            &existing,
            agent_run_id,
            &batch_id,
            canvas::OP_UPDATE,
            &now,
            Some(extra),
        )?;
        tx.commit()?;
        self.get_canvas(project_id, id)
    }

    /// Removes an archived canvas for good and appends a remove op. An active
    /// canvas is refused; missing rows return the no-rows error.
    pub fn delete_canvas(
        &self,
        project_id: &str,
        id: &str,
        agent_run_id: &str,
        batch_id: &str,
    ) -> Result<()> {
        let existing = self.get_canvas(project_id, id)?;
        if !existing.archived() {
            return Err(canvas::Error::NotArchived.into());
        }
        let tx = self.db.unchecked_transaction()?;
        let now = format_rfc3339(OffsetDateTime::now_utc());
        insert_canvas_op(
            &tx,
            &existing,
            agent_run_id,
            batch_id,
            canvas::OP_REMOVE,
            &now,
        )?;
        let removed = tx.execute(
            r#"DELETE FROM canvases WHERE project_id=? AND id=?"#,
            params![project_id, id],
        )?;
        if removed == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows.into());
        }
        tx.commit()?;
        Ok(())
    }

    fn write_canvas(
        &self,
        item: canvas::Canvas,
        agent_run_id: &str,
        batch_id: &str,
        op: &str,
        insert: bool,
    ) -> Result<canvas::Canvas> {
        let mut prepared = canvas::prepare(item)?;
        if prepared.id.is_empty() {
            prepared.id = ids::new_ulid();
        }
        let batch_id = if batch_id.is_empty() {
            ids::new_ulid()
        } else {
            batch_id.to_string()
        };
        let graph_json = canvas::marshal_graph(&prepared.graph)?;
        let visible = i64::from(prepared.agent_visible);
        let tx = self.db.unchecked_transaction()?;
        require_structure_node(&tx, &prepared.project_id, &prepared.scope_node_id)?;
        let now = format_rfc3339(OffsetDateTime::now_utc());
        if insert {
            tx.execute(
                r#"INSERT INTO canvases(id,project_id,name,type,agent_visible,graph_json,layout_json,scope_node_id,created_at,updated_at)
			VALUES(?,?,?,?,?,?,?,?,?,?)"#,
                params![
                    &prepared.id,
                    &prepared.project_id,
                    &prepared.name,
                    &prepared.r#type,
                    visible,
                    &graph_json,
                    &prepared.layout_json,
                    nullable(&prepared.scope_node_id),
                    &now,
                    &now,
                ],
            )?;
        } else {
            let existing = tx.query_row(
                r#"SELECT id FROM canvases WHERE project_id=? AND id=?"#,
                params![&prepared.project_id, &prepared.id],
                |row| row.get::<_, String>(0),
            );
            if existing.is_ok() {
                tx.execute(
                    r#"UPDATE canvases SET name=?, agent_visible=?, graph_json=?, layout_json=?, scope_node_id=?, updated_at=?
				WHERE project_id=? AND id=?"#,
                    params![
                        &prepared.name,
                        visible,
                        &graph_json,
                        &prepared.layout_json,
                        nullable(&prepared.scope_node_id),
                        &now,
                        &prepared.project_id,
                        &prepared.id,
                    ],
                )?;
            }
        }
        insert_canvas_op(&tx, &prepared, agent_run_id, &batch_id, op, &now)?;
        tx.commit()?;
        self.get_canvas(&prepared.project_id, &prepared.id)
    }
}

/// Checks a scope node against the project. An empty id is the whole project.
fn require_structure_node(q: &Connection, project_id: &str, node_id: &str) -> Result<()> {
    if node_id.is_empty() {
        return Ok(());
    }
    let found = q.query_row(
        r#"SELECT id FROM structure_nodes WHERE id=? AND project_id=? AND status<>?"#,
        params![node_id, project_id, structure::NODE_REMOVED],
        |row| row.get::<_, String>(0),
    );
    match found {
        Ok(_) => Ok(()),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(canvas::Error::UnknownScope.into()),
        Err(err) => Err(err.into()),
    }
}

fn insert_canvas_op(
    q: &Connection,
    item: &canvas::Canvas,
    agent_run_id: &str,
    batch_id: &str,
    op: &str,
    now: &str,
) -> Result<()> {
    insert_canvas_op_data(q, item, agent_run_id, batch_id, op, now, None)
}

fn insert_canvas_op_data(
    q: &Connection,
    item: &canvas::Canvas,
    agent_run_id: &str,
    batch_id: &str,
    op: &str,
    now: &str,
    extra: Option<Map<String, Value>>,
) -> Result<()> {
    let mut data = Map::new();
    data.insert("name".to_string(), Value::String(item.name.clone()));
    data.insert("type".to_string(), Value::String(item.r#type.clone()));
    data.insert("agent_visible".to_string(), Value::Bool(item.agent_visible));
    if let Some(extra) = extra {
        for (key, value) in extra {
            data.insert(key, value);
        }
    }
    let payload = serde_json::to_string(&Value::Object(data))
        .map_err(|err| Error::message(err.to_string()))?;
    q.execute(
        r#"INSERT INTO change_ops(id,project_id,agent_run_id,batch_id,entity,entity_id,op,data_json,node_id,created_at)
		VALUES(?,?,?,?,?,?,?,?,?,?)"#,
        params![
            ids::new_ulid(),
            &item.project_id,
            agent_run_id,
            batch_id,
            canvas::ENTITY,
            &item.id,
            op,
            &payload,
            &item.scope_node_id,
            now,
        ],
    )?;
    Ok(())
}

/// Reads a canvas in [`CANVAS_COLUMNS`] order, decoding its semantic graph.
fn scan_canvas(row: &Row<'_>) -> Result<canvas::Canvas> {
    let visible: i64 = row.get(4)?;
    let graph_raw: String = row.get(5)?;
    let created: String = row.get(8)?;
    let updated: String = row.get(9)?;
    let archived: String = row.get(10)?;
    Ok(canvas::Canvas {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        r#type: row.get(3)?,
        agent_visible: visible != 0,
        graph: canvas::unmarshal_graph(&graph_raw)?,
        layout_json: row.get(6)?,
        scope_node_id: row.get(7)?,
        created_at: parse_rfc3339(&created),
        updated_at: parse_rfc3339(&updated),
        archived_at: parse_optional_rfc3339(&archived),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use time::OffsetDateTime;

    use crate::providers::sqlite::Store;
    use crate::providers::sqlite::test_support::save_project;

    use super::*;

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    fn open(path: &std::path::Path) -> Store {
        Store::new(path).expect("open store")
    }

    fn save_structure_node(store: &Store, project_id: &str, id: &str, at: OffsetDateTime) {
        store
            .db
            .execute(
                r#"INSERT INTO structure_nodes(id,project_id,level,slug,path,title,created_at,updated_at)
		VALUES(?,?,'component','app','app','App',?,?)"#,
                params![id, project_id, format_rfc3339(at), format_rfc3339(at)],
            )
            .expect("structure node");
    }

    #[test]
    fn canvas_round_trip() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("canvas.db");
        let store = open(&path);
        let at = now();
        for id in ["proj-canvas", "proj-other"] {
            save_project(&store, id, at);
        }
        save_structure_node(&store, "proj-canvas", "node-1", at);

        let layout = r#"{"nodes":{"a":{"x":10,"y":20}},"groups":[],"token":"LAYOUT_ONLY_TOKEN"}"#;
        let created = store
            .create_canvas(
                canvas::Canvas {
                    project_id: "proj-canvas".to_string(),
                    name: "Platform".to_string(),
                    r#type: canvas::TYPE_CANVAS.to_string(),
                    agent_visible: true,
                    scope_node_id: "node-1".to_string(),
                    layout_json: layout.to_string(),
                    graph: canvas::Graph {
                        nodes: vec![
                            canvas::Node {
                                id: "lane".to_string(),
                                kind: "frame".to_string(),
                                label: "Platform".to_string(),
                                ..canvas::Node::default()
                            },
                            canvas::Node {
                                id: "a".to_string(),
                                kind: "service".to_string(),
                                label: "Import".to_string(),
                                props: serde_json::from_value(json!({"technology": "Go"}))
                                    .expect("props"),
                                ..canvas::Node::default()
                            },
                            canvas::Node {
                                id: "n".to_string(),
                                kind: canvas::KIND_NOTE.to_string(),
                                label: "ask design".to_string(),
                                ..canvas::Node::default()
                            },
                        ],
                        edges: vec![canvas::Edge {
                            id: "e".to_string(),
                            from: "a".to_string(),
                            to: "lane".to_string(),
                            kind: canvas::EDGE_DEPENDS_ON.to_string(),
                            ..canvas::Edge::default()
                        }],
                    },
                    ..canvas::Canvas::default()
                },
                "run-1",
                "batch-1",
            )
            .expect("create canvas");
        assert!(!created.id.is_empty(), "created = {created:?}");
        assert_eq!(created.scope_node_id, "node-1");
        assert!(created.agent_visible);
        assert_ne!(created.created_at, OffsetDateTime::UNIX_EPOCH);

        store
            .create_canvas(
                canvas::Canvas {
                    project_id: "proj-other".to_string(),
                    name: "Other".to_string(),
                    r#type: canvas::TYPE_CANVAS.to_string(),
                    ..canvas::Canvas::default()
                },
                "run-1",
                "batch-other",
            )
            .expect("other canvas");

        let (graph_raw, layout_raw): (String, String) = store
            .db
            .query_row(
                r#"SELECT graph_json, layout_json FROM canvases WHERE id=?"#,
                params![created.id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("stored canvas");
        assert!(
            !graph_raw.contains("LAYOUT_ONLY_TOKEN") && !graph_raw.contains(r#""source""#),
            "graph stored layout or export shape: {graph_raw}"
        );
        assert!(graph_raw.contains(r#""from""#), "graph = {graph_raw}");
        assert_eq!(layout_raw, layout);

        let (entity, op, node_id): (String, String, String) = store
            .db
            .query_row(
                r#"SELECT entity, op, node_id FROM change_ops WHERE entity_id=? AND op='create'"#,
                params![created.id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("change op");
        assert_eq!(
            (entity.as_str(), op.as_str(), node_id.as_str()),
            ("canvas", "create", "node-1")
        );

        let listed = store.list_canvases("proj-canvas").expect("list");
        assert_eq!(listed.len(), 1, "list = {listed:?}");
        assert_eq!(listed[0].id, created.id);
        assert_eq!(
            listed[0].graph.nodes[1].props.get("technology"),
            Some(&Value::String("Go".to_string()))
        );

        let mut changed = created.clone();
        changed.name = "Platform map".to_string();
        changed.layout_json = r#"{"nodes":{"a":{"x":1}}}"#.to_string();
        changed.agent_visible = false;
        let updated = store
            .update_canvas(changed, "run-1", "batch-2")
            .expect("update");
        assert_eq!(updated.name, "Platform map");
        assert!(!updated.agent_visible);
        assert_eq!(updated.r#type, canvas::TYPE_CANVAS);
        assert_eq!(updated.graph.edges.len(), 1);
        assert_eq!(updated.graph.edges[0].from, "a");
        assert_ne!(
            updated.layout_json, layout,
            "partial update kept the old layout"
        );

        store.close().expect("close");
        let reopened = open(&path);
        let got = reopened
            .get_canvas("proj-canvas", &created.id)
            .expect("reopen");
        assert_eq!(got.name, "Platform map");
        assert_eq!(got.graph.nodes[2].kind, canvas::KIND_NOTE);
        assert_eq!(got.scope_node_id, "node-1");

        reopened
            .db
            .execute(r#"DELETE FROM structure_nodes WHERE id='node-1'"#, [])
            .expect("delete node");
        let got = reopened
            .get_canvas("proj-canvas", &created.id)
            .expect("after node delete");
        assert_eq!(got.scope_node_id, "");

        let err = reopened
            .delete_canvas("proj-canvas", &created.id, "run-1", "batch-3")
            .expect_err("delete of an active canvas");
        assert!(
            matches!(err, Error::Canvas(canvas::Error::NotArchived)),
            "err = {err:?}"
        );
        assert_eq!(
            err.to_string(),
            "archive the canvas before deleting it permanently"
        );

        let archived = reopened
            .set_canvas_archived("proj-canvas", &created.id, true, at, "run-1", "batch-4")
            .expect("archive");
        assert!(archived.archived());
        let active = reopened.list_canvases("proj-canvas").expect("active");
        assert!(active.is_empty(), "active list after archive = {active:?}");
        let archived_list = reopened
            .list_archived_canvases("proj-canvas")
            .expect("archived");
        assert_eq!(archived_list.len(), 1);

        let restored = reopened
            .set_canvas_archived("proj-canvas", &created.id, false, at, "run-1", "batch-5")
            .expect("restore");
        assert!(!restored.archived());
        let active = reopened.list_canvases("proj-canvas").expect("active");
        assert_eq!(active.len(), 1);

        reopened
            .set_canvas_archived("proj-canvas", &created.id, true, at, "run-1", "batch-6")
            .expect("archive again");
        reopened
            .delete_canvas("proj-canvas", &created.id, "run-1", "batch-3")
            .expect("delete");
        let err = reopened
            .get_canvas("proj-canvas", &created.id)
            .expect_err("deleted get");
        assert!(err.is_no_rows(), "err = {err:?}");

        let removes: i64 = reopened
            .db
            .query_row(
                r#"SELECT COUNT(1) FROM change_ops WHERE entity='canvas' AND entity_id=? AND op='remove'"#,
                params![created.id],
                |row| row.get(0),
            )
            .expect("removes");
        assert_eq!(removes, 1);
    }

    #[test]
    fn canvas_rejects_invalid_graph_and_scope() {
        let holder = crate::providers::sqlite::test_support::TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "proj-canvas", at);

        let err = store
            .create_canvas(
                canvas::Canvas {
                    project_id: "proj-canvas".to_string(),
                    name: "Broken".to_string(),
                    r#type: canvas::TYPE_CANVAS.to_string(),
                    graph: canvas::Graph {
                        nodes: vec![canvas::Node {
                            id: "a".to_string(),
                            kind: "spaceship".to_string(),
                            ..canvas::Node::default()
                        }],
                        ..canvas::Graph::default()
                    },
                    ..canvas::Canvas::default()
                },
                "run-1",
                "batch-bad",
            )
            .expect_err("invalid graph");
        assert!(
            matches!(err, Error::Canvas(canvas::Error::InvalidGraph(_))),
            "invalid graph err = {err:?}"
        );
        assert!(
            err.to_string().starts_with("invalid canvas graph: "),
            "invalid graph err = {err}"
        );

        let err = store
            .create_canvas(
                canvas::Canvas {
                    project_id: "proj-canvas".to_string(),
                    name: "Scoped".to_string(),
                    r#type: canvas::TYPE_CANVAS.to_string(),
                    scope_node_id: "missing".to_string(),
                    ..canvas::Canvas::default()
                },
                "run-1",
                "batch-scope",
            )
            .expect_err("unknown scope");
        assert!(
            matches!(err, Error::Canvas(canvas::Error::UnknownScope)),
            "err = {err:?}"
        );
        assert_eq!(err.to_string(), "unknown scope node");

        let rows: i64 = store
            .db
            .query_row(r#"SELECT COUNT(1) FROM canvases"#, [], |row| row.get(0))
            .expect("rows");
        assert_eq!(rows, 0);
    }
}
