//! Port of `backend/domains/polter/brief.go`.
//!
//! The one read before an agent works: a card or a Structure node with its
//! Rules, open work, decisions, facts, milestone, canvas links, and docs,
//! deterministic and packed under a token budget.

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::domains::{canvas, context, decision, structure, work};

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
/// most important first: the target with its kind and technology, the Rules
/// that hold for it (its node's, its ancestors', and the project's), open tasks
/// and questions with the last handoff, the card's invariants, the decisions
/// that govern it, scope facts, the milestone goal, canvas links, and doc
/// titles. It is deterministic. A tight budget cuts lines from the end, so the
/// lowest-priority blocks go first, and the last line says what was cut; the
/// target and the Rules are never cut.
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
    Ok(pack_brief(&builder.blocks(), budget, 2))
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
            self.rules(),
            self.conflicts(),
            self.open_work(),
            self.invariants(),
            self.decisions(),
            self.facts(),
            self.milestone(),
            self.canvas(),
            self.docs(),
        ]
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
            let line = deploys_brief_line(self.focus);
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
        let line = deploys_brief_line(Some(node));
        if !line.is_empty() {
            lines.push(line);
        }
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
        if lines.is_empty() {
            return Vec::new();
        }
        let mut out = vec!["Rules:".to_string()];
        out.extend(lines);
        out
    }

    /// The card's own constraints.
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
        lines
    }

    /// The card's own decisions, the ones its node and ancestors cite, and the
    /// ones scoped to that path, each with its statement. The project
    /// principles follow as one line of titles: they apply everywhere, so the
    /// brief only names them. Superseded decisions are left out.
    fn decisions(&self) -> Vec<String> {
        let all = &self.input.snapshot.decisions;
        let mut seen: HashSet<String> = HashSet::new();
        let mut picked: Vec<decision::Decision> = Vec::new();
        if let Some(card) = self.card {
            cite_decisions(all, &card.decisions, &mut picked, &mut seen);
        }
        let paths = self.chain_paths();
        if let Some(focus) = self.focus {
            cite_decisions(all, &focus.decisions, &mut picked, &mut seen);
        }
        for node in &self.chain {
            cite_decisions(all, &node.decisions, &mut picked, &mut seen);
        }
        for path in &paths {
            for item in all {
                if item.scopes.iter().any(|scope| scope == path) {
                    push_decision(item.clone(), &mut picked, &mut seen);
                }
            }
        }
        let mut principles: Vec<String> = Vec::new();
        for item in all {
            if item.is_principle()
                && !seen.contains(&item.id)
                && decision::normalize(&item.status) != decision::STATE_SUPERSEDED
            {
                principles.push(decision::strip_number(&item.title));
            }
        }
        let mut lines: Vec<String> = Vec::new();
        if !picked.is_empty() {
            lines.push("Decisions:".to_string());
        }
        for item in &picked {
            let status = decision::normalize(&item.status);
            let mut line = format!(
                "- {} [{status}] {}",
                decision_handle(item),
                decision::strip_number(&item.title)
            );
            let statement = first_sentence(&item.body);
            if !statement.is_empty() {
                line.push_str(": ");
                line.push_str(&statement);
            }
            if !item.meta.status_reason.is_empty() && status == decision::STATE_CHALLENGED {
                line.push_str(&format!(" (challenged: {})", item.meta.status_reason));
            }
            lines.push(line);
        }
        if !principles.is_empty() {
            lines.push(format!("principles: {}", principles.join(" · ")));
        }
        lines
    }

    /// The facts scoped to the focus or an ancestor, nearest first.
    /// Project-wide facts are the same in every brief and are left to search.
    fn facts(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        let mut seen: HashSet<usize> = HashSet::new();
        for path in self.chain_paths() {
            for (i, fact) in self.input.snapshot.facts.iter().enumerate() {
                if !seen.contains(&i) && fact.scopes.iter().any(|scope| scope == &path) {
                    seen.insert(i);
                    lines.push(format!(
                        "- {}: {}",
                        fact.fact.truth_class, fact.fact.statement
                    ));
                }
            }
        }
        if lines.is_empty() {
            return Vec::new();
        }
        let mut out = vec!["Facts:".to_string()];
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

/// `kind: cli · tech: Go`, or empty when the node has neither.
// TODO(dedupe): a private copy of `kind_tech_line`, which lives in
// `structure_encode.rs` (Go declares it in `structure_encode.go`).
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
// TODO(dedupe): a private copy of `checkout_line`, which lives in
// `structure_encode.rs` (Go declares it in `structure_encode.go`).
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

/// The active Rules that hold for a node path: its own, its ancestors'
/// (nearest first), then the project-wide ones, each once and in stored order
/// within a level.
// TODO(dedupe): `rules_for` belongs to `rules.rs` (port of `rules.go`).
fn rules_for(rules: &[work::Rule], chain: &[String]) -> (Vec<Vec<work::Rule>>, Vec<work::Rule>) {
    let mut seen: HashSet<String> = HashSet::new();
    let mut own: Vec<Vec<work::Rule>> = vec![Vec::new(); chain.len()];
    for (i, path) in chain.iter().enumerate() {
        for rule in rules {
            if !seen.contains(&rule.id) && rule.scopes.iter().any(|scope| scope == path) {
                seen.insert(rule.id.clone());
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

/// Adds a decision once, leaving out the superseded ones.
fn push_decision(
    item: decision::Decision,
    picked: &mut Vec<decision::Decision>,
    seen: &mut HashSet<String>,
) {
    if seen.contains(&item.id) || decision::normalize(&item.status) == decision::STATE_SUPERSEDED {
        return;
    }
    seen.insert(item.id.clone());
    picked.push(item);
}

/// Resolves every reference and adds the decisions it names.
fn cite_decisions(
    all: &[decision::Decision],
    refs: &[String],
    picked: &mut Vec<decision::Decision>,
    seen: &mut HashSet<String>,
) {
    for reference in refs {
        if let Some(found) = decision::find(all, reference) {
            push_decision(found, picked, seen);
        }
    }
}

/// The Polter reference agents read for a decision: its number, or its id.
// TODO(dedupe): `decision_handle` belongs to `decisions.rs` (port of
// `decisions.go`).
fn decision_handle(item: &decision::Decision) -> String {
    match decision::number(&item.title) {
        Some(number) => decision::decision_ref(number),
        None => item.id.clone(),
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

/// Where the node runs and how its latest deploy went:
/// `deploys: production=vercel:web (failed: error · <deploy url>), local=docker:api (ok)`.
/// A failed deploy carries its page; the dashboard links stay in
/// `polterware deploys`.
fn deploys_brief_line(node: Option<&structure::Node>) -> String {
    let Some(node) = node else {
        return String::new();
    };
    if node.deploys.is_empty() {
        return String::new();
    }
    let mut parts: Vec<String> = Vec::with_capacity(node.deploys.len());
    for deploy in &node.deploys {
        let env = if deploy.environment.is_empty() {
            deploy.environment_id.as_str()
        } else {
            deploy.environment.as_str()
        };
        let mut part = format!("{env}={}:{}", deploy.provider, deploy.resource);
        if let Some(check) = &deploy.check {
            let state = if check.state == "failed" {
                non_empty(&[format!("failed: {}", check.summary), check.url.clone()]).join(" · ")
            } else {
                check.state.clone()
            };
            part.push_str(&format!(" ({state})"));
        }
        parts.push(part);
    }
    format!("deploys: {}", parts.join(", "))
}

/// The values that are not blank, unchanged.
fn non_empty(values: &[String]) -> Vec<String> {
    values
        .iter()
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .collect()
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
/// end. The first `keep` blocks (the target and the Rules) always stay whole,
/// and a block header or blank line is never left dangling.
pub(super) fn pack_brief(blocks: &[Vec<String>], budget: i32, keep: usize) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut floor = 0;
    for (i, block) in blocks.iter().enumerate() {
        if block.is_empty() {
            continue;
        }
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.extend(block.iter().cloned());
        if i < keep {
            floor = lines.len();
        }
    }
    floor = floor.max(lines.len().min(2));
    let mut text = format!("{}\n", lines.join("\n"));
    if context::estimate_tokens(&text) <= budget {
        return text;
    }
    let mut kept = lines.len();
    while kept > floor {
        kept -= 1;
        while kept > floor
            && (lines[kept - 1].is_empty()
                || (lines[kept - 1].ends_with(':') && !lines[kept - 1].contains(' ')))
        {
            kept -= 1;
        }
        let cut = lines[kept..].iter().filter(|line| !line.is_empty()).count();
        let note = format!("({cut} more lines cut to fit budget:{budget})");
        text = format!("{}\n{note}\n", lines[..kept].join("\n"));
        if context::estimate_tokens(&text) <= budget {
            return text;
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::canon;
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
        let decision = |id: &str, title: &str, status: &str, body: &str| decision::Decision {
            id: id.into(),
            title: title.into(),
            status: status.into(),
            body: body.into(),
            ..decision::Decision::default()
        };
        let scoped =
            |id: &str, title: &str, status: &str, body: &str, scopes: &[&str]| decision::Decision {
                scopes: scopes.iter().map(|scope| scope.to_string()).collect(),
                ..decision(id, title, status, body)
            };
        let challenged = decision::Decision {
            meta: decision::Meta {
                status_reason: "too slow".into(),
                ..decision::Meta::default()
            },
            ..scoped(
                "k4",
                "Decision 0004: Audit writes",
                "challenged",
                "Every write is audited.",
                &["app"],
            )
        };
        let rule = |id: &str, statement: &str, scopes: &[&str]| work::Rule {
            id: id.into(),
            statement: statement.into(),
            scopes: scopes.iter().map(|scope| scope.to_string()).collect(),
            ..work::Rule::default()
        };
        let fact = |statement: &str, truth_class: &str, scopes: &[&str]| work::Fact {
            fact: canon::Fact {
                statement: statement.into(),
                truth_class: truth_class.into(),
                ..canon::Fact::default()
            },
            scopes: scopes.iter().map(|scope| scope.to_string()).collect(),
            ..work::Fact::default()
        };
        BriefInput {
            snapshot: work::Snapshot {
                nodes: vec![system, api, db],
                cards: vec![card, old],
                decisions: vec![
                    decision(
                        "k1",
                        "Decision 0001: Sessions in cookies",
                        "active",
                        "## Decision\nSessions live in signed cookies. Tokens are not used.",
                    ),
                    decision(
                        "k2",
                        "Decision 0002: One API process",
                        "proposed",
                        "One process serves the API.",
                    ),
                    scoped("k3", "Decision 0003: Postgres", "superseded", "", &["app"]),
                    challenged,
                    decision("k5", "Local first", "active", ""),
                    // Any one context on the target's path picks a decision.
                    scoped(
                        "k6",
                        "Decision 0006: Cache reads",
                        "active",
                        "Reads go through a cache.",
                        &["app/db", "app/api"],
                    ),
                    scoped("k7", "Decision 0007: Batch jobs", "active", "", &["app/db"]),
                ],
                rules: vec![
                    rule("r1", "No secrets in logs", &["app"]),
                    rule("r2", "Handlers return typed errors", &["app/api"]),
                    rule("r3", "Migrations are reversible", &["app/db"]),
                    rule("r4", "Never commit credentials", &[]),
                ],
                facts: vec![
                    fact(
                        "The API runs on port 8080.",
                        canon::TRUTH_INFERRED,
                        &["app/api", "app/db"],
                    ),
                    fact("The app ships weekly.", canon::TRUTH_DECLARED, &["app"]),
                    fact("Project-wide fact.", canon::TRUTH_DECLARED, &[]),
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

Rules:
- Handlers return typed errors (from app/api)
- No secrets in logs (from app)
- Never commit credentials (whole project)

- [ ] T2 Login route
? Which hash? [blocking] (argon2 | bcrypt)
? Remember me?
handoff codex on feat/login: next: wire the route

! Rate limit attempts

Decisions:
- Decision-0001 [active] Sessions in cookies: Sessions live in signed cookies.
- Decision-0002 [proposed] One API process: One process serves the API.
- Decision-0006 [active] Cache reads: Reads go through a cache.
- Decision-0004 [challenged] Audit writes: Every write is audited. (challenged: too slow)
principles: Local first

Facts:
- inferred: The API runs on port 8080.
- declared: The app ships weekly.

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
            "- Decision-0002 [proposed]",
        ] {
            assert!(text.contains(want), "missing {want:?} in:\n{text}");
        }
        for unwanted in [
            "\"Old\"",
            "Remember me?",
            "milestone beta",
            "Decision-0001",
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
            "decision Decision-0002  proposed  \"One API process\"",
            "doc doc-1  \"API runbook\": Restart the API with make restart when the port is stuck.",
            "fact inferred: The API runs on port 8080.",
        ] {
            assert!(text.contains(want), "missing {want:?} in:\n{text}");
        }
        let got = search(&input, "api port", 0);
        assert!(
            got.contains("fact inferred") && !got.contains("node app/api"),
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
}
