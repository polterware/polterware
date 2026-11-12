//! Resolving a Structure path and the nodes under it.

use std::collections::{HashMap, HashSet};

use super::{NODE_REMOVED, Node};

/// A global filter over the project: one Structure node, or the whole project
/// when `node_id` is empty. `exact` drops the ancestors from the match.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scope {
    pub node_id: String,
    pub exact: bool,
}

impl Scope {
    /// Reports whether the scope narrows anything.
    pub fn active(&self) -> bool {
        !self.node_id.is_empty()
    }
}

/// A global filter over several Structure nodes. The match is the union of each
/// node's `Scope`. `exact` applies to every node. An empty `node_ids` is the
/// whole project.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MultiScope {
    pub node_ids: Vec<String>,
    pub exact: bool,
}

impl MultiScope {
    /// Reports whether the scope narrows anything.
    pub fn active(&self) -> bool {
        self.node_ids.iter().any(|id| !id.is_empty())
    }
}

/// The node IDs a scope matches. The default set matches everything; an empty
/// but resolved set matches only unscoped entities.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScopeSet {
    nodes: Option<HashMap<String, bool>>,
}

impl ScopeSet {
    /// A resolved set with no matches, which matches only unscoped entities.
    pub fn empty() -> Self {
        Self {
            nodes: Some(HashMap::new()),
        }
    }

    /// Reports whether an entity scoped to `node_id` is visible. An entity with
    /// no scope belongs to the whole project and is visible under any filter.
    pub fn matches(&self, node_id: &str) -> bool {
        match &self.nodes {
            None => true,
            Some(_) if node_id.is_empty() => true,
            Some(nodes) => nodes.get(node_id).copied().unwrap_or(false),
        }
    }

    /// Reports whether an entity with these contexts is inside the set: any one
    /// of them matches, and an entity with none belongs to the whole project.
    pub fn matches_any(&self, node_ids: &[String]) -> bool {
        if self.nodes.is_none() || node_ids.is_empty() {
            return true;
        }
        node_ids.iter().any(|id| self.matches(id))
    }

    /// Reports whether the set is the nil set, which matches everything.
    pub fn is_nil(&self) -> bool {
        self.nodes.is_none()
    }

