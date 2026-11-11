//! Port of `backend/providers/sqlite/scope.go`.
//!
//! An entity's contexts: the Structure nodes a Decision, a doc, or a Fact is
//! about. None means the whole project. The same file resolves a global scope
//! against the project tree.

use std::collections::HashMap;

use rusqlite::Connection;
use rusqlite::params;

use crate::domains::structure;

use super::{Error, Result};

/// Where an entity's scope is stored, by kind.
pub const SCOPE_DECISION: &str = "decision";
pub const SCOPE_FACT: &str = "fact";
pub const SCOPE_RULE: &str = "rule";
/// Any knowledge item, such as a doc.
pub const SCOPE_KNOWLEDGE: &str = "knowledge";
/// A derived Inbox item, keyed by its item key.
pub const SCOPE_ITEM: &str = "item";

/// The table that holds each scoped entity kind.
fn scope_table(kind: &str) -> Option<&'static str> {
    match kind {
        SCOPE_DECISION | SCOPE_KNOWLEDGE => Some("knowledge_items"),
        SCOPE_FACT => Some("canon_facts"),
        SCOPE_RULE => Some("rules"),
        _ => None,
    }
}

impl super::Store {
    /// Replaces an entity's contexts (or a derived item's, by item key) with
    /// `node_ids`; an empty list clears them, which means the whole project.
    /// Node paths are not accepted; resolve them first. Every node must be an
    /// active Structure node.
    pub fn set_scope(
        &self,
        project_id: &str,
        kind: &str,
        id: &str,
        node_ids: &[String],
    ) -> Result<()> {
        let node_ids = unique_ids(node_ids);
        for node_id in &node_ids {
            let found = self.db.query_row(
                r#"SELECT id FROM structure_nodes WHERE id=? AND project_id=? AND status<>'removed'"#,
                params![node_id, project_id],
                |row| row.get::<_, String>(0),
            );
            if found.is_err() {
                return Err(Error::message(format!(
                    "unknown structure node {node_id:?}"
                )));
            }
        }
        if kind != SCOPE_ITEM {
            let Some(table) = scope_table(kind) else {
                return Err(Error::message(format!("unknown scope kind {kind:?}")));
            };
            let found = self.db.query_row(
                &format!("SELECT id FROM {table} WHERE id=? AND project_id=?"),
                params![id, project_id],
                |row| row.get::<_, String>(0),
            );
            if found.is_err() {
                return Err(Error::message(format!("{kind} {id:?} not found")));
            }
        }
        let tx = self.db.unchecked_transaction()?;
        replace_contexts(&tx, project_id, id, &node_ids)?;
        tx.commit()?;
        Ok(())
    }

    /// Maps entity id to the paths of its context nodes, sorted.
    pub fn context_paths(&self, project_id: &str) -> Result<HashMap<String, Vec<String>>> {
        let mut out: HashMap<String, Vec<String>> = HashMap::new();
        self.each_row(
            r#"SELECT c.entity_id,n.path FROM entity_contexts c JOIN structure_nodes n ON n.id=c.node_id
		WHERE c.project_id=? ORDER BY c.entity_id, n.path"#,
            params![project_id],
            |row| {
                let id: String = row.get(0)?;
                let path: String = row.get(1)?;
                out.entry(id).or_default().push(path);
                Ok(())
            },
        )?;
        Ok(out)
    }

