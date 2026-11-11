//! Port of `backend/domains/polter/decisions.go`: the Decision planner, the
//! lifecycle text, and the list encoders.
//!
//! `plan_decisions` is one step of the planner `plan` runs after its nodes and
//! cards: it takes the project's Decisions and its Structure nodes by path (the
//! planner's live map, including nodes this text creates) and appends the ops
//! and diagnostics it plans.

use std::collections::HashMap;

use serde_json::json;

use crate::domains::{decision, structure, work};
use crate::platform::ids;

use super::model::{
    DECISION_NEW_REF, DecisionNode, Diagnostic, Document, SEVERITY_ERROR, go_quote,
};
use super::scopes::{parse_scopes, scope_is_active};

/// Plans the `###` decision blocks. Existing decisions move through the
/// lifecycle; `new` proposes one; `supersedes:` on the newer decision marks the
/// older one superseded in the same batch.
///
/// `decisions` is the project's stored Decisions and `nodes` its Structure nodes
/// by path; `diags` already holds the diagnostics of the earlier planning steps
/// (a failed line is skipped) and both `diags` and `ops` grow in place.
pub fn plan_decisions(
    doc: &Document,
    decisions: &[decision::Decision],
    nodes: &HashMap<String, structure::Node>,
    diags: &mut Vec<Diagnostic>,
    ops: &mut Vec<work::Op>,
) {
    if doc.decisions.is_empty() {
        return;
    }
    let mut batch = DecisionBatch {
        stored: decisions,
        nodes,
        changed: Vec::new(),
        diags,
        ops,
    };
    for node in &doc.decisions {
        if batch.failed(node.line) {
            continue;
        }
        if node.reference.eq_ignore_ascii_case(DECISION_NEW_REF) {
            batch.create(node);
            continue;
        }
        let Some(current) = decision::find(&batch.list(), &node.reference) else {
            batch.errorf(
                node.line,
                format!(
                    "unknown decision {}: use a Decision number, id, or ### new: Title to propose one",
                    go_quote(&node.reference)
                ),
            );
            continue;
        };
        batch.update(node, &current);
    }
}

/// Reads a decision's contexts.
struct ScopeRead {
    scopes: Vec<String>,
    /// False when the text names neither `scope:` nor `level:`.
    has: bool,
    /// False when the text was refused; nothing is planned then.
    ok: bool,
}

/// The decisions one text changes, by id, in the order they were changed. Go
/// keeps the same state in a map plus a `list` closure.
struct DecisionBatch<'a> {
    stored: &'a [decision::Decision],
    nodes: &'a HashMap<String, structure::Node>,
    changed: Vec<(String, decision::Decision)>,
    diags: &'a mut Vec<Diagnostic>,
    ops: &'a mut Vec<work::Op>,
}

