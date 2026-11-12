//! Compiling the candidate context items of a project under a token budget.

use std::collections::HashSet;

pub const DEFAULT_TOKEN_BUDGET: i32 = 1200;
pub const MAX_TOKEN_BUDGET: i32 = 8000;

/// Stable reason codes for why an item entered a compiled package.
pub const REASON_EXACT_ENTITY: &str = "exact_entity";
pub const REASON_KNOWLEDGE_LINK: &str = "knowledge_link";
pub const REASON_RELATIONSHIP_HOP: &str = "relationship_hop";
pub const REASON_FTS_HIT: &str = "fts_hit";
pub const REASON_TITLE_MATCH: &str = "title_match";
pub const REASON_PROJECT_SAMPLE: &str = "project_sample";
pub const REASON_FACT_FOR_ENTITY: &str = "fact_for_entity";
pub const REASON_FACT_QUERY_MATCH: &str = "fact_query_match";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Request {
    pub project_id: String,
    pub query: String,
    pub entity_id: String,
    pub token_budget: i32,
    /// Narrows candidates to one Structure node, its descendants and, unless
    /// [`Request::scope_exact`], its ancestors. [`Request::scope_node_ids`]
    /// adds more nodes; the package keeps the union. Empty is the whole
    /// project.
    pub scope_node_id: String,
    pub scope_node_ids: Vec<String>,
    pub scope_exact: bool,
}

