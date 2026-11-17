//! Decoding plus the reference checks, graph checks, and similarity hints.
//!
//! The stage hints live in `feature`, and the handoff hints with `GitView` in
//! `handoff`; the passes they feed (feature review, the handoff encoders) read
//! them here, and `lint` runs them with the rest of the reference checks.

use std::collections::{HashMap, HashSet};

use crate::domains::{structure, work};

use super::graph::check_graph;
use super::hints::{check_hints, find_card};
use super::model::*;

/// What reference-level lint checks against. `None` means the project could not
/// be resolved and only syntax is checked.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct References {
    /// Each Rule's id and statement.
    pub decisions: Vec<DecisionRef>,
    /// The project's current Work graph, for similarity hints.
    pub cards: Vec<work::Card>,
    /// The project's Structure nodes. A card `under:` is checked once any
    /// component exists.
    pub nodes: Vec<structure::Node>,
    /// The project's registered checkouts. Node `repo:` and task `[repo: …]`
    /// must name one; empty is the project root.
    pub repos: Vec<String>,
    /// The local repository view used to hint on handoff mismatches. `None`
    /// skips that check.
    pub git: Option<GitView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DecisionRef {
    pub id: String,
    /// The Rule's slug, which a card's `rules:` may cite.
    pub slug: String,
    pub title: String,
    /// The Rule status (proposed, active, under_review, …).
    pub status: String,
}

/// A local, network-free look at the project repository.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitView {
    pub available: bool,
    pub branch: String,
    pub files: Vec<String>,
    /// True when HEAD is an ancestor of local main or master.
    pub delivered: bool,
}

/// Decodes text and runs syntax checks, plus reference checks and similarity
/// hints when `references` is set. It returns the decoded document so callers
/// can continue to the planner.
pub fn lint(text: &str, references: Option<&References>) -> (Document, Vec<Diagnostic>) {
    let (doc, mut diags) = super::decode::decode(text);
    if let Some(references) = references {
        diags.extend(check_references(&doc, references));
        diags.extend(check_graph(&doc, &references.nodes, &references.cards));
        diags.extend(check_repos(&doc, &references.repos));
        diags.extend(check_hints(&doc, &references.cards));
        diags.extend(stage_hints(&doc, &references.cards));
        diags.extend(architecture_hints(
            &doc,
            &references.cards,
            &references.nodes,
        ));
        if let Some(git) = &references.git {
            diags.extend(handoff_hints(&doc, git));
        }
    }
    (doc, diags)
}

fn check_references(doc: &Document, references: &References) -> Vec<Diagnostic> {
    let mut known: HashSet<String> = HashSet::new();
    let mut numbers: HashSet<i64> = HashSet::new();
    for reference in &references.decisions {
        known.insert(reference.id.to_lowercase());
        if !reference.slug.is_empty() {
            known.insert(reference.slug.to_lowercase());
        }
        if let Some(number) = parse_decision_number(&reference.title) {
            numbers.insert(number);
        }
    }
    // `stale` hints at a cited Rule that no longer settles the question.
    let stale = |line: i32, reference: &str| -> Vec<Diagnostic> {
        for entry in &references.decisions {
            if !cites_decision(reference, &entry.id, &entry.slug, &entry.title)
                || work::binding_rule_status(&entry.status)
            {
                continue;
            }
            let state = entry.status.as_str();
            let mut message = format!("rule {} is {state}", entry.title);
            if state == work::RULE_REPLACED {
                message.push_str(": cite its successor");
            }
            return vec![Diagnostic::hint(line, message)];
        }
        Vec::new()
    };
    let mut diags = Vec::new();
    for card in &doc.cards {
        for entry in &card.meta {
            if entry.key != "rules" && entry.key != "decisions" {
                continue;
            }
            for reference in split_list(&entry.value) {
                if known.contains(&reference.to_lowercase()) {
                    diags.extend(stale(entry.line, &reference));
                    continue;
                }
                if let Some(number) = parse_decision_number(&reference)
                    && numbers.contains(&number)
                {
                    diags.extend(stale(entry.line, &reference));
                    continue;
                }
                diags.push(Diagnostic {
                    line: entry.line,
                    severity: SEVERITY_ERROR.to_string(),
                    message: format!(
                        "unknown rule {}: no Rule with that slug, number, or id in this project",
                        go_quote(&reference)
                    ),
                });
            }
        }
    }
    diags
}

