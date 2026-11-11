//! Port of `backend/compositions/wedge/scope.go`.

#[allow(unused_imports)]
use super::*;

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::Deserialize;

use crate::domains::inbox;
use crate::domains::polter;
use crate::domains::structure;
use crate::domains::work;
use crate::providers::sqlite::{
    self, ChangeRow, NodeScope, SCOPE_DECISION, SCOPE_FACT, SCOPE_ITEM, SCOPE_KNOWLEDGE,
    SCOPE_RULE, Store,
};

/// Derives the Inbox and keeps the items inside scope. An item's own contexts
/// win; otherwise a Question item takes its card's `under`. Everything else is
/// unscoped and stays.
pub fn list_inbox_scoped(
    store: &Store,
    project_id: &str,
    scope: &structure::MultiScope,
) -> sqlite::Result<inbox::Result> {
    let snapshot = store.load_work_snapshot(project_id)?;
    inbox_scoped(store, &snapshot, scope)
}

pub(crate) fn inbox_scoped(
    store: &Store,
    snapshot: &work::Snapshot,
    scope: &structure::MultiScope,
) -> sqlite::Result<inbox::Result> {
    let project_id = snapshot.project_id.clone();
    let result = list_inbox(store, snapshot)?;
    if !scope.active() {
        return Ok(result);
    }
    let resolved = store.load_scopes(&project_id, scope)?;
    let assigned = store.scope_map(&project_id, SCOPE_ITEM)?;
    let mut under: HashMap<String, String> = HashMap::new();
    for card in &snapshot.cards {
        under.insert(card.slug.clone(), card.under.clone());
    }
    let visible = |item: &inbox::Item| -> bool {
        if let Some(nodes) = assigned.get(&item.key) {
            return resolved.matches_any(nodes);
        }
        if item.kind == inbox::KIND_DECISION {
            let path = under.get(&item.card_slug).cloned().unwrap_or_default();
            return resolved.matches_path(&path);
        }
        true
    };
    let mut filtered = inbox::Result::default();
    for item in result.items {
        if visible(&item) {
            filtered.items.push(item);
        }
    }
    for routed in result.routed {
        if visible(&routed.item) {
            filtered.routed.push(routed);
        }
    }
    Ok(filtered)
}

/// Narrows the snapshot's cards and Decisions to `scope`. Structure nodes stay
/// whole so the map keeps its structure.
pub fn scope_snapshot(
    store: &Store,
    mut snapshot: work::Snapshot,
    scope: &structure::MultiScope,
) -> sqlite::Result<work::Snapshot> {
    if !scope.active() {
        return Ok(snapshot);
    }
    let resolved = store.load_scopes(&snapshot.project_id, scope)?;
    let scopes = store.scope_map(&snapshot.project_id, SCOPE_DECISION)?;
    snapshot.cards = filter_cards_by_scope(&resolved, &snapshot.cards);
    let mut kept = Vec::with_capacity(snapshot.decisions.len());
    for item in &snapshot.decisions {
        let contexts = scopes.get(&item.id).cloned().unwrap_or_default();
        if resolved.matches_any(&contexts) {
            kept.push(item.clone());
        }
    }
    snapshot.decisions = kept;
    Ok(snapshot)
}

/// Keeps the cards whose `under` path is inside scope. The resolved set may be
/// the union of several nodes. A card with no `under` belongs to the whole
/// project and stays.
pub fn filter_cards_by_scope(resolved: &NodeScope, cards: &[work::Card]) -> Vec<work::Card> {
    if resolved.set.is_nil() {
        return cards.to_vec();
    }
    cards
        .iter()
        .filter(|card| resolved.matches_path(&card.under))
        .cloned()
        .collect()
}

/// Keeps the facts with a context inside scope plus those with none.
pub fn filter_facts_by_scope(
    resolved: &NodeScope,
    scopes: &HashMap<String, Vec<String>>,
    facts: &[work::Fact],
) -> Vec<work::Fact> {
    if resolved.set.is_nil() {
        return facts.to_vec();
    }
    facts
        .iter()
        .filter(|fact| {
            let contexts = scopes.get(&fact.fact.id).cloned().unwrap_or_default();
            resolved.matches_any(&contexts)
        })
        .cloned()
        .collect()
}

/// Joins the legacy single node with the repeated list. Commas inside a value
/// split it. Blanks and duplicates are dropped. The single node comes first.
pub fn scope_refs(single: &str, many: &[String]) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<String> = Vec::new();
    let mut add = |reference: &str| {
        for part in reference.split(',') {
            let part = part.trim();
            if part.is_empty() || !seen.insert(part.to_string()) {
                continue;
            }
            out.push(part.to_string());
        }
    };
    add(single);
    for reference in many {
        add(reference);
    }
    out
}