impl Request {
    /// Returns the legacy node plus `scope_node_ids`, without blanks or
    /// duplicates.
    pub fn scope_ids(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for id in std::iter::once(&self.scope_node_id).chain(&self.scope_node_ids) {
            let id = id.trim();
            if id.is_empty() || !seen.insert(id) {
                continue;
            }
            out.push(id.to_string());
        }
        out
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Item {
    pub kind: String,
    pub id: String,
    pub title: String,
    pub excerpt: String,
    pub truth_class: String,
    pub source: String,
    pub path: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Package {
    pub project_id: String,
    pub query: String,
    pub entity_id: String,
    pub token_budget: i32,
    pub tokens_used: i32,
    pub items: Vec<Item>,
    pub truncated: bool,
}

pub fn normalize_budget(budget: i32) -> i32 {
    if budget <= 0 {
        return DEFAULT_TOKEN_BUDGET;
    }
    if budget > MAX_TOKEN_BUDGET {
        return MAX_TOKEN_BUDGET;
    }
    budget
}

pub fn estimate_tokens(text: &str) -> i32 {
    let n = text.chars().count() as i32;
    if n == 0 {
        return 0;
    }
    // Rough English heuristic: ~4 characters per token.
    let tokens = (n + 3) / 4;
    tokens.max(1)
}

pub fn excerpt(body: &str, max_runes: i32) -> String {
    let body = body.trim();
    let max_runes = if max_runes <= 0 {
        240
    } else {
        max_runes as usize
    };
    let mut out: String = body.chars().take(max_runes).collect();
    if body.chars().count() > max_runes {
        out.push('…');
    }
    out
}

fn item_token_cost(item: &Item) -> i32 {
    let mut cost = estimate_tokens(&item.title) + estimate_tokens(&item.excerpt) + 8;
    for reason in &item.reasons {
        cost += estimate_tokens(reason);
    }
    cost
}

/// Packs candidates under a token budget. Exact entity matches rank first,
/// then facts, linked knowledge, then search hits. Declared truth outranks
/// inferred.
pub fn compile(req: &Request, candidates: &[Item]) -> Package {
    let budget = normalize_budget(req.token_budget);
    let ranked = rank(candidates);
    let mut out = Package {
        project_id: req.project_id.clone(),
        query: req.query.clone(),
        entity_id: req.entity_id.clone(),
        token_budget: budget,
        items: Vec::with_capacity(ranked.len()),
        ..Package::default()
    };
    let mut used = 0;
    for mut item in ranked {
        let mut cost = item_token_cost(&item);
        if used + cost > budget && !out.items.is_empty() {
            out.truncated = true;
            break;
        }
        if used + cost > budget {
            item.excerpt = excerpt(&item.excerpt, 80);
            cost = item_token_cost(&item);
            if used + cost > budget {
                out.truncated = true;
                break;
            }
        }
        out.items.push(item);
        used += cost;
    }
    out.tokens_used = used;
    out
}

fn rank(items: &[Item]) -> Vec<Item> {
    let mut out = items.to_vec();
    for i in 0..out.len() {
        for j in i + 1..out.len() {
            if rank_score(&out[j]) > rank_score(&out[i]) {
                out.swap(i, j);
            }
        }
    }
    out
}

fn rank_score(item: &Item) -> i32 {
    let mut score = 0;
    match item.truth_class.as_str() {
        "verified" => score += 45,
        "declared" => score += 40,
        "observed" => score += 30,
        "operational" => score += 20,
        "inferred" => score += 10,
        _ => {}
    }
    match item.kind.as_str() {
        "entity" => score += 50,
        "fact" => score += 45,
        "decision" => score += 35,
        "doc" => score += 30,
        _ => {}
    }
    score
}

/// Appends unique reason codes in order.
pub fn merge_reasons(existing: &[String], added: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(existing.len() + added.len());
    for reason in existing.iter().chain(added) {
        if reason.is_empty() {
            continue;
        }
        if !seen.insert(reason) {
            continue;
        }
        out.push(reason.clone());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repeated(s: &str, n: usize) -> String {
        s.repeat(n)
    }

    #[test]
    fn compile_respects_budget() {
        let items = [
            Item {
                kind: "entity".to_string(),
                id: "e1".to_string(),
                title: "API Gateway".to_string(),
                excerpt: repeated("x", 400),
                truth_class: "declared".to_string(),
                source: "layout".to_string(),
                reasons: vec![REASON_EXACT_ENTITY.to_string()],
                ..Item::default()
            },
            Item {
                kind: "doc".to_string(),
                id: "d1".to_string(),
                title: "Architecture".to_string(),
                excerpt: repeated("y", 400),
                truth_class: "declared".to_string(),
                source: "knowledge".to_string(),
                reasons: vec![REASON_FTS_HIT.to_string()],
                ..Item::default()
            },
            Item {
                kind: "doc".to_string(),
                id: "d2".to_string(),
                title: "Extra".to_string(),
                excerpt: repeated("z", 400),
                truth_class: "inferred".to_string(),
                source: "knowledge".to_string(),
                reasons: vec![REASON_FTS_HIT.to_string()],
                ..Item::default()
            },
        ];
        let pkg = compile(
            &Request {
                project_id: "p1".to_string(),
                token_budget: 200,
                ..Request::default()
            },
            &items,
        );
        assert!(!pkg.items.is_empty(), "expected at least one item");
        assert!(
            pkg.tokens_used <= pkg.token_budget,
            "over budget: {} > {}",
            pkg.tokens_used,
            pkg.token_budget
        );
        assert_eq!(pkg.items[0].kind, "entity", "expected entity first");
    }

    #[test]
    fn compile_preserves_reasons() {
        let items = [Item {
            kind: "fact".to_string(),
            id: "f1".to_string(),
            title: "Uses Go modules".to_string(),
            excerpt: "module example".to_string(),
            truth_class: "declared".to_string(),
            source: "polterware".to_string(),
            path: "canon/fact/f1".to_string(),
            reasons: vec![REASON_PROJECT_SAMPLE.to_string()],
        }];
        let pkg = compile(
            &Request {
                project_id: "p1".to_string(),
                token_budget: 500,
                ..Request::default()
            },
            &items,
        );
        assert_eq!(pkg.items.len(), 1, "expected 1 item");
        assert_eq!(
            pkg.items[0].reasons,
            vec![REASON_PROJECT_SAMPLE.to_string()]
        );
    }

    #[test]
    fn fact_outranks_doc() {
        let items = [
            Item {
                kind: "doc".to_string(),
                id: "d1".to_string(),
                title: "Doc".to_string(),
                excerpt: "body".to_string(),
                truth_class: "declared".to_string(),
                source: "knowledge".to_string(),
                ..Item::default()
            },
            Item {
                kind: "fact".to_string(),
                id: "f1".to_string(),
                title: "Fact".to_string(),
                excerpt: "claim".to_string(),
                truth_class: "declared".to_string(),
                source: "polterware".to_string(),
                ..Item::default()
            },
        ];
        let pkg = compile(
            &Request {
                project_id: "p1".to_string(),
                token_budget: 800,
                ..Request::default()
            },
            &items,
        );
        assert!(pkg.items.len() >= 2, "expected both items");
        assert_eq!(pkg.items[0].kind, "fact", "expected fact first");
    }

    #[test]
    fn budget_trims_heavy_items() {
        let heavy = Item {
            kind: "fact".to_string(),
            id: "f1".to_string(),
            title: "Heavy".to_string(),
            excerpt: repeated("a", 800),
            truth_class: "declared".to_string(),
            source: "polterware".to_string(),
            reasons: vec![REASON_PROJECT_SAMPLE.to_string()],
            ..Item::default()
        };
        let light = Item {
            kind: "entity".to_string(),
            id: "e1".to_string(),
            title: "API".to_string(),
            excerpt: "svc".to_string(),
            truth_class: "declared".to_string(),
            source: "layout".to_string(),
            reasons: vec![REASON_EXACT_ENTITY.to_string()],
            ..Item::default()
        };
        let pkg = compile(
            &Request {
                project_id: "p1".to_string(),
                token_budget: item_token_cost(&light) + 40,
                ..Request::default()
            },
            &[heavy, light],
        );
        assert!(!pkg.items.is_empty(), "expected at least the light item");
        assert_eq!(
            pkg.items[0].kind, "entity",
            "expected entity packed first under tight budget"
        );
    }

    #[test]
    fn merge_reasons_dedupes() {
        let got = merge_reasons(
            &[REASON_FTS_HIT.to_string(), REASON_TITLE_MATCH.to_string()],
            &[
                REASON_FTS_HIT.to_string(),
                REASON_PROJECT_SAMPLE.to_string(),
            ],
        );
        assert_eq!(got.len(), 3, "expected 3 unique reasons");
        assert_eq!(
            got,
            vec![
                REASON_FTS_HIT.to_string(),
                REASON_TITLE_MATCH.to_string(),
                REASON_PROJECT_SAMPLE.to_string()
            ]
        );
    }
}
