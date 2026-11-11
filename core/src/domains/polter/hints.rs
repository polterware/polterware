//! Port of `backend/domains/polter/hints.go`: deterministic similarity checks
//! against the project's current cards.
//!
//! Duplicate warnings for new cards, questions, and invariants, and a placement
//! suggestion for new cards without `under:`. They never block a write. Matching
//! is word-set overlap after normalization (case, accents, plurals, stop words,
//! word order); no embeddings.

use std::collections::{HashMap, HashSet};

use crate::domains::work;

use super::model::{CardNode, Diagnostic, Document, go_quote};

/// Checks a text against the project's active cards.
pub fn check_hints(doc: &Document, cards: &[work::Card]) -> Vec<Diagnostic> {
    let active: Vec<&work::Card> = cards
        .iter()
        .filter(|card| card.status == work::CARD_ACTIVE)
        .collect();
    if active.is_empty() {
        return Vec::new();
    }
    let mut diags = Vec::new();
    let mut hint = |line: i32, message: String| diags.push(Diagnostic::hint(line, message));
    for node in &doc.cards {
        let existing = active.iter().find(|card| card.slug == node.slug).copied();
        if existing.is_none() {
            if node.meta_value("status") == Some(work::CARD_REMOVED) {
                continue;
            }
            if let Some((matched, _)) = similar_card(node, &active) {
                hint(
                    node.line,
                    format!(
                        "similar to card {} ({}): same? reuse the slug",
                        go_quote(&matched.slug),
                        matched.title
                    ),
                );
            }
            if node.meta_value("under").is_none()
                && let Some(under) = suggest_under(node, &active)
            {
                hint(
                    node.line,
                    format!(
                        "no under: similar cards sit under {0}: add under: {1}",
                        go_quote(&under),
                        under
                    ),
                );
            }
        }
        for question in &node.questions {
            if question.removed || existing.is_some_and(|card| has_question(card, &question.text)) {
                continue;
            }
            if let Some((slug, text, _)) = similar_question(&question.text, &active) {
                hint(
                    question.line,
                    format!(
                        "similar to open question on {}: {}: same? keep one",
                        go_quote(&slug),
                        go_quote(&text)
                    ),
                );
            }
        }
        for invariant in &node.invariants {
            if invariant.removed
                || existing.is_some_and(|card| has_invariant(card, &invariant.text))
            {
                continue;
            }
            if let Some((slug, text, _)) = similar_invariant(&invariant.text, &active) {
                hint(
                    invariant.line,
                    format!(
                        "similar to invariant on {}: {}: same? keep one",
                        go_quote(&slug),
                        go_quote(&text)
                    ),
                );
            }
        }
    }
    diags
}

pub(crate) fn find_card<'a>(cards: &[&'a work::Card], slug: &str) -> Option<&'a work::Card> {
    cards.iter().find(|card| card.slug == slug).copied()
}

fn has_question(card: &work::Card, text: &str) -> bool {
    let key = work::normalize_text(text);
    card.questions
        .iter()
        .any(|question| work::normalize_text(&question.text) == key)
}

fn has_invariant(card: &work::Card, text: &str) -> bool {
    let key = work::normalize_text(text);
    card.invariants
        .iter()
        .any(|invariant| work::normalize_text(&invariant.text) == key)
}

fn card_words(slug: &str, title: &str) -> WordSet {
    let mut words = WordSet::new(title);
    words.add(&WordSet::new(&slug.replace('-', " ")));
    words
}

/// Returns the closest existing card to a new card node.
fn similar_card<'a>(node: &CardNode, cards: &[&'a work::Card]) -> Option<(&'a work::Card, f64)> {
    let words = card_words(&node.slug, &node.title);
    let mut best: Option<&work::Card> = None;
    let mut best_score = 0.0;
    for card in cards {
        if let Some(score) = words.similar(&card_words(&card.slug, &card.title))
            && score > best_score
        {
            best = Some(card);
            best_score = score;
        }
    }
    best.filter(|_| best_score > 0.0)
        .map(|card| (card, best_score))
}