/// Turns one node id, path, or slug into a [`structure::Scope`]. An empty ref
/// is the whole project. Removed nodes are unknown. The ref is one node: commas
/// are not separators here (assignment stays a single node).
pub fn resolve_scope_ref(
    store: &Store,
    project_id: &str,
    reference: &str,
    exact: bool,
) -> sqlite::Result<structure::Scope> {
    let reference = reference.trim();
    if reference.is_empty() {
        return Ok(structure::Scope::default());
    }
    let scope = resolve_scope_ids(store, project_id, &[reference.to_string()], exact)?;
    Ok(structure::Scope {
        node_id: scope.node_ids[0].clone(),
        exact,
    })
}

/// Turns node ids, paths, or slugs into one [`structure::MultiScope`]. An empty
/// list is the whole project. Commas inside a ref split it. The first unknown
/// ref is an error. Removed nodes are unknown.
pub fn resolve_scope_refs(
    store: &Store,
    project_id: &str,
    refs: &[String],
    exact: bool,
) -> sqlite::Result<structure::MultiScope> {
    resolve_scope_ids(store, project_id, &scope_refs("", refs), exact)
}

fn resolve_scope_ids(
    store: &Store,
    project_id: &str,
    refs: &[String],
    exact: bool,
) -> sqlite::Result<structure::MultiScope> {
    if refs.is_empty() {
        return Ok(structure::MultiScope::default());
    }
    let nodes = store.structure_nodes(project_id)?;
    let mut ids: Vec<String> = Vec::with_capacity(refs.len());
    let mut seen: HashSet<String> = HashSet::new();
    for reference in refs {
        let mut id = String::new();
        for node in &nodes {
            if node.status == structure::NODE_REMOVED {
                continue;
            }
            if node.id == *reference || node.path == *reference || node.slug == *reference {
                id = node.id.clone();
                break;
            }
        }
        if id.is_empty() {
            return Err(sqlite::Error::message(format!(
                "unknown structure node {:?}: list them with polterware structure",
                reference
            )));
        }
        if seen.insert(id.clone()) {
            ids.push(id);
        }
    }
    Ok(structure::MultiScope {
        node_ids: ids,
        exact,
    })
}

/// Resolves one scope reference against the project Structure.
pub fn load_node_scope(
    store: &Store,
    project_id: &str,
    reference: &str,
    exact: bool,
) -> sqlite::Result<NodeScope> {
    load_node_scopes(store, project_id, &[reference.to_string()], exact)
}

/// Resolves several scope references and returns their union.
pub fn load_node_scopes(
    store: &Store,
    project_id: &str,
    refs: &[String],
    exact: bool,
) -> sqlite::Result<NodeScope> {
    let scope = resolve_scope_refs(store, project_id, refs, exact)?;
    store.load_scopes(project_id, &scope)
}

/// Maps entity id (or inbox item key) to its context node ids for every
/// knowledge item, fact, Rule, and derived item that has any.
pub fn entity_scope_index(
    store: &Store,
    project_id: &str,
) -> sqlite::Result<HashMap<String, Vec<String>>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    for kind in [SCOPE_KNOWLEDGE, SCOPE_FACT, SCOPE_RULE, SCOPE_ITEM] {
        let mapped = store.scope_map(project_id, kind)?;
        for (id, nodes) in mapped {
            out.insert(id, nodes);
        }
    }
    Ok(out)
}

/// Keeps changes whose subjects are inside scope, plus changes that name no
/// scoped subject. Subjects are matched by the contexts already stored on
/// knowledge, facts, and evidence. A nil set matches everything.
pub fn filter_changes_by_scope(
    resolved: &NodeScope,
    scopes: &HashMap<String, Vec<String>>,
    rows: &[ChangeRow],
) -> Vec<ChangeRow> {
    if resolved.set.is_nil() {
        return rows.to_vec();
    }
    rows.iter()
        .filter(|row| change_in_scope(resolved, scopes, &row.payload_json))
        .cloned()
        .collect()
}

/// The part of a change payload scope filtering reads. Go unmarshals the whole
/// payload and keeps the subjects it has; a payload without them names nothing.
#[derive(Debug, Default, Deserialize)]
struct ScopeChangePayload {
    #[serde(default)]
    subject_ids: Vec<String>,
}

fn change_in_scope(
    resolved: &NodeScope,
    scopes: &HashMap<String, Vec<String>>,
    payload: &str,
) -> bool {
    let Ok(body) = serde_json::from_str::<ScopeChangePayload>(payload) else {
        return true;
    };
    if body.subject_ids.is_empty() {
        return true;
    }
    for id in &body.subject_ids {
        let contexts = scopes.get(id).cloned().unwrap_or_default();
        if resolved.matches_any(&contexts) {
            return true;
        }
    }
    false
}

