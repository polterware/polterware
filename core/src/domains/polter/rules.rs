//! Port of `backend/domains/polter/rules.go`: the Rule planner, a node's `!`
//! lines, and the Rules that hold for a path.
//!
//! `plan_rules` and `node_rules` are planner steps: they take the project's
//! Rules (a mutable copy for rules, which this text updates as it plans) and its
//! Structure nodes by path, and append their ops and diagnostics.

use std::collections::HashMap;

use serde_json::json;

use crate::domains::{structure, work};
use crate::platform::ids;

use super::model::{Diagnostic, Document, SEVERITY_ERROR, StructureNode, go_quote};
use super::scopes::{parse_scopes, scope_is_active};

/// Writes the Rule blocks: a new slug creates a Rule (it needs a statement), a
/// known slug or id updates it, and `status: removed` retires it.
///
/// `rules` starts as the project's active Rules and is updated as ops are
/// planned (the planner's `rules`); `nodes` is the project's Structure nodes by
/// path, used to check `scope:`.
pub fn plan_rules(
    doc: &Document,
    rules: &mut Vec<work::Rule>,
    nodes: &HashMap<String, structure::Node>,
    diags: &mut Vec<Diagnostic>,
    ops: &mut Vec<work::Op>,
) {
    for node in &doc.rules {
        let mut already_failed = failed(diags, node.line);
        for entry in &node.meta {
            already_failed = already_failed || failed(diags, entry.line);
        }
        if already_failed {
            continue;
        }
        let index = rules
            .iter()
            .position(|rule| rule.id == node.reference || rule.slug == node.reference);
        let status = node.meta_value("status").map(str::to_string);
        let mut scopes: Option<Vec<String>> = None;
        if let Some(value) = node.meta_value("scope") {
            let parsed = parse_scopes(value);
            if !check_scopes(
                &mut *diags,
                nodes,
                node.meta_line("scope"),
                parsed.as_slice(),
            ) {
                continue;
            }
            scopes = Some(parsed);
        }
        let Some(index) = index else {
            if status.as_deref() == Some(work::RULE_REMOVED) {
                diags.push(Diagnostic::error(
                    node.line,
                    format!(
                        "cannot remove unknown rule {}: check the slug",
                        go_quote(&node.reference)
                    ),
                ));
                continue;
            }
            if node.statement.is_empty() {
                diags.push(Diagnostic::error(
                    node.line,
                    format!(
                        "unknown rule {}: add a statement (#### rule {}: Statement)",
                        go_quote(&node.reference),
                        node.reference
                    ),
                ));
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
            ops.push(work::Op {
                entity: work::ENTITY_RULE.into(),
                kind: work::OP_CREATE.into(),
                entity_id: rule.id.clone(),
                card_slug: rule.slug.clone(),
                label: rule.statement.clone(),
                fields,
                ..work::Op::default()
            });
            rules.push(rule);
            continue;
        };
        let mut current = rules[index].clone();
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
        ops.push(work::Op {
            entity: work::ENTITY_RULE.into(),
            kind: kind.into(),
            entity_id: current.id.clone(),
            card_slug: current.handle().to_string(),
            label: current.statement.clone(),
            fields,
            before,
            ..work::Op::default()
        });
        rules[index] = current;
    }
}

/// Plans a node's `!` lines: each is a Rule whose only context is that node,
/// matched by its normalized text. `[removed]` retires it.
pub fn node_rules(
    node: &StructureNode,
    stored: &structure::Node,
    rules: &mut Vec<work::Rule>,
    ops: &mut Vec<work::Op>,
) {
    for invariant in &node.invariants {
        let key = work::normalize_text(&invariant.text);
        let index = rules.iter().position(|rule| {
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
            ops.push(work::Op {
                entity: work::ENTITY_RULE.into(),
                kind: work::OP_CREATE.into(),
                entity_id: rule.id.clone(),
                node_id: stored.id.clone(),
                card_slug: stored.path.clone(),
                label: invariant.text.clone(),
                fields,
                ..work::Op::default()
            });
            rules.push(rule);
            continue;
        };
        let current = rules[index].clone();
        if invariant.removed {
            rules.remove(index);
            let mut fields = work::Fields::new();
            fields.insert("status".into(), json!(work::RULE_REMOVED));
            let mut before = work::Fields::new();
            before.insert("status".into(), json!(current.status));
            ops.push(work::Op {
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
            ops.push(work::Op {
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
            rules[index].statement = invariant.text.clone();
        }
    }
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
            if !seen.contains(&rule.id.as_str()) && rule.scopes.contains(path) {
                seen.push(rule.id.as_str());
                own[i].push(rule.clone());
            }
        }
    }
    let project = rules
        .iter()
        .filter(|rule| rule.scopes.is_empty())
        .cloned()
        .collect();
    (own, project)
}

/// Reports whether a diagnostic on `line` is an error.
fn failed(diags: &[Diagnostic], line: i32) -> bool {
    diags
        .iter()
        .any(|diag| diag.line == line && diag.severity == SEVERITY_ERROR)
}

/// Reports each path that is not an active Structure node.
///
/// TODO(dedupe): the planner keeps one `checkScopes`; this copy waits for
/// `plan.rs` to land.
fn check_scopes(
    diags: &mut Vec<Diagnostic>,
    nodes: &HashMap<String, structure::Node>,
    line: i32,
    paths: &[String],
) -> bool {
    let mut ok = true;
    for path in paths {
        if !scope_is_active(nodes, path) {
            diags.push(Diagnostic::error(
                line,
                format!(
                    "unknown scope {}: use a Structure node path",
                    go_quote(path)
                ),
            ));
            ok = false;
        }
    }
    ok
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::polter::decode::decode;
    use crate::domains::polter::encode::HEADER;
    use crate::domains::polter::model::has_errors;

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

    fn nodes_of(snapshot: &work::Snapshot) -> HashMap<String, structure::Node> {
        snapshot
            .nodes
            .iter()
            .map(|node| (node.path.clone(), node.clone()))
            .collect()
    }

    fn plan_text(
        snapshot: &work::Snapshot,
        text: &str,
    ) -> (Vec<work::Op>, Vec<Diagnostic>, Vec<work::Rule>) {
        let (doc, diags) = decode(text);
        assert!(!has_errors(&diags), "decode: {diags:?}");
        let mut rules = snapshot.rules.clone();
        let mut diags = diags;
        let mut ops = Vec::new();
        plan_rules(&doc, &mut rules, &nodes_of(snapshot), &mut diags, &mut ops);
        (ops, diags, rules)
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
        let mut rules = snapshot.rules.clone();
        let mut ops = Vec::new();
        node_rules(&doc.nodes[0], &snapshot.nodes[0], &mut rules, &mut ops);
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
        let mut rules = snapshot.rules.clone();
        let mut ops = Vec::new();
        node_rules(&doc.nodes[0], &snapshot.nodes[0], &mut rules, &mut ops);
        assert!(
            ops.len() == 1 && ops[0].kind == work::OP_REMOVE,
            "removed line: {ops:?}"
        );
        assert_eq!(ops[0].fields["status"], json!(work::RULE_REMOVED), "status");
        assert!(rules.is_empty(), "the retired Rule is gone: {rules:?}");
        // A removed line for a Rule that is not stored plans nothing.
        let (doc, _) = decode(
            &(HEADER.to_string() + "# core: Core\nlevel: domain\n\n! Log every write [removed]\n"),
        );
        let mut ops = Vec::new();
        node_rules(&doc.nodes[0], &snapshot.nodes[0], &mut Vec::new(), &mut ops);
        assert!(ops.is_empty(), "nothing to remove: {ops:?}");
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