impl DecisionBatch<'_> {
    /// The decisions as the text has left them: the stored order, each changed
    /// one substituted, then the ones this text created.
    fn list(&self) -> Vec<decision::Decision> {
        let mut out: Vec<decision::Decision> = Vec::with_capacity(self.stored.len() + 1);
        for stored in self.stored {
            match self.changed.iter().find(|entry| entry.0 == stored.id) {
                Some((_, changed)) => out.push(changed.clone()),
                None => out.push(stored.clone()),
            }
        }
        for (_, changed) in &self.changed {
            if !out.iter().any(|entry| entry.id == changed.id) {
                out.push(changed.clone());
            }
        }
        out
    }

    fn set(&mut self, changed: decision::Decision) {
        let id = changed.id.clone();
        match self.changed.iter_mut().find(|entry| entry.0 == id) {
            Some((_, slot)) => *slot = changed,
            None => self.changed.push((id, changed)),
        }
    }

    fn errorf(&mut self, line: i32, message: String) {
        self.diags.push(Diagnostic::error(line, message));
    }

    fn failed(&self, line: i32) -> bool {
        self.diags
            .iter()
            .any(|diag| diag.line == line && diag.severity == SEVERITY_ERROR)
    }

    fn create(&mut self, node: &DecisionNode) {
        if node.title.is_empty() {
            self.errorf(
                node.line,
                "a new decision needs a title: write \"### new: Title\"".to_string(),
            );
            return;
        }
        let all = self.list();
        let mut title = node.title.clone();
        if let Some(number) = decision::number(&title) {
            for existing in &all {
                if let Some(other) = decision::number(&existing.title)
                    && other == number
                {
                    self.errorf(
                        node.line,
                        format!(
                            "Decision number {} is taken by {}: omit the number and it is assigned",
                            number,
                            go_quote(&existing.title)
                        ),
                    );
                    return;
                }
            }
        } else {
            let next = decision::next_number(&all);
            title = decision::title_for(next, &title);
        }
        let status = match node.meta_value("status") {
            Some(status) => status.to_string(),
            None if node.meta_value("supersedes").is_some() => decision::STATE_ACTIVE.to_string(),
            None => decision::STATE_PROPOSED.to_string(),
        };
        let supersedes = node.meta_value("supersedes").map(str::to_string);
        if status != decision::STATE_PROPOSED && status != decision::STATE_ACTIVE {
            self.errorf(
                node.line,
                format!("a new decision starts proposed or active, not {status}"),
            );
            return;
        }
        if supersedes.is_some() && status != decision::STATE_ACTIVE {
            self.errorf(
                node.line,
                "supersedes needs status: active on the new decision".to_string(),
            );
            return;
        }
        let trigger = node.meta_value("trigger").unwrap_or_default().to_string();
        let reason = node.meta_value("reason").unwrap_or_default().to_string();
        let scopes = self.decision_scopes(node);
        if !scopes.ok {
            return;
        }
        let id = ids::new_ulid();
        let created = decision::Decision {
            id: id.clone(),
            title: title.clone(),
            status: status.clone(),
            scopes: scopes.scopes.clone(),
            meta: decision::Meta {
                review_trigger: trigger.clone(),
                ..decision::Meta::default()
            },
            ..decision::Decision::default()
        };
        let mut fields = work::Fields::new();
        fields.insert("title".into(), json!(title));
        fields.insert("body".into(), json!(node.body));
        fields.insert("status".into(), json!(status));
        fields.insert("review_trigger".into(), json!(trigger));
        fields.insert("scope".into(), json!(scopes.scopes));
        self.ops.push(work::Op {
            entity: work::ENTITY_DECISION.into(),
            kind: work::OP_CREATE.into(),
            entity_id: id,
            card_slug: title.clone(),
            label: title,
            fields,
            ..work::Op::default()
        });
        self.set(created.clone());
        if let Some(supersedes) = &supersedes {
            self.supersede(node, supersedes, &created, &reason);
        }
    }

    fn update(&mut self, node: &DecisionNode, current: &decision::Decision) {
        let mut fields = work::Fields::new();
        let mut before = work::Fields::new();
        let mut updated = current.clone();
        let has_status = node.meta_value("status").is_some();
        let supersedes = node.meta_value("supersedes").map(str::to_string);
        let reason = node.meta_value("reason").map(str::to_string);
        let mut status = node.meta_value("status").map(str::to_string);
        if status.is_none()
            && supersedes.is_some()
            && decision::normalize(&current.status) == decision::STATE_PROPOSED
        {
            status = Some(decision::STATE_ACTIVE.to_string());
        }
        let current_state = decision::normalize(&current.status);
        if let Some(status) = &status
            && status.as_str() != current_state
        {
            if status.as_str() == decision::STATE_SUPERSEDED {
                self.errorf(
                    node.line,
                    format!(
                        "{} is superseded by naming it in the newer decision: write supersedes: {} there",
                        current.title,
                        decision_handle(current)
                    ),
                );
                return;
            }
            if !decision::can_transition(&current.status, status) {
                self.errorf(
                    node.line,
                    format!(
                        "{} cannot move {} → {}: {}",
                        current.title,
                        current_state,
                        status,
                        transition_hint(&current.status)
                    ),
                );
                return;
            }
            if status.as_str() == decision::STATE_CHALLENGED
                && reason.as_deref().unwrap_or_default().trim().is_empty()
                && reason.is_none()
            {
                self.errorf(
                    node.line,
                    format!(
                        "challenging {} needs a reason: add reason: <why it is under review>",
                        current.title
                    ),
                );
                return;
            }
            set_field(
                &mut fields,
                &mut before,
                "status",
                json!(current_state),
                json!(status),
            );
            updated.status = status.clone();
        }
        if supersedes.is_some() && decision::normalize(&updated.status) != decision::STATE_ACTIVE {
            self.errorf(
                node.line,
                format!(
                    "{} must be active to supersede another decision",
                    current.title
                ),
            );
            return;
        }
        if let Some(trigger) = node.meta_value("trigger")
            && trigger != current.meta.review_trigger.as_str()
        {
            set_field(
                &mut fields,
                &mut before,
                "review_trigger",
                json!(current.meta.review_trigger),
                json!(trigger),
            );
            updated.meta.review_trigger = trigger.to_string();
        }
        let scopes = self.decision_scopes(node);
        if !scopes.ok {
            return;
        }
        if scopes.has && scopes.scopes != current.scopes {
            set_field(
                &mut fields,
                &mut before,
                "scope",
                json!(current.scopes),
                json!(scopes.scopes),
            );
            updated.scopes = scopes.scopes;
        }
        if let Some(reason) = &reason
            && has_status
            && reason.as_str() != current.meta.status_reason
        {
            set_field(
                &mut fields,
                &mut before,
                "status_reason",
                json!(current.meta.status_reason),
                json!(reason),
            );
            updated.meta.status_reason = reason.clone();
        }
        if node.body_set {
            set_field(
                &mut fields,
                &mut before,
                "body",
                json!(""),
                json!(node.body),
            );
        }
        if !node.title.is_empty() && node.title != current.title {
            set_field(
                &mut fields,
                &mut before,
                "title",
                json!(current.title),
                json!(node.title),
            );
            updated.title = node.title.clone();
        }
        if !fields.is_empty() {
            self.ops.push(work::Op {
                entity: work::ENTITY_DECISION.into(),
                kind: work::OP_UPDATE.into(),
                entity_id: current.id.clone(),
                card_slug: current.title.clone(),
                label: updated.title.clone(),
                fields,
                before,
                ..work::Op::default()
            });
            self.set(updated.clone());
        }
        if let Some(supersedes) = &supersedes {
            let reason = reason.unwrap_or_default();
            self.supersede(node, supersedes, &updated, &reason);
        }
    }

    /// Marks the decision named by `target` as superseded by `newer`.
    fn supersede(
        &mut self,
        node: &DecisionNode,
        target: &str,
        newer: &decision::Decision,
        reason: &str,
    ) {
        let all = self.list();
        let Some(old) = decision::find(&all, target) else {
            self.errorf(
                node.line,
                format!(
                    "unknown decision {} in supersedes: no Decision with that number or id",
                    go_quote(target)
                ),
            );
            return;
        };
        if decision::normalize(&old.status) == decision::STATE_SUPERSEDED {
            if old.meta.superseded_by != newer.id {
                self.errorf(
                    node.line,
                    format!(
                        "{} is already superseded: cite its successor instead",
                        old.title
                    ),
                );
            }
            return;
        }
        if !decision::can_transition(&old.status, decision::STATE_SUPERSEDED) {
            self.errorf(
                node.line,
                format!(
                    "{} is {} and cannot be superseded: activate it first",
                    old.title,
                    decision::normalize(&old.status)
                ),
            );
            return;
        }
        if let Err(err) = decision::check_supersede(&all, &old.id, &newer.id) {
            self.errorf(node.line, err.to_string());
            return;
        }
        let mut fields = work::Fields::new();
        fields.insert("status".into(), json!(decision::STATE_SUPERSEDED));
        fields.insert("superseded_by".into(), json!(newer.id));
        let mut before = work::Fields::new();
        before.insert("status".into(), json!(decision::normalize(&old.status)));
        before.insert("superseded_by".into(), json!(old.meta.superseded_by));
        if !reason.is_empty() {
            fields.insert("status_reason".into(), json!(reason));
            before.insert("status_reason".into(), json!(old.meta.status_reason));
        }
        let mut superseded = old.clone();
        superseded.status = decision::STATE_SUPERSEDED.to_string();
        superseded.meta.superseded_by = newer.id.clone();
        self.set(superseded);
        self.ops.push(work::Op {
            entity: work::ENTITY_DECISION.into(),
            kind: work::OP_UPDATE.into(),
            entity_id: old.id,
            card_slug: old.title.clone(),
            label: old.title,
            fields,
            before,
            ..work::Op::default()
        });
    }

    /// Reads a decision's contexts from `scope:` (a list of node paths) and
    /// `level: principle` (no contexts: the whole project). `has` is false when
    /// the text names neither, so an update keeps the stored contexts.
    fn decision_scopes(&mut self, node: &DecisionNode) -> ScopeRead {
        let value = node.meta_value("scope");
        let level = node.meta_value("level");
        let principle =
            level.is_some_and(|level| level.trim().eq_ignore_ascii_case(decision::LEVEL_PRINCIPLE));
        let scopes = value.map(parse_scopes).unwrap_or_default();
        if principle && !scopes.is_empty() {
            self.errorf(
                node.line,
                "level: principle applies to the whole project: drop scope: or level:".to_string(),
            );
            return ScopeRead {
                scopes: Vec::new(),
                has: false,
                ok: false,
            };
        }
        if principle {
            return ScopeRead {
                scopes: Vec::new(),
                has: true,
                ok: true,
            };
        }
        if value.is_none() {
            return ScopeRead {
                scopes: Vec::new(),
                has: false,
                ok: true,
            };
        }
        let line = node
            .meta
            .iter()
            .rev()
            .find(|entry| entry.key == "scope")
            .map_or(node.line, |entry| entry.line);
        if !self.check_scopes(line, &scopes) {
            return ScopeRead {
                scopes: Vec::new(),
                has: false,
                ok: false,
            };
        }
        ScopeRead {
            scopes,
            has: true,
            ok: true,
        }
    }

    /// Reports each path that is not an active Structure node.
    ///
    /// TODO(dedupe): the planner keeps one `checkScopes`; this copy waits for
    /// `plan.rs` to land.
    fn check_scopes(&mut self, line: i32, paths: &[String]) -> bool {
        let mut ok = true;
        for path in paths {
            if !scope_is_active(self.nodes, path) {
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

/// Records a changed field with its previous value.
fn set_field(
    fields: &mut work::Fields,
    before: &mut work::Fields,
    key: &str,
    old: serde_json::Value,
    value: serde_json::Value,
) {
    fields.insert(key.to_string(), value);
    before.insert(key.to_string(), old);
}

/// The reference of a decision: its number, or its id when it has none.
pub fn decision_handle(decision: &decision::Decision) -> String {
    if let Some(number) = decision::number(&decision.title) {
        return decision::decision_ref(number);
    }
    decision.id.clone()
}

/// The reminder of the legal moves out of `from`.
pub fn transition_hint(from: &str) -> &'static str {
    match decision::normalize(from) {
        decision::STATE_PROPOSED => "a proposal is activated first (status: active)",
        decision::STATE_SUPERSEDED => "a superseded decision is final: propose a new one instead",
        _ => "legal moves are proposed → active → challenged → active|superseded",
    }
}

/// Reports whether a decision in this status constrains work.
pub fn decision_binding(status: &str) -> bool {
    decision::binding(status)
}

/// The lifecycle state of a stored Decision status.
pub fn decision_state(status: &str) -> &'static str {
    decision::normalize(status)
}

/// Lists decisions, one line each, optionally only those in `state`. A header
/// counts every state so the whole lifecycle is visible.
pub fn encode_decisions(decisions: &[decision::Decision], state: &str) -> String {
    let mut counts: HashMap<&str, i32> = HashMap::new();
    for entry in decisions {
        *counts
            .entry(decision::normalize(&entry.status))
            .or_insert(0) += 1;
    }
    let header: Vec<String> = decision::STATES
        .iter()
        .map(|state| format!("{state} {}", counts.get(state).copied().unwrap_or(0)))
        .collect();
    let mut out = format!("decisions: {}\n", header.join(" · "));
    let mut shown = 0;
    for entry in decisions {
        let state_of = decision::normalize(&entry.status);
        if !state.is_empty() && state_of != state {
            continue;
        }
        shown += 1;
        let mut line = format!("- {} · {}", entry.title, state_of);
        if entry.is_principle() {
            line.push_str(" · principle");
        }
        if !entry.meta.review_trigger.is_empty() && decision::binding(state_of) {
            line.push_str(" · review when: ");
            line.push_str(&entry.meta.review_trigger);
        }
        if !entry.meta.superseded_by.is_empty()
            && let Some(next) = decision::find(decisions, &entry.meta.superseded_by)
        {
            line.push_str(" · superseded by ");
            line.push_str(&decision_handle(&next));
        }
        out.push_str(&line);
        out.push('\n');
    }
    if shown == 0 {
        out.push_str("(none)\n");
    }
    out
}

/// Renders one decision as a Polter block plus its chain and the Structure
/// nodes that cite it.
pub fn encode_decision(
    target: &decision::Decision,
    all: &[decision::Decision],
    node_paths: &[String],
) -> String {
    let mut node = DecisionNode {
        reference: decision_handle(target),
        body: target.body.clone(),
        ..DecisionNode::default()
    };
    if decision::number(&target.title).is_some() {
        node.title = decision::strip_number(&target.title);
    } else {
        node.title = target.title.clone();
    }
    let mut add = |key: &str, value: &str| {
        if !value.is_empty() {
            node.meta.push(super::model::MetaEntry::new(key, value, 0));
        }
    };
    add("status", decision::normalize(&target.status));
    add("trigger", &target.meta.review_trigger);
    add("scope", &target.scopes.join(", "));
    add("reason", &target.meta.status_reason);
    let mut out = String::new();
    out.push_str(super::encode::HEADER);
    out.push('\n');
    write_decision_node(&mut out, &node);
    let mut notes: Vec<String> = Vec::new();
    for entry in all {
        if entry.meta.superseded_by == target.id {
            notes.push(format!("supersedes: {}", decision_handle(entry)));
        }
    }
    if let Some(next) = decision::find(all, &target.meta.superseded_by) {
        notes.push(format!("superseded by: {}", decision_handle(&next)));
    }
    if !node_paths.is_empty() {
        notes.push(format!("governs: {}", node_paths.join(", ")));
    }
    if !notes.is_empty() {
        out.push('\n');
        out.push_str(&notes.join("\n"));
        out.push('\n');
    }
    out
}

/// The `### ref: Title` block of one decision.
///
/// TODO(dedupe): `encode.rs` keeps the private `write_decision` this copies; it
/// is not reachable from a sibling module.
fn write_decision_node(out: &mut String, decision: &DecisionNode) {
    if decision.title.is_empty() {
        out.push_str(&format!("### {}\n", decision.reference));
    } else {
        out.push_str(&format!("### {}: {}\n", decision.reference, decision.title));
    }
    let mut meta: Vec<String> = Vec::new();
    for key in super::model::DECISION_META_KEYS {
        if let Some(value) = decision.meta_value(key)
            && !value.is_empty()
        {
            meta.push(format!("{key}: {value}"));
        }
    }
    if !meta.is_empty() {
        out.push_str(&meta.join(" · "));
        out.push('\n');
    }
    if !decision.body.is_empty() {
        out.push('\n');
        out.push_str(&decision.body);
        out.push('\n');
    }
}

/// Returns the paths of active Structure nodes whose decisions: cite `target`.
pub fn nodes_citing(nodes: &[structure::Node], target: &decision::Decision) -> Vec<String> {
    let mut paths = Vec::new();
    for node in nodes {
        if node.status == structure::NODE_REMOVED {
            continue;
        }
        if node
            .decisions
            .iter()
            .any(|reference| super::lint::cites_decision(reference, &target.id, &target.title))
        {
            paths.push(node.path.clone());
        }
    }
    paths
}

/// One lifecycle change composed by the CLI and the Desktop.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DecisionInput {
    /// `new` or a Decision number, id, or title. Go calls this field `Ref`.
    pub reference: String,
    pub title: String,
    pub status: String,
    pub supersedes: String,
    pub trigger: String,
    pub reason: String,
    pub body: String,
}

