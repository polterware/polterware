//! The edges between canvas nodes, and the change entries a changed one raises.

use std::collections::{HashMap, HashSet};

use super::{Graph, REF_NODE};

/// An edge between two Structure nodes, read through the canvas nodes that cite
/// them. How the project's parts relate lives only here: a Structure node
/// declares no dependencies of its own.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Relation {
    /// Structure node id.
    pub from: String,
    /// Structure node id.
    pub to: String,
    pub kind: String,
    pub from_label: String,
    pub to_label: String,
}

impl Relation {
    fn key(&self) -> String {
        format!("{}\x00{}\x00{}", self.from, self.to, self.kind)
    }
}

fn cites_structure_node(node: &super::Node) -> bool {
    matches!(&node.r#ref, Some(reference) if reference.kind == REF_NODE && !reference.id.is_empty())
}

fn cited_id(node: &super::Node) -> &str {
    node.r#ref
        .as_ref()
        .map_or("", |reference| reference.id.as_str())
}

/// Lists the graph's edges whose two ends cite a Structure node, once each, in a
/// stable order.
pub fn relations(graph: &Graph) -> Vec<Relation> {
    let cites: HashMap<&str, &super::Node> = graph
        .nodes
        .iter()
        .filter(|node| cites_structure_node(node))
        .map(|node| (node.id.as_str(), node))
        .collect();
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<Relation> = Vec::new();
    for edge in &graph.edges {
        let (Some(from), Some(to)) = (cites.get(edge.from.as_str()), cites.get(edge.to.as_str()))
        else {
            continue;
        };
        let relation = Relation {
            from: cited_id(from).to_string(),
            to: cited_id(to).to_string(),
            kind: edge.kind.clone(),
            from_label: from.label.clone(),
            to_label: to.label.clone(),
        };
        if !seen.insert(relation.key()) {
            continue;
        }
        out.push(relation);
    }
    out.sort_by_key(|relation| relation.key());
    out
}

/// Compares the relations of two versions of a canvas.
pub fn relation_changes(before: &Graph, after: &Graph) -> (Vec<Relation>, Vec<Relation>) {
    let mut old: HashSet<String> = HashSet::new();
    for relation in relations(before) {
        old.insert(relation.key());
    }
    let mut next: HashSet<String> = HashSet::new();
    let mut added: Vec<Relation> = Vec::new();
    for relation in relations(after) {
        if !old.contains(&relation.key()) {
            added.push(relation.clone());
        }
        next.insert(relation.key());
    }
    let mut removed: Vec<Relation> = Vec::new();
    for relation in relations(before) {
        if !next.contains(&relation.key()) {
            removed.push(relation);
        }
    }
    (added, removed)
}

/// Names one changed relation ("Connect CLI → API (calls)") or counts several.
pub fn relation_summary(added: &[Relation], removed: &[Relation]) -> String {
    match (added.len(), removed.len()) {
        (1, 0) => format!(
            "Connect {} → {} ({})",
            added[0].from_label, added[0].to_label, added[0].kind
        ),
        (0, 1) => format!(
            "Disconnect {} → {} ({})",
            removed[0].from_label, removed[0].to_label, removed[0].kind
        ),
        _ => format!("Change {} relations", added.len() + removed.len()),
    }
}

/// The Structure nodes a set of changed relations touch.
pub fn relation_subjects(added: &[Relation], removed: &[Relation]) -> Vec<String> {
    let mut seen: HashSet<&str> = HashSet::new();
    let mut out: Vec<String> = Vec::new();
    for relation in added.iter().chain(removed.iter()) {
        for id in [&relation.from, &relation.to] {
            if seen.insert(id.as_str()) {
                out.push(id.clone());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::canvas::{
        EDGE_CALLS, EDGE_RELATES, Edge, KIND_COMPONENT, KIND_NOTE, Node, REF_NODE, Ref,
    };

    fn relation_node(id: &str, label: &str, cited: &str) -> Node {
        Node {
            id: id.to_string(),
            kind: KIND_COMPONENT.to_string(),
            label: label.to_string(),
            r#ref: Some(Ref {
                kind: REF_NODE.to_string(),
                id: cited.to_string(),
            }),
            ..Node::default()
        }
    }

    fn relation_graph(edges: Vec<Edge>) -> Graph {
        Graph {
            nodes: vec![
                relation_node("a", "CLI", "cli"),
                relation_node("b", "API", "api"),
                Node {
                    id: "n".to_string(),
                    kind: KIND_NOTE.to_string(),
                    label: "idea".to_string(),
                    ..Node::default()
                },
            ],
            edges,
        }
    }

    fn edge(id: &str, from: &str, to: &str, kind: &str) -> Edge {
        Edge {
            id: id.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            kind: kind.to_string(),
            ..Edge::default()
        }
    }

    #[test]
    fn test_relation_changes_list_only_edges_between_structure_nodes() {
        let before = relation_graph(vec![edge("e0", "n", "b", EDGE_RELATES)]);
        let after = relation_graph(vec![
            edge("e0", "n", "b", EDGE_RELATES),
            edge("e1", "a", "b", EDGE_CALLS),
        ]);
        let (added, removed) = relation_changes(&before, &after);
        assert_eq!(
            (added.len(), removed.len()),
            (1, 0),
            "added {added:?} removed {removed:?}"
        );
        assert_eq!(
            relation_summary(&added, &removed),
            "Connect CLI → API (calls)"
        );
        assert_eq!(relation_subjects(&added, &removed), vec!["cli", "api"]);
        let (added, removed) = relation_changes(&after, &before);
        assert_eq!(
            relation_summary(&added, &removed),
            "Disconnect CLI → API (calls)"
        );
    }

    #[test]
    fn test_relation_changes_ignore_moves_and_redrawn_edges() {
        let before = relation_graph(vec![edge("e1", "a", "b", EDGE_CALLS)]);
        // The same relation drawn again under a new edge id is no change.
        let after = relation_graph(vec![edge("e9", "a", "b", EDGE_CALLS)]);
        let (added, removed) = relation_changes(&before, &after);
        assert_eq!(
            added.len() + removed.len(),
            0,
            "added {added:?} removed {removed:?}"
        );
    }

    #[test]
    fn relation_summary_counts_several_changes() {
        let before = relation_graph(vec![]);
        let after = relation_graph(vec![
            edge("e1", "a", "b", EDGE_CALLS),
            edge("e2", "b", "a", EDGE_RELATES),
        ]);
        let (added, removed) = relation_changes(&before, &after);
        assert_eq!(added.len(), 2);
        assert_eq!(relation_summary(&added, &removed), "Change 2 relations");
        assert_eq!(relation_subjects(&added, &removed), vec!["api", "cli"]);
    }
}
