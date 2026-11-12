//! Shared Polter types: the decoded AST, its metadata entries, and the
//! constants the whole package reads.
//!
//! The decoder, the planners, and the encoders all read these types from here,
//! so the grammar has one definition.

/// The grammar version this codec reads and writes.
pub const VERSION: i32 = 1;

pub const SEVERITY_ERROR: &str = "error";
pub const SEVERITY_HINT: &str = "hint";

/// One line-numbered lint result. `message` already contains the fix.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diagnostic {
    pub line: i32,
    pub severity: String,
    pub message: String,
}

impl Diagnostic {
    /// Builds an error diagnostic.
    pub fn error(line: i32, message: impl Into<String>) -> Self {
        Self {
            line,
            severity: SEVERITY_ERROR.to_string(),
            message: message.into(),
        }
    }

    /// Builds a hint diagnostic.
    pub fn hint(line: i32, message: impl Into<String>) -> Self {
        Self {
            line,
            severity: SEVERITY_HINT.to_string(),
            message: message.into(),
        }
    }
}

/// Reports whether any diagnostic is an error.
pub fn has_errors(diags: &[Diagnostic]) -> bool {
    diags.iter().any(|d| d.severity == SEVERITY_ERROR)
}

/// The decoded AST of one Polter text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Document {
    pub version: i32,
    pub summary: String,
    pub rationale: String,
    pub nodes: Vec<StructureNode>,
    pub cards: Vec<CardNode>,
    pub decisions: Vec<DecisionNode>,
    pub facts: Vec<FactNode>,
    pub rules: Vec<RuleNode>,
    pub conflicts: Vec<ConflictNode>,
}

/// One `### ref: Title` block: a Decision lifecycle change. `reference` is
/// `new` for a proposal, otherwise a Decision id, number, or title. The body
/// after the metadata is the Decision text. Go calls this field `Ref`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DecisionNode {
    pub line: i32,
    pub reference: String,
    pub title: String,
    pub meta: Vec<MetaEntry>,
    pub body: String,
    pub body_set: bool,
}

impl DecisionNode {
    /// Returns the last value given for `key`.
    pub fn meta_value(&self, key: &str) -> Option<&str> {
        last_value(&self.meta, key)
    }
}

/// The decision metadata keys. `supersedes:` names the older decision the
/// block's decision replaces; `trigger:` is the review trigger; `reason:`
/// explains a challenge or a supersede; `level: principle` pins a decision for
/// the whole project.
pub const DECISION_META_KEYS: [&str; 6] = [
    "status",
    "level",
    "supersedes",
    "trigger",
    "reason",
    "scope",
];

/// The ref that proposes a new decision.
pub const DECISION_NEW_REF: &str = "new";

/// A domain or a component. `path` is the heading path (`core` or
/// `core/import`); the store materializes it with `under:`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructureNode {
    pub line: i32,
    pub slug: String,
    pub path: String,
    pub title: String,
    pub meta: Vec<MetaEntry>,
    pub intent: String,
    pub intent_set: bool,
    pub invariants: Vec<InvariantNode>,
}

impl StructureNode {
    /// Returns the last value given for `key`.
    pub fn meta_value(&self, key: &str) -> Option<&str> {
        last_value(&self.meta, key)
    }

