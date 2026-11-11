//! Port of `backend/domains/polter/conflicts.go`: the Conflict planner and its
//! text.
//!
//! `plan_conflicts` is a planner step: it raises, decides, and withdraws
//! Conflicts against the project's Rules (the planner's live list, including
//! ones this text creates) and Decisions.

use serde_json::json;

use crate::domains::{decision, work};
use crate::platform::ids;

use super::model::{CONFLICT_HEADING_PREFIX, Diagnostic, Document, SEVERITY_ERROR, go_quote};

/// Reads "a | b | c" into its trimmed, non-empty parts.
pub fn split_options(value: &str) -> Vec<String> {
    value
        .split('|')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

/// Writes the Conflict blocks. A new slug raises one: it needs the request and
/// the Rule or Decision it goes against. A known slug updates it; `answer:`
/// decides it and `status: withdrawn` drops it.
///
/// `rules` is the planner's live Rule list; `snapshot` supplies the Decisions,
/// cards, and stored Conflicts, and gains the ones this text raises.
pub fn plan_conflicts(
    doc: &Document,
    rules: &[work::Rule],
    snapshot: &mut work::Snapshot,
    diags: &mut Vec<Diagnostic>,
    ops: &mut Vec<work::Op>,
) {
    for node in &doc.conflicts {
        let mut already_failed = failed(diags, node.line);
        for entry in &node.meta {
            already_failed = already_failed || failed(diags, entry.line);
        }
        if already_failed {
            continue;
        }
        let mut fields = work::Fields::new();
        if let Some(reference) = node.meta_value("rule").filter(|value| !value.is_empty()) {
            let Some(rule) = rules
                .iter()
                .find(|rule| rule.id == reference || rule.slug == reference)
            else {
                diags.push(Diagnostic::error(
                    node.line,
                    format!(
                        "unknown rule {}: use a Rule slug or id (polterware query rules)",
                        go_quote(reference)
                    ),
                ));
                continue;
            };
            fields.insert("rule_id".into(), json!(rule.id));
        }
        if let Some(reference) = node
            .meta_value("decision")
            .filter(|value| !value.is_empty())
        {
            let Some(found) = decision::find(&snapshot.decisions, reference) else {
                diags.push(Diagnostic::error(
                    node.line,
                    format!(
                        "unknown decision {}: use a Decision number or id",
                        go_quote(reference)
                    ),
                ));
                continue;
            };
            fields.insert("decision_id".into(), json!(found.id));
        }
        if let Some(slug) = node.meta_value("card").filter(|value| !value.is_empty()) {
            if snapshot.card(slug).is_none() {
                diags.push(Diagnostic::error(
                    node.line,
                    format!("unknown card {}", go_quote(slug)),
                ));
                continue;
            }
            fields.insert("card".into(), json!(slug));
        }
        if let Some(value) = node.meta_value("options") {
            fields.insert("options".into(), json!(split_options(value)));
        }
        let answer = node.meta_value("answer").map(str::to_string);
        let status = node.meta_value("status").map(str::to_string);
        let current = snapshot.conflict(&node.reference).cloned();
        let Some(current) = current else {
            if node.request.is_empty() {
                diags.push(Diagnostic::error(
                    node.line,
                    format!(
                        "unknown conflict {}: add the request (#### conflict {}: Request)",
                        go_quote(&node.reference),
                        node.reference
                    ),
                ));
                continue;
            }
            if !fields.contains_key("rule_id") && !fields.contains_key("decision_id") {
                diags.push(Diagnostic::error(
                    node.line,
                    format!(
                        "conflict {} needs the rule: or decision: it goes against",
                        go_quote(&node.reference)
                    ),
                ));
                continue;
            }
            if answer.is_some() || status.as_deref() == Some(work::CONFLICT_WITHDRAWN) {
                diags.push(Diagnostic::error(
                    node.line,
                    format!(
                        "unknown conflict {}: raise it before deciding or withdrawing it",
                        go_quote(&node.reference)
                    ),
                ));
                continue;
            }
            fields.insert("slug".into(), json!(node.reference));
            fields.insert("request".into(), json!(node.request));
            let id = ids::new_ulid();
            snapshot.conflicts.push(work::Conflict {
                id: id.clone(),
                slug: node.reference.clone(),
                request: node.request.clone(),
                state: work::CONFLICT_OPEN.to_string(),
                ..work::Conflict::default()
            });
            ops.push(work::Op {
                entity: work::ENTITY_CONFLICT.into(),
                kind: work::OP_CREATE.into(),
                entity_id: id,
                card_slug: node.reference.clone(),
                label: node.request.clone(),
                fields,
                ..work::Op::default()
            });
            continue;
        };
        let mut before = work::Fields::new();
        if !node.request.is_empty() && node.request != current.request {
            fields.insert("request".into(), json!(node.request));
            before.insert("request".into(), json!(current.request));
        }
        let mut kind = work::OP_UPDATE;
        if let Some(answer) = &answer {
            if answer.is_empty() {
                diags.push(Diagnostic::error(
                    node.line,
                    "answer: is empty: write the decision".to_string(),
                ));
                continue;
            }
            fields.insert("answer".into(), json!(answer));
            before.insert("answer".into(), json!(current.answer));
            fields.insert("state".into(), json!(work::CONFLICT_DECIDED));
            before.insert("state".into(), json!(current.state));
        } else if let Some(status) = &status
            && *status != current.state
        {
            fields.insert("state".into(), json!(status));
            before.insert("state".into(), json!(current.state));
            if status.as_str() == work::CONFLICT_WITHDRAWN {
                kind = work::OP_REMOVE;
            } else {
                fields.insert("answer".into(), json!(""));
                before.insert("answer".into(), json!(current.answer));
            }
        }
        if fields.is_empty() {
            continue;
        }
        ops.push(work::Op {
            entity: work::ENTITY_CONFLICT.into(),
            kind: kind.into(),
            entity_id: current.id.clone(),
            card_slug: current.slug.clone(),
            label: current.request.clone(),
            fields,
            before,
            ..work::Op::default()
        });
    }
}

/// The Polter text that decides a Conflict with an answer.
pub fn conflict_text(slug: &str, answer: &str, summary: &str, rationale: &str) -> String {
    format!(
        "---\npolter: 1\nsummary: {summary}\nrationale: {rationale}\n---\n\n#### {CONFLICT_HEADING_PREFIX}{slug}\nanswer: {answer}\n"
    )
}

/// Names what a Conflict goes against: the Rule's statement and the Decision's
/// title, empty when it names none or the record is gone.
pub fn conflict_against(
    conflict: &work::Conflict,
    snapshot: &work::Snapshot,
) -> (work::Rule, String) {
    let mut rule = work::Rule::default();
    let mut decision_title = String::new();
    for candidate in &snapshot.rules {
        if candidate.id == conflict.rule_id {
            rule = candidate.clone();
        }
    }
    for candidate in &snapshot.decisions {
        if candidate.id == conflict.decision {
            decision_title = candidate.title.clone();
        }
    }
    (rule, decision_title)
}

/// Renders one Conflict as a Polter block with its state, what it goes against,
/// and the answer once a person decided it.
pub fn encode_conflict(conflict: &work::Conflict, snapshot: &work::Snapshot) -> String {
    let mut out = format!(
        "#### {CONFLICT_HEADING_PREFIX}{}: {}\nstate: {}\n",
        conflict.slug, conflict.request, conflict.state
    );
    let (rule, decision_title) = conflict_against(conflict, snapshot);
    if !rule.id.is_empty() {
        out.push_str(&format!("rule: {} ({})\n", rule.handle(), rule.statement));
    }
    if !decision_title.is_empty() {
        out.push_str(&format!("decision: {decision_title}\n"));
    }
    if !conflict.card_slug.is_empty() {
        out.push_str(&format!("card: {}\n", conflict.card_slug));
    }
    if !conflict.options.is_empty() {
        out.push_str(&format!("options: {}\n", conflict.options.join(" | ")));
    }
    if !conflict.answer.is_empty() {
        out.push_str(&format!("answer: {}\n", conflict.answer));
    }
    out
}

/// Lists the open Conflicts, then the decided ones, one line each.
pub fn encode_conflicts(snapshot: &work::Snapshot) -> String {
    let mut open: Vec<String> = Vec::new();
    let mut decided: Vec<String> = Vec::new();
    for conflict in &snapshot.conflicts {
        match conflict.state.as_str() {
            work::CONFLICT_OPEN => open.push(format!("- {}: {}", conflict.slug, conflict.request)),
            work::CONFLICT_DECIDED => decided.push(format!(
                "- {}: {} -> {}",
                conflict.slug, conflict.request, conflict.answer
            )),
            _ => {}
        }
    }
    if open.is_empty() && decided.is_empty() {
        return "No conflicts.\n".to_string();
    }
    let mut out = String::new();
    if !open.is_empty() {
        out.push_str(&format!("Open conflicts:\n{}\n", open.join("\n")));
    }
    if !decided.is_empty() {
        out.push_str(&format!("Decided conflicts:\n{}\n", decided.join("\n")));
    }
    out
}

/// Lists the Conflicts a person decided since a time, so an agent picks up the
/// answers at its next status read.
pub fn encode_status_conflicts(snapshot: &work::Snapshot, since: &str) -> String {
    let mut out = String::new();
    for conflict in &snapshot.conflicts {
        if conflict.state == work::CONFLICT_DECIDED && conflict.decided_at.as_str() >= since {
            out.push_str(&format!(
                "conflict decided: {}: {} -> {}\n",
                conflict.slug, conflict.request, conflict.answer
            ));
        }
    }
    out
}

/// Reports whether a diagnostic on `line` is an error.
fn failed(diags: &[Diagnostic], line: i32) -> bool {
    diags
        .iter()
        .any(|diag| diag.line == line && diag.severity == SEVERITY_ERROR)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::polter::decode::decode;
    use crate::domains::polter::encode::HEADER;
    use crate::domains::polter::model::has_errors;

    fn conflict_snapshot() -> work::Snapshot {
        work::Snapshot {
            project_id: "p".into(),
            rules: vec![work::Rule {
                id: "r1".into(),
                slug: "local-only".into(),
                statement: "Never call the network.".into(),
                status: work::RULE_ACTIVE.into(),
                ..work::Rule::default()
            }],
            ..work::Snapshot::default()
        }
    }

    fn plan_text(snapshot: &mut work::Snapshot, text: &str) -> (Vec<work::Op>, Vec<Diagnostic>) {
        let (doc, diags) = decode(text);
        assert!(!has_errors(&diags), "decode: {diags:?}");
        let rules = snapshot.rules.clone();
        let mut diags = diags;
        let mut ops = Vec::new();
        plan_conflicts(&doc, &rules, snapshot, &mut diags, &mut ops);
        (ops, diags)
    }

    #[test]
    fn conflict_raise_decide_withdraw() {
        let mut snapshot = conflict_snapshot();
        let (ops, diags) = plan_text(
            &mut snapshot,
            &(HEADER.to_string()
                + "#### conflict use-redis: Add a Redis cache - for the sessions\nrule: local-only\noptions: Keep local | Allow Redis\n"),
        );
        assert!(
            !has_errors(&diags)
                && ops.len() == 1
                && ops[0].entity == work::ENTITY_CONFLICT
                && ops[0].kind == work::OP_CREATE,
            "raise: {ops:?} {diags:?}"
        );
        assert_eq!(ops[0].fields["rule_id"], json!("r1"), "rule id");
        assert_eq!(
            ops[0].fields["request"],
            json!("Add a Redis cache - for the sessions"),
            "request"
        );
        assert_eq!(
            ops[0].fields["options"],
            json!(["Keep local", "Allow Redis"]),
            "options"
        );

        let mut bare = conflict_snapshot();
        let (_, diags) = plan_text(
            &mut bare,
            &(HEADER.to_string() + "#### conflict lonely: Do something\n"),
        );
        assert!(
            has_errors(&diags),
            "a conflict must name the rule or decision it goes against"
        );

        let mut snapshot = conflict_snapshot();
        snapshot.conflicts = vec![work::Conflict {
            id: "c1".into(),
            slug: "use-redis".into(),
            request: "Add a Redis cache".into(),
            rule_id: "r1".into(),
            state: work::CONFLICT_OPEN.into(),
            ..work::Conflict::default()
        }];
        let (ops, diags) = plan_text(
            &mut snapshot,
            &conflict_text(
                "use-redis",
                "Keep local - for now",
                "Decide",
                "A person decided",
            ),
        );
        assert!(
            !has_errors(&diags)
                && ops.len() == 1
                && ops[0].fields["state"] == json!(work::CONFLICT_DECIDED)
                && ops[0].fields["answer"] == json!("Keep local - for now"),
            "decide: {ops:?} {diags:?}"
        );
        let (ops, diags) = plan_text(
            &mut snapshot,
            &(HEADER.to_string() + "#### conflict use-redis\nstatus: withdrawn\n"),
        );
        assert!(
            !has_errors(&diags) && ops.len() == 1 && ops[0].kind == work::OP_REMOVE,
            "withdraw: {ops:?} {diags:?}"
        );

        snapshot.conflicts[0].state = work::CONFLICT_DECIDED.into();
        snapshot.conflicts[0].answer = "Keep local".into();
        let text = encode_conflict(&snapshot.conflicts[0], &snapshot);
        assert!(
            text.contains("rule: local-only (Never call the network.)")
                && text.contains("answer: Keep local"),
            "encode: {text}"
        );
    }

    #[test]
    fn encodes_open_and_decided_conflicts() {
        let mut snapshot = conflict_snapshot();
        assert_eq!(encode_conflicts(&snapshot), "No conflicts.\n", "empty");
        snapshot.conflicts = vec![
            work::Conflict {
                id: "c1".into(),
                slug: "use-redis".into(),
                request: "Add Redis".into(),
                state: work::CONFLICT_OPEN.into(),
                options: vec!["Keep local".into(), "Allow Redis".into()],
                card_slug: "cache".into(),
                ..work::Conflict::default()
            },
            work::Conflict {
                id: "c2".into(),
                slug: "old".into(),
                request: "Old request".into(),
                state: work::CONFLICT_DECIDED.into(),
                answer: "No".into(),
                decided_at: "2026-11-10T00:00:00Z".into(),
                ..work::Conflict::default()
            },
        ];
        let text = encode_conflicts(&snapshot);
        assert_eq!(
            text,
            "Open conflicts:\n- use-redis: Add Redis\nDecided conflicts:\n- old: Old request -> No\n"
        );
        let status = encode_status_conflicts(&snapshot, "2026-11-01T00:00:00Z");
        assert_eq!(status, "conflict decided: old: Old request -> No\n");
        assert!(
            encode_status_conflicts(&snapshot, "2026-11-11T00:00:00Z").is_empty(),
            "a newer since lists nothing"
        );
        assert_eq!(split_options(" a | b | "), vec!["a", "b"], "options");
    }
}