    /// Returns entity ID (or item key) to its context node IDs for every entity
    /// of `kind` in the project that has contexts. Entities without any are
    /// absent.
    pub fn scope_map(&self, project_id: &str, kind: &str) -> Result<HashMap<String, Vec<String>>> {
        let query = if kind == SCOPE_ITEM {
            // Item keys are not rows of any entity table.
            String::from(
                r#"SELECT c.entity_id,c.node_id FROM entity_contexts c WHERE c.project_id=?
			AND NOT EXISTS (SELECT 1 FROM knowledge_items t WHERE t.id=c.entity_id)
			AND NOT EXISTS (SELECT 1 FROM canon_facts t WHERE t.id=c.entity_id)
			AND NOT EXISTS (SELECT 1 FROM rules t WHERE t.id=c.entity_id)
			ORDER BY c.entity_id, c.node_id"#,
            )
        } else {
            let Some(table) = scope_table(kind) else {
                return Err(Error::message(format!("unknown scope kind {kind:?}")));
            };
            format!(
                r#"SELECT c.entity_id,c.node_id FROM entity_contexts c JOIN {table} t ON t.id=c.entity_id
			WHERE c.project_id=? ORDER BY c.entity_id, c.node_id"#
            )
        };
        let mut out: HashMap<String, Vec<String>> = HashMap::new();
        self.each_row(&query, params![project_id], |row| {
            let id: String = row.get(0)?;
            let node: String = row.get(1)?;
            out.entry(id).or_default().push(node);
            Ok(())
        })?;
        Ok(out)
    }
}

/// Writes an entity's contexts as exactly `node_ids`.
pub fn replace_contexts(
    tx: &Connection,
    project_id: &str,
    id: &str,
    node_ids: &[String],
) -> Result<()> {
    tx.execute(
        r#"DELETE FROM entity_contexts WHERE project_id=? AND entity_id=?"#,
        params![project_id, id],
    )?;
    for node_id in unique_ids(node_ids) {
        tx.execute(
            r#"INSERT INTO entity_contexts(project_id,entity_id,node_id) VALUES(?,?,?)"#,
            params![project_id, id, node_id],
        )?;
    }
    Ok(())
}

/// Resolves Structure node paths and writes them as an entity's contexts. An
/// empty list clears them.
pub fn set_context_paths(
    tx: &Connection,
    project_id: &str,
    id: &str,
    paths: &[String],
) -> Result<()> {
    let mut node_ids = Vec::with_capacity(paths.len());
    for path in paths {
        let node_id = tx
            .query_row(
                r#"SELECT id FROM structure_nodes WHERE project_id=? AND path=? AND status<>'removed'"#,
                params![project_id, path],
                |row| row.get::<_, String>(0),
            )
            .map_err(|_| {
                Error::message(format!(
                    "scope {path:?} is not a Structure node path: list them with polterware structure"
                ))
            })?;
        node_ids.push(node_id);
    }
    replace_contexts(tx, project_id, id, &node_ids)
}

/// Removes empty and duplicate ids, keeping the first occurrence.
pub fn unique_ids(ids: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        if id.is_empty() || !seen.insert(id.clone()) {
            continue;
        }
        out.push(id.clone());
    }
    out
}

/// The resolved filter for one project: the matching node set and the nodes it
/// was built from, so callers can also match by node path.
#[derive(Debug, Clone, Default)]
pub struct NodeScope {
    pub set: structure::ScopeSet,
    pub nodes: Vec<structure::Node>,
}

impl NodeScope {
    /// Reports whether an entity scoped to `node_id` is visible.
    pub fn matches(&self, node_id: &str) -> bool {
        self.set.matches(node_id)
    }

    /// Reports whether an entity with these contexts is visible: one of them is
    /// inside the scope, or it has none (the whole project).
    pub fn matches_any(&self, node_ids: &[String]) -> bool {
        self.set.matches_any(node_ids)
    }

    /// Reports whether an entity scoped by node path is visible. An empty path
    /// is unscoped and always matches.
    pub fn matches_path(&self, path: &str) -> bool {
        if self.set.is_nil() || path.is_empty() {
            return true;
        }
        for node in &self.nodes {
            if node.path == path {
                return self.set.contains(&node.id);
            }
        }
        false
    }
}

impl super::Store {
    /// Resolves one node against the project's Structure. An inactive scope
    /// returns a [`NodeScope`] whose set is nil and matches everything.
    pub fn load_scope(&self, project_id: &str, scope: &structure::Scope) -> Result<NodeScope> {
        let ids = if scope.node_id.is_empty() {
            Vec::new()
        } else {
            vec![scope.node_id.clone()]
        };
        self.load_scopes(
            project_id,
            &structure::MultiScope {
                node_ids: ids,
                exact: scope.exact,
            },
        )
    }