/// Renders the Polter text for one decision change with the given change
/// summary and rationale. Metadata values are flattened so they cannot end
/// their line early.
pub fn compose_decision(input: &DecisionInput, summary: &str, rationale: &str) -> String {
    let mut node = DecisionNode {
        reference: input.reference.clone(),
        title: flatten(&input.title),
        body: input.body.trim().to_string(),
        ..DecisionNode::default()
    };
    let mut add = |key: &str, value: &str| {
        let value = flatten(value);
        if !value.is_empty() {
            node.meta.push(super::model::MetaEntry::new(key, value, 0));
        }
    };
    add("status", &input.status);
    add("supersedes", &input.supersedes);
    add("trigger", &input.trigger);
    add("reason", &input.reason);
    super::encode::format(&Document {
        summary: flatten(summary),
        rationale: flatten(rationale),
        decisions: vec![node],
        ..Document::default()
    })
}

/// Makes a value safe for a `key: value` metadata line.
pub fn flatten(value: &str) -> String {
    let value = value.split_whitespace().collect::<Vec<&str>>().join(" ");
    let value = value.replace(" · ", ", ");
    let value = value.replace(" • ", ", ");
    value.replace(" - ", " – ")
}

#[cfg(test)]
mod tests {
    use super::super::model::has_errors;
    use super::*;
    use crate::domains::polter::decode::{decode, has_diag};
    use crate::domains::polter::lint::{DecisionRef, References, lint};