    pub fn len(&self) -> usize {
        self.nodes.as_ref().map_or(0, HashMap::len)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains(&self, node_id: &str) -> bool {
        self.nodes
            .as_ref()
            .is_some_and(|nodes| nodes.get(node_id).copied().unwrap_or(false))
    }

    pub fn insert(&mut self, node_id: String) {
        self.nodes
            .get_or_insert_with(HashMap::new)
            .insert(node_id, true);
    }

    pub fn ids(&self) -> impl Iterator<Item = &String> {
        self.nodes.iter().flat_map(|nodes| nodes.keys())
    }
}

/// Returns the node itself and its descendants, plus its ancestors unless the
/// scope is exact. Removed nodes are skipped. An unknown node yields an empty
/// set that matches only unscoped entities.
pub fn resolve_scope(nodes: &[Node], scope: &Scope) -> ScopeSet {
    if !scope.active() {
        return ScopeSet::default();
    }
    let mut by_id: HashMap<&str, &Node> = HashMap::new();
    let mut children: HashMap<&str, Vec<&str>> = HashMap::new();
    for node in nodes {
        if node.status == NODE_REMOVED {
            continue;
        }
        by_id.insert(node.id.as_str(), node);
        children
            .entry(node.parent_id.as_str())
            .or_default()
            .push(node.id.as_str());
    }
    let mut set = ScopeSet::empty();
    let Some(root) = by_id.get(scope.node_id.as_str()) else {
        return set;
    };
    walk_down(&scope.node_id, &children, &mut set);
    if scope.exact {
        return set;
    }
    let mut parent = root.parent_id.as_str();
    while !parent.is_empty() {
        let Some(node) = by_id.get(parent) else {
            break;
        };
        set.insert(parent.to_string());
        parent = node.parent_id.as_str();
    }
    set
}

fn walk_down(id: &str, children: &HashMap<&str, Vec<&str>>, set: &mut ScopeSet) {
    if set.contains(id) {
        return;
    }
    set.insert(id.to_string());
    if let Some(kids) = children.get(id) {
        for child in kids {
            walk_down(child, children, set);
        }
    }
}

/// Returns the union of `resolve_scope` for each node. An empty scope matches
/// everything (the default set). A node that is unknown or removed contributes
/// nothing; when every node is skipped the set is empty and matches only
/// unscoped entities.
pub fn resolve_scopes(nodes: &[Node], scope: &MultiScope) -> ScopeSet {
    if !scope.active() {
        return ScopeSet::default();
    }
    let mut set = ScopeSet::empty();
    for id in &scope.node_ids {
        if id.is_empty() {
            continue;
        }
        let resolved = resolve_scope(
            nodes,
            &Scope {
                node_id: id.clone(),
                exact: scope.exact,
            },
        );
        for node_id in resolved.ids() {
            set.insert(node_id.clone());
        }
    }
    set
}

/// The set keyed by node path, for Work cards whose `under` names a node path.
pub fn path_scope_set(nodes: &[Node], set: &ScopeSet) -> Option<HashSet<String>> {
    if set.is_nil() {
        return None;
    }
    let mut paths = HashSet::new();
    for node in nodes {
        if set.contains(&node.id) {
            paths.insert(node.path.clone());
        }
    }
    Some(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::structure::NODE_ACTIVE;

    fn scope_nodes() -> Vec<Node> {
        let node = |id: &str, parent: &str, path: &str, status: &str| Node {
            id: id.to_string(),
            parent_id: parent.to_string(),
            path: path.to_string(),
            status: status.to_string(),
            ..Node::default()
        };
        vec![
            node("root", "", "app", NODE_ACTIVE),
            node("auth", "root", "app/auth", NODE_ACTIVE),
            node("login", "auth", "app/auth/login", NODE_ACTIVE),
            node("gone", "auth", "app/auth/gone", NODE_REMOVED),
            node("orphan", "gone", "app/auth/gone/orphan", NODE_ACTIVE),
            node("billing", "root", "app/billing", NODE_ACTIVE),
            node("invoices", "billing", "app/billing/invoices", NODE_ACTIVE),
        ]
    }

    fn has(set: &ScopeSet, ids: &[&str]) -> bool {
        ids.iter().all(|id| set.contains(id))
    }

    #[test]
    fn resolve_scopes_union() {
        let nodes = scope_nodes();
        let set = resolve_scopes(
            &nodes,
            &MultiScope {
                node_ids: vec!["login".to_string(), "billing".to_string()],
                exact: false,
            },
        );
        assert!(
            has(&set, &["login", "auth", "root", "billing", "invoices"]),
            "union = {set:?}"
        );
        assert!(
            !set.contains("gone") && !set.contains("orphan"),
            "removed subtree leaked: {set:?}"
        );
        let single = resolve_scope(
            &nodes,
            &Scope {
                node_id: "login".to_string(),
                exact: false,
            },
        );
        assert!(
            has(&single, &["login", "auth", "root"])
                && !single.contains("billing")
                && !single.contains("invoices"),
            "single = {single:?}"
        );
    }

    #[test]
    fn resolve_scopes_parent_includes_children() {
        let set = resolve_scopes(
            &scope_nodes(),
            &MultiScope {
                node_ids: vec!["auth".to_string()],
                exact: false,
            },
        );
        assert!(
            has(&set, &["auth", "login", "root"])
                && !set.contains("gone")
                && !set.contains("orphan")
                && !set.contains("billing"),
            "parent = {set:?}"
        );
    }

    #[test]
    fn resolve_scopes_exact_drops_ancestors() {
        let set = resolve_scopes(
            &scope_nodes(),
            &MultiScope {
                node_ids: vec!["auth".to_string(), "billing".to_string()],
                exact: true,
            },
        );
        assert!(
            has(&set, &["auth", "login", "billing", "invoices"])
                && !set.contains("root")
                && !set.contains("gone")
                && !set.contains("orphan"),
            "exact = {set:?}"
        );
    }

    #[test]
    fn resolve_scopes_skips_removed_and_unknown() {
        let nodes = scope_nodes();
        let only_removed = resolve_scopes(
            &nodes,
            &MultiScope {
                node_ids: vec!["gone".to_string()],
                exact: false,
            },
        );
        assert!(
            only_removed.is_empty() && !only_removed.is_nil(),
            "removed node = {only_removed:?}"
        );
        let unknown = resolve_scopes(
            &nodes,
            &MultiScope {
                node_ids: vec!["missing".to_string()],
                exact: false,
            },
        );
        assert!(
            unknown.is_empty() && !unknown.is_nil(),
            "unknown node = {unknown:?}"
        );
        let mixed = resolve_scopes(
            &nodes,
            &MultiScope {
                node_ids: vec!["gone".to_string(), "invoices".to_string(), String::new()],
                exact: false,
            },
        );
        assert!(
            has(&mixed, &["invoices", "billing", "root"])
                && !mixed.contains("gone")
                && !mixed.contains("auth")
                && mixed.len() == 3,
            "mixed = {mixed:?}"
        );
    }

    #[test]
    fn resolve_scopes_inactive_matches_everything() {
        assert!(
            resolve_scopes(&scope_nodes(), &MultiScope::default()).is_nil(),
            "empty scope must match everything"
        );
        assert!(
            resolve_scopes(
                &scope_nodes(),
                &MultiScope {
                    node_ids: vec![String::new(), String::new()],
                    exact: false,
                }
            )
            .is_nil(),
            "blank nodes must match everything"
        );
    }

    #[test]
    fn scope_set_matches_unscoped_always() {
        let open = ScopeSet::default();
        assert!(
            open.matches("auth") && open.matches(""),
            "the nil set must match every entity"
        );
        let empty = ScopeSet::empty();
        assert!(
            empty.matches("") && !empty.matches("auth"),
            "the empty set matches only unscoped entities"
        );
        let filtered = resolve_scopes(
            &scope_nodes(),
            &MultiScope {
                node_ids: vec!["auth".to_string()],
                exact: true,
            },
        );
        assert!(
            filtered.matches("") && filtered.matches("login") && !filtered.matches("billing"),
            "matches = login:{} billing:{} unscoped:{}",
            filtered.matches("login"),
            filtered.matches("billing"),
            filtered.matches("")
        );
    }
}
