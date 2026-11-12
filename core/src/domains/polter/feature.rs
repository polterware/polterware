//! Port of `backend/domains/polter/feature.go`: the feature flow.
//!
//! Review computes the Shape impact of one card (what it touches and inherits)
//! and the checks of every step; the gates turn that review into the Polter
//! text that moves a card from shape to plan and from plan to build.

use std::collections::HashMap;

use crate::domains::{structure, work};

use super::hints::{WordSet, card_words};
use super::*;

// The feature flow steps (`FEATURE_STEP_SHAPE`, `FEATURE_STEP_CONTRACT`,
// `FEATURE_STEP_PLAN`) and `valid_feature_step` live in `format.rs`, the query
// parser that reads them; the flow reads them from there.

/// Question routes: where an open question waits.
pub const ROUTE_INBOX: &str = "inbox";
pub const ROUTE_DEFERRED: &str = "deferred";
pub const ROUTE_DELEGATED: &str = "delegated";
pub const ROUTE_ANSWERED: &str = "answered";

const MAX_SUGGESTIONS: usize = 3;
const MAX_FEATURE_TITLE: usize = 60;
const MAX_CARD_SLUG_LENGTH: usize = 48;

/// What a card touches and inherits, and what stops it from moving on. Checks
/// carry the step they belong to.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FeatureReview {
    pub card: work::Card,
    /// The node named by the card's `under:`, if any.
    pub under: Option<structure::Node>,
    /// Component paths for a card without `under:`.
    pub suggested: Vec<String>,
    /// The `under:` chain (top-down) plus components whose words overlap the
    /// card title and intent.
    pub touches: Vec<structure::Node>,
    pub rules: Vec<ReviewRule>,
    pub inherited: Vec<ReviewInvariant>,
    pub questions: Vec<ReviewQuestion>,
    pub checks: Vec<FeatureCheck>,
    /// Whether the project has any component a card could sit under.
    components: bool,
}

/// One Rule that applies to the card. `from` is `card` or the node path that
/// cites it. `status` is empty when no Rule matches.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReviewRule {
    pub reference: String,
    pub id: String,
    pub title: String,
    pub status: String,
    pub from: String,
}

/// One acceptance criterion (a card invariant, or a Rule that holds for the
/// card), with the node path it came from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReviewInvariant {
    /// The invariant's id, or the Rule's id when the text came from a Rule.
    pub id: String,
    pub text: String,
    /// The node path the invariant came from, or `card` for the card's own.
    pub from: String,
}

/// One open or answered question on the card.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReviewQuestion {
    pub id: String,
    pub text: String,
    pub blocking: bool,
    pub options: Vec<String>,
    pub route: String,
    pub answer: String,
}

/// One finding for a flow step. Severity is [`SEVERITY_ERROR`] (the gate
/// refuses) or [`SEVERITY_HINT`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FeatureCheck {
    pub step: String,
    pub severity: String,
    pub message: String,
}

/// Computes the Shape impact and the checks of every step for one active card.
/// `routes` maps a question id to its Inbox route; a missing id is
/// [`ROUTE_INBOX`].
pub fn review_feature(
    snapshot: &work::Snapshot,
    slug: &str,
    rules: &[DecisionRef],
    routes: &HashMap<String, String>,
) -> Result<FeatureReview, String> {
    let card = match snapshot.card(slug) {
        Some(card) if card.status == work::CARD_ACTIVE => card.clone(),
        _ => {
            return Err(format!(
                "unknown card {}: list cards with polterware query cards",
                go_quote(slug)
            ));
        }
    };
    let mut review = FeatureReview {
        card: card.clone(),
        ..FeatureReview::default()
    };
    let nodes = &snapshot.nodes;
    review.components = nodes.iter().any(|node| {
        node.status != structure::NODE_REMOVED && node.level == structure::LEVEL_COMPONENT
    });

    let mut chain: Vec<structure::Node> = Vec::new();
    if !card.under.is_empty() {
        review.under = nodes
            .iter()
            .find(|node| node.status != structure::NODE_REMOVED && node.path == card.under)
            .cloned();
    }
    if let Some(under) = &review.under {
        let ancestors = ancestor_chain(nodes, under);
        for node in ancestors.iter().rev() {
            chain.push(node.clone());
        }
        chain.push(under.clone());
    }

    let text = format!("{} {}", card.title, card.intent);
    let related = suggest_components(nodes, &snapshot.cards, &text, &card.slug);
    if card.under.is_empty() {
        review.suggested = related.iter().take(MAX_SUGGESTIONS).cloned().collect();
    }
    review.touches = chain.clone();
    for path in &related {
        if review.touches.iter().any(|node| node.path == *path) {
            continue;
        }
        if let Some(node) = snapshot.node_by_path(path) {
            review.touches.push(node.clone());
        }
    }

    review.rules = review_rules(&card, &chain, rules);
    for node in &chain {
        for rule in &snapshot.rules {
            if rule.binding() && rule.scopes.contains(&node.path) {
                review.inherited.push(ReviewInvariant {
                    id: rule.id.clone(),
                    text: rule.statement.clone(),
                    from: node.path.clone(),
                });
            }
        }
    }
    for question in &card.questions {
        let mut item = ReviewQuestion {
            id: question.id.clone(),
            text: question.text.clone(),
            blocking: question.blocking,
            options: question.options.clone(),
            ..ReviewQuestion::default()
        };
        match question.state.as_str() {
            work::QUESTION_ANSWERED => {
                item.route = ROUTE_ANSWERED.to_string();
                item.answer = question.answer.clone();
            }
            work::QUESTION_OPEN => {
                item.route = routes.get(&question.id).cloned().unwrap_or_default();
                if item.route.is_empty() {
                    item.route = ROUTE_INBOX.to_string();
                }
            }
            _ => continue,
        }
        review.questions.push(item);
    }
    review.checks = feature_checks(&review);
    Ok(review)
}

