//! The Rule planner, a node's `!`
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
/// known slug or id updates it, and `status: removed` retires it. A Rule carries
/// an optional `why:`, one of five statuses, and `replaced-by:` naming the Rule
/// that replaced it, which sets the status to replaced.
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
        let why = node.why_set.then(|| node.why.clone());
        let mut scopes: Option<Vec<String>> = None;
        if let Some(value) = node.meta_value("scope") {
            let parsed = parse_scopes(value);
            if !p.check_scopes(node.meta_line("scope"), parsed.as_slice()) {
                continue;
            }
            scopes = Some(parsed);
        }
        // The Rule `replaced-by` names, resolved to the stored Rule so its id
        // is written even when the text gives its slug.
        let replaced_reference = node
            .meta_value("replaced-by")
            .map(str::to_string)
            .filter(|value| !value.is_empty());
        let replaced = match &replaced_reference {
            Some(reference) => {
                let found = p
                    .rules
                    .iter()
                    .position(|rule| rule.id == *reference || rule.slug == *reference);
                match found {
                    Some(at) if p.rules[at].status == work::RULE_REMOVED => {
                        p.errorf(
                            node.meta_line("replaced-by"),
                            format!(
                                "replaced-by names a removed rule {}: use a live Rule",
                                go_quote(reference)
                            ),
                        );
                        continue;
                    }
                    Some(at) => Some(at),
                    None => {
                        p.errorf(
                            node.meta_line("replaced-by"),
                            format!(
                                "replaced-by names an unknown rule {}: use the Rule's slug or id",
                                go_quote(reference)
                            ),
                        );
                        continue;
                    }
                }
            }
            None => None,
        };
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
            if status.as_deref() == Some(work::RULE_REPLACED) && replaced.is_none() {
                p.errorf(
                    node.meta_line("status"),
                    "status: replaced needs replaced-by: <the rule that replaced it>".to_string(),
                );
                continue;
            }
            if status.as_deref() == Some(work::RULE_REPLACED) && replaced.is_none() {
                p.errorf(
                    node.meta_line("status"),
                    "status: replaced needs replaced-by: <the rule that replaced it>".to_string(),
                );
                continue;
            }
            let replaced_by = replaced.map(|at| p.rules[at].id.clone());
            let mut rule = work::Rule {
                id: ids::new_ulid(),
                slug: node.reference.clone(),
                statement: node.statement.clone(),
                why: why.clone().unwrap_or_default(),
                scopes: scopes.clone().unwrap_or_default(),
                status: status
                    .clone()
                    .unwrap_or_else(|| work::RULE_ACTIVE.to_string()),
                replaced_by: replaced_by.clone().unwrap_or_default(),
                ..work::Rule::default()
            };
            if replaced_by.is_some() {
                rule.status = work::RULE_REPLACED.to_string();
            }
            let mut fields = work::Fields::new();
            fields.insert("slug".into(), json!(rule.slug));
            fields.insert("statement".into(), json!(rule.statement));
            if let Some(scopes) = &scopes {
                fields.insert("scope".into(), json!(scopes));
            }
            if let Some(why) = &why {
                fields.insert("why".into(), json!(why));
            }
            if rule.status != work::RULE_ACTIVE {
                fields.insert("status".into(), json!(rule.status));
            }
            if let Some(replaced_by) = &replaced_by {
                fields.insert("replaced_by".into(), json!(replaced_by));
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
        if let Some(why) = &why
            && *why != current.why
        {
            fields.insert("why".into(), json!(why));
            before.insert("why".into(), json!(current.why));
            current.why = why.clone();
        }
        let mut new_status = current.status.clone();
        let mut new_replaced = current.replaced_by.clone();
        if status.as_deref() == Some(work::RULE_REMOVED) {
            new_status = work::RULE_REMOVED.to_string();
            new_replaced.clear();
        } else if let Some(at) = replaced {
            if at == index {
                p.errorf(
                    node.meta_line("replaced-by"),
                    format!(
                        "a rule cannot replace itself: {}",
                        go_quote(&node.reference)
                    ),
                );
                continue;
            }
            new_replaced = p.rules[at].id.clone();
            new_status = work::RULE_REPLACED.to_string();
        } else if let Some(given) = &status {
            if given == work::RULE_REPLACED {
                if new_replaced.is_empty() {
                    p.errorf(
                        node.meta_line("status"),
                        "status: replaced needs replaced-by: <the rule that replaced it>"
                            .to_string(),
                    );
                    continue;
                }
                new_status = work::RULE_REPLACED.to_string();
            } else {
                new_replaced.clear();
                new_status = given.clone();
            }
        }
        if new_replaced != current.replaced_by {
            fields.insert("replaced_by".into(), json!(new_replaced));
            before.insert("replaced_by".into(), json!(current.replaced_by));
            current.replaced_by = new_replaced;
        }
        if new_status != current.status {
            fields.insert("status".into(), json!(new_status));
            before.insert("status".into(), json!(current.status));
            current.status = new_status;
        }
        if fields.is_empty() {
            continue;
        }
        let kind = if current.status == work::RULE_REMOVED {
            work::OP_REMOVE
        } else {
            work::OP_UPDATE
        };
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

/// Returns the paths of active Structure nodes that cite `rule` in `rules:`.
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

    fn lifecycle_snapshot() -> work::Snapshot {
        work::Snapshot {
            rules: vec![
                work::Rule {
                    id: "r1".into(),
                    slug: "old".into(),
                    statement: "Keep the old law.".into(),
                    why: "The first reason.".into(),
                    status: work::RULE_ACTIVE.into(),
                    ..work::Rule::default()
                },
                work::Rule {
                    id: "r2".into(),
                    slug: "new".into(),
                    statement: "Keep the new law.".into(),
                    status: work::RULE_ACTIVE.into(),
                    ..work::Rule::default()
                },
            ],
            ..rule_snapshot()
        }
    }

    #[test]
    fn a_rule_writes_its_why_status_and_replacement() {
        let snapshot = lifecycle_snapshot();
        let (ops, diags, rules) = plan_text(
            &snapshot,
            &(HEADER.to_string()
                + "#### rule draft: Draft the law.\nstatus: proposed\nwhy: Not agreed yet.\n"),
        );
        assert!(!has_errors(&diags), "create: {diags:?}");
        assert_eq!(ops[0].fields["status"], json!(work::RULE_PROPOSED));
        assert_eq!(ops[0].fields["why"], json!("Not agreed yet."));
        assert_eq!(rules[2].status, work::RULE_PROPOSED, "the new Rule");

        let (ops, diags, rules) = plan_text(
            &snapshot,
            &(HEADER.to_string() + "#### rule old\nreplaced-by: new\n"),
        );
        assert!(!has_errors(&diags), "replace: {diags:?}");
        assert_eq!(ops[0].fields["status"], json!(work::RULE_REPLACED));
        assert_eq!(ops[0].fields["replaced_by"], json!("r2"), "stored id");
        assert_eq!(rules[0].status, work::RULE_REPLACED);
        assert_eq!(rules[0].replaced_by, "r2");

        let (ops, diags, _) = plan_text(
            &snapshot,
            &(HEADER.to_string() + "#### rule old\nwhy: The law moved.\n"),
        );
        assert!(!has_errors(&diags), "why: {diags:?}");
        assert_eq!(ops[0].fields["why"], json!("The law moved."));

        let (ops, diags, _) = plan_text(&snapshot, &(HEADER.to_string() + "#### rule old\nwhy:\n"));
        assert!(!has_errors(&diags), "clear why: {diags:?}");
        assert_eq!(ops[0].fields["why"], json!(""), "why: clears the reason");
    }

    #[test]
    fn rule_lifecycle_errors() {
        let snapshot = lifecycle_snapshot();
        for (text, fragment) in [
            ("#### rule old\nstatus: replaced\n", "needs replaced-by"),
            ("#### rule old\nreplaced-by: old\n", "cannot replace itself"),
            ("#### rule old\nreplaced-by: ghost\n", "unknown rule"),
            ("#### rule old\nreplaced-by: removed-one\n", "removed rule"),
        ] {
            let mut snapshot = snapshot.clone();
            if fragment == "removed rule" {
                snapshot.rules.push(work::Rule {
                    id: "r3".into(),
                    slug: "removed-one".into(),
                    statement: "Retired.".into(),
                    status: work::RULE_REMOVED.into(),
                    ..work::Rule::default()
                });
            }
            let (_, diags, _) = plan_text(&snapshot, &(HEADER.to_string() + text));
            assert!(
                has_errors(&diags) && diags.iter().any(|d| d.message.contains(fragment)),
                "{text}: {diags:?}"
            );
        }
        let (_, diags, _) = plan_text(
            &snapshot,
            &(HEADER.to_string() + "#### rule old\nreplaced-by: ghost\n"),
        );
        assert!(
            has_errors(&diags) && diags.iter().any(|d| d.message.contains("unknown rule")),
            "an unknown target is refused: {diags:?}"
        );
    }

    #[test]
    fn only_active_and_under_review_rules_bind() {
        let rule = |status: &str, statement: &str| work::Rule {
            status: status.into(),
            statement: statement.into(),
            ..work::Rule::default()
        };
        let rules = vec![
            rule(work::RULE_PROPOSED, "Proposed"),
            rule(work::RULE_ACTIVE, "Active"),
            rule(work::RULE_UNDER_REVIEW, "Review"),
            rule(work::RULE_REPLACED, "Replaced"),
            rule(work::RULE_REMOVED, "Removed"),
        ];
        let (_, project) = rules_for(&rules, &[]);
        let binding: Vec<&str> = project.iter().map(|rule| rule.statement.as_str()).collect();
        assert_eq!(binding, vec!["Active", "Review"]);
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
