//! The one read before an agent works.
//!
//! The one read before an agent works: a card or a Structure node with its
//! Rules, open work, milestone, canvas links, and docs, deterministic and packed
//! under a token budget.

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::domains::{canvas, context, structure, work};

use super::structure_encode::{checkout_line, kind_tech_line};
use super::*;

/// The error [`brief`] returns when a target is neither a card slug nor a
/// Structure node path or id. The caller may still resolve it as a file path
/// and try again with the node it falls under.
pub const ERR_UNKNOWN_BRIEF_TARGET: &str = "unknown brief target";

/// The errors a brief returns, one per Go sentinel error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BriefError {
    /// The target named no active card and no active Structure node.
    UnknownTarget(String),
}

impl fmt::Display for BriefError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownTarget(target) => write!(
                f,
                "{ERR_UNKNOWN_BRIEF_TARGET} {}: use a card slug, a Structure path, or a file path (polterware search <terms> finds them)",
                go_quote(target)
            ),
        }
    }
}

impl std::error::Error for BriefError {}

/// What a brief draws from. The caller loads it; a brief only reads it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BriefInput {
    pub snapshot: work::Snapshot,
    /// The project's active docs. A brief lists only those with a context on
    /// the target's node or one of its ancestors.
    pub docs: Vec<Doc>,
    /// The agent-visible canvases, with linked nodes already carrying their
    /// Structure title.
    pub canvases: Vec<canvas::Canvas>,
}

/// One doc with the Structure nodes it is about. Search reads the body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Doc {
    pub id: String,
    pub title: String,
    pub doc_type: String,
    pub scope_node_ids: Vec<String>,
    pub body: String,
}

/// Renders what an agent reads before it works on a card or a Structure node,
/// most important first: the target with its kind and technology; on a card its
/// spec (behavior, edge cases, plan), its acceptance criteria, and its open
/// tasks and questions with the last handoff; then the Rules that hold for it
/// (its node's, its ancestors', and the project's), its milestone goal, canvas
/// links, and doc titles. It is deterministic. A tight budget cuts lines from
/// the end, so the lowest-priority blocks go first, and the last line says what
/// was cut; the target and the Rules are never cut.
pub fn brief(input: &BriefInput, target: &str, budget: i32) -> Result<String, BriefError> {
    let budget = context::normalize_budget(budget);
    let target = target.trim();
    let Some((card, node)) = resolve_brief_target(&input.snapshot, target) else {
        return Err(BriefError::UnknownTarget(target.to_string()));
    };
    let mut focus = node;
    if let Some(card) = card {
        focus = active_node(&input.snapshot.nodes, &card.under);
    }
    let chain = match focus {
        Some(node) => ancestor_chain(&input.snapshot.nodes, node),
        None => Vec::new(),
    };
    let builder = BriefBuilder {
        input,
        card,
        focus,
        chain,
    };
    Ok(pack_brief(&builder.blocks(), budget, &[0, 4]))
}

/// Finds a card by slug, then a node by path or id.
#[allow(clippy::type_complexity)]
fn resolve_brief_target<'a>(
    snapshot: &'a work::Snapshot,
    target: &str,
) -> Option<(Option<&'a work::Card>, Option<&'a structure::Node>)> {
    if target.is_empty() {
        return None;
    }
    if let Some(card) = snapshot
        .cards
        .iter()
        .find(|card| card.status == work::CARD_ACTIVE && card.slug == target)
    {
        return Some((Some(card), None));
    }
    if let Some(node) = snapshot.nodes.iter().find(|node| {
        node.status != structure::NODE_REMOVED && (node.path == target || node.id == target)
    }) {
        return Some((None, Some(node)));
    }
    None
}

struct BriefBuilder<'a> {
    input: &'a BriefInput,
    card: Option<&'a work::Card>,
    /// The target node, or the node the card sits under.
    focus: Option<&'a structure::Node>,
    /// `focus`'s ancestors, nearest first.
    chain: Vec<structure::Node>,
}