/// Ranks active components by the words they share with `text`: their path,
/// title, intent, and the cards placed under them. A component needs one
/// shared path or title word, or two shared words overall. `skip_slug` leaves
/// that card out of the card words.
pub fn suggest_components(
    nodes: &[structure::Node],
    cards: &[work::Card],
    text: &str,
    skip_slug: &str,
) -> Vec<String> {
    let words = WordSet::new(text);
    if words.is_empty() {
        return Vec::new();
    }
    struct Scored {
        path: String,
        shared: usize,
        named: usize,
    }
    let mut ranked: Vec<Scored> = Vec::new();
    for node in nodes {
        if node.status == structure::NODE_REMOVED || node.level != structure::LEVEL_COMPONENT {
            continue;
        }
        let name = WordSet::new(&format!("{} {}", node.path.replace('/', " "), node.title));
        let mut all = WordSet::new(&node.intent);
        all.add(&name);
        for card in cards {
            if card.status == work::CARD_ACTIVE && card.under == node.path && card.slug != skip_slug
            {
                all.add(&card_words(&card.slug, &card.title));
            }
        }
        let shared = words.shared(&all);
        let named = words.shared(&name);
        if named == 0 && shared < 2 {
            continue;
        }
        ranked.push(Scored {
            path: node.path.clone(),
            shared,
            named,
        });
    }
    ranked.sort_by(|a, b| {
        b.shared
            .cmp(&a.shared)
            .then_with(|| b.named.cmp(&a.named))
            .then_with(|| a.path.cmp(&b.path))
    });
    ranked.into_iter().map(|item| item.path).collect()
}

/// Renders the review for one step: the Shape impact, the step's own section
/// (contract invariants or tasks), its checks, and the next command when
/// nothing blocks.
pub fn encode_feature_review(review: &FeatureReview, step: &str) -> String {
    let card = &review.card;
    let mut out = String::new();
    out.push_str(&format!(
        "{}  {}  {}\n",
        card.slug,
        card.stage,
        go_quote(&card.title)
    ));
    if let Some(under) = &review.under {
        out.push_str(&format!(
            "Under: {} {}\n",
            under.path,
            go_quote(&under.title)
        ));
    } else if !card.under.is_empty() {
        out.push_str(&format!("Under: {}\n", card.under));
    } else if !review.suggested.is_empty() {
        out.push_str(&format!(
            "Under: none (suggested {})\n",
            review.suggested.join(", ")
        ));
    } else {
        out.push_str("Under: none\n");
    }
    if !review.touches.is_empty() {
        let paths: Vec<&str> = review
            .touches
            .iter()
            .map(|node| node.path.as_str())
            .collect();
        out.push_str(&format!("Touches: {}\n", paths.join(", ")));
    }
    for item in &review.rules {
        out.push_str(&format!("Applies: {}\n", rule_line(item)));
    }
    for invariant in &review.inherited {
        out.push_str(&format!(
            "Inherits: ! {} (from {})\n",
            invariant.text, invariant.from
        ));
    }
    for question in &review.questions {
        out.push_str(&format!("Open: {}\n", question_line(question)));
    }
    match step {
        FEATURE_STEP_CONTRACT => {
            for invariant in &card.invariants {
                if invariant.state == work::INVARIANT_ACTIVE {
                    out.push_str(&format!("Contract: ! {}\n", invariant.text));
                }
            }
        }
        FEATURE_STEP_PLAN => {
            for task in &card.tasks {
                if task.state == work::TASK_DROPPED {
                    continue;
                }
                out.push_str(&format!("Task: {}\n", review_task_line(task)));
            }
        }
        _ => {}
    }
    let mut blocked = false;
    for check in review.step_checks(step) {
        out.push_str(&format!("{}: {}\n", check.severity, check.message));
        blocked = blocked || check.severity == SEVERITY_ERROR;
    }
    if !blocked {
        if card.stage == work::STAGE_SHAPE
            && (step == FEATURE_STEP_SHAPE || step == FEATURE_STEP_CONTRACT)
        {
            out.push_str(&format!(
                "next: polterware feature contract {} --accept\n",
                card.slug
            ));
        } else if step == FEATURE_STEP_PLAN && card.stage == work::STAGE_PLAN {
            out.push_str(&format!(
                "next: polterware feature plan {} --approve\n",
                card.slug
            ));
        }
    }
    out
}

/// The checks of one step, in order.
impl FeatureReview {
    pub fn step_checks(&self, step: &str) -> Vec<FeatureCheck> {
        self.checks
            .iter()
            .filter(|check| check.step == step)
            .cloned()
            .collect()
    }
}

