//! Port of `backend/domains/work` (`work.go`; the package's own file).
//!
//! The relational Work model: cards (features) with tasks, open questions, and
//! contract invariants. Agents write it through the Polter codec; every
//! mutation is an Op recorded in the change operation log.

mod claim;
mod milestone;

pub use claim::*;
pub use milestone::*;

use std::sync::LazyLock;

use regex::Regex;
use serde_json::{Map, Value};
use time::OffsetDateTime;

use crate::domains::structure;

pub const STAGE_SHAPE: &str = "shape";
pub const STAGE_PLAN: &str = "plan";
pub const STAGE_BUILD: &str = "build";
pub const STAGE_VERIFY: &str = "verify";
pub const STAGE_DELIVERED: &str = "delivered";

pub const KIND_FEATURE: &str = "feature";
pub const KIND_ARCHITECTURE: &str = "architecture";

pub const CARD_ACTIVE: &str = "active";
pub const CARD_REMOVED: &str = "removed";

pub const TASK_OPEN: &str = "open";
pub const TASK_DONE: &str = "done";
pub const TASK_DROPPED: &str = "dropped";

pub const QUESTION_OPEN: &str = "open";
pub const QUESTION_ANSWERED: &str = "answered";
pub const QUESTION_REMOVED: &str = "removed";

pub const INVARIANT_ACTIVE: &str = "active";
pub const INVARIANT_REMOVED: &str = "removed";

/// The card stages in board order.
pub const STAGES: [&str; 5] = [
    STAGE_SHAPE,
    STAGE_PLAN,
    STAGE_BUILD,
    STAGE_VERIFY,
    STAGE_DELIVERED,
];

/// Reports whether `stage` is one of the card stages.
pub fn valid_stage(stage: &str) -> bool {
    STAGES.contains(&stage)
}

/// Reports whether `status` is a card status.
pub fn valid_card_status(status: &str) -> bool {
    status == CARD_ACTIVE || status == CARD_REMOVED
}

static SLUG_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").expect("the slug pattern"));

/// Reports whether `slug` is a lowercase kebab-case handle.
pub fn valid_slug(slug: &str) -> bool {
    slug.len() <= 80 && SLUG_PATTERN.is_match(slug)
}

/// Folds case, accents, punctuation, and whitespace so that two spellings of
/// the same task or question compare equal.
pub fn normalize_text(value: &str) -> String {
    use unicode_normalization::UnicodeNormalization;

    let mut out = String::new();
    let mut space = false;
    for r in value.nfd() {
        if unicode_normalization::char::is_combining_mark(r) {
            continue;
        }
        if r.is_alphabetic() || r.is_numeric() {
            if space && !out.is_empty() {
                out.push(' ');
            }
            space = false;
            if let Some(lower) = r.to_lowercase().next() {
                out.push(lower);
            }
        } else {
            space = true;
        }
    }
    out
}

/// Reports whether `kind` is a card kind.
pub fn valid_kind(kind: &str) -> bool {
    kind == KIND_FEATURE || kind == KIND_ARCHITECTURE
}

#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub id: String,
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub intent: String,
    /// What the feature does: the spec's `behavior:` section.
    pub behavior: String,
    /// The cases the feature must handle: the spec's `edge-cases:` section.
    pub edge_cases: String,
    /// Architecture, APIs, and files touched: the `plan:` section.
    pub plan: String,
    pub stage: String,
    pub status: String,
    pub owner: String,
    /// `feature` or `architecture` (a structural Structure change).
    pub kind: String,
    pub under: String,
    pub decisions: Vec<String>,
    /// The milestone the card ships in; empty is the Backlog. `milestone` is
    /// that milestone's slug, as a `milestone:` line writes it.
    pub milestone_id: String,
    pub milestone: String,
    pub position: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub tasks: Vec<Task>,
    pub questions: Vec<Question>,
    pub invariants: Vec<Invariant>,
    /// Ordered oldest first. The last row for a branch is current.
    pub handoffs: Vec<Handoff>,
}

impl Default for Card {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            slug: String::new(),
            title: String::new(),
            intent: String::new(),
            behavior: String::new(),
            edge_cases: String::new(),
            plan: String::new(),
            stage: String::new(),
            status: String::new(),
            owner: String::new(),
            kind: String::new(),
            under: String::new(),
            decisions: Vec::new(),
            milestone_id: String::new(),
            milestone: String::new(),
            position: 0,
            created_at: OffsetDateTime::UNIX_EPOCH,
            updated_at: OffsetDateTime::UNIX_EPOCH,
            tasks: Vec::new(),
            questions: Vec::new(),
            invariants: Vec::new(),
            handoffs: Vec::new(),
        }
    }
}