    fn decision_snapshot() -> work::Snapshot {
        work::Snapshot {
            project_id: "p1".into(),
            nodes: vec![
                structure::Node {
                    id: "n-core".into(),
                    path: "core".into(),
                    level: structure::LEVEL_DOMAIN.into(),
                    status: structure::NODE_ACTIVE.into(),
                    ..structure::Node::default()
                },
                structure::Node {
                    id: "n-api".into(),
                    parent_id: "n-core".into(),
                    path: "core/api".into(),
                    level: structure::LEVEL_COMPONENT.into(),
                    status: structure::NODE_ACTIVE.into(),
                    ..structure::Node::default()
                },
            ],
            decisions: vec![
                decision::Decision {
                    id: "a1".into(),
                    title: "Decision 0001: SQLite".into(),
                    status: "accepted".into(),
                    ..decision::Decision::default()
                },
                decision::Decision {
                    id: "a2".into(),
                    title: "Decision 0002: Import".into(),
                    status: "active".into(),
                    scopes: vec!["core".into()],
                    ..decision::Decision::default()
                },
                decision::Decision {
                    id: "a3".into(),
                    title: "Decision 0003: Context".into(),
                    status: "proposed".into(),
                    ..decision::Decision::default()
                },
                decision::Decision {
                    id: "a4".into(),
                    title: "Decision 0004: Old".into(),
                    status: "superseded".into(),
                    meta: decision::Meta {
                        superseded_by: "a2".into(),
                        ..decision::Meta::default()
                    },
                    ..decision::Decision::default()
                },
            ],
            ..work::Snapshot::default()
        }
    }