impl BriefBuilder<'_> {
    fn blocks(&self) -> Vec<Vec<String>> {
        vec![
            self.target(),
            self.spec(),
            self.invariants(),
            self.open_work(),
            self.rules(),
            self.conflicts(),
            self.milestone(),
            self.canvas(),
            self.docs(),
        ]
    }

    /// The card's spec: what it does (`behavior:`), the cases it must handle
    /// (`edge-cases:`), and the technical plan (`plan:`), in that order.
    fn spec(&self) -> Vec<String> {
        let Some(card) = self.card else {
            return Vec::new();
        };
        let mut lines: Vec<String> = Vec::new();
        for (label, body) in [
            ("Behavior:", &card.behavior),
            ("Edge cases:", &card.edge_cases),
            ("Plan:", &card.plan),
        ] {
            let body = body.trim();
            if body.is_empty() {
                continue;
            }
            if !lines.is_empty() {
                lines.push(String::new());
            }
            lines.push(label.to_string());
            lines.extend(body.split('\n').map(str::to_string));
        }
        lines
    }

    fn target(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        if let Some(card) = self.card {
            lines.push(format!("## {}: {}", card.slug, card.title));
            let mut meta = vec![format!("stage: {}", card.stage)];
            if !card.under.is_empty() {
                meta.push(format!("under: {}", card.under));
            }
            if !card.milestone.is_empty() {
                meta.push(format!("milestone: {}", card.milestone));
            }
            let claim = open_claim(card);
            if !claim.is_empty() {
                meta.push(format!("claim: {claim}"));
            }
            lines.push(meta.join(" · "));
            let line = node_meta(self.focus, false);
            if !line.is_empty() {
                lines.push(line);
            }
            if !card.intent.is_empty() {
                lines.push(card.intent.clone());
            }
            return lines;
        }
        let Some(node) = self.focus else {
            return lines;
        };
        lines.push(format!("# {}: {}", node.path, node.title));
        lines.push(node_meta(Some(node), true));
        if !node.intent.is_empty() {
            lines.push(node.intent.clone());
        }
        lines
    }

    /// The card's Conflicts: open ones wait for a person, and a decided one
    /// carries the answer to follow.
    fn conflicts(&self) -> Vec<String> {
        let Some(card) = self.card else {
            return Vec::new();
        };
        let mut lines: Vec<String> = Vec::new();
        for conflict in &self.input.snapshot.conflicts {
            if conflict.card_slug != card.slug {
                continue;
            }
            match conflict.state.as_str() {
                work::CONFLICT_OPEN => lines.push(format!(
                    "- {}: {} (open: wait for a person)",
                    conflict.slug, conflict.request
                )),
                work::CONFLICT_DECIDED => lines.push(format!(
                    "- {}: {} -> {}",
                    conflict.slug, conflict.request, conflict.answer
                )),
                _ => {}
            }
        }
        if lines.is_empty() {
            return Vec::new();
        }
        let mut out = vec!["Conflicts:".to_string()];
        out.extend(lines);
        out
    }

    fn open_work(&self) -> Vec<String> {
        if let Some(card) = self.card {
            let mut lines = brief_tasks(card);
            lines.extend(brief_questions(card, ""));
            if let Some(handoff) = card.latest_handoff_any() {
                lines.push(handoff_line(handoff));
            }
            return lines;
        }
        let Some(node) = self.focus else {
            return Vec::new();
        };
        let mut lines: Vec<String> = Vec::new();
        let mut questions: Vec<String> = Vec::new();
        for card in cards_within(&self.input.snapshot, node) {
            if card.stage == work::STAGE_DELIVERED {
                continue;
            }
            let mut line = format!("- {}  {}  {}", card.slug, card.stage, go_quote(&card.title));
            let open = brief_questions(card, "").len();
            if open > 0 {
                line.push_str(&format!("  {open} open?"));
            }
            lines.push(line);
            for question in brief_questions(card, &card.slug) {
                if question.contains("[blocking]") {
                    questions.push(question);
                }
            }
        }
        lines.extend(questions);
        lines
    }

    /// Every Rule that holds for the target: the focus node's own, its
    /// ancestors' (nearest first, marked with the node they come from), then
    /// the project-wide ones. Agents must follow them.
    fn rules(&self) -> Vec<String> {
        let paths = self.chain_paths();
        let (own, project) = rules_for(&self.input.snapshot.rules, &paths);
        let mut lines: Vec<String> = Vec::new();
        for (i, rules) in own.iter().enumerate() {
            for rule in rules {
                if i == 0 && self.card.is_none() {
                    lines.push(format!("- {}", rule.statement));
                } else {
                    lines.push(format!("- {} (from {})", rule.statement, paths[i]));
                }
            }
        }
        for rule in &project {
            lines.push(format!("- {} (whole project)", rule.statement));
        }
        // A Rule the card cites by `rules:` binds it even outside its chain.
        if let Some(card) = self.card {
            let shown: Vec<&str> = own
                .iter()
                .flatten()
                .chain(project.iter())
                .map(|rule| rule.id.as_str())
                .collect();
            for rule in &self.input.snapshot.rules {
                if !rule.binding() || shown.contains(&rule.id.as_str()) {
                    continue;
                }
                if card.decisions.iter().any(|reference| {
                    super::lint::cites_decision(reference, &rule.id, &rule.slug, &rule.statement)
                }) {
                    lines.push(format!("- {} (cited)", rule.statement));
                }
            }
        }
        if lines.is_empty() {
            return Vec::new();
        }
        let mut out = vec!["Rules:".to_string()];
        out.extend(lines);
        out
    }

    /// The card's acceptance criteria.
    fn invariants(&self) -> Vec<String> {
        let Some(card) = self.card else {
            return Vec::new();
        };
        let mut lines: Vec<String> = Vec::new();
        for invariant in &card.invariants {
            if invariant.state != work::INVARIANT_REMOVED {
                lines.push(format!("! {}", invariant.text));
            }
        }
        if lines.is_empty() {
            return Vec::new();
        }
        let mut out = vec!["Acceptance:".to_string()];
        out.extend(lines);
        out
    }

    fn milestone(&self) -> Vec<String> {
        let Some(card) = self.card else {
            return Vec::new();
        };
        if card.milestone_id.is_empty() {
            return Vec::new();
        }
        for milestone in &self.input.snapshot.milestones {
            if milestone.id != card.milestone_id {
                continue;
            }
            let mut line = format!(
                "milestone {}: {} · {}",
                milestone.slug, milestone.name, milestone.state
            );
            if !milestone.target_date.is_empty() {
                line.push_str(" · target ");
                line.push_str(&milestone.target_date);
            }
            let mut lines = vec![line];
            if !milestone.goal.is_empty() {
                lines.push(format!("goal: {}", milestone.goal));
            }
            if !milestone.exit_criteria.is_empty() {
                lines.push(format!("exit: {}", milestone.exit_criteria.join("; ")));
            }
            return lines;
        }
        Vec::new()
    }

    /// Per agent-visible canvas, the links of the canvas nodes that cite the
    /// focus node.
    fn canvas(&self) -> Vec<String> {
        let Some(focus) = self.focus else {
            return Vec::new();
        };
        let mut lines: Vec<String> = Vec::new();
        for item in &self.input.canvases {
            let mut labels: HashMap<String, String> = HashMap::new();
            let mut linked: HashSet<String> = HashSet::new();
            for node in &item.graph.nodes {
                labels.insert(node.id.clone(), node.label.clone());
                if node
                    .r#ref
                    .as_ref()
                    .is_some_and(|item| item.kind == canvas::REF_NODE && item.id == focus.id)
                {
                    linked.insert(node.id.clone());
                }
            }
            if linked.is_empty() {
                continue;
            }
            let mut edges = 0;
            for edge in &item.graph.edges {
                if !linked.contains(&edge.from) && !linked.contains(&edge.to) {
                    continue;
                }
                edges += 1;
                let from = labels.get(&edge.from).map_or("", String::as_str);
                let to = labels.get(&edge.to).map_or("", String::as_str);
                lines.push(format!(
                    "- {}: {} --{}--> {}",
                    item.name, from, edge.kind, to
                ));
            }
            if edges == 0 {
                lines.push(format!("- {}: {} (no links)", item.name, focus.title));
            }
        }
        if lines.is_empty() {
            return Vec::new();
        }
        let mut out = vec!["Canvas:".to_string()];
        out.extend(lines);
        out
    }

    fn docs(&self) -> Vec<String> {
        let Some(focus) = self.focus else {
            return Vec::new();
        };
        let mut ids = vec![focus.id.clone()];
        for node in &self.chain {
            ids.push(node.id.clone());
        }
        let mut lines: Vec<String> = Vec::new();
        let mut seen: HashSet<usize> = HashSet::new();
        for id in &ids {
            for (i, doc) in self.input.docs.iter().enumerate() {
                if seen.contains(&i) || !doc.scope_node_ids.iter().any(|scope| scope == id) {
                    continue;
                }
                seen.insert(i);
                lines.push(format!("- {}", doc.title));
            }
        }
        if lines.is_empty() {
            return Vec::new();
        }
        let mut out = vec!["Docs:".to_string()];
        out.extend(lines);
        out
    }

    /// The focus path and its ancestors' paths, nearest first.
    fn chain_paths(&self) -> Vec<String> {
        let Some(focus) = self.focus else {
            return Vec::new();
        };
        let mut paths = vec![focus.path.clone()];
        for node in &self.chain {
            paths.push(node.path.clone());
        }
        paths
    }
}