/// Derives the project's Inbox. It only reads.
pub(crate) fn list_inbox(
    store: &Store,
    snapshot: &work::Snapshot,
) -> sqlite::Result<inbox::Result> {
    let mut conflicts: Vec<inbox::Conflict> = Vec::new();
    for conflict in &snapshot.conflicts {
        if conflict.state != work::CONFLICT_OPEN {
            continue;
        }
        let (rule, decision_title) = polter::conflict_against(conflict, snapshot);
        let mut details = BTreeMap::new();
        details.insert("conflict".to_string(), conflict.slug.clone());
        let mut against: Vec<String> = Vec::new();
        if !rule.id.is_empty() {
            details.insert("rule_id".to_string(), rule.id.clone());
            details.insert("rule".to_string(), rule.statement.clone());
            against.push(format!("Rule: {}", rule.statement));
        }
        if !decision_title.is_empty() {
            details.insert("decision_id".to_string(), conflict.decision.clone());
            details.insert("decision".to_string(), decision_title.clone());
            against.push(format!("Decision: {decision_title}"));
        }
        conflicts.push(inbox::Conflict {
            slug: conflict.slug.clone(),
            request: conflict.request.clone(),
            against: against.join("; "),
            card_slug: conflict.card_slug.clone(),
            options: conflict.options.clone(),
            details,
        });
    }
    let mut questions: Vec<inbox::Question> = Vec::new();
    for card in &snapshot.cards {
        if card.status != work::CARD_ACTIVE {
            continue;
        }
        for question in &card.questions {
            if question.state != work::QUESTION_OPEN {
                continue;
            }
            questions.push(inbox::Question {
                id: question.id.clone(),
                card_slug: card.slug.clone(),
                text: question.text.clone(),
                blocking: question.blocking,
                options: question.options.clone(),
            });
        }
    }
    let links = store.list_node_deploy_links(&snapshot.project_id)?;
    let connections = store.list_connections(&snapshot.project_id)?;
    let probes: Vec<inbox::ProviderConnection> = connections
        .into_iter()
        .map(|connection| inbox::ProviderConnection {
            provider_id: connection.provider_id,
            environment_id: connection.environment_id,
            status: connection.status,
        })
        .collect();
    let dispositions = store.list_inbox_dispositions(&snapshot.project_id)?;
    let mut deploys = inbox::failed_deploys(&links);
    deploys.extend(inbox::unconnected_deploys(&links, &probes));
    Ok(inbox::derive(
        &conflicts,
        &questions,
        &deploys,
        &dispositions,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::structure::ScopeSet;

    fn resolved_set(ids: &[&str]) -> NodeScope {
        let mut set = ScopeSet::empty();
        for id in ids {
            set.insert((*id).to_string());
        }
        NodeScope {
            set,
            nodes: Vec::new(),
        }
    }

    #[test]
    fn refs_merge_single_repeated_and_commas() {
        let got = scope_refs(
            " auth , billing ",
            &[
                "billing".to_string(),
                "login, auth".to_string(),
                String::new(),
                " invoices ".to_string(),
            ],
        );
        assert_eq!(got.join("|"), "auth|billing|login|invoices");
        assert!(scope_refs("", &[]).is_empty(), "empty refs");
    }

    #[test]
    fn filters_changes_by_scope() {
        let resolved = resolved_set(&["auth"]);
        let scopes: HashMap<String, Vec<String>> = HashMap::from([
            ("fact-auth".to_string(), vec!["auth".to_string()]),
            ("fact-bill".to_string(), vec!["billing".to_string()]),
            (
                "fact-both".to_string(),
                vec!["billing".to_string(), "auth".to_string()],
            ),
        ]);
        let row = |step: &str, payload: &str| ChangeRow {
            step_id: step.to_string(),
            agent_run_id: String::new(),
            project_id: "p".to_string(),
            actor_kind: String::new(),
            agent_label: String::new(),
            run_status: String::new(),
            run_task: String::new(),
            summary: String::new(),
            payload_json: payload.to_string(),
            created_at: time::OffsetDateTime::UNIX_EPOCH,
        };
        let rows = vec![
            row("open", r#"{"summary":"note"}"#),
            row("bad-json", "{"),
            row("unscoped", r#"{"subject_ids":["other"]}"#),
            row("in", r#"{"subject_ids":["fact-auth"]}"#),
            row("out", r#"{"subject_ids":["fact-bill"]}"#),
            row("either", r#"{"subject_ids":["fact-bill","fact-auth"]}"#),
            row("any-context", r#"{"subject_ids":["fact-both"]}"#),
        ];
        let filtered = filter_changes_by_scope(&resolved, &scopes, &rows);
        let kept: Vec<&str> = filtered.iter().map(|row| row.step_id.as_str()).collect();
        assert_eq!(
            kept.join("|"),
            "open|bad-json|unscoped|in|either|any-context"
        );
        assert_eq!(
            filter_changes_by_scope(&NodeScope::default(), &scopes, &rows).len(),
            rows.len(),
            "a nil set keeps every row"
        );
    }
}