    fn plan_decision_text(body: &str) -> (Vec<work::Op>, Vec<Diagnostic>) {
        let (doc, diags) = decode(&format!("---\npolter: 1\n---\n{body}"));
        assert!(!has_errors(&diags), "decode: {diags:?}");
        let snapshot = decision_snapshot();
        let nodes: HashMap<String, structure::Node> = snapshot
            .nodes
            .iter()
            .map(|node| (node.path.clone(), node.clone()))
            .collect();
        let mut diags = diags;
        let mut ops = Vec::new();
        plan_decisions(&doc, &snapshot.decisions, &nodes, &mut diags, &mut ops);
        (ops, diags)
    }

    #[test]
    fn decision_decode_and_format() {
        let text = "---\npolter: 1\n---\n\n### new: Local core\nstatus: active · trigger: multi-user sync\n\nOne process owns SQLite.\n";
        let (doc, diags) = decode(text);
        assert!(
            diags.is_empty() && doc.decisions.len() == 1,
            "doc = {doc:?}"
        );
        let got = &doc.decisions[0];
        assert_eq!(got.reference, "new", "ref");
        assert_eq!(got.title, "Local core", "title");
        assert_eq!(got.body, "One process owns SQLite.", "body");
        assert_eq!(got.meta_value("status"), Some("active"), "status");
        let (again, _) = decode(&super::super::encode::format(&doc));
        assert_eq!(
            super::super::encode::format(&again),
            super::super::encode::format(&doc),
            "round trip differs"
        );
    }

