//! Port of `backend/domains/polter/structure_encode.go`: the plain-text
//! Structure graph an agent reads.

use crate::domains::{structure, work};

use super::model::go_quote;

/// Renders the graph, or one node and its children, as plain text.
pub fn encode_structure(
    project_name: &str,
    rows: &[structure::Row],
    cards: &[work::Card],
    rules: &[work::Rule],
    focus: &str,
) -> String {
    let project_name = if project_name.is_empty() {
        "project"
    } else {
        project_name
    };
    let mut out = String::new();
    if !focus.is_empty() {
        let row = rows
            .iter()
            .find(|row| row.node.path == focus)
            .or_else(|| rows.iter().find(|row| row.node.id == focus));
        let Some(row) = row else {
            return format!(
                "unknown node {}: list the graph with polterware structure\n",
                go_quote(focus)
            );
        };
        write_structure_row(&mut out, project_name, row, rows, cards, rules);
        return out;
    }
    out.push_str(&format!("{project_name}\n"));
    let mut roots = 0;
    for row in rows {
        if !row.node.parent_id.is_empty() {
            continue;
        }
        roots += 1;
        out.push_str(&format!(
            "{}  {}  {}{}  {}\n",
            row.node.path,
            row.node.level,
            status_label(&row.derived),
            migration_suffix(&row.node),
            go_quote(&row.node.title)
        ));
    }
    if roots == 0 {
        out.push_str("No Structure nodes.\n");
    }
    out
}

fn write_structure_row(
    out: &mut String,
    project_name: &str,
    row: &structure::Row,
    rows: &[structure::Row],
    cards: &[work::Card],
    rules: &[work::Rule],
) {
    out.push_str(&format!(
        "{}\n",
        structure_breadcrumb(project_name, row, rows)
    ));
    out.push_str(&format!(
        "{}  {}  {}\n",
        row.node.path,
        row.node.level,
        status_label(&row.derived)
    ));
    let line = kind_tech_line(&row.node);
    if !line.is_empty() {
        out.push_str(&format!("{line}\n"));
    }
    if !row.node.intent.is_empty() {
        out.push_str(&format!("Intent: {}\n", row.node.intent));
    }
    let mut wrote_rules = false;
    for rule in rules {
        if !rule.scopes.iter().any(|scope| scope == &row.node.path) {
            continue;
        }
        if !wrote_rules {
            out.push_str("Rules:\n");
            wrote_rules = true;
        }
        out.push_str(&format!("- {}\n", rule.statement));
    }
    if !row.node.decisions.is_empty() {
        out.push_str(&format!("Decisions: {}\n", row.node.decisions.join(", ")));
    }
    if row.node.migrating() {
        out.push_str(&format!(
            "Migrating: {} -> {}\n",
            row.node.migration_from, row.node.migration_to
        ));
    }
    let checkout = checkout_line(&row.node);
    if !checkout.is_empty() {
        out.push_str(&format!("Checkout: {checkout}\n"));
    }
    out.push_str(&format!(
        "Reality: {} features · {} open questions\n",
        row.feature_count, row.open_questions
    ));
    out.push_str("Children:\n");
    let mut children = 0;
    for child in rows {
        if child.node.parent_id != row.node.id {
            continue;
        }
        children += 1;
        out.push_str(&format!(
            "  {}  {}  {}  {}\n",
            child.node.path,
            child.node.level,
            status_label(&child.derived),
            go_quote(&child.node.title)
        ));
    }
    for card in cards {
        if card.status != work::CARD_ACTIVE || card.under != row.node.path {
            continue;
        }
        children += 1;
        out.push_str(&format!(
            "  {}  {}  {}\n",
            card.slug,
            card.stage,
            go_quote(&card.title)
        ));
    }
    if children == 0 {
        out.push_str("  (none)\n");
    }
}

fn structure_breadcrumb(
    project_name: &str,
    row: &structure::Row,
    rows: &[structure::Row],
) -> String {
    let mut parts: Vec<&str> = Vec::new();
    let mut current = row;
    loop {
        parts.push(&current.node.title);
        let Some(parent) = rows
            .iter()
            .find(|item| item.node.id == current.node.parent_id)
        else {
            break;
        };
        current = parent;
    }
    parts.reverse();
    format!("{project_name} › {}", parts.join(" › "))
}

fn status_label(status: &str) -> &'static str {
    match status {
        "" => "n/a",
        structure::STATUS_IN_PROGRESS => "in progress",
        structure::STATUS_DELIVERED => "delivered",
        _ => "proposed",
    }
}

fn migration_suffix(node: &structure::Node) -> String {
    if !node.migrating() {
        return String::new();
    }
    format!(
        "  migrating {} -> {}",
        node.migration_from, node.migration_to
    )
}