/// `level: component · kind: cli · tech: Go · repo: api · path: src`. The level
/// is left out for the node a card sits under.
fn node_meta(node: Option<&structure::Node>, with_level: bool) -> String {
    let Some(node) = node else {
        return String::new();
    };
    let mut parts: Vec<String> = Vec::new();
    if with_level {
        parts.push(format!("level: {}", node.level));
    }
    for line in [kind_tech_line(node), checkout_line(node)] {
        if !line.is_empty() {
            parts.push(line);
        }
    }
    parts.join(" · ")
}

/// The first open task's claim: `holder`, or `holder (scope)`.
fn open_claim(card: &work::Card) -> String {
    for task in &card.tasks {
        let Some(claim) = &task.claim else {
            continue;
        };
        if task.state != work::TASK_OPEN {
            continue;
        }
        if !claim.scope.is_empty() {
            return format!("{} ({})", claim.holder, claim.scope);
        }
        return claim.holder.clone();
    }
    String::new()
}

fn brief_tasks(card: &work::Card) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for task in &card.tasks {
        if task.state != work::TASK_OPEN {
            continue;
        }
        let mut line = "- [ ] ".to_string();
        if task.number > 0 {
            line.push_str(&format!("T{} ", task.number));
        }
        line.push_str(&task.title);
        if !task.repo.is_empty() {
            line.push_str(&format!(" [repo: {}]", task.repo));
        }
        if !task.scope.is_empty() {
            line.push_str(&format!(" `{}`", task.scope));
        }
        lines.push(line);
    }
    lines
}