    /// The line of the node's `key:` entry, or the node heading.
    pub fn meta_line(&self, key: &str) -> i32 {
        meta_entry_line(&self.meta, key, self.line)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetaEntry {
    pub key: String,
    pub value: String,
    pub line: i32,
}

impl MetaEntry {
    /// Builds an entry.
    pub fn new(key: impl Into<String>, value: impl Into<String>, line: i32) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            line,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CardNode {
    pub line: i32,
    pub slug: String,
    pub title: String,
    pub meta: Vec<MetaEntry>,
    pub intent: String,
    pub intent_set: bool,
    /// What the feature does: the spec's `behavior:` section.
    pub behavior: String,
    pub behavior_set: bool,
    /// The cases the feature must handle: the spec's `edge-cases:` section.
    pub edge_cases: String,
    pub edge_cases_set: bool,
    /// Architecture, APIs, and files touched: the `plan:` section.
    pub plan: String,
    pub plan_set: bool,
    pub tasks: Vec<TaskNode>,
    pub questions: Vec<QuestionNode>,
    pub invariants: Vec<InvariantNode>,
    pub handoff: Option<HandoffNode>,
    /// Claim, done, and release lines. They act on stored tasks for the
    /// session's agent and are never encoded back.
    pub coordination: Vec<CoordNode>,
    /// `confirm:` lines: one acceptance criterion or one Rule confirmed for
    /// this card. They are never encoded back.
    pub confirmations: Vec<ConfirmNode>,
}

/// What a `confirm:` line confirms: one acceptance criterion (`!` invariant) or
/// one Rule.
pub const CONFIRM_INVARIANT: &str = "invariant";
pub const CONFIRM_RULE: &str = "rule";

/// One confirmation line: `confirm: ! <criterion text>` or
/// `confirm: rule <slug or id>`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConfirmNode {
    pub line: i32,
    /// [`CONFIRM_INVARIANT`] or [`CONFIRM_RULE`].
    pub kind: String,
    /// The criterion's text, or the Rule's slug or id.
    pub target: String,
}

pub const COORD_CLAIM: &str = "claim";
pub const COORD_DONE: &str = "done";
pub const COORD_RELEASE: &str = "release";

/// One coordination line: `claim: T2`, `done: T2`, `release: T2`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CoordNode {
    pub line: i32,
    pub action: String,
    pub number: i32,
}

/// One handoff block inside a card. `branch` is required.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HandoffNode {
    pub line: i32,
    pub branch: String,
    pub agent: String,
    pub goal: String,
    pub done: String,
    pub in_progress: String,
    pub next: String,
    pub touched: Vec<String>,
}

impl CardNode {
    /// Returns the last value given for `key`.
    pub fn meta_value(&self, key: &str) -> Option<&str> {
        last_value(&self.meta, key)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaskNode {
    pub line: i32,
    pub number: i32,
    pub title: String,
    pub state: String,
    /// The checkout from `[repo: api]` or an `api:` scope prefix.
    pub repo: String,
    /// The glob without its checkout prefix.
    pub scope: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QuestionNode {
    pub line: i32,
    pub text: String,
    pub blocking: bool,
    pub options: Vec<String>,
    pub removed: bool,
    /// The decision written after " -> "; it closes the question.
    pub answer: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InvariantNode {
    pub line: i32,
    pub text: String,
    pub removed: bool,
}

/// The card metadata keys grammar v1 accepts. `rules:` lists the Rules that
/// govern the card; the retired `decisions:` key is read as the same list.
pub const META_KEYS: [&str; 7] = [
    "stage",
    "owner",
    "under",
    "milestone",
    "rules",
    "status",
    "kind",
];

/// The card sections that carry the spec and the plan. Each starts with its key
/// on a line of its own and keeps the lines after it as a Markdown body: the
/// counterpart of a Rule's `why:`. A body runs until the next section, a `#` or
/// `##` heading, or a `#### rule` / `#### conflict` block, so headings deeper
/// than the card, `!` lines, and checklists inside it stay text.
pub const CARD_SECTION_KEYS: [&str; 3] = ["behavior", "edge-cases", "plan"];

/// The Structure node metadata keys. `repo:` names a registered checkout
/// (empty is the project root); `path:` is a folder inside it. `kind:` is what
/// a component is and `tech:` its technology. `rules:` lists the Rules that
/// hold for the node; the retired `decisions:` key is read as the same list.
pub const NODE_META_KEYS: [&str; 11] = [
    "level",
    "kind",
    "tech",
    "under",
    "repo",
    "dir",
    "path",
    "rules",
    "deploys",
    "status",
    "migrating",
];

/// A `#### slug-or-id: Statement` canon Fact block.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FactNode {
    pub line: i32,
    pub reference: String,
    pub statement: String,
    pub meta: Vec<MetaEntry>,
}

/// The keys a Fact block takes.
pub const FACT_META_KEYS: [&str; 2] = ["kind", "scope"];

impl FactNode {
    /// Returns the last value given for `key`.
    pub fn meta_value(&self, key: &str) -> Option<&str> {
        last_value(&self.meta, key)
    }

    /// The line of the fact's `key:` entry, or the fact heading.
    pub fn meta_line(&self, key: &str) -> i32 {
        meta_entry_line(&self.meta, key, self.line)
    }
}

/// A `#### rule slug: Statement` block. A Rule is the law agents read before
/// they work; `scope:` names the Structure nodes it holds for, none is the
/// whole project, `why:` is the optional Markdown reason, `status:` is its
/// lifecycle state, and `replaced-by:` names the Rule that replaced it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuleNode {
    pub line: i32,
    pub reference: String,
    pub statement: String,
    pub meta: Vec<MetaEntry>,
    pub why: String,
    /// Reports whether the block carried `why:`, which may clear the reason.
    pub why_set: bool,
}

/// The single-line keys a Rule block takes. `why:` starts a body and is
/// handled separately.
pub const RULE_META_KEYS: [&str; 3] = ["scope", "status", "replaced-by"];

/// Starts a Rule heading inside `#### `.
pub const RULE_HEADING_PREFIX: &str = "rule ";

impl RuleNode {
    /// Returns the last value given for `key`.
    pub fn meta_value(&self, key: &str) -> Option<&str> {
        last_value(&self.meta, key)
    }

    /// The line of the rule's `key:` entry, or the rule heading.
    pub fn meta_line(&self, key: &str) -> i32 {
        meta_entry_line(&self.meta, key, self.line)
    }
}

/// A `#### conflict slug: Request` block: a request that goes against a Rule,
/// raised instead of acting against it. Each metadata line is one key, so a
/// request or an answer may hold any text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConflictNode {
    pub line: i32,
    pub reference: String,
    pub request: String,
    pub meta: Vec<MetaEntry>,
}

/// The keys a Conflict block takes.
pub const CONFLICT_META_KEYS: [&str; 6] =
    ["rule", "decision", "card", "options", "answer", "status"];

/// Starts a Conflict heading inside `#### `.
pub const CONFLICT_HEADING_PREFIX: &str = "conflict ";

impl ConflictNode {
    /// Returns the last value given for `key`.
    pub fn meta_value(&self, key: &str) -> Option<&str> {
        last_value(&self.meta, key)
    }