/// `kind: database · tech: Postgres`, or empty when the node has neither.
fn kind_tech_line(node: &structure::Node) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !node.kind.is_empty() {
        parts.push(format!("kind: {}", node.kind));
    }
    if !node.tech.is_empty() {
        parts.push(format!("tech: {}", node.tech));
    }
    parts.join(" · ")
}

/// `repo: api · path: src/auth`, or empty when the node sits at the root of the
/// project checkout.
fn checkout_line(node: &structure::Node) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !node.repo.is_empty() {
        parts.push(format!("repo: {}", node.repo));
    }
    if !node.folder.is_empty() {
        parts.push(format!("path: {}", node.folder));
    }
    parts.join(" · ")
}

/// The node's ancestors, nearest first, stopping at a removed or missing
/// parent.
pub fn ancestor_chain(nodes: &[structure::Node], node: &structure::Node) -> Vec<structure::Node> {
    let mut chain: Vec<structure::Node> = Vec::new();
    let mut current = node.clone();
    while !current.parent_id.is_empty() {
        let Some(parent) = nodes.iter().find(|item| item.id == current.parent_id) else {
            break;
        };
        if parent.status == structure::NODE_REMOVED {
            break;
        }
        chain.push(parent.clone());
        current = parent.clone();
    }
    chain
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, parent: &str, path: &str, level: &str, title: &str) -> structure::Node {
        structure::Node {
            id: id.into(),
            parent_id: parent.into(),
            path: path.into(),
            level: level.into(),
            title: title.into(),
            status: structure::NODE_ACTIVE.into(),
            ..structure::Node::default()
        }
    }

    fn rows() -> Vec<structure::Row> {
        vec![
            structure::Row {
                node: node("data", "", "data", structure::LEVEL_DOMAIN, "Data"),
                derived: structure::STATUS_IN_PROGRESS.into(),
                ..structure::Row::default()
            },
            structure::Row {
                node: structure::Node {
                    kind: "database".into(),
                    tech: "Postgres".into(),
                    ..node("db", "data", "data/db", structure::LEVEL_COMPONENT, "DB")
                },
                derived: structure::STATUS_PROPOSED.into(),
                ..structure::Row::default()
            },
        ]
    }

    #[test]
    fn shows_kind_and_tech() {
        let out = encode_structure("demo", &rows(), &[], &[], "data/db");
        assert!(
            out.contains("kind: database · tech: Postgres\n"),
            "out = {out}"
        );
        let out = encode_structure("demo", &rows(), &[], &[], "data");
        assert!(!out.contains("kind:"), "a node without kind prints no line");
    }

    #[test]
    fn renders_the_whole_graph() {
        let out = encode_structure("demo", &rows(), &[], &[], "");
        assert!(out.starts_with("demo\n"), "{out}");
        assert!(
            out.contains("data  domain  in progress  \"Data\"\n"),
            "{out}"
        );
        assert!(!out.contains("No Structure nodes"), "{out}");
        assert!(encode_structure("", &[], &[], &[], "").contains("project"));
        assert!(encode_structure("demo", &[], &[], &[], "").contains("No Structure nodes."));

        let card = work::Card {
            slug: "import".into(),
            title: "Import".into(),
            stage: work::STAGE_BUILD.into(),
            status: work::CARD_ACTIVE.into(),
            under: "data/db".into(),
            ..work::Card::default()
        };
        let rule = work::Rule {
            id: "r1".into(),
            statement: "No network calls".into(),
            scopes: vec!["data/db".into()],
            ..work::Rule::default()
        };
        let out = encode_structure("demo", &rows(), &[card], &[rule], "data/db");
        assert!(out.contains("demo › Data › DB\n"), "{out}");
        assert!(out.contains("Rules:\n- No network calls\n"), "{out}");
        assert!(
            out.contains("Reality: 0 features · 0 open questions\n"),
            "{out}"
        );
        assert!(out.contains("  import  build  \"Import\"\n"), "{out}");
    }

    #[test]
    fn names_an_unknown_focus() {
        let out = encode_structure("demo", &rows(), &[], &[], "nope");
        assert_eq!(
            out,
            "unknown node \"nope\": list the graph with polterware structure\n"
        );
    }

    #[test]
    fn walks_the_ancestor_chain() {
        let nodes = vec![
            node("s", "", "app", structure::LEVEL_DOMAIN, "App"),
            node("d", "s", "app/core", structure::LEVEL_DOMAIN, "Core"),
            node("c", "d", "app/core/api", structure::LEVEL_COMPONENT, "API"),
        ];
        let chain = ancestor_chain(&nodes, &nodes[2]);
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].path, "app/core", "nearest first");
        assert_eq!(chain[1].path, "app", "then the root");
    }
}