/// Reports node `repo:` and task repo values that name no registered checkout.
pub fn check_repos(doc: &Document, repos: &[String]) -> Vec<Diagnostic> {
    let mut known: HashSet<&str> = HashSet::new();
    known.insert("");
    for repo in repos {
        known.insert(repo);
    }
    let unknown = |line: i32, repo: &str| Diagnostic {
        line,
        severity: SEVERITY_ERROR.to_string(),
        message: format!(
            "unknown repo {}: link a local clone with dir: <folder>, or list repositories with polterware repo list",
            go_quote(repo)
        ),
    };
    let mut diags = Vec::new();
    for node in &doc.nodes {
        for entry in &node.meta {
            if entry.key == "repo" && !known.contains(entry.value.as_str()) {
                diags.push(unknown(entry.line, &entry.value));
            }
        }
    }
    for card in &doc.cards {
        for task in &card.tasks {
            if !known.contains(task.repo.as_str()) {
                diags.push(unknown(task.line, &task.repo));
            }
        }
    }
    diags
}

/// Reads the number from "Decision-0008", "ADR-8", "Verdict-8", or a numbered
/// title.
pub fn parse_decision_number(value: &str) -> Option<i64> {
    let lower = value.trim().to_lowercase();
    let rest = [
        "decision-",
        "decision ",
        "adr-",
        "adr ",
        "verdict-",
        "verdict ",
    ]
    .iter()
    .find_map(|prefix| lower.strip_prefix(prefix))?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

/// Reports whether a `rules:` entry (or the retired `decisions:` key) names the
/// Rule with `id`, `slug`, and `title`, by id, by slug, or by a migrated number.
pub fn cites_decision(reference: &str, id: &str, slug: &str, title: &str) -> bool {
    let reference = reference.trim();
    if reference.eq_ignore_ascii_case(id)
        || (!slug.is_empty() && reference.eq_ignore_ascii_case(slug))
    {
        return true;
    }
    let Some(number) = parse_decision_number(reference) else {
        return false;
    };
    parse_decision_number(title) == Some(number)
}

/// Reminds that a delivered Architecture Change should clear the `migrating:`
/// state of the node it changes.
pub fn architecture_hints(
    doc: &Document,
    cards: &[work::Card],
    nodes: &[structure::Node],
) -> Vec<Diagnostic> {
    let mut stored: HashMap<&str, &work::Card> = HashMap::new();
    for card in cards {
        stored.insert(card.slug.as_str(), card);
    }
    let mut diags = Vec::new();
    for card in &doc.cards {
        let Some(stage) = card.meta_value("stage") else {
            continue;
        };
        if stage != work::STAGE_DELIVERED {
            continue;
        }
        let previous = stored.get(card.slug.as_str()).copied();
        let mut kind = card.meta_value("kind").unwrap_or_default().to_string();
        let mut under = card.meta_value("under").unwrap_or_default().to_string();
        if let Some(previous) = previous {
            if card.meta_value("kind").is_none() {
                kind = previous.kind.clone();
            }
            if card.meta_value("under").is_none() {
                under = previous.under.clone();
            }
        }
        if kind != work::KIND_ARCHITECTURE || under.is_empty() {
            continue;
        }
        for node in nodes {
            if node.path == under
                && node.status != structure::NODE_REMOVED
                && node.migrating()
                && !clears_migration(doc, &under)
            {
                diags.push(Diagnostic::hint(
                    card.line,
                    format!(
                        "architecture change {} is delivered but node {} is still migrating {} -> {}: clear migrating: on it",
                        card.slug, under, node.migration_from, node.migration_to
                    ),
                ));
            }
        }
    }
    diags
}

fn clears_migration(doc: &Document, path: &str) -> bool {
    for node in &doc.nodes {
        if node.path != path {
            continue;
        }
        if node
            .meta_value("migrating")
            .is_some_and(|value| value.trim().is_empty())
        {
            return true;
        }
    }
    false
}

/// Hints when a text moves a card to a later stage with a blocking question
/// still open.
pub fn stage_hints(doc: &Document, cards: &[work::Card]) -> Vec<Diagnostic> {
    let active: Vec<&work::Card> = cards.iter().collect();
    let mut diags = Vec::new();
    for node in &doc.cards {
        let Some(entry) = node.meta.iter().rev().find(|meta| meta.key == "stage") else {
            continue;
        };
        let Some(stored) = find_card(&active, &node.slug) else {
            continue;
        };
        if stage_index(&entry.value) <= stage_index(&stored.stage) {
            continue;
        }
        for question in &stored.questions {
            if question.state != work::QUESTION_OPEN
                || !question.blocking
                || answered_in(node, &question.text)
            {
                continue;
            }
            diags.push(Diagnostic::hint(
                entry.line,
                format!(
                    "stage {} with blocking question {} open: answer it with `? {} -> <answer>`",
                    entry.value,
                    go_quote(&question.text),
                    question.text
                ),
            ));
        }
    }
    diags
}

fn stage_index(stage: &str) -> i32 {
    work::STAGES
        .iter()
        .position(|candidate| *candidate == stage)
        .map_or(-1, |index| index as i32)
}

fn answered_in(node: &CardNode, text: &str) -> bool {
    let want = work::normalize_text(text);
    node.questions
        .iter()
        .any(|question| !question.answer.is_empty() && work::normalize_text(&question.text) == want)
}

/// Compares declared handoff branches and touched files with git. Mismatches
/// are hints; they never block the write.
pub fn handoff_hints(doc: &Document, git: &GitView) -> Vec<Diagnostic> {
    if !git.available {
        return Vec::new();
    }
    let observed = file_set(&git.files);
    let mut diags = Vec::new();
    for card in &doc.cards {
        let Some(handoff) = &card.handoff else {
            continue;
        };
        if !handoff.branch.is_empty() && !git.branch.is_empty() && handoff.branch != git.branch {
            diags.push(Diagnostic::hint(
                handoff.line,
                format!(
                    "handoff branch {} does not match git branch {}",
                    go_quote(&handoff.branch),
                    go_quote(&git.branch)
                ),
            ));
        }
        if handoff.touched.is_empty() {
            continue;
        }
        let declared = file_set(&handoff.touched);
        let mut missing: Vec<&String> = observed
            .iter()
            .filter(|path| !declared.contains(*path))
            .collect();
        let mut extra: Vec<&String> = declared
            .iter()
            .filter(|path| !observed.contains(*path))
            .collect();
        if missing.is_empty() && extra.is_empty() {
            continue;
        }
        missing.sort();
        extra.sort();
        diags.push(Diagnostic::hint(
            handoff.line,
            format!(
                "touched files differ from the git diff: not declared {}; not in the diff {}",
                list_or_none(&missing),
                list_or_none(&extra)
            ),
        ));
    }
    diags
}

fn file_set(paths: &[String]) -> HashSet<String> {
    let mut out = HashSet::new();
    for path in paths {
        let path = path.trim().replace('\\', "/");
        if path.is_empty() || path == "." {
            continue;
        }
        out.insert(path);
    }
    out
}

fn list_or_none(paths: &[&String]) -> String {
    if paths.is_empty() {
        return "none".to_string();
    }
    paths
        .iter()
        .map(|path| path.as_str())
        .collect::<Vec<&str>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::polter::decode::{SAMPLE, has_diag};

    fn node(id: &str, path: &str, level: &str) -> structure::Node {
        structure::Node {
            id: id.into(),
            path: path.into(),
            level: level.into(),
            status: structure::NODE_ACTIVE.into(),
            ..structure::Node::default()
        }
    }

    #[test]
    fn lint_resolves_decision_references() {
        let (_, diags) = lint(
            SAMPLE,
            Some(&References {
                decisions: vec![DecisionRef {
                    id: "decision-x".into(),
                    title: "Decision 0007: Deterministic import".into(),
                    ..DecisionRef::default()
                }],
                ..References::default()
            }),
        );
        assert!(
            !has_errors(&diags),
            "Decision-0007 should resolve: {diags:?}"
        );
        let (_, diags) = lint(SAMPLE, Some(&References::default()));
        assert!(has_diag(&diags, 10, "unknown rule"), "{diags:?}");
    }

    #[test]
    fn lint_hints_at_a_superseded_decision() {
        let references = References {
            decisions: vec![DecisionRef {
                id: "decision-x".into(),
                title: "Decision 0007: Deterministic import".into(),
                status: work::RULE_REPLACED.into(),
                ..DecisionRef::default()
            }],
            ..References::default()
        };
        let (_, diags) = lint(SAMPLE, Some(&references));
        assert!(!has_errors(&diags), "{diags:?}");
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains("cite its successor")),
            "{diags:?}"
        );
    }

    #[test]
    fn a_card_under_needs_a_component() {
        let nodes = vec![node("n1", "core/import", structure::LEVEL_COMPONENT)];
        let (_, diags) = lint(
            "---\npolter: 1\n---\n## git-import: Git import\nunder: missing\n",
            Some(&References {
                nodes: nodes.clone(),
                ..References::default()
            }),
        );
        assert!(has_diag(&diags, 4, "unknown under"), "{diags:?}");
        let (_, diags) = lint(
            "---\npolter: 1\n---\n## git-import: Git import\nunder: core/import\n",
            Some(&References {
                nodes: nodes.clone(),
                ..References::default()
            }),
        );
        assert!(!has_errors(&diags), "a component under passes: {diags:?}");
        let (_, diags) = lint(
            "---\npolter: 1\n---\n## git-import: Git import\nunder: core/import\n",
            Some(&References::default()),
        );
        assert!(
            !has_errors(&diags),
            "with no components, a legacy under passes: {diags:?}"
        );
    }

    #[test]
    fn lint_reports_a_node_parent_that_was_not_defined() {
        let (_, diags) = lint(
            "---\npolter: 1\n---\n# core/import: Import\nlevel: component\nunder: core\n",
            Some(&References::default()),
        );
        assert!(has_diag(&diags, 4, "unknown under"), "{diags:?}");
    }

    #[test]
    fn lint_accepts_a_component_under_a_domain() {
        let nodes = vec![node("s1", "app", structure::LEVEL_DOMAIN)];
        let (_, diags) = lint(
            "---\npolter: 1\n---\n# app/cli: CLI\nlevel: component\nunder: app\n",
            Some(&References {
                nodes,
                ..References::default()
            }),
        );
        assert!(!has_errors(&diags), "{diags:?}");
    }

    #[test]
    fn lint_rejects_the_retired_uses_key() {
        let nodes = vec![node("s1", "app", structure::LEVEL_DOMAIN)];
        let (_, diags) = lint(
            "---\npolter: 1\n---\n# app/cli: CLI\nlevel: component\nunder: app\nuses: app\n",
            Some(&References {
                nodes,
                ..References::default()
            }),
        );
        assert!(has_diag(&diags, 7, "uses: is retired"), "{diags:?}");
    }

    #[test]
    fn lint_reports_unknown_repos() {
        let (_, diags) = lint(
            "---\npolter: 1\n---\n## a: A\n- [ ] T1 Fix [repo: missing]\n",
            Some(&References {
                repos: vec!["api".into()],
                ..References::default()
            }),
        );
        assert!(has_diag(&diags, 5, "unknown repo"), "{diags:?}");
    }

    #[test]
    fn lint_hints_at_a_stage_move_with_a_blocking_question() {
        let cards = vec![work::Card {
            slug: "git-import".into(),
            stage: work::STAGE_BUILD.into(),
            status: work::CARD_ACTIVE.into(),
            questions: vec![work::Question {
                text: "Include merge commits?".into(),
                blocking: true,
                state: work::QUESTION_OPEN.into(),
                ..work::Question::default()
            }],
            ..work::Card::default()
        }];
        let (_, diags) = lint(
            "---\npolter: 1\n---\n## git-import: Git import\nstage: verify\n",
            Some(&References {
                cards,
                ..References::default()
            }),
        );
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains("with blocking question")),
            "{diags:?}"
        );
        assert!(!has_errors(&diags), "{diags:?}");
    }

    #[test]
    fn lint_hints_at_a_handoff_that_does_not_match_git() {
        let text = "---\npolter: 1\n---\n## a: A\nhandoff:\nbranch: feat/x\ntouched: src/a.go\n";
        let git = GitView {
            available: true,
            branch: "main".into(),
            files: vec!["src/b.go".into()],
            delivered: false,
        };
        let (_, diags) = lint(
            text,
            Some(&References {
                git: Some(git),
                ..References::default()
            }),
        );
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains("does not match git branch")),
            "{diags:?}"
        );
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains("not declared src/b.go")),
            "{diags:?}"
        );
        assert!(!has_errors(&diags), "{diags:?}");
    }

    #[test]
    fn lint_hints_that_a_delivered_change_still_migrates() {
        let card = work::Card {
            slug: "move-core".into(),
            kind: work::KIND_ARCHITECTURE.into(),
            under: "core".into(),
            status: work::CARD_ACTIVE.into(),
            ..work::Card::default()
        };
        let nodes = vec![structure::Node {
            migration_from: "old".into(),
            migration_to: "new".into(),
            ..node("c", "core", structure::LEVEL_DOMAIN)
        }];
        let (_, diags) = lint(
            "---\npolter: 1\n---\n## move-core: Move\nstage: delivered\n",
            Some(&References {
                cards: vec![card],
                nodes,
                ..References::default()
            }),
        );
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains("is still migrating old -> new")),
            "{diags:?}"
        );
    }

    #[test]
    fn cites_by_id_or_number() {
        assert!(cites_decision("decision-x", "decision-x", "", "x"));
        assert!(cites_decision(
            "Decision-0007",
            "decision-x",
            "",
            "Decision 0007: T"
        ));
        assert!(cites_decision(
            "ADR-7",
            "decision-x",
            "",
            "Decision 0007: T"
        ));
        assert!(!cites_decision(
            "Decision-0008",
            "decision-x",
            "",
            "Decision 0007: T"
        ));
        assert!(!cites_decision(
            "nope",
            "decision-x",
            "",
            "Decision 0007: T"
        ));
        assert!(cites_decision(
            "Toolbar-Fade",
            "rule-1",
            "toolbar-fade",
            "Fade."
        ));
        assert!(!cites_decision("", "rule-1", "", "Fade."));
    }

    #[test]
    fn a_card_cites_a_rule_by_its_slug() {
        let text = "---\npolter: 1\nsummary: s\nrationale: r\n---\n\n## fade: Fade\nstage: shape \u{b7} rules: toolbar-fade\n\nIntent: fade.\n";
        let references = References {
            decisions: vec![DecisionRef {
                id: "rule-1".into(),
                slug: "toolbar-fade".into(),
                title: "Page toolbars float on the fade.".into(),
                status: "active".into(),
            }],
            ..References::default()
        };
        let (_, diags) = lint(text, Some(&references));
        assert!(!has_errors(&diags), "{diags:?}");
    }
}