/// The open questions, blocking ones first; `from` names the card when the
/// brief is about a node.
fn brief_questions(card: &work::Card, from: &str) -> Vec<String> {
    let mut blocking: Vec<String> = Vec::new();
    let mut other: Vec<String> = Vec::new();
    for question in &card.questions {
        if question.state != work::QUESTION_OPEN {
            continue;
        }
        let mut line = format!("? {}", question.text);
        if question.blocking {
            line.push_str(" [blocking]");
        }
        if !question.options.is_empty() {
            line.push_str(&format!(" ({})", question.options.join(" | ")));
        }
        if !from.is_empty() {
            line.push_str(&format!(" — {from}"));
        }
        if question.blocking {
            blocking.push(line);
        } else {
            other.push(line);
        }
    }
    blocking.extend(other);
    blocking
}

fn handoff_line(handoff: &work::Handoff) -> String {
    let mut line = "handoff".to_string();
    if !handoff.agent.is_empty() {
        line.push_str(&format!(" {}", handoff.agent));
    }
    if !handoff.branch.is_empty() {
        line.push_str(&format!(" on {}", handoff.branch));
    }
    let mut parts: Vec<String> = Vec::new();
    for (key, value) in [
        ("next", &handoff.next),
        ("in progress", &handoff.in_progress),
    ] {
        if !value.is_empty() {
            parts.push(format!("{key}: {value}"));
        }
    }
    if parts.is_empty() && !handoff.goal.is_empty() {
        parts.push(format!("goal: {}", handoff.goal));
    }
    if parts.is_empty() {
        return line;
    }
    format!("{line}: {}", parts.join(" · "))
}

