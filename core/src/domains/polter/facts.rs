//! Port of `backend/domains/polter/facts.go`: the canon Fact planner.
//!
//! `plan_facts` is a planner step: it diffs the `#### slug: Statement` blocks
//! against the project's Facts and appends the ops and diagnostics it plans.

use serde_json::json;

use crate::domains::{canon, work};

use super::model::{Document, go_quote};
use super::plan::Planner;
use super::scopes::parse_scopes;

/// Plans the Fact blocks: a new slug creates a Fact (it needs a statement), a
/// known slug or id updates it, and `kind:` and `scope:` replace the stored
/// values.
pub fn plan_facts(p: &mut Planner, doc: &Document) {
    let mut state: Vec<work::Fact> = p.snapshot.facts.clone();
    for node in &doc.facts {
        let mut already_failed = p.failed(node.line);
        for entry in &node.meta {
            already_failed = already_failed || p.failed(entry.line);
        }
        if already_failed {
            continue;
        }
        let mut current = work::Fact::default();
        let mut index: Option<usize> = None;
        for (i, fact) in state.iter().enumerate() {
            if fact.fact.id == node.reference || fact.slug == node.reference {
                current = fact.clone();
                index = Some(i);
                break;
            }
        }
        let mut fields = work::Fields::new();
        let mut before = work::Fields::new();
        if index.is_none() {
            if node.statement.is_empty() {
                p.errorf(
                    node.line,
                    format!(
                        "unknown fact {}: add a statement (#### slug: Statement)",
                        go_quote(&node.reference)
                    ),
                );
                continue;
            }
            current = work::Fact {
                fact: canon::Fact {
                    id: canon::new_fact_id(),
                    statement: node.statement.clone(),
                    truth_class: canon::TRUTH_DECLARED.to_string(),
                    ..canon::Fact::default()
                },
                slug: node.reference.clone(),
                ..work::Fact::default()
            };
            fields.insert("slug".into(), json!(current.slug));
            fields.insert("statement".into(), json!(current.fact.statement));
            fields.insert("kind".into(), json!(current.fact.truth_class));
        } else if !node.statement.is_empty() && node.statement != current.fact.statement {
            fields.insert("statement".into(), json!(node.statement));
            before.insert("statement".into(), json!(current.fact.statement));
            current.fact.statement = node.statement.clone();
        }
        if let Some(kind) = node.meta_value("kind") {
            if !canon::valid_fact_truth_class(kind) {
                p.errorf(
                    node.meta_line("kind"),
                    format!(
                        "unknown fact kind {}: use declared|inferred|verified",
                        go_quote(kind)
                    ),
                );
                continue;
            }
            if kind != current.fact.truth_class.as_str() {
                fields.insert("kind".into(), json!(kind));
                before.insert("kind".into(), json!(current.fact.truth_class));
                current.fact.truth_class = kind.to_string();
            }
        }
        if let Some(value) = node.meta_value("scope") {
            let scopes = parse_scopes(value);
            if !p.check_scopes(node.meta_line("scope"), scopes.as_slice()) {
                continue;
            }
            if scopes != current.scopes {
                fields.insert("scope".into(), json!(scopes.clone()));
                before.insert("scope".into(), json!(current.scopes));
                current.scopes = scopes;
            }
        }
        if fields.is_empty() {
            continue;
        }
        let kind = match index {
            None => {
                state.push(current.clone());
                work::OP_CREATE
            }
            Some(index) => {
                state[index] = current.clone();
                work::OP_UPDATE
            }
        };
        p.ops.push(work::Op {
            entity: work::ENTITY_FACT.into(),
            kind: kind.into(),
            entity_id: current.fact.id.clone(),
            card_slug: current.fact.id.clone(),
            label: current.fact.statement.clone(),
            fields,
            before,
            ..work::Op::default()
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::domains::structure;

    use super::*;
    use crate::domains::polter::decode::decode;
    use crate::domains::polter::encode::{HEADER, format};
    use crate::domains::polter::model::has_errors;
    use crate::domains::polter::model::{Diagnostic, SEVERITY_ERROR};

    fn fact_snapshot() -> work::Snapshot {
        work::Snapshot {
            project_id: "p".into(),
            nodes: vec![structure::Node {
                id: "n".into(),
                path: "core".into(),
                level: structure::LEVEL_DOMAIN.into(),
                status: structure::NODE_ACTIVE.into(),
                ..structure::Node::default()
            }],
            ..work::Snapshot::default()
        }
    }

    fn plan_text(
        facts: &[work::Fact],
        nodes: &HashMap<String, structure::Node>,
        text: &str,
    ) -> (Vec<work::Op>, Vec<Diagnostic>) {
        let (doc, diags) = decode(text);
        assert!(!has_errors(&diags), "decode: {diags:?}");
        let snapshot = work::Snapshot {
            facts: facts.to_vec(),
            nodes: nodes.values().cloned().collect(),
            ..work::Snapshot::default()
        };
        let mut planner = Planner::new(&snapshot, "");
        planner.diags = diags;
        plan_facts(&mut planner, &doc);
        (planner.ops, planner.diags)
    }

    fn nodes_of(snapshot: &work::Snapshot) -> HashMap<String, structure::Node> {
        snapshot
            .nodes
            .iter()
            .map(|node| (node.path.clone(), node.clone()))
            .collect()
    }

    #[test]
    fn fact_decode_and_format() {
        let text = format!(
            "{HEADER}\n## work: Work\nstage: plan\n### new: Decision\nstatus: proposed\n\nDecision body.\n#### local-core: One process owns SQLite.\nkind: inferred · scope: core\n#### rule no-network: Never call the network.\nscope: core\n# core: Core\nlevel: domain\n"
        );
        let (doc, diags) = decode(&text);
        assert!(
            !has_errors(&diags)
                && doc.facts.len() == 1
                && doc.rules.len() == 1
                && doc.cards.len() == 1
                && doc.decisions.len() == 1
                && doc.nodes.len() == 1,
            "doc={doc:?} diags={diags:?}"
        );
        let fact = &doc.facts[0];
        assert_eq!(fact.reference, "local-core", "ref");
        assert_eq!(fact.statement, "One process owns SQLite.", "statement");
        let (again, diags) = decode(&format(&doc));
        assert!(
            !has_errors(&diags) && format(&again) == format(&doc),
            "round trip: {} {diags:?}",
            format(&again)
        );
    }

    #[test]
    fn fact_lint_line_numbers() {
        for (body, line, message) in [
            (
                "#### local-core: Local core\nkind: operational\n",
                5,
                "unknown fact kind",
            ),
            (
                "#### local-core: Local core\nkind: observed\n",
                5,
                "unknown fact kind",
            ),
            (
                "#### local-core: Local core\nevidence: file:go.mod\n",
                5,
                "evidence: is retired",
            ),
            (
                "#### local-core: Local core\nowner: me\n",
                5,
                "unknown fact key",
            ),
            (
                "#### local-core: Local core\n#### local-core: Duplicate\n",
                5,
                "duplicate fact",
            ),
        ] {
            let (_, diags) = super::super::lint::lint(
                &format!("{HEADER}{body}"),
                Some(&super::super::lint::References::default()),
            );
            let found = diags.iter().any(|diag| {
                diag.line == line
                    && diag.severity == SEVERITY_ERROR
                    && diag.message.contains(message)
            });
            assert!(found, "{body}: {diags:?}");
        }
    }

    #[test]
    fn fact_plan_create_update_and_omission() {
        let snapshot = fact_snapshot();
        let nodes = nodes_of(&snapshot);
        for kind in ["", "declared", "inferred", "verified"] {
            let mut text = format!("{HEADER}#### local-core: Local core\nscope: core\n");
            if !kind.is_empty() {
                text.push_str(&format!("kind: {kind}\n"));
            }
            let (ops, diags) = plan_text(&snapshot.facts, &nodes, &text);
            assert!(
                !has_errors(&diags)
                    && ops.len() == 1
                    && ops[0].entity == work::ENTITY_FACT
                    && ops[0].kind == work::OP_CREATE,
                "ops={ops:?} diags={diags:?}"
            );
            let want = if kind.is_empty() { "declared" } else { kind };
            assert_eq!(ops[0].fields["kind"], json!(want), "kind");
            let plan = super::super::plan::format_plan(&ops);
            assert!(
                plan.starts_with(&format!("+ fact {} \"Local core\"", ops[0].entity_id)),
                "plan = {plan:?}"
            );
            assert_eq!(
                super::super::plan::summarize(&ops),
                "1 op (1 fact)",
                "summary"
            );
        }
        let facts = [work::Fact {
            fact: canon::Fact {
                id: "fact-1".into(),
                statement: "Local core".into(),
                truth_class: "declared".into(),
                ..canon::Fact::default()
            },
            slug: "local-core".into(),
            scopes: vec!["core".into()],
        }];
        for reference in ["local-core", "fact-1"] {
            let (ops, diags) = plan_text(
                &facts,
                &nodes,
                &format!("{HEADER}#### {reference}: Renamed\n"),
            );
            assert!(
                !has_errors(&diags)
                    && ops.len() == 1
                    && ops[0].kind == work::OP_UPDATE
                    && ops[0].fields.len() == 1
                    && ops[0].before["statement"] == json!("Local core"),
                "ops={ops:?} diags={diags:?}"
            );
            let (ops, diags) = plan_text(&facts, &nodes, &format!("{HEADER}#### {reference}\n"));
            assert!(
                !has_errors(&diags) && ops.is_empty(),
                "omission ops={ops:?} diags={diags:?}"
            );
        }
        let (ops, diags) = plan_text(&facts, &nodes, &format!("{HEADER}#### fact-1\nscope:\n"));
        assert!(
            !has_errors(&diags) && ops.len() == 1 && ops[0].fields.len() == 1,
            "clear: {ops:?} {diags:?}"
        );
        let (_, diags) = plan_text(
            &facts,
            &nodes,
            &format!("{HEADER}#### local-core\nscope: missing\n"),
        );
        assert!(has_errors(&diags) && diags[0].line == 5, "scope: {diags:?}");
    }
}