    #[test]
    fn decision_decode_errors() {
        let (_, diags) = decode("---\npolter: 1\n---\n### Decision-2\nstatus: dead\nowner: me\n");
        assert_eq!(diags.len(), 2, "diags = {diags:?}");
    }

    #[test]
    fn plan_new_decision_gets_next_number() {
        let (ops, diags) = plan_decision_text("### new: Bounded output\ntrigger: model change\n");
        assert!(
            !has_errors(&diags) && ops.len() == 1,
            "ops = {ops:?} diags = {diags:?}"
        );
        assert_eq!(ops[0].entity, work::ENTITY_DECISION, "entity");
        assert_eq!(ops[0].kind, work::OP_CREATE, "kind");
        assert_eq!(
            ops[0].fields["title"],
            json!("Decision 0005: Bounded output"),
            "title"
        );
        assert_eq!(
            ops[0].fields["status"],
            json!(decision::STATE_PROPOSED),
            "status"
        );
    }

    #[test]
    fn plan_decision_transitions() {
        let (ops, diags) = plan_decision_text("### Decision-0003\nstatus: active\n");
        assert!(
            !has_errors(&diags)
                && ops.len() == 1
                && ops[0].entity_id == "a3"
                && ops[0].fields["status"] == json!(decision::STATE_ACTIVE),
            "activate: ops = {ops:?} diags = {diags:?}"
        );
        // A legacy "accepted" row is already active: no change.
        let (ops, diags) = plan_decision_text("### Decision-0001\nstatus: active\n");
        assert!(
            !has_errors(&diags) && ops.is_empty(),
            "no-op: ops = {ops:?} diags = {diags:?}"
        );
        for body in [
            "### Decision-0003\nstatus: challenged · reason: x\n", // proposed cannot be challenged
            "### Decision-0004\nstatus: active\n",                 // superseded is final
            "### Decision-0002\nstatus: challenged\n",             // challenge needs a reason
            "### Decision-0002\nstatus: superseded\n",             // superseded only via successor
            "### Decision-0099\nstatus: active\n",                 // unknown
        ] {
            let (_, diags) = plan_decision_text(body);
            assert!(has_errors(&diags), "expected an error for {body:?}");
        }
        let (ops, diags) = plan_decision_text(
            "### Decision-0002\nstatus: challenged · reason: multi-user sync arrived\n",
        );
        assert!(
            !has_errors(&diags)
                && ops.len() == 1
                && ops[0].fields["status_reason"] == json!("multi-user sync arrived"),
            "challenge: ops = {ops:?} diags = {diags:?}"
        );
    }

