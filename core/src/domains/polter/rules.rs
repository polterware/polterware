//! Port of `backend/domains/polter/rules.go`: the Rule planner, a node's `!`
//! lines, and the Rules that hold for a path.
//!
//! `plan_rules` and `node_rules` are planner steps: they update the planner's
//! live Rules and Structure nodes by path, and append their ops and
//! diagnostics.

use serde_json::json;

use crate::domains::{structure, work};
use crate::platform::ids;

use super::model::{Document, StructureNode, go_quote};
use super::plan::Planner;
use super::scopes::parse_scopes;

/// Writes the Rule blocks: a new slug creates a Rule (it needs a statement), a
/// known slug or id updates it, and `status: removed` retires it.
pub fn plan_rules(p: &mut Planner, doc: &Document) {
    for node in &doc.rules {
        let mut already_failed = p.failed(node.line);
        for entry in &node.meta {
            already_failed = already_failed || p.failed(entry.line);
        }
        if already_failed {
            continue;
        }
        let index = p
            .rules
            .iter()
            .position(|rule| rule.id == node.reference || rule.slug == node.reference);
        let status = node.meta_value("status").map(str::to_string);
        let mut scopes: Option<Vec<String>> = None;
        if let Some(value) = node.meta_value("scope") {
            let parsed = parse_scopes(value);
            if !p.check_scopes(node.meta_line("scope"), parsed.as_slice()) {
                continue;
            }
            scopes = Some(parsed);
        }
        let Some(index) = index else {
            if status.as_deref() == Some(work::RULE_REMOVED) {
                p.errorf(
                    node.line,
                    format!(
                        "cannot remove unknown rule {}: check the slug",
                        go_quote(&node.reference)
                    ),
                );
                continue;
            }
            if node.statement.is_empty() {
                p.errorf(
                    node.line,
                    format!(
                        "unknown rule {}: add a statement (#### rule {}: Statement)",
                        go_quote(&node.reference),
                        node.reference
                    ),
                );
                continue;
            }
            let rule = work::Rule {
                id: ids::new_ulid(),
                slug: node.reference.clone(),
                statement: node.statement.clone(),
                scopes: scopes.clone().unwrap_or_default(),
                status: work::RULE_ACTIVE.to_string(),
                ..work::Rule::default()
            };
            let mut fields = work::Fields::new();
            fields.insert("slug".into(), json!(rule.slug));
            fields.insert("statement".into(), json!(rule.statement));
            if let Some(scopes) = &scopes {
                fields.insert("scope".into(), json!(scopes));
            }
            p.ops.push(work::Op {
                entity: work::ENTITY_RULE.into(),
                kind: work::OP_CREATE.into(),
                entity_id: rule.id.clone(),
                card_slug: rule.slug.clone(),
                label: rule.statement.clone(),
                fields,
                ..work::Op::default()
            });
            p.rules.push(rule);
            continue;
        };
        let mut current = p.rules[index].clone();
        let mut fields = work::Fields::new();
        let mut before = work::Fields::new();
        if !node.statement.is_empty() && node.statement != current.statement {
            fields.insert("statement".into(), json!(node.statement));
            before.insert("statement".into(), json!(current.statement));
            current.statement = node.statement.clone();
        }
        if let Some(scopes) = &scopes
            && *scopes != current.scopes
        {
            fields.insert("scope".into(), json!(scopes));
            before.insert("scope".into(), json!(current.scopes));
            current.scopes = scopes.clone();
        }
        let mut kind = work::OP_UPDATE;
        if status.as_deref() == Some(work::RULE_REMOVED) {
            kind = work::OP_REMOVE;
            fields.insert("status".into(), json!(work::RULE_REMOVED));
            before.insert("status".into(), json!(current.status));
            current.status = work::RULE_REMOVED.to_string();
        }
        if fields.is_empty() {
            continue;
        }
        p.ops.push(work::Op {
            entity: work::ENTITY_RULE.into(),
            kind: kind.into(),
            entity_id: current.id.clone(),
            card_slug: current.handle().to_string(),
            label: current.statement.clone(),
            fields,
            before,
            ..work::Op::default()
        });
        p.rules[index] = current;
    }
}