/// The Polter text that moves a card through a gate: contract (shape → plan)
/// or plan (plan → build). It returns error diagnostics instead when the card
/// is at another stage or a gate check fails.
pub fn advance_text(review: &FeatureReview, step: &str) -> (String, Vec<Diagnostic>) {
    let card = &review.card;
    let (from, to) = match step {
        FEATURE_STEP_CONTRACT => (work::STAGE_SHAPE, work::STAGE_PLAN),
        FEATURE_STEP_PLAN => (work::STAGE_PLAN, work::STAGE_BUILD),
        _ => {
            return (
                String::new(),
                vec![Diagnostic {
                    line: 0,
                    severity: SEVERITY_ERROR.to_string(),
                    message: format!("unknown step {}: use contract or plan", go_quote(step)),
                }],
            );
        }
    };
    let mut diags: Vec<Diagnostic> = Vec::new();
    if card.stage != from {
        diags.push(Diagnostic {
            line: 0,
            severity: SEVERITY_ERROR.to_string(),
            message: format!(
                "card {} is in {}: the {} gate moves a card from {}",
                card.slug, card.stage, step, from
            ),
        });
    }
    for check in review.step_checks(step) {
        if check.severity == SEVERITY_ERROR {
            diags.push(Diagnostic {
                line: 0,
                severity: SEVERITY_ERROR.to_string(),
                message: check.message,
            });
        }
    }
    if !diags.is_empty() {
        return (String::new(), diags);
    }
    (
        format!("{HEADER}\n## {}\nstage: {}\n", card.slug, to),
        Vec::new(),
    )
}

/// The generated summary and rationale for a gate.
pub fn advance_envelope(slug: &str, step: &str) -> (String, String) {
    if step == FEATURE_STEP_PLAN {
        return (
            format!("Approve plan for {slug}"),
            "The tasks are ready to build; agents can claim them".to_string(),
        );
    }
    (
        format!("Accept contract for {slug}"),
        "Shape is settled: no blocking question is open and the contract is recorded".to_string(),
    )
}

/// The Polter text for a new card in shape. The slug is unique among every
/// card, removed ones included, so a new feature never restores an old card.
pub fn create_feature_text(
    snapshot: &work::Snapshot,
    intent: &str,
    title: &str,
    under: &str,
    kind: &str,
) -> Result<(String, String), String> {
    let intent = one_line(intent);
    if intent.is_empty() {
        return Err("intent is required: say what the feature should do and why".to_string());
    }
    let mut title = one_line(title);
    if title.is_empty() {
        title = feature_title(&intent);
    }
    let existing: Vec<String> = snapshot
        .cards
        .iter()
        .map(|card| card.slug.clone())
        .collect();
    let slug = unique_card_slug(&existing, &title);
    let mut meta = format!("stage: {}", work::STAGE_SHAPE);
    let under = under.trim();
    if !under.is_empty() {
        meta.push_str(&format!(" · under: {under}"));
    }
    let kind = kind.trim().to_lowercase();
    if !kind.is_empty() && !work::valid_kind(&kind) {
        return Err(format!(
            "unknown kind {}: use {}|{}",
            go_quote(&kind),
            work::KIND_FEATURE,
            work::KIND_ARCHITECTURE
        ));
    }
    if kind == work::KIND_ARCHITECTURE {
        meta.push_str(&format!(" · kind: {kind}"));
    }
    let text = format!("{HEADER}\n## {slug}: {title}\n{meta}\n\n{intent}\n");
    Ok((slug, text))
}

/// The generated summary and rationale for a new feature.
pub fn create_envelope(title: &str) -> (String, String) {
    (
        format!("Create feature: {title}"),
        "Record the intent before shaping, so impact and open questions are visible".to_string(),
    )
}

/// The intent's first sentence, cut at a word to about 60 characters.
pub fn feature_title(intent: &str) -> String {
    let intent = one_line(intent);
    let mut end = intent.len();
    for (i, r) in intent.char_indices() {
        if matches!(r, '.' | '?' | '!' | ';')
            && (i + 1 == intent.len() || intent.as_bytes()[i + 1] == b' ')
        {
            end = i;
            break;
        }
    }
    let intent = &intent[..end];
    let runes: Vec<char> = intent.chars().collect();
    if runes.len() <= MAX_FEATURE_TITLE {
        return intent.trim().to_string();
    }
    let mut cut: String = runes[..MAX_FEATURE_TITLE].iter().collect();
    if let Some(i) = cut.rfind(' ')
        && i > 0
    {
        cut.truncate(i);
    }
    trim_right_punct_space(&cut)
}