    #[test]
    fn plan_supersede() {
        let (ops, diags) = plan_decision_text(
            "### new: Cloud store\nsupersedes: Decision-0001 · reason: teams need shared state\n",
        );
        assert!(
            !has_errors(&diags) && ops.len() == 2,
            "ops = {ops:?} diags = {diags:?}"
        );
        let (create, old) = (&ops[0], &ops[1]);
        assert_eq!(
            create.fields["status"],
            json!(decision::STATE_ACTIVE),
            "successor status"
        );
        assert_eq!(old.entity_id, "a1", "old entity");
        assert_eq!(
            old.fields["status"],
            json!(decision::STATE_SUPERSEDED),
            "old status"
        );
        assert_eq!(
            old.fields["superseded_by"],
            json!(create.entity_id),
            "old superseded_by"
        );
        // An existing proposal that supersedes is activated in the same batch.
        let (ops, diags) = plan_decision_text("### Decision-0003\nsupersedes: Decision-0001\n");
        assert!(
            !has_errors(&diags)
                && ops.len() == 2
                && ops[0].fields["status"] == json!(decision::STATE_ACTIVE),
            "existing successor: ops = {ops:?} diags = {diags:?}"
        );
        // A superseded decision cannot be the successor, and cannot be superseded twice.
        let (_, diags) = plan_decision_text("### Decision-0004\nsupersedes: Decision-0001\n");
        assert!(has_errors(&diags), "superseded successor should fail");
        let (_, diags) = plan_decision_text("### Decision-0003\nsupersedes: Decision-0004\n");
        assert!(
            has_errors(&diags),
            "superseding a superseded decision should fail"
        );
    }

    #[test]
    fn lint_hints_on_stale_decisions() {
        let references = References {
            decisions: vec![
                DecisionRef {
                    id: "a2".into(),
                    title: "Decision 0002: Import".into(),
                    status: "superseded".into(),
                },
                DecisionRef {
                    id: "a3".into(),
                    title: "Decision 0003: Context".into(),
                    status: "accepted".into(),
                },
            ],
            ..References::default()
        };
        let (_, diags) = lint(
            "---\npolter: 1\n---\n## one: One\ndecisions: Decision-0002, Decision-0003\n",
            Some(&references),
        );
        let hints: Vec<&Diagnostic> = diags
            .iter()
            .filter(|d| d.severity == super::super::model::SEVERITY_HINT)
            .filter(|d| d.message.contains("decision Decision"))
            .collect();
        assert!(
            hints.len() == 1 && hints[0].message.contains("superseded"),
            "hints = {hints:?} (diags {diags:?})"
        );
    }

    #[test]
    fn parses_decision_queries() {
        let query = super::super::format::parse_query("decisions challenged").expect("a query");
        assert_eq!(query.kind, super::super::format::QUERY_DECISIONS, "kind");
        assert_eq!(query.slug, "challenged", "slug");
        let query = super::super::format::parse_query("decision Decision 0007").expect("a query");
        assert_eq!(query.kind, super::super::format::QUERY_DECISION, "kind");
        assert_eq!(query.slug, "Decision 0007", "slug");
        assert!(
            super::super::format::parse_query("decisions bogus").is_err(),
            "unknown state should fail"
        );
    }