/// Plans a node's `!` lines: each is a Rule whose only context is that node,
/// matched by its normalized text. `[removed]` retires it.
pub fn node_rules(p: &mut Planner, node: &StructureNode, stored: &structure::Node) {
    for invariant in &node.invariants {
        let key = work::normalize_text(&invariant.text);
        let index = p.rules.iter().position(|rule| {
            rule.scopes.len() == 1
                && rule.scopes[0] == stored.path
                && work::normalize_text(&rule.statement) == key
        });
        let Some(index) = index else {
            if invariant.removed {
                continue;
            }
            let rule = work::Rule {
                id: ids::new_ulid(),
                statement: invariant.text.clone(),
                scopes: vec![stored.path.clone()],
                status: work::RULE_ACTIVE.to_string(),
                ..work::Rule::default()
            };
            let mut fields = work::Fields::new();
            fields.insert("statement".into(), json!(invariant.text));
            fields.insert("scope".into(), json!([stored.path]));
            p.ops.push(work::Op {
                entity: work::ENTITY_RULE.into(),
                kind: work::OP_CREATE.into(),
                entity_id: rule.id.clone(),
                node_id: stored.id.clone(),
                card_slug: stored.path.clone(),
                label: invariant.text.clone(),
                fields,
                ..work::Op::default()
            });
            p.rules.push(rule);
            continue;
        };
        let current = p.rules[index].clone();
        if invariant.removed {
            p.rules.remove(index);
            let mut fields = work::Fields::new();
            fields.insert("status".into(), json!(work::RULE_REMOVED));
            let mut before = work::Fields::new();
            before.insert("status".into(), json!(current.status));
            p.ops.push(work::Op {
                entity: work::ENTITY_RULE.into(),
                kind: work::OP_REMOVE.into(),
                entity_id: current.id.clone(),
                node_id: stored.id.clone(),
                card_slug: stored.path.clone(),
                label: current.statement.clone(),
                fields,
                before,
                ..work::Op::default()
            });
            continue;
        }
        if invariant.text != current.statement {
            let mut fields = work::Fields::new();
            fields.insert("statement".into(), json!(invariant.text));
            let mut before = work::Fields::new();
            before.insert("statement".into(), json!(current.statement));
            p.ops.push(work::Op {
                entity: work::ENTITY_RULE.into(),
                kind: work::OP_UPDATE.into(),
                entity_id: current.id.clone(),
                node_id: stored.id.clone(),
                card_slug: stored.path.clone(),
                label: invariant.text.clone(),
                fields,
                before,
                ..work::Op::default()
            });
            p.rules[index].statement = invariant.text.clone();
        }
    }
}

/// Lists Rules, one line each, optionally only those in `status`. A header
/// counts every status so the whole lifecycle is visible.
pub fn encode_rules(rules: &[work::Rule], status: &str) -> String {
    let mut counts: std::collections::HashMap<&str, i32> = std::collections::HashMap::new();
    for rule in rules {
        *counts.entry(rule.status.as_str()).or_insert(0) += 1;
    }
    let header: Vec<String> = work::RULE_STATUSES
        .iter()
        .map(|state| format!("{state} {}", counts.get(state).copied().unwrap_or(0)))
        .collect();
    let mut out = format!("rules: {}\n", header.join(" · "));
    let mut shown = 0;
    for rule in rules {
        if !status.is_empty() && rule.status != status {
            continue;
        }
        shown += 1;
        let mut line = format!("- {} · {} · {}", rule.handle(), rule.status, rule.statement);
        if rule.scopes.is_empty() {
            line.push_str(" · whole project");
        } else {
            line.push_str(&format!(" · scope: {}", rule.scopes.join(", ")));
        }
        out.push_str(&line);
        out.push('\n');
    }
    if shown == 0 {
        out.push_str("(none)\n");
    }
    out
}

/// Returns the paths of active Structure nodes whose `decisions:` cite `rule`.
pub fn nodes_citing_rule(nodes: &[structure::Node], rule: &work::Rule) -> Vec<String> {
    let mut paths = Vec::new();
    for node in nodes {
        if node.status == structure::NODE_REMOVED {
            continue;
        }
        if node
            .decisions
            .iter()
            .any(|reference| super::lint::cites_decision(reference, &rule.id, &rule.statement))
        {
            paths.push(node.path.clone());
        }
    }
    paths
}