/// The active cards under `node` or any node below it.
fn cards_within<'a>(snapshot: &'a work::Snapshot, node: &structure::Node) -> Vec<&'a work::Card> {
    let prefix = format!("{}/", node.path);
    snapshot
        .cards
        .iter()
        .filter(|card| {
            card.status == work::CARD_ACTIVE
                && (card.under == node.path || card.under.starts_with(&prefix))
        })
        .collect()
}

/// The active node at `path`.
fn active_node<'a>(nodes: &'a [structure::Node], path: &str) -> Option<&'a structure::Node> {
    nodes
        .iter()
        .find(|node| node.status != structure::NODE_REMOVED && node.path == path)
}

/// The opening sentence of a body, up to 240 runes; Markdown heading lines are
/// skipped.
#[allow(dead_code)]
fn first_sentence(body: &str) -> String {
    let mut prose: Vec<&str> = Vec::new();
    for line in body.split('\n') {
        if !line.trim().starts_with('#') {
            prose.push(line);
        }
    }
    let joined = prose.join(" ");
    let text = joined.split_whitespace().collect::<Vec<_>>().join(" ");
    let text = match text.find(". ") {
        Some(i) => &text[..=i],
        None => text.as_str(),
    };
    context::excerpt(text, 240)
}

/// Joins the blocks with blank lines and, over budget, drops lines from the
/// end of the last block that is not kept. The blocks in `keep` (the target and
/// the Rules) always stay whole, and a block header or blank line is never left
/// dangling.
pub(super) fn pack_brief(blocks: &[Vec<String>], budget: i32, keep: &[usize]) -> String {
    let keep_block = |i: usize| keep.contains(&i);
    let render = |blocks: &[Vec<String>]| {
        let mut lines: Vec<String> = Vec::new();
        for block in blocks.iter().filter(|block| !block.is_empty()) {
            if !lines.is_empty() {
                lines.push(String::new());
            }
            lines.extend(block.iter().cloned());
        }
        lines
    };
    let total: usize = blocks.iter().map(|block| block.len()).sum();
    let mut body: Vec<Vec<String>> = blocks.to_vec();
    let lines = render(&body);
    let mut text = format!("{}\n", lines.join("\n"));
    if context::estimate_tokens(&text) <= budget {
        return text;
    }
    let floor = lines.len().min(2);
    loop {
        let Some(i) = (0..body.len())
            .rev()
            .find(|&i| !body[i].is_empty() && !keep_block(i))
        else {
            return text;
        };
        body[i].pop();
        while body[i]
            .last()
            .is_some_and(|line| line.is_empty() || (line.ends_with(':') && !line.contains(' ')))
        {
            body[i].pop();
        }
        let lines = render(&body);
        if lines.len() <= floor {
            return text;
        }
        let kept_lines: usize = body.iter().map(|block| block.len()).sum();
        let cut = total.saturating_sub(kept_lines);
        let note = format!("({cut} more lines cut to fit budget:{budget})");
        text = format!("{}\n{note}\n", lines.join("\n"));
        if context::estimate_tokens(&text) <= budget {
            return text;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::polter::search::search;

    fn brief_fixture() -> BriefInput {
        let system = structure::Node {
            id: "s".into(),
            path: "app".into(),
            level: structure::LEVEL_DOMAIN.into(),
            title: "App".into(),
            status: structure::NODE_ACTIVE.into(),
            ..structure::Node::default()
        };
        let api = structure::Node {
            id: "c".into(),
            parent_id: "s".into(),
            path: "app/api".into(),
            level: structure::LEVEL_COMPONENT.into(),
            title: "API".into(),
            status: structure::NODE_ACTIVE.into(),
            kind: "service".into(),
            tech: "Go".into(),
            folder: "backend".into(),
            decisions: vec!["Decision-0002".into()],
            ..structure::Node::default()
        };
        let db = structure::Node {
            id: "d".into(),
            parent_id: "s".into(),
            path: "app/db".into(),
            level: structure::LEVEL_COMPONENT.into(),
            title: "DB".into(),
            status: structure::NODE_ACTIVE.into(),
            ..structure::Node::default()
        };
        let card = work::Card {
            id: "card-1".into(),
            slug: "login".into(),
            title: "Login".into(),
            stage: work::STAGE_BUILD.into(),
            status: work::CARD_ACTIVE.into(),
            under: "app/api".into(),
            intent: "Sign people in.".into(),
            behavior: "Sign people in with a session cookie.".into(),
            edge_cases: "The cookie expires every hour.".into(),
            plan: "A session table and a login route.".into(),
            decisions: vec!["Decision-0001".into()],
            milestone_id: "m1".into(),
            milestone: "beta".into(),
            tasks: vec![
                work::Task {
                    number: 1,
                    title: "Session table".into(),
                    state: work::TASK_DONE.into(),
                    ..work::Task::default()
                },
                work::Task {
                    number: 2,
                    title: "Login route".into(),
                    state: work::TASK_OPEN.into(),
                    claim: Some(work::Claim {
                        holder: "codex".into(),
                        ..work::Claim::default()
                    }),
                    ..work::Task::default()
                },
            ],
            questions: vec![
                work::Question {
                    text: "Remember me?".into(),
                    state: work::QUESTION_OPEN.into(),
                    ..work::Question::default()
                },
                work::Question {
                    text: "Which hash?".into(),
                    blocking: true,
                    state: work::QUESTION_OPEN.into(),
                    options: vec!["argon2".into(), "bcrypt".into()],
                    ..work::Question::default()
                },
            ],
            invariants: vec![work::Invariant {
                text: "Rate limit attempts".into(),
                state: work::INVARIANT_ACTIVE.into(),
                ..work::Invariant::default()
            }],
            handoffs: vec![work::Handoff {
                agent: "codex".into(),
                branch: "feat/login".into(),
                next: "wire the route".into(),
                ..work::Handoff::default()
            }],
            ..work::Card::default()
        };
        let old = work::Card {
            slug: "old".into(),
            title: "Old".into(),
            stage: work::STAGE_DELIVERED.into(),
            status: work::CARD_ACTIVE.into(),
            under: "app/api".into(),
            ..work::Card::default()
        };
        let rule = |id: &str, statement: &str, scopes: &[&str]| work::Rule {
            id: id.into(),
            statement: statement.into(),
            scopes: scopes.iter().map(|scope| scope.to_string()).collect(),
            ..work::Rule::default()
        };
        BriefInput {
            snapshot: work::Snapshot {
                nodes: vec![system, api, db],
                cards: vec![card, old],
                rules: vec![
                    rule("r1", "No secrets in logs", &["app"]),
                    rule("r2", "Handlers return typed errors", &["app/api"]),
                    rule("r3", "Migrations are reversible", &["app/db"]),
                    rule("r4", "Never commit credentials", &[]),
                ],
                milestones: vec![work::Milestone {
                    id: "m1".into(),
                    slug: "beta".into(),
                    name: "Beta".into(),
                    state: work::MILESTONE_ACTIVE.into(),
                    target_date: "2026-12-01".into(),
                    goal: "Outside users sign in.".into(),
                    exit_criteria: vec!["10 users".into(), "no P1".into()],
                    ..work::Milestone::default()
                }],
                ..work::Snapshot::default()
            },
            docs: vec![
                Doc {
                    id: "doc-1".into(),
                    title: "API runbook".into(),
                    doc_type: "doc".into(),
                    scope_node_ids: vec!["d".into(), "c".into()],
                    ..Doc::default()
                },
                Doc {
                    id: "doc-2".into(),
                    title: "Style guide".into(),
                    doc_type: "doc".into(),
                    scope_node_ids: vec!["s".into()],
                    ..Doc::default()
                },
                Doc {
                    id: "doc-3".into(),
                    title: "Unrelated".into(),
                    scope_node_ids: vec!["d".into()],
                    ..Doc::default()
                },
            ],
            canvases: vec![canvas::Canvas {
                name: "Architecture".into(),
                graph: canvas::Graph {
                    nodes: vec![
                        canvas::Node {
                            id: "n1".into(),
                            label: "API".into(),
                            r#ref: Some(canvas::Ref {
                                kind: canvas::REF_NODE.into(),
                                id: "c".into(),
                            }),
                            ..canvas::Node::default()
                        },
                        canvas::Node {
                            id: "n2".into(),
                            label: "Postgres".into(),
                            ..canvas::Node::default()
                        },
                    ],
                    edges: vec![canvas::Edge {
                        from: "n1".into(),
                        to: "n2".into(),
                        kind: canvas::EDGE_READS.into(),
                        ..canvas::Edge::default()
                    }],
                },
                ..canvas::Canvas::default()
            }],
        }
    }

    #[test]
    fn brief_card_orders_blocks_by_priority() {
        let text = brief(&brief_fixture(), "login", 0).expect("a known card");
        let want = r#"## login: Login
stage: build · under: app/api · milestone: beta · claim: codex
kind: service · tech: Go · path: backend
Sign people in.

Behavior:
Sign people in with a session cookie.

Edge cases:
The cookie expires every hour.

Plan:
A session table and a login route.

Acceptance:
! Rate limit attempts

- [ ] T2 Login route
? Which hash? [blocking] (argon2 | bcrypt)
? Remember me?
handoff codex on feat/login: next: wire the route

Rules:
- Handlers return typed errors (from app/api)
- No secrets in logs (from app)
- Never commit credentials (whole project)

milestone beta: Beta · active · target 2026-12-01
goal: Outside users sign in.
exit: 10 users; no P1

Canvas:
- Architecture: API --reads--> Postgres

Docs:
- API runbook
- Style guide
"#;
        assert_eq!(text, want, "brief");
    }

    #[test]
    fn brief_node_lists_open_cards_and_blocking_questions() {
        let text = brief(&brief_fixture(), "app/api", 0).expect("a known node");
        for want in [
            "# app/api: API\nlevel: component · kind: service · tech: Go · path: backend\n",
            "\nRules:\n- Handlers return typed errors\n- No secrets in logs (from app)\n- Never commit credentials (whole project)\n",
            "- login  build  \"Login\"  2 open?",
            "? Which hash? [blocking] (argon2 | bcrypt) — login",
        ] {
            assert!(text.contains(want), "missing {want:?} in:\n{text}");
        }
        for unwanted in [
            "\"Old\"",
            "Remember me?",
            "milestone beta",
            "Migrations are reversible",
        ] {
            assert!(
                !text.contains(unwanted),
                "unexpected {unwanted:?} in:\n{text}"
            );
        }
    }

    #[test]
    fn brief_is_deterministic_and_reports_unknown_targets() {
        let first = brief(&brief_fixture(), "login", 0).expect("a known card");
        let second = brief(&brief_fixture(), "login", 0).expect("a known card");
        assert_eq!(
            first, second,
            "brief changed between two reads of the same graph"
        );
        assert_eq!(
            brief(&brief_fixture(), "backend/api/main.go", 0),
            Err(BriefError::UnknownTarget("backend/api/main.go".to_string())),
            "an unknown target names the error"
        );
    }

    #[test]
    fn search_lists_every_kind_one_line_each() {
        let mut input = brief_fixture();
        input.docs[0].body =
            "Restart the API with make restart when the port is stuck.".to_string();
        let text = search(&input, "API", 0);
        for want in [
            "node app/api  component  \"API\"",
            "doc doc-1  \"API runbook\": Restart the API with make restart when the port is stuck.",
        ] {
            assert!(text.contains(want), "missing {want:?} in:\n{text}");
        }
        let got = search(&input, "api port", 0);
        assert!(
            got.contains("doc doc-1") && !got.contains("node app/api"),
            "every term must match: {got}"
        );
        assert_eq!(
            search(&input, "nothing-like-this", 0),
            "no matches for \"nothing-like-this\"\n",
            "empty search"
        );
    }

    #[test]
    fn search_finds_rules() {
        assert!(
            search(&brief_fixture(), "credentials", 0)
                .contains("rule r4: Never commit credentials"),
            "search finds a Rule"
        );
    }

    #[test]
    fn brief_never_cuts_rules() {
        let text = brief(&brief_fixture(), "login", 40).expect("a known card");
        for want in [
            "## login: Login",
            "Rules:\n",
            "- Never commit credentials (whole project)\n",
            "more lines cut to fit budget:40",
        ] {
            assert!(
                text.contains(want),
                "missing {want:?} in a tight brief:\n{text}"
            );
        }
        assert!(!text.contains("Canvas:"), "the tail should be cut:\n{text}");
    }

    #[test]
    fn brief_shows_checkout() {
        let nodes = vec![structure::Node {
            id: "c".into(),
            path: "auth".into(),
            level: structure::LEVEL_COMPONENT.into(),
            title: "Auth".into(),
            status: structure::NODE_ACTIVE.into(),
            repo: "api".into(),
            folder: "src/auth".into(),
            ..structure::Node::default()
        }];
        let input = BriefInput {
            snapshot: work::Snapshot {
                nodes,
                ..work::Snapshot::default()
            },
            ..BriefInput::default()
        };
        let text = brief(&input, "auth", 800).expect("a known node");
        assert!(
            text.starts_with("# auth: Auth\nlevel: component · repo: api · path: src/auth\n"),
            "brief = {text}"
        );
    }

    #[test]
    fn brief_budget_cuts_from_the_end() {
        let system = structure::Node {
            id: "s".into(),
            path: "polterware".into(),
            level: structure::LEVEL_DOMAIN.into(),
            title: "Polterware".into(),
            status: structure::NODE_ACTIVE.into(),
            ..structure::Node::default()
        };
        let domain = structure::Node {
            id: "d".into(),
            parent_id: "s".into(),
            path: "core".into(),
            level: structure::LEVEL_DOMAIN.into(),
            title: "Core".into(),
            status: structure::NODE_ACTIVE.into(),
            ..structure::Node::default()
        };
        let component = structure::Node {
            id: "c".into(),
            parent_id: "d".into(),
            path: "core/import".into(),
            level: structure::LEVEL_COMPONENT.into(),
            title: "Import".into(),
            status: structure::NODE_ACTIVE.into(),
            ..structure::Node::default()
        };
        let card = work::Card {
            slug: "git-import".into(),
            title: "Git import".into(),
            stage: work::STAGE_BUILD.into(),
            status: work::CARD_ACTIVE.into(),
            under: "core/import".into(),
            intent: "Import HEAD as evidence.".into(),
            tasks: vec![work::Task {
                number: 1,
                title: "Upsert commit evidence".into(),
                state: work::TASK_OPEN.into(),
                ..work::Task::default()
            }],
            questions: vec![work::Question {
                text: "Include merge commits?".into(),
                blocking: true,
                state: work::QUESTION_OPEN.into(),
                options: vec!["yes".into(), "no".into()],
                ..work::Question::default()
            }],
            ..work::Card::default()
        };
        let input = BriefInput {
            snapshot: work::Snapshot {
                nodes: vec![system, domain, component],
                cards: vec![card],
                rules: vec![work::Rule {
                    id: "r1".into(),
                    statement: "No network calls during scan".into(),
                    scopes: vec!["core".into()],
                    ..work::Rule::default()
                }],
                ..work::Snapshot::default()
            },
            ..BriefInput::default()
        };
        let full = brief(&input, "git-import", 800).expect("a known card");
        assert!(
            full.contains("- No network calls during scan (from core)")
                && full.contains("? Include merge commits? [blocking] (yes | no)")
                && !full.contains("cut to fit"),
            "full brief = {full}"
        );
        let tight = brief(&input, "git-import", 30).expect("a known card");
        assert!(
            tight.contains("(from core)")
                && !tight.contains("Include merge commits")
                && tight
                    .starts_with("## git-import: Git import\nstage: build · under: core/import\n")
                && tight.contains("lines cut to fit budget:30"),
            "a tight budget keeps the heading and the Rules and cuts the rest: {tight}"
        );
    }

    #[test]
    fn brief_card_shows_a_cited_rule_outside_its_chain() {
        let mut input = brief_fixture();
        input.snapshot.rules.push(work::Rule {
            id: "rule-far".into(),
            slug: "far-away".into(),
            statement: "Keep the far rule.".into(),
            scopes: vec!["somewhere/else".into()],
            ..work::Rule::default()
        });
        let slug = input.snapshot.cards[0].slug.clone();
        input.snapshot.cards[0].decisions.push("far-away".into());
        let text = brief(&input, &slug, 0).expect("brief");
        assert!(text.contains("- Keep the far rule. (cited)\n"), "{text}");
    }
}