    #[test]
    fn encodes_decisions() {
        let snapshot = decision_snapshot();
        let out = encode_decisions(&snapshot.decisions, "");
        assert!(
            out.contains("active 2") && out.contains("superseded by Decision-0002"),
            "out = {out}"
        );
        let detail = encode_decision(
            &snapshot.decisions[1],
            &snapshot.decisions,
            &["core/import".to_string()],
        );
        assert!(
            detail.contains("### Decision-0002: Import")
                && detail.contains("supersedes: Decision-0004")
                && detail.contains("governs: core/import"),
            "detail = {detail}"
        );
    }

    #[test]
    fn compose_decision_flattens_values() {
        let text = compose_decision(
            &DecisionInput {
                reference: "Decision-0002".into(),
                status: "challenged".into(),
                reason: "cost · latency - both".into(),
                ..DecisionInput::default()
            },
            "Challenge",
            "why",
        );
        let (doc, diags) = decode(&text);
        assert!(
            diags.is_empty() && doc.decisions.len() == 1,
            "doc = {doc:?} diags = {diags:?}\n{text}"
        );
        assert_eq!(
            doc.decisions[0].meta_value("reason"),
            Some("cost, latency – both"),
            "reason"
        );
    }

    #[test]
    fn decision_contexts() {
        // scope: takes a list; the stored list is sorted and deduplicated.
        let (ops, diags) = plan_decision_text(
            "### new: Local first\nstatus: active · scope: core/api, core, core/api\n",
        );
        assert!(
            !has_errors(&diags)
                && ops.len() == 1
                && ops[0].fields["scope"] == json!(["core", "core/api"]),
            "ops = {ops:?} diags = {diags:?}"
        );
        let (ops, diags) = plan_decision_text("### Decision-0002\nscope: core, core/api\n");
        assert!(
            !has_errors(&diags)
                && ops.len() == 1
                && ops[0].fields["scope"] == json!(["core", "core/api"])
                && ops[0].before["scope"] == json!(["core"]),
            "update ops = {ops:?} diags = {diags:?}"
        );
        // Writing the stored contexts again changes nothing.
        let (ops, diags) = plan_decision_text("### Decision-0002\nscope: core\n");
        assert!(
            !has_errors(&diags) && ops.is_empty(),
            "unchanged ops = {ops:?} diags = {diags:?}"
        );
        // level: principle is still read: it clears the contexts.
        let (ops, diags) = plan_decision_text("### Decision-0002\nlevel: principle\n");
        assert!(
            !has_errors(&diags)
                && ops.len() == 1
                && ops[0].fields["scope"].as_array().is_some_and(Vec::is_empty),
            "principle ops = {ops:?} diags = {diags:?}"
        );
        let (_, diags) = plan_decision_text("### Decision-0002\nlevel: principle · scope: core\n");
        assert!(
            has_errors(&diags),
            "a principle with contexts must be rejected"
        );
        let (_, diags) = plan_decision_text("### Decision-0002\nscope: core, nowhere\n");
        assert!(
            has_errors(&diags),
            "an unknown context path must be rejected"
        );
        let (_, diags) = decode("---\npolter: 1\n---\n### Decision-0002\nlevel: pinned\n");
        assert!(has_errors(&diags), "an unknown level must be rejected");
        let scoped = decision::Decision {
            id: "a2".into(),
            title: "Decision 0002: Import".into(),
            status: "active".into(),
            scopes: vec!["core".into(), "core/api".into()],
            ..decision::Decision::default()
        };
        let text = encode_decision(&scoped, std::slice::from_ref(&scoped), &[]);
        assert!(
            text.contains("scope: core, core/api"),
            "encoded decision lacks its contexts:\n{text}"
        );
        let (doc, diags) = decode(&text);
        assert!(
            !has_errors(&diags) && doc.decisions.len() == 1,
            "round trip decode: {diags:?}"
        );
        assert_eq!(
            doc.decisions[0].meta_value("scope").map(parse_scopes),
            Some(scoped.scopes.clone()),
            "round trip scope"
        );
        let pinned = decision::Decision {
            id: "a5".into(),
            title: "Decision 0005: Local".into(),
            status: "active".into(),
            ..decision::Decision::default()
        };
        let got = encode_decisions(&[pinned, scoped], "");
        assert!(
            got.contains("Decision 0005: Local · active · principle")
                && !got.contains("Import · active · principle"),
            "decision list marks principles by having no contexts:\n{got}"
        );
    }

    #[test]
    fn unknown_supersede_target_is_refused() {
        let (_, diags) = plan_decision_text("### new: Cloud store\nsupersedes: Decision-0099\n");
        assert!(has_diag(&diags, 4, "unknown decision"), "diags = {diags:?}");
    }
}