impl Card {
    /// Returns the newest handoff recorded for `branch`.
    pub fn latest_handoff(&self, branch: &str) -> Option<&Handoff> {
        self.handoffs
            .iter()
            .rev()
            .find(|handoff| handoff.branch == branch)
    }

    /// Returns the newest handoff on the card, any branch.
    pub fn latest_handoff_any(&self) -> Option<&Handoff> {
        self.handoffs.last()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Task {
    pub id: String,
    pub card_id: String,
    pub number: i32,
    pub title: String,
    pub state: String,
    /// A project_repos name; empty is the project root checkout.
    pub repo: String,
    pub scope: String,
    pub position: i32,
    /// The task's active claim, or `None` when the task is free.
    pub claim: Option<Claim>,
}

/// Joins one commit of a checkout (an empty repo is the project root) to one
/// task: one an agent or a person linked. Subject, URL (the commit on GitHub,
/// or empty), and CommittedAt are read from the checkout when linking.
#[derive(Debug, Clone, PartialEq)]
pub struct CommitLink {
    pub id: String,
    pub project_id: String,
    pub repo: String,
    pub sha: String,
    pub task_id: String,
    pub subject: String,
    pub url: String,
    pub committed_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl Default for CommitLink {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            repo: String::new(),
            sha: String::new(),
            task_id: String::new(),
            subject: String::new(),
            url: String::new(),
            committed_at: OffsetDateTime::UNIX_EPOCH,
            created_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Question {
    pub id: String,
    pub card_id: String,
    pub text: String,
    pub blocking: bool,
    pub options: Vec<String>,
    pub state: String,
    pub answer: String,
    pub position: i32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Invariant {
    pub id: String,
    pub card_id: String,
    pub text: String,
    pub state: String,
    pub position: i32,
}

/// One session record for a card on a Git branch. Rows are append-only; the
/// latest row for a card and branch is the current one.
#[derive(Debug, Clone, PartialEq)]
pub struct Handoff {
    pub id: String,
    pub project_id: String,
    pub card_id: String,
    pub branch: String,
    pub goal: String,
    pub done: String,
    pub in_progress: String,
    pub next: String,
    pub touched: Vec<String>,
    pub agent: String,
    pub created_at: OffsetDateTime,
}

impl Default for Handoff {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            card_id: String::new(),
            branch: String::new(),
            goal: String::new(),
            done: String::new(),
            in_progress: String::new(),
            next: String::new(),
            touched: Vec::new(),
            agent: String::new(),
            created_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

/// The current Work graph of one project, plus its Structure nodes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Snapshot {
    pub project_id: String,
    pub cards: Vec<Card>,
    pub nodes: Vec<structure::Node>,
    /// The project's repositories, which components link by name.
    pub repos: Vec<String>,
    /// The project's environments. A `deploys:` line names one by id or by
    /// name.
    pub environments: Vec<NamedEnvironment>,
    /// Every Rule of the project, in position order.
    pub rules: Vec<Rule>,
    /// Every Conflict of the project, newest first.
    pub conflicts: Vec<Conflict>,
    /// Every milestone, archived ones included, so a card's stored milestone
    /// still resolves.
    pub milestones: Vec<Milestone>,
}

/// A short imperative constraint agents read before they work. Scopes are the
/// paths of the Structure nodes it applies to, sorted; none is the whole
/// project, and a node's Rules hold for every node below it.
///
/// A Rule is the only law: it carries an optional Markdown `why` (the old
/// Decision body) and, when replaced, the id of the Rule that replaced it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub id: String,
    pub slug: String,
    pub statement: String,
    pub why: String,
    pub scopes: Vec<String>,
    pub status: String,
    pub replaced_by: String,
    pub position: i32,
}

impl Default for Rule {
    fn default() -> Self {
        Self {
            id: String::new(),
            slug: String::new(),
            statement: String::new(),
            why: String::new(),
            scopes: Vec::new(),
            status: RULE_ACTIVE.to_string(),
            replaced_by: String::new(),
            position: 0,
        }
    }
}

/// Rule states.
pub const RULE_PROPOSED: &str = "proposed";
pub const RULE_ACTIVE: &str = "active";
pub const RULE_UNDER_REVIEW: &str = "under_review";
pub const RULE_REPLACED: &str = "replaced";
pub const RULE_REMOVED: &str = "removed";

/// The Rule states in display order.
pub const RULE_STATUSES: [&str; 5] = [
    RULE_PROPOSED,
    RULE_ACTIVE,
    RULE_UNDER_REVIEW,
    RULE_REPLACED,
    RULE_REMOVED,
];

/// Reports whether `status` is a Rule status.
pub fn valid_rule_status(status: &str) -> bool {
    RULE_STATUSES.contains(&status)
}

/// Reports whether a Rule in `status` constrains work: active and under review
/// Rules bind until they are resolved.
pub fn binding_rule_status(status: &str) -> bool {
    status == RULE_ACTIVE || status == RULE_UNDER_REVIEW
}

/// A request that goes against a Rule. An agent raises it instead of acting
/// against the law; a person decides it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Conflict {
    pub id: String,
    pub slug: String,
    pub request: String,
    pub rule_id: String,
    /// The Rule named by a legacy `decision:` line, read as a Rule id.
    pub decision: String,
    pub card_slug: String,
    pub options: Vec<String>,
    pub state: String,
    pub answer: String,
    /// `created_at` and `decided_at` are RFC 3339; `decided_at` is empty while
    /// open.
    pub created_at: String,
    pub decided_at: String,
}

/// Conflict states.
pub const CONFLICT_OPEN: &str = "open";
pub const CONFLICT_DECIDED: &str = "decided";
pub const CONFLICT_WITHDRAWN: &str = "withdrawn";

impl Snapshot {
    /// Returns the project's Conflict with this slug.
    pub fn conflict(&self, slug: &str) -> Option<&Conflict> {
        self.conflicts.iter().find(|conflict| conflict.slug == slug)
    }

    /// Reports whether `name` is a registered checkout. Empty is the root.
    pub fn has_repo(&self, name: &str) -> bool {
        if name.is_empty() {
            return true;
        }
        self.repos.iter().any(|repo| repo == name)
    }

    /// Returns the node with `path`, including removed nodes.
    pub fn node_by_path(&self, path: &str) -> Option<&structure::Node> {
        self.nodes.iter().find(|node| node.path == path)
    }

    /// Returns the card with `slug`, if present.
    pub fn card(&self, slug: &str) -> Option<&Card> {
        self.cards.iter().find(|card| card.slug == slug)
    }
}

impl Rule {
    /// The Polter reference of a Rule: its slug, or its id when it has none.
    pub fn handle(&self) -> &str {
        if self.slug.is_empty() {
            &self.id
        } else {
            &self.slug
        }
    }

    /// Reports whether this Rule constrains work: active and under review
    /// Rules bind until they are resolved.
    pub fn binding(&self) -> bool {
        binding_rule_status(&self.status)
    }
}

/// A project environment a `deploys:` line can name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NamedEnvironment {
    pub id: String,
    pub name: String,
}

/// One acceptance criterion (`invariant`) or Rule (`rule`) a person or an agent
/// confirmed for a card. `target_id` is the invariant's or the Rule's id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Confirmation {
    pub card_id: String,
    pub kind: String,
    pub target_id: String,
    pub confirmed_by: String,
    pub confirmed_at: OffsetDateTime,
}

impl Default for Confirmation {
    fn default() -> Self {
        Self {
            card_id: String::new(),
            kind: String::new(),
            target_id: String::new(),
            confirmed_by: String::new(),
            confirmed_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

/// What a confirmation is about.
pub const CONFIRMATION_INVARIANT: &str = "invariant";
pub const CONFIRMATION_RULE: &str = "rule";

pub const ENTITY_CARD: &str = "card";
pub const ENTITY_TASK: &str = "task";
pub const ENTITY_QUESTION: &str = "question";
pub const ENTITY_INVARIANT: &str = "invariant";
pub const ENTITY_HANDOFF: &str = "handoff";
pub const ENTITY_CLAIM: &str = "claim";
pub const ENTITY_NODE: &str = "node";
pub const ENTITY_RULE: &str = "rule";
pub const ENTITY_CONFLICT: &str = "conflict";
pub const ENTITY_MILESTONE: &str = "milestone";
/// One acceptance criterion or Rule a person or an agent confirmed for a card.
pub const ENTITY_CONFIRMATION: &str = "confirmation";

pub const OP_CREATE: &str = "create";
pub const OP_UPDATE: &str = "update";
pub const OP_REMOVE: &str = "remove";

/// A column map: Go's `map[string]any`.
pub type Fields = Map<String, Value>;

/// One planned mutation. `fields` holds only the columns that change (column
/// name to new value); for creates it holds every column.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Op {
    pub entity: String,
    pub kind: String,
    pub entity_id: String,
    pub card_id: String,
    pub card_slug: String,
    /// The Structure node an invariant belongs to. `card_id` stays empty.
    pub node_id: String,
    /// The human handle used in plan output: the task number (`T2`), or the
    /// question or invariant text.
    pub label: String,
    pub fields: Fields,
    /// Previous values of changed fields, for plan output only.
    pub before: Fields,
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;

    #[test]
    fn stages_are_in_board_order() {
        let want = ["shape", "plan", "build", "verify", "delivered"];
        assert_eq!(STAGES.len(), want.len(), "STAGES = {STAGES:?}");
        for (i, stage) in want.iter().enumerate() {
            assert_eq!(STAGES[i], *stage, "STAGES[{i}]");
        }
    }

    #[test]
    fn valid_stages() {
        for stage in STAGES {
            assert!(valid_stage(stage), "{stage:?} must be valid");
        }
        for stage in ["", "Shape", "done", "review", " build"] {
            assert!(!valid_stage(stage), "{stage:?} must be invalid");
        }
    }

    #[test]
    fn valid_card_status_and_kind() {
        assert!(
            valid_card_status(CARD_ACTIVE)
                && valid_card_status(CARD_REMOVED)
                && !valid_card_status("")
                && !valid_card_status("archived"),
            "card status validation is wrong"
        );
        assert!(
            valid_kind(KIND_FEATURE)
                && valid_kind(KIND_ARCHITECTURE)
                && !valid_kind("")
                && !valid_kind("bug"),
            "card kind validation is wrong"
        );
    }

    #[test]
    fn valid_slugs() {
        for slug in ["a", "git-import", "decision-0007", "v2", "a1-b2-c3"] {
            assert!(valid_slug(slug), "{slug:?} must be valid");
        }
        let long = "a".repeat(81);
        for slug in [
            "",
            "Git-Import",
            "-lead",
            "trail-",
            "double--dash",
            "under_score",
            "has space",
            "café",
            long.as_str(),
        ] {
            assert!(!valid_slug(slug), "{slug:?} must be invalid");
        }
        assert!(
            valid_slug(&long[..80]),
            "80 characters is the limit and must be valid"
        );
    }

    #[test]
    fn normalize_text_folds_case_accents_and_punctuation() {
        let same = [
            ("Upsert commit evidence", "  upsert   COMMIT evidence! "),
            ("Café résumé", "cafe resume"),
            ("Include merge commits?", "include merge commits"),
            (
                "wire-into ImportProjectSources",
                "wire into importprojectsources",
            ),
        ];
        for (a, b) in same {
            let (a, b) = (normalize_text(a), normalize_text(b));
            assert_eq!(a, b, "the spellings must normalize equal");
        }
        assert_ne!(
            normalize_text("Task one"),
            normalize_text("Task two"),
            "different text must stay different"
        );
        assert_eq!(
            normalize_text("  ?!  "),
            "",
            "punctuation only must normalize to empty"
        );
    }

    #[test]
    fn card_latest_handoff() {
        let base = OffsetDateTime::UNIX_EPOCH;
        let card = Card {
            handoffs: vec![
                Handoff {
                    id: "h1".into(),
                    branch: "main".into(),
                    goal: "first".into(),
                    created_at: base,
                    ..Handoff::default()
                },
                Handoff {
                    id: "h2".into(),
                    branch: "feat/x".into(),
                    goal: "other branch".into(),
                    created_at: base + Duration::minutes(1),
                    ..Handoff::default()
                },
                Handoff {
                    id: "h3".into(),
                    branch: "main".into(),
                    goal: "second".into(),
                    created_at: base + Duration::minutes(2),
                    ..Handoff::default()
                },
            ],
            ..Card::default()
        };
        let got = card.latest_handoff("main").expect("main has a handoff");
        assert_eq!(got.id, "h3", "LatestHandoff(main)");
        let got = card.latest_handoff("feat/x").expect("feat/x has a handoff");
        assert_eq!(got.id, "h2", "LatestHandoff(feat/x)");
        assert!(
            card.latest_handoff("nope").is_none(),
            "unknown branch must report not found"
        );
        let got = card.latest_handoff_any().expect("the card has handoffs");
        assert_eq!(got.id, "h3", "LatestHandoffAny");
        assert!(
            Card::default().latest_handoff_any().is_none(),
            "a card without handoffs must report not found"
        );
    }

    #[test]
    fn snapshot_lookups() {
        let snapshot = Snapshot {
            project_id: "p".into(),
            cards: vec![
                Card {
                    slug: "git-import".into(),
                    ..Card::default()
                },
                Card {
                    slug: "structure".into(),
                    ..Card::default()
                },
            ],
            nodes: vec![structure::Node {
                path: "core/import".into(),
                status: "removed".into(),
                ..structure::Node::default()
            }],
            repos: vec!["api".into()],
            ..Snapshot::default()
        };
        let card = snapshot.card("structure").expect("the card is present");
        assert_eq!(card.slug, "structure", "Card(structure)");
        assert!(
            snapshot.card("missing").is_none(),
            "unknown slug must not be found"
        );
        let node = snapshot
            .node_by_path("core/import")
            .expect("removed nodes stay readable");
        assert_eq!(
            node.status, "removed",
            "NodeByPath must include removed nodes"
        );
        assert!(
            snapshot.node_by_path("core").is_none(),
            "paths match exactly"
        );
        assert!(
            snapshot.has_repo("") && snapshot.has_repo("api") && !snapshot.has_repo("web"),
            "HasRepo: empty is the root, listed names match, others do not"
        );
    }
}