/// The kebab slug of `title`, suffixed `-2`, `-3`, … when taken. It mirrors the
/// Desktop's uniqueCardSlug, folding accents first.
pub fn unique_card_slug(existing: &[String], title: &str) -> String {
    let mut base = work::normalize_text(title).replace(' ', "-");
    let runes: Vec<char> = base.chars().collect();
    if runes.len() > MAX_CARD_SLUG_LENGTH {
        base = runes[..MAX_CARD_SLUG_LENGTH]
            .iter()
            .collect::<String>()
            .trim_end_matches('-')
            .to_string();
    }
    if !work::valid_slug(&base) {
        base = "card".to_string();
    }
    let used = |candidate: &str| existing.iter().any(|slug| slug == candidate);
    if !used(&base) {
        return base;
    }
    let mut n: u64 = 2;
    loop {
        let candidate = format!("{base}-{n}");
        if !used(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

fn review_rules(
    card: &work::Card,
    chain: &[structure::Node],
    rules: &[DecisionRef],
) -> Vec<ReviewRule> {
    let mut out: Vec<ReviewRule> = Vec::new();
    let mut seen: HashMap<String, bool> = HashMap::new();
    let mut add = |reference: &str, from: &str| {
        let mut item = ReviewRule {
            reference: reference.to_string(),
            from: from.to_string(),
            ..ReviewRule::default()
        };
        for candidate in rules {
            if cites_decision(reference, &candidate.id, &candidate.title) {
                item.id = candidate.id.clone();
                item.title = candidate.title.clone();
                item.status = candidate.status.clone();
                break;
            }
        }
        let key = if item.id.is_empty() {
            reference.to_lowercase()
        } else {
            item.id.clone()
        };
        if seen.contains_key(&key) {
            return;
        }
        seen.insert(key, true);
        out.push(item);
    };
    for reference in &card.decisions {
        add(reference, "card");
    }
    for node in chain {
        for reference in &node.decisions {
            add(reference, &node.path);
        }
    }
    out
}

fn feature_checks(review: &FeatureReview) -> Vec<FeatureCheck> {
    let card = &review.card;
    let mut checks: Vec<FeatureCheck> = Vec::new();
    let mut add = |step: &str, severity: &str, message: String| {
        checks.push(FeatureCheck {
            step: step.to_string(),
            severity: severity.to_string(),
            message,
        });
    };

    // Shape.
    if card.intent.trim().is_empty() {
        add(
            FEATURE_STEP_SHAPE,
            SEVERITY_HINT,
            "no intent: write what we want and why as the card paragraph".to_string(),
        );
    }
    if card.under.is_empty() && review.components {
        if !review.suggested.is_empty() {
            add(
                FEATURE_STEP_SHAPE,
                SEVERITY_HINT,
                format!(
                    "no under: suggested {}: add under: {}",
                    review.suggested.join(", "),
                    review.suggested[0]
                ),
            );
        } else {
            add(
                FEATURE_STEP_SHAPE,
                SEVERITY_HINT,
                "no under: pick a component from polterware structure".to_string(),
            );
        }
    }

    // Contract and Plan share the blocking-question gate.
    for question in &review.questions {
        if question.route == ROUTE_ANSWERED || !question.blocking {
            continue;
        }
        for step in [FEATURE_STEP_CONTRACT, FEATURE_STEP_PLAN] {
            add(
                step,
                SEVERITY_ERROR,
                format!(
                    "blocking question open: {}: answer it with `? {} -> <answer>`",
                    go_quote(&question.text),
                    question.text
                ),
            );
        }
    }

    // Contract.
    if card.behavior.trim().is_empty() {
        add(
            FEATURE_STEP_CONTRACT,
            SEVERITY_ERROR,
            "no behavior: write what the feature does as a behavior: section".to_string(),
        );
    }
    let mut own = 0;
    for invariant in &card.invariants {
        if invariant.state != work::INVARIANT_ACTIVE {
            continue;
        }
        own += 1;
        let words = WordSet::new(&invariant.text);
        for inherited in &review.inherited {
            if words.similar(&WordSet::new(&inherited.text)).is_some() {
                add(
                    FEATURE_STEP_CONTRACT,
                    SEVERITY_HINT,
                    format!(
                        "invariant {} repeats {} from {}: drop it, the card already inherits it",
                        go_quote(&invariant.text),
                        go_quote(&inherited.text),
                        inherited.from
                    ),
                );
                break;
            }
        }
    }
    if own == 0 && review.inherited.is_empty() {
        add(
            FEATURE_STEP_CONTRACT,
            SEVERITY_HINT,
            "no contract: add `! invariant` lines the feature must keep".to_string(),
        );
    }
    for item in &review.rules {
        if item.id.is_empty() {
            add(
                FEATURE_STEP_CONTRACT,
                SEVERITY_HINT,
                format!("rule {} matches no Rule in this project", item.reference),
            );
        } else if !item.status.is_empty() && !work::binding_rule_status(&item.status) {
            add(
                FEATURE_STEP_CONTRACT,
                SEVERITY_HINT,
                format!("rule {} is {}, not binding", item.title, item.status),
            );
        }
    }

    // Plan.
    if card.plan.trim().is_empty() {
        add(
            FEATURE_STEP_PLAN,
            SEVERITY_ERROR,
            "no plan: write architecture, APIs, and files touched as a plan: section".to_string(),
        );
    }
    let mut open = 0;
    for task in &card.tasks {
        if task.state != work::TASK_OPEN {
            continue;
        }
        open += 1;
        if task.scope.is_empty() {
            add(
                FEATURE_STEP_PLAN,
                SEVERITY_HINT,
                format!(
                    "T{} has no scope: add a backticked glob so a claim reserves its files",
                    task.number
                ),
            );
        }
    }
    if open == 0 {
        add(
            FEATURE_STEP_PLAN,
            SEVERITY_ERROR,
            "no open tasks: add `- [ ] Task` lines before approving the plan".to_string(),
        );
    }
    checks
}

fn rule_line(item: &ReviewRule) -> String {
    let mut line = item.reference.clone();
    if !item.title.is_empty() {
        line = item.title.clone();
    }
    if !item.status.is_empty() {
        line.push_str(&format!(" ({})", item.status));
    } else if item.id.is_empty() {
        line.push_str(" (no Rule)");
    }
    if item.from != "card" {
        line.push_str(&format!(" via {}", item.from));
    }
    line
}

fn question_line(question: &ReviewQuestion) -> String {
    let mut line = format!("? {}", question.text);
    if question.blocking {
        line.push_str(" [blocking]");
    }
    if !question.options.is_empty() {
        line.push_str(&format!(" ({})", question.options.join(" | ")));
    }
    if question.route == ROUTE_ANSWERED {
        return format!("{line} -> {}", question.answer);
    }
    format!("{line} → {}", question.route)
}

fn review_task_line(task: &work::Task) -> String {
    let mark = if task.state == work::TASK_DONE {
        "[x]"
    } else {
        "[ ]"
    };
    let mut line = format!("{mark} T{} {}", task.number, task.title);
    if !task.repo.is_empty() {
        line.push_str(&format!(" [repo: {}]", task.repo));
    }
    if !task.scope.is_empty() {
        line.push_str(&format!(" `{}`", task.scope));
    }
    line
}

/// Joins the words of `text` on one line, as Go's `strings.Fields` does.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// Trims trailing punctuation and space, as Go's
/// `strings.TrimRightFunc(cut, IsPunct || IsSpace)`.
fn trim_right_punct_space(text: &str) -> String {
    text.trim_end_matches(|c: char| c.is_whitespace() || is_punct(c))
        .to_string()
}

/// The Unicode P* general categories, as Go's `unicode.IsPunct` reads them:
/// `(low, high, stride)` in code point order.
const PUNCT_RANGES: [(u32, u32, u32); 174] = [
    (0x0021, 0x0023, 1),
    (0x0025, 0x002a, 1),
    (0x002c, 0x002f, 1),
    (0x003a, 0x003b, 1),
    (0x003f, 0x0040, 1),
    (0x005b, 0x005d, 1),
    (0x005f, 0x007b, 28),
    (0x007d, 0x00a1, 36),
    (0x00a7, 0x00ab, 4),
    (0x00b6, 0x00b7, 1),
    (0x00bb, 0x00bf, 4),
    (0x037e, 0x0387, 9),
    (0x055a, 0x055f, 1),
    (0x0589, 0x058a, 1),
    (0x05be, 0x05c0, 2),
    (0x05c3, 0x05c6, 3),
    (0x05f3, 0x05f4, 1),
    (0x0609, 0x060a, 1),
    (0x060c, 0x060d, 1),
    (0x061b, 0x061d, 2),
    (0x061e, 0x061f, 1),
    (0x066a, 0x066d, 1),
    (0x06d4, 0x0700, 44),
    (0x0701, 0x070d, 1),
    (0x07f7, 0x07f9, 1),
    (0x0830, 0x083e, 1),
    (0x085e, 0x0964, 262),
    (0x0965, 0x0970, 11),
    (0x09fd, 0x0a76, 121),
    (0x0af0, 0x0c77, 391),
    (0x0c84, 0x0df4, 368),
    (0x0e4f, 0x0e5a, 11),
    (0x0e5b, 0x0f04, 169),
    (0x0f05, 0x0f12, 1),
    (0x0f14, 0x0f3a, 38),
    (0x0f3b, 0x0f3d, 1),
    (0x0f85, 0x0fd0, 75),
    (0x0fd1, 0x0fd4, 1),
    (0x0fd9, 0x0fda, 1),
    (0x104a, 0x104f, 1),
    (0x10fb, 0x1360, 613),
    (0x1361, 0x1368, 1),
    (0x1400, 0x166e, 622),
    (0x169b, 0x169c, 1),
    (0x16eb, 0x16ed, 1),
    (0x1735, 0x1736, 1),
    (0x17d4, 0x17d6, 1),
    (0x17d8, 0x17da, 1),
    (0x1800, 0x180a, 1),
    (0x1944, 0x1945, 1),
    (0x1a1e, 0x1a1f, 1),
    (0x1aa0, 0x1aa6, 1),
    (0x1aa8, 0x1aad, 1),
    (0x1b4e, 0x1b4f, 1),
    (0x1b5a, 0x1b60, 1),
    (0x1b7d, 0x1b7f, 1),
    (0x1bfc, 0x1bff, 1),
    (0x1c3b, 0x1c3f, 1),
    (0x1c7e, 0x1c7f, 1),
    (0x1cc0, 0x1cc7, 1),
    (0x1cd3, 0x2010, 829),
    (0x2011, 0x2027, 1),
    (0x2030, 0x2043, 1),
    (0x2045, 0x2051, 1),
    (0x2053, 0x205e, 1),
    (0x207d, 0x207e, 1),
    (0x208d, 0x208e, 1),
    (0x2308, 0x230b, 1),
    (0x2329, 0x232a, 1),
    (0x2768, 0x2775, 1),
    (0x27c5, 0x27c6, 1),
    (0x27e6, 0x27ef, 1),
    (0x2983, 0x2998, 1),
    (0x29d8, 0x29db, 1),
    (0x29fc, 0x29fd, 1),
    (0x2cf9, 0x2cfc, 1),
    (0x2cfe, 0x2cff, 1),
    (0x2d70, 0x2e00, 144),
    (0x2e01, 0x2e2e, 1),
    (0x2e30, 0x2e4f, 1),
    (0x2e52, 0x2e5d, 1),
    (0x3001, 0x3003, 1),
    (0x3008, 0x3011, 1),
    (0x3014, 0x301f, 1),
    (0x3030, 0x303d, 13),
    (0x30a0, 0x30fb, 91),
    (0xa4fe, 0xa4ff, 1),
    (0xa60d, 0xa60f, 1),
    (0xa673, 0xa67e, 11),
    (0xa6f2, 0xa6f7, 1),
    (0xa874, 0xa877, 1),
    (0xa8ce, 0xa8cf, 1),
    (0xa8f8, 0xa8fa, 1),
    (0xa8fc, 0xa92e, 50),
    (0xa92f, 0xa95f, 48),
    (0xa9c1, 0xa9cd, 1),
    (0xa9de, 0xa9df, 1),
    (0xaa5c, 0xaa5f, 1),
    (0xaade, 0xaadf, 1),
    (0xaaf0, 0xaaf1, 1),
    (0xabeb, 0xfd3e, 20819),
    (0xfd3f, 0xfe10, 209),
    (0xfe11, 0xfe19, 1),
    (0xfe30, 0xfe52, 1),
    (0xfe54, 0xfe61, 1),
    (0xfe63, 0xfe68, 5),
    (0xfe6a, 0xfe6b, 1),
    (0xff01, 0xff03, 1),
    (0xff05, 0xff0a, 1),
    (0xff0c, 0xff0f, 1),
    (0xff1a, 0xff1b, 1),
    (0xff1f, 0xff20, 1),
    (0xff3b, 0xff3d, 1),
    (0xff3f, 0xff5b, 28),
    (0xff5d, 0xff5f, 2),
    (0xff60, 0xff65, 1),
    (0x10100, 0x10102, 1),
    (0x1039f, 0x103d0, 49),
    (0x1056f, 0x10857, 744),
    (0x1091f, 0x1093f, 32),
    (0x10a50, 0x10a58, 1),
    (0x10a7f, 0x10af0, 113),
    (0x10af1, 0x10af6, 1),
    (0x10b39, 0x10b3f, 1),
    (0x10b99, 0x10b9c, 1),
    (0x10d6e, 0x10ead, 319),
    (0x10ed0, 0x10f55, 133),
    (0x10f56, 0x10f59, 1),
    (0x10f86, 0x10f89, 1),
    (0x11047, 0x1104d, 1),
    (0x110bb, 0x110bc, 1),
    (0x110be, 0x110c1, 1),
    (0x11140, 0x11143, 1),
    (0x11174, 0x11175, 1),
    (0x111c5, 0x111c8, 1),
    (0x111cd, 0x111db, 14),
    (0x111dd, 0x111df, 1),
    (0x11238, 0x1123d, 1),
    (0x112a9, 0x113d4, 299),
    (0x113d5, 0x113d7, 2),
    (0x113d8, 0x1144b, 115),
    (0x1144c, 0x1144f, 1),
    (0x1145a, 0x1145b, 1),
    (0x1145d, 0x114c6, 105),
    (0x115c1, 0x115d7, 1),
    (0x11641, 0x11643, 1),
    (0x11660, 0x1166c, 1),
    (0x116b9, 0x1173c, 131),
    (0x1173d, 0x1173e, 1),
    (0x1183b, 0x11944, 265),
    (0x11945, 0x11946, 1),
    (0x119e2, 0x11a3f, 93),
    (0x11a40, 0x11a46, 1),
    (0x11a9a, 0x11a9c, 1),
    (0x11a9e, 0x11aa2, 1),
    (0x11b00, 0x11b09, 1),
    (0x11be1, 0x11c41, 96),
    (0x11c42, 0x11c45, 1),
    (0x11c70, 0x11c71, 1),
    (0x11ef7, 0x11ef8, 1),
    (0x11f43, 0x11f4f, 1),
    (0x11fff, 0x12470, 1137),
    (0x12471, 0x12474, 1),
    (0x12ff1, 0x12ff2, 1),
    (0x16a6e, 0x16a6f, 1),
    (0x16af5, 0x16b37, 66),
    (0x16b38, 0x16b3b, 1),
    (0x16b44, 0x16d6d, 553),
    (0x16d6e, 0x16d6f, 1),
    (0x16e97, 0x16e9a, 1),
    (0x16fe2, 0x1bc9f, 19645),
    (0x1da87, 0x1da8b, 1),
    (0x1e5ff, 0x1e95e, 863),
    (0x1e95f, 0x1e95f, 1),
];

fn is_punct(c: char) -> bool {
    let r = c as u32;
    let index = PUNCT_RANGES.partition_point(|&(low, _, _)| low <= r);
    if index == 0 {
        return false;
    }
    let (low, high, stride) = PUNCT_RANGES[index - 1];
    r <= high && (r - low).is_multiple_of(stride)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::polter::decode::decode;

    fn feature_snapshot(card: work::Card) -> work::Snapshot {
        work::Snapshot {
            project_id: "p1".into(),
            nodes: vec![
                structure::Node {
                    id: "n-core".into(),
                    level: structure::LEVEL_DOMAIN.into(),
                    path: "core".into(),
                    title: "Core".into(),
                    status: structure::NODE_ACTIVE.into(),
                    decisions: vec!["Decision 0007".into()],
                    ..structure::Node::default()
                },
                structure::Node {
                    id: "n-import".into(),
                    parent_id: "n-core".into(),
                    level: structure::LEVEL_COMPONENT.into(),
                    path: "core/import".into(),
                    title: "Import".into(),
                    intent: "Deterministic import of project files and git metadata".into(),
                    status: structure::NODE_ACTIVE.into(),
                    ..structure::Node::default()
                },
                structure::Node {
                    id: "n-auth".into(),
                    parent_id: "n-core".into(),
                    level: structure::LEVEL_COMPONENT.into(),
                    path: "core/auth".into(),
                    title: "Auth".into(),
                    intent: "Bearer sessions".into(),
                    status: structure::NODE_ACTIVE.into(),
                    ..structure::Node::default()
                },
            ],
            cards: vec![card],
            rules: vec![work::Rule {
                id: "r1".into(),
                statement: "No network calls during scan".into(),
                scopes: vec!["core".into()],
                status: work::RULE_ACTIVE.into(),
                ..work::Rule::default()
            }],
            ..work::Snapshot::default()
        }
    }

    fn feature_decisions() -> Vec<DecisionRef> {
        vec![
            DecisionRef {
                id: "decision-7".into(),
                title: "Decision 0007: Deterministic import only".into(),
                status: "active".into(),
            },
            DecisionRef {
                id: "decision-9".into(),
                title: "Decision 0009: Checkouts".into(),
                status: "proposed".into(),
            },
        ]
    }

    fn has_check(review: &FeatureReview, step: &str, severity: &str, fragment: &str) -> bool {
        review.checks.iter().any(|check| {
            check.step == step && check.severity == severity && check.message.contains(fragment)
        })
    }

    #[test]
    fn review_feature_impact() {
        let card = work::Card {
            slug: "git-import".into(),
            title: "Git metadata import".into(),
            stage: work::STAGE_SHAPE.into(),
            status: work::CARD_ACTIVE.into(),
            intent: "Import git commits and branch as evidence".into(),
            under: "core/import".into(),
            decisions: vec!["Decision-0009".into()],
            questions: vec![
                work::Question {
                    id: "q1".into(),
                    text: "Include merge commits?".into(),
                    blocking: true,
                    state: work::QUESTION_OPEN.into(),
                    ..work::Question::default()
                },
                work::Question {
                    id: "q2".into(),
                    text: "How many commits?".into(),
                    state: work::QUESTION_OPEN.into(),
                    ..work::Question::default()
                },
                work::Question {
                    id: "q3".into(),
                    text: "Shallow clones?".into(),
                    state: work::QUESTION_ANSWERED.into(),
                    answer: "skip".into(),
                    ..work::Question::default()
                },
            ],
            invariants: vec![work::Invariant {
                text: "No network calls during the scan".into(),
                state: work::INVARIANT_ACTIVE.into(),
                ..work::Invariant::default()
            }],
            ..work::Card::default()
        };
        let mut routes = HashMap::new();
        routes.insert("q2".to_string(), ROUTE_DELEGATED.to_string());
        let review = review_feature(
            &feature_snapshot(card),
            "git-import",
            &feature_decisions(),
            &routes,
        )
        .expect("the card is active");

        let under = review.under.as_ref().expect("under resolves");
        assert_eq!(under.path, "core/import", "under path");
        assert!(review.suggested.is_empty(), "suggested with an under:");

        let touches: Vec<&str> = review
            .touches
            .iter()
            .map(|node| node.path.as_str())
            .collect();
        assert_eq!(touches.join(","), "core,core/import", "touches");

        assert_eq!(review.rules.len(), 2, "rules: {:?}", review.rules);
        assert_eq!(review.rules[0].id, "decision-9", "the card's rule");
        assert_eq!(review.rules[1].from, "core", "the node's rule");

        assert_eq!(
            review.inherited.len(),
            1,
            "inherited: {:?}",
            review.inherited
        );
        assert_eq!(
            review.inherited[0].from, "core",
            "inherited from the domain"
        );

        let mut got: HashMap<String, String> = HashMap::new();
        for question in &review.questions {
            got.insert(question.id.clone(), question.route.clone());
        }
        assert_eq!(got.get("q1").map(String::as_str), Some(ROUTE_INBOX));
        assert_eq!(got.get("q2").map(String::as_str), Some(ROUTE_DELEGATED));
        assert_eq!(got.get("q3").map(String::as_str), Some(ROUTE_ANSWERED));

        assert!(
            has_check(
                &review,
                FEATURE_STEP_CONTRACT,
                SEVERITY_ERROR,
                "Include merge commits?"
            ) && has_check(
                &review,
                FEATURE_STEP_PLAN,
                SEVERITY_ERROR,
                "Include merge commits?"
            ),
            "blocking question should gate contract and plan: {:?}",
            review.checks
        );
        assert!(
            has_check(&review, FEATURE_STEP_CONTRACT, SEVERITY_HINT, "repeats"),
            "near-duplicate inherited invariant should hint: {:?}",
            review.checks
        );
        assert!(
            has_check(
                &review,
                FEATURE_STEP_CONTRACT,
                SEVERITY_HINT,
                "is proposed, not binding"
            ),
            "proposed decision should hint: {:?}",
            review.checks
        );
        assert!(
            has_check(&review, FEATURE_STEP_PLAN, SEVERITY_ERROR, "no open tasks"),
            "plan without tasks should be refused: {:?}",
            review.checks
        );

        let text = encode_feature_review(&review, FEATURE_STEP_SHAPE);
        for want in [
            "Under: core/import",
            "Touches: core, core/import",
            "Inherits: ! No network calls during scan (from core)",
            "→ delegated",
        ] {
            assert!(
                text.contains(want),
                "encoded review misses {want:?}:\n{text}"
            );
        }
    }

    #[test]
    fn review_feature_suggests_under() {
        let card = work::Card {
            slug: "commit-evidence".into(),
            title: "Commit evidence import".into(),
            stage: work::STAGE_SHAPE.into(),
            status: work::CARD_ACTIVE.into(),
            intent: "Import git commits".into(),
            ..work::Card::default()
        };
        let snapshot = feature_snapshot(card.clone());
        let review = review_feature(&snapshot, "commit-evidence", &[], &HashMap::new())
            .expect("the card is active");
        assert_eq!(
            review.suggested.first().map(String::as_str),
            Some("core/import"),
            "suggested = {:?}",
            review.suggested
        );
        assert!(
            has_check(
                &review,
                FEATURE_STEP_SHAPE,
                SEVERITY_HINT,
                "add under: core/import"
            ),
            "checks = {:?}",
            review.checks
        );
        assert!(
            review_feature(&snapshot, "missing", &[], &HashMap::new()).is_err(),
            "unknown card should fail"
        );
    }

    #[test]
    fn advance_text_gates() {
        let mut card = work::Card {
            slug: "git-import".into(),
            title: "Git import".into(),
            stage: work::STAGE_SHAPE.into(),
            status: work::CARD_ACTIVE.into(),
            questions: vec![work::Question {
                id: "q1".into(),
                text: "Merge commits?".into(),
                blocking: true,
                state: work::QUESTION_OPEN.into(),
                ..work::Question::default()
            }],
            ..work::Card::default()
        };
        let review = review_feature(
            &feature_snapshot(card.clone()),
            "git-import",
            &[],
            &HashMap::new(),
        )
        .expect("the card is active");
        let (_, diags) = advance_text(&review, FEATURE_STEP_CONTRACT);
        assert!(
            has_errors(&diags),
            "blocking question should refuse the contract gate"
        );

        card.questions[0].state = work::QUESTION_ANSWERED.into();
        card.questions[0].answer = "no".into();
        let review = review_feature(
            &feature_snapshot(card.clone()),
            "git-import",
            &[],
            &HashMap::new(),
        )
        .expect("the card is active");
        let (_, diags) = advance_text(&review, FEATURE_STEP_CONTRACT);
        assert!(
            diags
                .iter()
                .any(|diag| diag.message.contains("no behavior")),
            "the contract gate needs a behavior: {diags:?}"
        );

        card.behavior = "Import git commits as evidence.".into();
        let review = review_feature(
            &feature_snapshot(card.clone()),
            "git-import",
            &[],
            &HashMap::new(),
        )
        .expect("the card is active");
        let (text, diags) = advance_text(&review, FEATURE_STEP_CONTRACT);
        assert!(
            diags.is_empty() && text.contains("## git-import\nstage: plan\n"),
            "contract gate: text={text:?} diags={diags:?}"
        );
        let (_, diags) = advance_text(&review, FEATURE_STEP_PLAN);
        assert!(has_errors(&diags), "plan gate from shape should be refused");

        card.stage = work::STAGE_PLAN.into();
        let review = review_feature(
            &feature_snapshot(card.clone()),
            "git-import",
            &[],
            &HashMap::new(),
        )
        .expect("the card is active");
        let (_, diags) = advance_text(&review, FEATURE_STEP_PLAN);
        assert!(
            has_errors(&diags),
            "plan gate without open tasks should be refused"
        );
        assert!(
            diags.iter().any(|diag| diag.message.contains("no plan")),
            "the plan gate needs a plan: {diags:?}"
        );
        card.tasks = vec![work::Task {
            number: 1,
            title: "Read HEAD".into(),
            state: work::TASK_OPEN.into(),
            ..work::Task::default()
        }];
        let review = review_feature(
            &feature_snapshot(card.clone()),
            "git-import",
            &[],
            &HashMap::new(),
        )
        .expect("the card is active");
        let (_, diags) = advance_text(&review, FEATURE_STEP_PLAN);
        assert!(
            diags.iter().any(|diag| diag.message.contains("no plan")),
            "tasks alone do not pass the plan gate: {diags:?}"
        );

        card.plan = "Read HEAD, then stamp each fact.\nFiles: backend/git/*".into();
        let review = review_feature(
            &feature_snapshot(card.clone()),
            "git-import",
            &[],
            &HashMap::new(),
        )
        .expect("the card is active");
        let (text, diags) = advance_text(&review, FEATURE_STEP_PLAN);
        assert!(
            diags.is_empty() && text.contains("stage: build"),
            "plan gate: text={text:?} diags={diags:?}"
        );
        assert!(
            has_check(&review, FEATURE_STEP_PLAN, SEVERITY_HINT, "T1 has no scope"),
            "task without scope should hint: {:?}",
            review.checks
        );
    }

    #[test]
    fn creates_feature_text() {
        let snapshot = work::Snapshot {
            cards: vec![work::Card {
                slug: "import-git-commits".into(),
                status: work::CARD_REMOVED.into(),
                ..work::Card::default()
            }],
            ..work::Snapshot::default()
        };
        let (slug, text) = create_feature_text(
            &snapshot,
            "Import git commits. Then\n- [ ] not a task",
            "",
            "core/import",
            "",
        )
        .expect("the intent is present");
        assert_eq!(
            slug, "import-git-commits-2",
            "a removed card keeps its slug"
        );
        let (doc, diags) = decode(&text);
        assert!(
            !has_errors(&diags) && doc.cards.len() == 1 && doc.cards[0].tasks.is_empty(),
            "decode: {diags:?} {:?}",
            doc.cards
        );
        assert_eq!(
            doc.cards[0].meta_value("under"),
            Some("core/import"),
            "under"
        );
        assert!(
            create_feature_text(&snapshot, "   ", "", "", "").is_err(),
            "empty intent should fail"
        );
    }

    #[test]
    fn feature_title_and_slug() {
        assert_eq!(
            feature_title("Import HEAD as evidence. Also commits."),
            "Import HEAD as evidence",
            "the first sentence"
        );
        let long = "word ".repeat(20);
        let got = feature_title(&long);
        assert!(
            got.chars().count() <= 60 && !got.ends_with(' '),
            "long title = {got:?}"
        );
        assert_eq!(
            unique_card_slug(&["cafe-import".to_string()], "Café import"),
            "cafe-import-2",
            "an accents-folded slug that is taken"
        );
        assert_eq!(
            unique_card_slug(&[], "!!!"),
            "card",
            "an empty slug falls back to card"
        );
    }

    #[test]
    fn create_feature_text_architecture() {
        let (_, text) = create_feature_text(
            &work::Snapshot::default(),
            "Move sync to Connect",
            "",
            "core",
            "architecture",
        )
        .expect("architecture is a kind");
        assert!(text.contains("kind: architecture"), "text = {text:?}");
        assert!(
            create_feature_text(&work::Snapshot::default(), "x", "", "", "bogus").is_err(),
            "expected an error for an unknown kind"
        );
    }
}