fn similar_question(text: &str, cards: &[&work::Card]) -> Option<(String, String, f64)> {
    let words = WordSet::new(text);
    let mut found: Option<(String, String, f64)> = None;
    let mut best_score = 0.0;
    for card in cards {
        for question in &card.questions {
            if question.state != work::QUESTION_OPEN {
                continue;
            }
            if let Some(score) = words.similar(&WordSet::new(&question.text))
                && score > best_score
            {
                found = Some((card.slug.clone(), question.text.clone(), score));
                best_score = score;
            }
        }
    }
    found.filter(|_| best_score > 0.0)
}

fn similar_invariant(text: &str, cards: &[&work::Card]) -> Option<(String, String, f64)> {
    let words = WordSet::new(text);
    let mut found: Option<(String, String, f64)> = None;
    let mut best_score = 0.0;
    for card in cards {
        for invariant in &card.invariants {
            if invariant.state != work::INVARIANT_ACTIVE {
                continue;
            }
            if let Some(score) = words.similar(&WordSet::new(&invariant.text))
                && score > best_score
            {
                found = Some((card.slug.clone(), invariant.text.clone(), score));
                best_score = score;
            }
        }
    }
    found.filter(|_| best_score > 0.0)
}

/// Picks the `under:` value whose path and cards share the most words with a
/// new card. It needs one shared path word or two shared words overall, and
/// stays silent on a tie.
fn suggest_under(node: &CardNode, cards: &[&work::Card]) -> Option<String> {
    let words = card_words(&node.slug, &node.title);
    struct Candidate {
        path: WordSet,
        all: WordSet,
    }
    let mut candidates: HashMap<String, Candidate> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    for card in cards {
        if card.under.is_empty() {
            continue;
        }
        if !candidates.contains_key(&card.under) {
            candidates.insert(
                card.under.clone(),
                Candidate {
                    path: WordSet::new(&card.under),
                    all: WordSet::new(&card.under),
                },
            );
            order.push(card.under.clone());
        }
        if let Some(candidate) = candidates.get_mut(&card.under) {
            candidate.all.add(&card_words(&card.slug, &card.title));
        }
    }
    let (mut best, mut best_shared, mut best_path, mut tie) =
        (String::new(), 0usize, 0usize, false);
    for under in &order {
        let Some(candidate) = candidates.get(under) else {
            continue;
        };
        let shared = words.shared(&candidate.all);
        let path_shared = words.shared(&candidate.path);
        if path_shared == 0 && shared < 2 {
            continue;
        }
        if shared > best_shared || (shared == best_shared && path_shared > best_path) {
            best = under.clone();
            best_shared = shared;
            best_path = path_shared;
            tie = false;
        } else if shared == best_shared && path_shared == best_path {
            tie = true;
        }
    }
    if best.is_empty() || tie {
        return None;
    }
    Some(best)
}

/// A normalized bag of meaningful words.
#[derive(Debug, Clone, Default)]
struct WordSet(HashSet<String>);

impl WordSet {
    fn new(text: &str) -> Self {
        let mut words = HashSet::new();
        for word in work::normalize_text(text).split_whitespace() {
            if is_stop_word(word) {
                continue;
            }
            words.insert(singular(word));
        }
        Self(words)
    }

    fn add(&mut self, other: &WordSet) {
        for word in &other.0 {
            self.0.insert(word.clone());
        }
    }

    fn shared(&self, other: &WordSet) -> usize {
        self.0.intersection(&other.0).count()
    }

    /// Reports the Jaccard score when two sets are close enough to hint: at
    /// least half their union shared, and two shared words unless they are
    /// equal.
    fn similar(&self, other: &WordSet) -> Option<f64> {
        let shared = self.shared(other);
        if shared == 0 {
            return None;
        }
        let score = shared as f64 / (self.0.len() + other.0.len() - shared) as f64;
        if score >= 0.5 && (shared >= 2 || score == 1.0) {
            return Some(score);
        }
        None
    }
}