/// Renders one Rule as text: its statement, status, scope, why, successor, and
/// the Structure nodes that cite it.
pub fn encode_rule(rule: &work::Rule, all: &[work::Rule], node_paths: &[String]) -> String {
    let mut out = format!(
        "rule {} [{}] {}\n",
        rule.handle(),
        rule.status,
        rule.statement
    );
    if rule.scopes.is_empty() {
        out.push_str("scope: whole project\n");
    } else {
        out.push_str(&format!("scope: {}\n", rule.scopes.join(", ")));
    }
    if !rule.replaced_by.is_empty() {
        let handle = all
            .iter()
            .find(|candidate| candidate.id == rule.replaced_by)
            .map_or(rule.replaced_by.as_str(), work::Rule::handle);
        out.push_str(&format!("replaced by: {handle}\n"));
    }
    if !rule.why.is_empty() {
        out.push_str(&format!("\n{}\n", rule.why.trim_end_matches('\n')));
    }
    if !node_paths.is_empty() {
        out.push_str(&format!("\nholds for: {}\n", node_paths.join(", ")));
    }
    out
}

/// Returns the active Rules that hold for a node path: its own, its ancestors'
/// (nearest first), then the project-wide ones, each once and in stored order
/// within a level.
pub fn rules_for(
    rules: &[work::Rule],
    chain: &[String],
) -> (Vec<Vec<work::Rule>>, Vec<work::Rule>) {
    let mut seen: Vec<&str> = Vec::new();
    let mut own: Vec<Vec<work::Rule>> = vec![Vec::new(); chain.len()];
    for (i, path) in chain.iter().enumerate() {
        for rule in rules {
            if rule.binding() && !seen.contains(&rule.id.as_str()) && rule.scopes.contains(path) {
                seen.push(rule.id.as_str());
                own[i].push(rule.clone());
            }
        }
    }
    let project = rules
        .iter()
        .filter(|rule| rule.binding() && rule.scopes.is_empty())
        .cloned()
        .collect();
    (own, project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::polter::decode::decode;
    use crate::domains::polter::encode::HEADER;
    use crate::domains::polter::model::{Diagnostic, has_errors};

    fn rule_snapshot() -> work::Snapshot {
        work::Snapshot {
            project_id: "p".into(),
            nodes: vec![structure::Node {
                id: "n".into(),
                slug: "core".into(),
                path: "core".into(),
                title: "Core".into(),
                level: structure::LEVEL_DOMAIN.into(),
                status: structure::NODE_ACTIVE.into(),
                ..structure::Node::default()
            }],
            ..work::Snapshot::default()
        }
    }

    fn plan_text(
        snapshot: &work::Snapshot,
        text: &str,
    ) -> (Vec<work::Op>, Vec<Diagnostic>, Vec<work::Rule>) {
        let (doc, diags) = decode(text);
        assert!(!has_errors(&diags), "decode: {diags:?}");
        let mut planner = Planner::new(snapshot, "");
        planner.diags = diags;
        plan_rules(&mut planner, &doc);
        (planner.ops, planner.diags, planner.rules)
    }

    #[test]
    fn rule_plan_create_update_remove_and_node_lines() {
        let snapshot = rule_snapshot();
        let (ops, diags, _) = plan_text(
            &snapshot,
            &(HEADER.to_string()
                + "#### rule no-network: Never call the network.\nscope: core\n#### rule small-prs: Keep changes small.\n"),
        );
        assert!(
            !has_errors(&diags) && ops.len() == 2,
            "ops = {ops:?} diags = {diags:?}"
        );
        assert_eq!(ops[0].entity, work::ENTITY_RULE, "entity");
        assert_eq!(ops[0].kind, work::OP_CREATE, "kind");
        assert_eq!(ops[0].fields["scope"], json!(["core"]), "scope");
        assert!(
            !ops[1].fields.contains_key("scope"),
            "a rule without scope is for the whole project: {:?}",
            ops[1].fields
        );

        let snapshot = work::Snapshot {
            rules: vec![work::Rule {
                id: "r1".into(),
                slug: "no-network".into(),
                statement: "Never call the network.".into(),
                scopes: vec!["core".into()],
                status: work::RULE_ACTIVE.into(),
                ..work::Rule::default()
            }],
            ..rule_snapshot()
        };
        let (ops, diags, _) = plan_text(
            &snapshot,
            &(HEADER.to_string() + "#### rule no-network\nstatus: removed\n"),
        );
        assert!(
            !has_errors(&diags) && ops.len() == 1 && ops[0].kind == work::OP_REMOVE,
            "remove: {ops:?} {diags:?}"
        );
        let (_, diags, _) = plan_text(
            &snapshot,
            &(HEADER.to_string() + "#### rule ghost\nstatus: removed\n"),
        );
        assert!(
            has_errors(&diags),
            "removing an unknown rule must fail: {diags:?}"
        );

        // A node's ! line is a Rule for that node; the same text is the same Rule.
        let (doc, diags) = decode(
            &(HEADER.to_string()
                + "# core: Core\nlevel: domain\n\n! Never call the network.\n! Log every write\n"),
        );
        assert!(!has_errors(&diags), "decode: {diags:?}");
        let mut planner = Planner::new(&snapshot, "");
        node_rules(&mut planner, &doc.nodes[0], &snapshot.nodes[0]);
        let ops = &planner.ops;
        assert!(
            ops.len() == 1
                && ops[0].entity == work::ENTITY_RULE
                && ops[0].label == "Log every write",
            "node lines: {ops:?} {diags:?}"
        );
        assert_eq!(ops[0].node_id, "n", "the op names the stored node");
        assert_eq!(ops[0].card_slug, "core", "the op names the path");
    }

    #[test]
    fn removes_a_node_rule_line() {
        let mut snapshot = rule_snapshot();
        snapshot.rules = vec![work::Rule {
            id: "r1".into(),
            statement: "Never call the network.".into(),
            scopes: vec!["core".into()],
            status: work::RULE_ACTIVE.into(),
            ..work::Rule::default()
        }];
        let (doc, _) = decode(
            &(HEADER.to_string()
                + "# core: Core\nlevel: domain\n\n! Never call the network. [removed]\n"),
        );
        let mut planner = Planner::new(&snapshot, "");
        node_rules(&mut planner, &doc.nodes[0], &snapshot.nodes[0]);
        assert!(
            planner.ops.len() == 1 && planner.ops[0].kind == work::OP_REMOVE,
            "removed line: {:?}",
            planner.ops
        );
        assert_eq!(
            planner.ops[0].fields["status"],
            json!(work::RULE_REMOVED),
            "status"
        );
        assert!(
            planner.rules.is_empty(),
            "the retired Rule is gone: {:?}",
            planner.rules
        );
        // A removed line for a Rule that is not stored plans nothing.
        let (doc, _) = decode(
            &(HEADER.to_string() + "# core: Core\nlevel: domain\n\n! Log every write [removed]\n"),
        );
        let mut planner = Planner::new(&rule_snapshot(), "");
        node_rules(&mut planner, &doc.nodes[0], &snapshot.nodes[0]);
        assert!(
            planner.ops.is_empty(),
            "nothing to remove: {:?}",
            planner.ops
        );
    }

    #[test]
    fn rules_for_a_chain_and_the_project() {
        let rules = vec![
            work::Rule {
                id: "r1".into(),
                statement: "Own".into(),
                scopes: vec!["core/import".into()],
                ..work::Rule::default()
            },
            work::Rule {
                id: "r2".into(),
                statement: "Parent".into(),
                scopes: vec!["core".into()],
                ..work::Rule::default()
            },
            work::Rule {
                id: "r3".into(),
                statement: "Everyone".into(),
                ..work::Rule::default()
            },
        ];
        let chain = vec!["core/import".to_string(), "core".to_string()];
        let (own, project) = rules_for(&rules, &chain);
        assert_eq!(own.len(), 2, "one level per chain path");
        assert_eq!(own[0][0].statement, "Own", "the node's own Rule");
        assert_eq!(own[1][0].statement, "Parent", "its ancestor's");
        assert_eq!(project.len(), 1, "the project-wide Rules");
        assert_eq!(project[0].statement, "Everyone");
        // A Rule is listed once even when two paths name it.
        let rules = vec![work::Rule {
            id: "r1".into(),
            statement: "Both".into(),
            scopes: vec!["core".into(), "core/import".into()],
            ..work::Rule::default()
        }];
        let (own, _) = rules_for(&rules, &chain);
        assert!(
            own[0].len() == 1 && own[1].is_empty(),
            "a Rule is listed once, at the nearest level: {own:?}"
        );
    }
}