    /// Resolves several nodes against the project's Structure. The set is the
    /// union of each node's scope. An inactive scope matches everything.
    pub fn load_scopes(
        &self,
        project_id: &str,
        scope: &structure::MultiScope,
    ) -> Result<NodeScope> {
        if !scope.active() {
            return Ok(NodeScope::default());
        }
        let nodes = self.load_structure_nodes(project_id)?;
        Ok(NodeScope {
            set: structure::resolve_scopes(&nodes, scope),
            nodes,
        })
    }

    /// Returns every Structure node of the project, removed ones included.
    pub fn structure_nodes(&self, project_id: &str) -> Result<Vec<structure::Node>> {
        self.load_structure_nodes(project_id)
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::params;
    use time::OffsetDateTime;

    use crate::domains::knowledge;
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    use super::*;

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    fn doc(id: &str, project_id: &str, at: OffsetDateTime) -> knowledge::Item {
        knowledge::Item {
            id: id.to_string(),
            project_id: project_id.to_string(),
            title: id.to_string(),
            kind: knowledge::KIND_DOC.to_string(),
            doc_type: knowledge::DOC_TYPE_DOC.to_string(),
            created_at: at,
            updated_at: at,
            ..knowledge::Item::default()
        }
    }

    #[test]
    fn entity_contexts_match_any_node() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "p", at);
        let stamp = crate::providers::sqlite::format_rfc3339(at);
        for (id, parent, level, path) in [
            ("app", None, "domain", "app"),
            ("api", Some("app"), "component", "app/api"),
            ("web", None, "domain", "web"),
        ] {
            store
                .db
                .execute(
                    "INSERT INTO structure_nodes(id,project_id,parent_id,level,slug,path,title,created_at,updated_at) VALUES(?,'p',?,?,?,?,?,?,?)",
                    params![id, parent, level, id, path, id, stamp, stamp],
                )
                .expect("insert node");
        }
        for id in ["both", "web-only", "none"] {
            store.save_knowledge(doc(id, "p", at)).expect("save doc");
        }
        store
            .set_scope(
                "p",
                SCOPE_KNOWLEDGE,
                "both",
                &["web".to_string(), "api".to_string(), "api".to_string()],
            )
            .expect("scope both");
        store
            .set_scope("p", SCOPE_KNOWLEDGE, "web-only", &["web".to_string()])
            .expect("scope web-only");
        let err = store
            .set_scope("p", SCOPE_KNOWLEDGE, "none", &["missing".to_string()])
            .expect_err("unknown node");
        assert_eq!(err.to_string(), "unknown structure node \"missing\"");
        assert_eq!(
            store
                .set_scope("p", "nope", "both", &[])
                .expect_err("unknown kind")
                .to_string(),
            "unknown scope kind \"nope\""
        );

        let scopes = store.scope_map("p", SCOPE_KNOWLEDGE).expect("scopes");
        assert_eq!(
            scopes.get("both"),
            Some(&vec!["api".to_string(), "web".to_string()])
        );
        assert_eq!(scopes.get("none").map(Vec::len).unwrap_or(0), 0);
        // A filter on app keeps the doc with one context under it and the doc with none.
        let resolved = store
            .load_scopes(
                "p",
                &structure::MultiScope {
                    node_ids: vec!["app".to_string()],
                    exact: false,
                },
            )
            .expect("scope");
        let mut kept = Vec::new();
        for id in ["both", "web-only", "none"] {
            let contexts = scopes.get(id).cloned().unwrap_or_default();
            if resolved.matches_any(&contexts) {
                kept.push(id);
            }
        }
        assert_eq!(kept, vec!["both", "none"]);
        // An empty list clears the contexts: the doc is about the whole project again.
        store
            .set_scope("p", SCOPE_KNOWLEDGE, "both", &[])
            .expect("clear");
        let scopes = store.scope_map("p", SCOPE_KNOWLEDGE).expect("scopes");
        assert_eq!(scopes.get("both").map(Vec::len).unwrap_or(0), 0);
    }
}