    /// The line of the conflict's `key:` entry, or the conflict heading.
    pub fn meta_line(&self, key: &str) -> i32 {
        meta_entry_line(&self.meta, key, self.line)
    }
}

/// Reads the last value given for `key`, as Go's `MetaValue` does.
fn last_value<'a>(meta: &'a [MetaEntry], key: &str) -> Option<&'a str> {
    meta.iter()
        .rev()
        .find(|entry| entry.key == key)
        .map(|entry| entry.value.as_str())
}

/// The line of the last `key:` entry, or `fallback`.
fn meta_entry_line(meta: &[MetaEntry], key: &str, fallback: i32) -> i32 {
    meta.iter()
        .rev()
        .find(|entry| entry.key == key)
        .map_or(fallback, |entry| entry.line)
}

/// Reports whether `value` is a stored Rule id: 26 uppercase base32
/// characters, the handle of a Rule with no slug.
pub fn valid_rule_id(value: &str) -> bool {
    value.len() == 26
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || b.is_ascii_uppercase())
}

/// Reads a comma separated list into its trimmed, non-empty parts.
pub fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

/// Quotes a value the way Go's `%q` quotes a string in these messages: a
/// double-quoted literal with the usual escapes.
pub fn go_quote(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for r in value.chars() {
        match r {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{7}' => out.push_str("\\a"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{b}' => out.push_str("\\v"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Normalizes a node `path:` value: a folder relative to its checkout. Empty
/// is the checkout root. Absolute paths and ".." are refused; the refused
/// value is returned unchanged so the caller can report it.
///
/// The folder text is cleaned once here, so a stored path and the one an agent
/// writes for it are the same.
pub fn clean_folder(value: &str) -> (String, bool) {
    let value = value.trim();
    if value.is_empty() {
        return (String::new(), true);
    }
    if value.starts_with('/') || value.contains('\\') || value.contains(['*', '?', '[']) {
        return (value.to_string(), false);
    }
    let cleaned = path_clean(value);
    if cleaned == "." {
        return (String::new(), true);
    }
    if cleaned == ".." || cleaned.starts_with("../") {
        return (value.to_string(), false);
    }
    (cleaned, true)
}

/// Go's `path.Clean`: the shortest lexical form of a slash separated path.
fn path_clean(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let rooted = path.starts_with('/');
    let src: Vec<u8> = path.as_bytes().to_vec();
    let mut buf: Vec<u8> = vec![0; src.len()];
    let (mut w, mut r, mut dotdot) = (0usize, 0usize, 0usize);
    if rooted {
        buf[0] = b'/';
        w = 1;
        r = 1;
        dotdot = 1;
    }
    while r < src.len() {
        if src[r] == b'/' || (src[r] == b'.' && (r + 1 == src.len() || src[r + 1] == b'/')) {
            r += 1;
        } else if src[r] == b'.'
            && r + 1 < src.len()
            && src[r + 1] == b'.'
            && (r + 2 == src.len() || src[r + 2] == b'/')
        {
            r += 2;
            if w > dotdot {
                w -= 1;
                while w > dotdot && buf[w] != b'/' {
                    w -= 1;
                }
            } else if !rooted {
                if w > 0 {
                    buf[w] = b'/';
                    w += 1;
                }
                buf[w] = b'.';
                w += 1;
                buf[w] = b'.';
                w += 1;
                dotdot = w;
            }
        } else {
            if (rooted && w != 1) || (!rooted && w != 0) {
                buf[w] = b'/';
                w += 1;
            }
            while r < src.len() && src[r] != b'/' {
                buf[w] = src[r];
                w += 1;
                r += 1;
            }
        }
    }
    if w == 0 {
        return ".".to_string();
    }
    String::from_utf8_lossy(&buf[..w]).into_owned()
}

/// Splits a `migrating:` value ("A -> B" or "A → B") into its two sides. An
/// empty value clears the state and returns empty sides.
pub fn parse_migration(value: &str) -> Option<(String, String)> {
    let value = value.trim();
    if value.is_empty() {
        return Some((String::new(), String::new()));
    }
    let value = value.replacen('→', "->", 1);
    let (left, right) = value.split_once("->")?;
    let (from, to) = (left.trim(), right.trim());
    if from.is_empty() || to.is_empty() {
        return None;
    }
    Some((from.to_string(), to.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_folder_normalizes_and_refuses() {
        assert_eq!(clean_folder(""), (String::new(), true), "empty is the root");
        assert_eq!(
            clean_folder(" src/auth/ "),
            ("src/auth".to_string(), true),
            "surrounding space and a trailing slash are dropped"
        );
        assert_eq!(
            clean_folder("./src//auth/."),
            ("src/auth".to_string(), true),
            "dot segments collapse"
        );
        assert_eq!(
            clean_folder("/src"),
            ("/src".to_string(), false),
            "an absolute path is refused"
        );
        assert_eq!(
            clean_folder("src/../../x"),
            ("src/../../x".to_string(), false),
            "a path that climbs out is refused"
        );
        assert_eq!(
            clean_folder("src/*"),
            ("src/*".to_string(), false),
            "a glob is refused"
        );
    }

    #[test]
    fn parse_migration_reads_both_arrows() {
        assert_eq!(
            parse_migration(" old design -> new design "),
            Some(("old design".to_string(), "new design".to_string())),
            "an ascii arrow"
        );
        assert_eq!(
            parse_migration("old → new"),
            Some(("old".to_string(), "new".to_string())),
            "the arrow character"
        );
        assert_eq!(
            parse_migration(""),
            Some((String::new(), String::new())),
            "empty clears"
        );
        assert_eq!(parse_migration("old design"), None, "no arrow is invalid");
        assert_eq!(parse_migration("-> new"), None, "an empty side is invalid");
    }

    #[test]
    fn split_list_drops_blanks() {
        assert_eq!(
            split_list("core, , core/import,"),
            vec!["core".to_string(), "core/import".to_string()],
            "blanks are dropped"
        );
        assert!(split_list("").is_empty(), "empty is an empty list");
    }

    #[test]
    fn rule_ids_are_26_base32_characters() {
        assert!(valid_rule_id("01J8ZQ4Y7XK3M9V2N5P6R7T8S9"), "a ULID");
        assert!(!valid_rule_id("local-only"), "a slug is not an id");
        assert!(!valid_rule_id("01j8zq4y7xk3m9v2n5p6r7t8s9"), "lowercase");
    }

    #[test]
    fn quotes_like_go() {
        assert_eq!(go_quote("core/import"), "\"core/import\"");
        assert_eq!(go_quote("a \"b\""), "\"a \\\"b\\\"\"");
        assert_eq!(go_quote("tab\there"), "\"tab\\there\"");
    }
}