fn singular(word: &str) -> String {
    let len = word.len();
    if len > 4 && word.ends_with("ies") {
        return format!("{}y", &word[..len - 3]);
    }
    if len > 3
        && word.ends_with('s')
        && !word.ends_with("ss")
        && !word.ends_with("us")
        && !word.ends_with("is")
    {
        return word[..len - 1].to_string();
    }
    word.to_string()
}

fn is_stop_word(word: &str) -> bool {
    matches!(
        word,
        "a" | "an"
            | "and"
            | "are"
            | "as"
            | "at"
            | "be"
            | "by"
            | "can"
            | "do"
            | "does"
            | "for"
            | "from"
            | "has"
            | "have"
            | "how"
            | "if"
            | "in"
            | "into"
            | "is"
            | "it"
            | "its"
            | "not"
            | "of"
            | "on"
            | "or"
            | "should"
            | "so"
            | "that"
            | "the"
            | "this"
            | "to"
            | "we"
            | "what"
            | "when"
            | "where"
            | "which"
            | "who"
            | "why"
            | "will"
            | "with"
            | "add"
            | "new"
            | "support"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::polter::decode::decode;

    fn cards() -> Vec<work::Card> {
        vec![
            work::Card {
                slug: "git-import".into(),
                title: "Git import".into(),
                status: work::CARD_ACTIVE.into(),
                under: "core/import".into(),
                questions: vec![work::Question {
                    text: "Include merge commits?".into(),
                    state: work::QUESTION_OPEN.into(),
                    ..work::Question::default()
                }],
                invariants: vec![work::Invariant {
                    text: "No network calls during scan".into(),
                    state: work::INVARIANT_ACTIVE.into(),
                    ..work::Invariant::default()
                }],
                ..work::Card::default()
            },
            work::Card {
                slug: "commits-panel".into(),
                title: "Commits panel".into(),
                status: work::CARD_ACTIVE.into(),
                under: "desktop/reality".into(),
                ..work::Card::default()
            },
        ]
    }

    #[test]
    fn hints_at_duplicates_and_placement() {
        let (doc, diags) = decode(
            "---\npolter: 1\n---\n## import-git-metadata: Import Git metadata\n? Should we include merge commits?\n! No network calls during the scan\n\n## import-cache: Import cache\n",
        );
        assert!(diags.is_empty(), "{diags:?}");
        let diags = check_hints(&doc, &cards());
        assert!(
            diags.iter().all(|d| d.severity == "hint"),
            "hints must not be errors: {diags:?}"
        );
        for (line, fragment) in [
            (4, "similar to card \"git-import\""),
            (5, "similar to open question on \"git-import\""),
            (6, "similar to invariant on \"git-import\""),
            (8, "add under: core/import"),
        ] {
            assert!(
                diags
                    .iter()
                    .any(|d| d.line == line && d.message.contains(fragment)),
                "missing hint {line} {fragment:?} in {diags:?}"
            );
        }
        assert!(
            !diags
                .iter()
                .any(|d| d.line == 8 && d.message.contains("similar to card")),
            "import-cache is not a duplicate: {diags:?}"
        );
    }

    #[test]
    fn a_known_card_stays_silent() {
        let (doc, _) =
            decode("---\npolter: 1\n---\n## git-import: Git import\n? Include merge commits?\n");
        let diags = check_hints(&doc, &cards());
        assert!(diags.is_empty(), "known card should be silent: {diags:?}");
    }

    #[test]
    fn normalizes_words() {
        assert_eq!(singular("commits"), "commit");
        assert_eq!(singular("queries"), "query");
        assert_eq!(singular("status"), "status");
        assert_eq!(singular("class"), "class");
        assert!(is_stop_word("the"));
        let words = WordSet::new("The Git Imports");
        assert!(words.0.contains("git"), "stop words drop and plurals fold");
        assert!(words.0.contains("import"), "the plural folds");
    }

    #[test]
    fn no_active_cards_hint_nothing() {
        let (doc, _) = decode("---\npolter: 1\n---\n## import-cache: Import cache\n");
        assert!(check_hints(&doc, &[]).is_empty());
    }
}
