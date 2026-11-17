//! A Structure node: its level, kind, checkout, and derived status.

use std::collections::{HashMap, HashSet};

pub const LEVEL_DOMAIN: &str = "domain";
pub const LEVEL_COMPONENT: &str = "component";

pub const NODE_ACTIVE: &str = "active";
pub const NODE_REMOVED: &str = "removed";

pub const STATUS_PROPOSED: &str = "proposed";
pub const STATUS_IN_PROGRESS: &str = "in_progress";
pub const STATUS_DELIVERED: &str = "delivered";

/// The node levels from the top of the graph down to a component.
pub const LEVELS: [&str; 2] = [LEVEL_DOMAIN, LEVEL_COMPONENT];

/// Reports whether `level` is domain or component.
pub fn valid_level(level: &str) -> bool {
    LEVELS.contains(&level)
}

/// The level a node's parent has. A domain has none: it sits under the project.
pub fn parent_level(level: &str) -> Option<&'static str> {
    match level {
        LEVEL_COMPONENT => Some(LEVEL_DOMAIN),
        _ => None,
    }
}

/// Reports whether a node at `level` may sit under a parent at `parent_level`.
/// Only a component has a parent: a domain. With none, it sits loose under the
/// project.
pub fn allowed_parent(level: &str, parent: &str) -> bool {
    match parent_level(level) {
        Some(want) => want == parent,
        None => false,
    }
}

/// One Structure node. The Rules that hold for it live apart, with the node
/// among their contexts.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Node {
    pub id: String,
    pub project_id: String,
    pub parent_id: String,
    pub level: String,
    pub slug: String,
    pub path: String,
    pub title: String,
    pub intent: String,
    pub status: String,
    pub decisions: Vec<String>,
    pub position: i32,
    /// A `project_repos` name; empty is the project root checkout.
    pub repo: String,
    /// A directory inside that checkout (`src/auth`). `path` stays the node
    /// identity and is never a filesystem folder.
    pub folder: String,
    /// What a component is (see `COMPONENT_KINDS`); other levels have none.
    pub kind: String,
    /// The free-text technology (`Go`, `Postgres`), the canvas technology prop.
    pub tech: String,
    /// `migration_from` and `migration_to` describe a node moving between
    /// designs. Both empty means it is not migrating.
    pub migration_from: String,
    pub migration_to: String,
}

impl Node {
    /// Reports whether the node has a migration state.
    pub fn migrating(&self) -> bool {
        !self.migration_from.is_empty() || !self.migration_to.is_empty()
    }
}

/// A Work card reduced to what status derivation needs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Feature {
    pub slug: String,
    pub title: String,
    pub under: String,
    pub stage: String,
    pub open_questions: i32,
}

/// A node plus the status and counts derived from its descendants.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Row {
    pub node: Node,
    pub derived: String,
    pub child_count: usize,
    pub feature_count: usize,
    pub open_questions: i32,
}

/// Reports proposed, in_progress, or delivered. No features is proposed. Every
/// feature delivered is delivered.
pub fn derive_status(stages: &[&str]) -> &'static str {
    if stages.is_empty() {
        return STATUS_PROPOSED;
    }
    for stage in stages {
        if *stage != "delivered" {
            return STATUS_IN_PROGRESS;
        }
    }
    STATUS_DELIVERED
}

/// Attaches derived status and counts. Removed nodes are left out.
pub fn decorate(nodes: &[Node], features: &[Feature]) -> Vec<Row> {
    let active: Vec<&Node> = nodes
        .iter()
        .filter(|node| node.status != NODE_REMOVED)
        .collect();
    let mut children: HashMap<&str, Vec<&Node>> = HashMap::new();
    for node in &active {
        children
            .entry(node.parent_id.as_str())
            .or_default()
            .push(node);
    }
    let mut component_paths: HashMap<&str, HashSet<&str>> = HashMap::new();
    for node in &active {
        walk_paths(node, &children, &mut component_paths);
    }
    let mut rows = Vec::with_capacity(active.len());
    for node in &active {
        let paths = &component_paths[node.id.as_str()];
        let mut stages: Vec<&str> = Vec::new();
        let mut questions = 0;
        let mut count = 0;
        for feature in features {
            if feature.under.is_empty() || !paths.contains(feature.under.as_str()) {
                continue;
            }
            count += 1;
            stages.push(feature.stage.as_str());
            questions += feature.open_questions;
        }
        rows.push(Row {
            node: (*node).clone(),
            derived: derive_status(&stages).to_string(),
            child_count: children.get(node.id.as_str()).map_or(0, Vec::len),
            feature_count: count,
            open_questions: questions,
        });
    }
    rows
}

/// Collects the paths of the components at or under `node` into `memo`.
fn walk_paths<'a>(
    node: &'a Node,
    children: &HashMap<&'a str, Vec<&'a Node>>,
    memo: &mut HashMap<&'a str, HashSet<&'a str>>,
) {
    if memo.contains_key(node.id.as_str()) {
        return;
    }
    let mut paths: HashSet<&str> = HashSet::new();
    if node.level == LEVEL_COMPONENT {
        paths.insert(node.path.as_str());
    }
    if let Some(kids) = children.get(node.id.as_str()) {
        for child in kids {
            walk_paths(child, children, memo);
            if let Some(child_paths) = memo.get(child.id.as_str()) {
                paths.extend(child_paths.iter().copied());
            }
        }
    }
    memo.insert(node.id.as_str(), paths);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_status_of_stages() {
        assert_eq!(derive_status(&[]), STATUS_PROPOSED);
        assert_eq!(derive_status(&["delivered", "delivered"]), STATUS_DELIVERED);
        assert_eq!(derive_status(&["build"]), STATUS_IN_PROGRESS);
    }

    #[test]
    fn decorate_counts_descendant_features() {
        let nodes = [
            Node {
                id: "s".to_string(),
                path: "app".to_string(),
                level: LEVEL_DOMAIN.to_string(),
                title: "App".to_string(),
                status: NODE_ACTIVE.to_string(),
                ..Node::default()
            },
            Node {
                id: "d".to_string(),
                parent_id: "s".to_string(),
                path: "app/core".to_string(),
                level: LEVEL_DOMAIN.to_string(),
                title: "Core".to_string(),
                status: NODE_ACTIVE.to_string(),
                ..Node::default()
            },
            Node {
                id: "c".to_string(),
                parent_id: "d".to_string(),
                path: "app/core/import".to_string(),
                level: LEVEL_COMPONENT.to_string(),
                title: "Import".to_string(),
                status: NODE_ACTIVE.to_string(),
                ..Node::default()
            },
        ];
        let features = [Feature {
            slug: "git-import".to_string(),
            under: "app/core/import".to_string(),
            stage: "build".to_string(),
            open_questions: 2,
            ..Feature::default()
        }];
        let rows = decorate(&nodes, &features);
        assert_eq!(rows.len(), 3, "rows = {}", rows.len());
        let by_path: HashMap<&str, &Row> = rows
            .iter()
            .map(|row| (row.node.path.as_str(), row))
            .collect();
        let app = by_path["app"];
        assert!(
            app.feature_count == 1 && app.derived == STATUS_IN_PROGRESS && app.open_questions == 2,
            "system = {app:?}"
        );
        assert!(
            by_path["app/core/import"].child_count == 0 && by_path["app/core"].child_count == 1,
            "children import={} domain={}",
            by_path["app/core/import"].child_count,
            by_path["app/core"].child_count
        );
    }
}
