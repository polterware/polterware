//! Port of `backend/domains/polter/scopes.go`: the `scope:` value shared by
//! Rules, Decisions, Facts, and derived items.

use std::collections::HashMap;

use crate::domains::structure;

use super::model::go_quote;
use super::plan::Planner;

/// Reads a `scope:` value: Structure node paths separated by commas.
///
/// Blanks and duplicates are dropped and the list is sorted, so two writes of
/// the same contexts compare equal. An empty value is an empty list (the whole
/// project).
pub fn parse_scopes(value: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for part in value.split(',') {
        let part = part.trim();
        if part.is_empty() || out.iter().any(|scope| scope == part) {
            continue;
        }
        out.push(part.to_string());
    }
    out.sort();
    out
}

/// Reports whether `path` names an active Structure node.
pub(crate) fn scope_is_active(nodes: &HashMap<String, structure::Node>, path: &str) -> bool {
    nodes
        .get(path)
        .is_some_and(|node| node.status != structure::NODE_REMOVED)
}

impl Planner {
    /// Reports each path that is not an active Structure node as an error on
    /// `line`, and answers whether every path held.
    pub(crate) fn check_scopes(&mut self, line: i32, paths: &[String]) -> bool {
        let mut ok = true;
        for path in paths {
            if !scope_is_active(&self.nodes, path) {
                self.errorf(
                    line,
                    format!(
                        "unknown scope {}: use a Structure node path",
                        go_quote(path)
                    ),
                );
                ok = false;
            }
        }
        ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scopes_sorted_and_deduplicated() {
        assert_eq!(
            parse_scopes(" core, , core/import, core "),
            vec!["core".to_string(), "core/import".to_string()],
            "blanks and duplicates are dropped and the list is sorted"
        );
        assert!(parse_scopes("").is_empty(), "empty is the whole project");
        assert!(parse_scopes("  ").is_empty(), "blanks are empty too");
    }

    #[test]
    fn an_unknown_or_removed_scope_is_not_active() {
        let mut nodes: HashMap<String, structure::Node> = HashMap::new();
        nodes.insert(
            "core".to_string(),
            structure::Node {
                path: "core".into(),
                status: structure::NODE_ACTIVE.into(),
                ..structure::Node::default()
            },
        );
        nodes.insert(
            "old".to_string(),
            structure::Node {
                path: "old".into(),
                status: structure::NODE_REMOVED.into(),
                ..structure::Node::default()
            },
        );
        assert!(scope_is_active(&nodes, "core"));
        assert!(!scope_is_active(&nodes, "old"), "a removed node");
        assert!(!scope_is_active(&nodes, "missing"), "an unknown node");
    }
}
