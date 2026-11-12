//! Port of `backend/domains/polter/decode.go`: the decoder, the document AST
//! entry points, and the metadata grammar.
//!
//! `Decode` parses Polter text into a [`Document`]. Syntax problems come back
//! as diagnostics and decoding always continues, so every error is reported at
//! once.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use crate::domains::{canon, structure, work};

use super::glob::valid_pattern;
use super::model::*;

/// The states a retired `###` decision block may name. The block only parses so
/// the planner can point old text at `#### rule`.
pub const DECISION_STATES: [&str; 4] = ["proposed", "active", "challenged", "superseded"];

/// Reports whether `value` is a retired decision state.
pub fn valid_decision_state(value: &str) -> bool {
    DECISION_STATES.contains(&value)
}

/// Reports whether `value` is a retired decision level.
pub fn valid_decision_level(value: &str) -> bool {
    matches!(value, "decision" | "principle")
}

/// A card task line: `- [ ] T1 Title`.
static TASK_LINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[-*+][\t\n\f\r ]+\[([ xX\-])\][\t\n\f\r ]*(.*)$").expect("the task line pattern")
});

/// A checkbox line whose box is malformed.
static BAD_TASK_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[-*+][\t\n\f\r ]+\[[^\]]*\]").expect("the bad task pattern"));

static TASK_NUMBER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[Tt]([0-9]+)[\t\n\f\r ]+(.*)$").expect("the task number pattern")
});

static TRAILING_SCOPE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.*?)[\t\n\x0c\r ]*`([^`]+)`$").expect("the scope pattern"));

static META_KEY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([a-z_]+):[\t\n\f\r ]*(.*)$").expect("the metadata pattern"));

/// A card spec section: `behavior:`, `edge-cases:`, or `plan:`.
static CARD_SECTION_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(behavior|edge-cases|plan):[\t\n\f\r ]*(.*)$").expect("the card section pattern")
});

/// A Rule's single-line metadata key; `replaced-by:` carries a hyphen.
static RULE_META_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([a-z][a-z-]*):[\t\n\f\r ]*(.*)$").expect("the rule metadata pattern")
});

static TRAILING_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.*?)[\t\n\f\r ]*\[([a-zA-Z]+)\]$").expect("the tag pattern"));

static TRAILING_REPO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.*?)[\t\n\f\r ]*\[repo:[\t\n\f\r ]*([^\]]*)\]$").expect("the repo pattern")
});

static TRAILING_OPTS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.*?)[\t\n\f\r ]*\(([^()]*\|[^()]*)\)$").expect("the options pattern")
});

static META_SPLITTER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[\t\n\f\r ]+[·•][\t\n\f\r ]+|[\t\n\f\r ]+-[\t\n\f\r ]+")
        .expect("the metadata splitter pattern")
});

static COORD_LINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(claim|done|release):[\t\n\f\r ]*(.*)$").expect("the coordination pattern")
});

static COORD_TASK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[Tt]([1-9][0-9]*)$").expect("the coordination task pattern"));

static CONFIRM_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^confirm:[\t\n\f\r ]*(.*)$").expect("the confirm pattern"));

/// Bounds `tech:` so it stays a technology name, not prose.
pub const MAX_TECH_RUNES: usize = 80;

/// Parses Polter text into a [`Document`].
pub fn decode(text: &str) -> (Document, Vec<Diagnostic>) {
    let mut d = Decoder {
        lines: text
            .replace("\r\n", "\n")
            .split('\n')
            .map(str::to_string)
            .collect(),
        ..Decoder::default()
    };
    d.run();
    (d.doc, d.diags)
}

#[derive(Default)]
struct Decoder {
    lines: Vec<String>,
    doc: Document,
    diags: Vec<Diagnostic>,

    card: Option<CardNode>,
    node: Option<StructureNode>,
    decision: Option<DecisionNode>,
    fact: Option<FactNode>,
    fact_refs: HashMap<String, i32>,
    rule: Option<RuleNode>,
    conflict: Option<ConflictNode>,
    rule_refs: HashMap<String, i32>,
    dec_refs: HashMap<String, i32>,
    in_meta: bool,
    in_handoff: bool,
    in_fence: bool,
    intent: Vec<String>,
    /// The card section being read (`behavior`, `edge-cases`, or `plan`) and
    /// its body so far.
    card_section: Option<String>,
    section: Vec<String>,
    slug_lines: HashMap<String, i32>,
    path_lines: HashMap<String, i32>,
    saw_version: bool,
}

impl Decoder {
    fn errorf(&mut self, line: i32, message: String) {
        self.diags.push(Diagnostic::error(line, message));
    }

    fn run(&mut self) {
        let start = self.frontmatter();
        if !self.saw_version {
            self.errorf(
                1,
                "missing version header: start with \"---\", \"polter: 1\", \"---\"".to_string(),
            );
        }
        for i in start..self.lines.len() {
            let raw = self.lines[i].clone();
            self.line(i as i32 + 1, &raw);
        }
        self.close_card();
        self.close_node();
        self.close_decision();
        self.close_fact();
        self.close_rule();
        self.close_conflict();
    }

    /// Consumes the leading `---` block and returns the next line index.
    fn frontmatter(&mut self) -> usize {
        let mut first = 0;
        while first < self.lines.len() && self.lines[first].trim().is_empty() {
            first += 1;
        }
        if first >= self.lines.len() || self.lines[first].trim() != "---" {
            return 0;
        }
        let mut i = first + 1;
        while i < self.lines.len() {
            let raw = self.lines[i].trim().to_string();
            if raw == "---" {
                return i + 1;
            }
            if raw.is_empty() || raw.starts_with('#') {
                i += 1;
                continue;
            }
            let found = META_KEY.captures(&raw);
            let key = found
                .as_ref()
                .map(|m| m.get(1).expect("group 1").as_str())
                .filter(|key| matches!(*key, "polter" | "summary" | "rationale"));
            let Some(key) = key.map(str::to_string) else {
                self.errorf(
                    i as i32 + 1,
                    format!(
                        "unknown frontmatter line {}: use polter, summary, or rationale",
                        go_quote(&raw)
                    ),
                );
                i += 1;
                continue;
            };
            let value = found
                .as_ref()
                .and_then(|m| m.get(2))
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            match key.as_str() {
                "polter" => {
                    let parsed = value.parse::<i32>().ok();
                    let version = parsed.unwrap_or(0);
                    if parsed != Some(VERSION) {
                        self.errorf(
                            i as i32 + 1,
                            format!("unsupported version {}: use polter: 1", go_quote(&value)),
                        );
                    }
                    self.doc.version = version;
                    self.saw_version = true;
                }
                "summary" => self.doc.summary = value,
                "rationale" => self.doc.rationale = value,
                _ => {}
            }
            i += 1;
        }
        self.errorf(
            first as i32 + 1,
            "unclosed frontmatter: end it with \"---\"".to_string(),
        );
        self.lines.len()
    }

    fn line(&mut self, n: i32, raw: &str) {
        let trimmed = raw.trim();

        if self.in_fence {
            if self.card_section.is_some() {
                self.section.push(raw.to_string());
            } else {
                self.intent.push(raw.to_string());
            }
            if trimmed.starts_with("```") {
                self.in_fence = false;
            }
            return;
        }

        if let Some(body) = trimmed.strip_prefix("#### ") {
            self.end_handoff();
            let body = body.trim();
            if let Some(rest) = body.strip_prefix(RULE_HEADING_PREFIX) {
                self.rule_heading(n, rest.trim());
            } else if let Some(rest) = body.strip_prefix(CONFLICT_HEADING_PREFIX) {
                self.conflict_heading(n, rest.trim());
            } else if body.starts_with("decision ") || body == "decision" {
                self.close_rule();
                self.errorf(
                    n,
                    "decisions are Rules now: use #### rule <slug>: Statement".to_string(),
                );
            } else {
                self.fact_heading(n, body);
            }
            return;
        }
        if let Some(body) = trimmed.strip_prefix("### ") {
            self.end_handoff();
            self.decision_heading(n, body.trim());
            return;
        }
        if let Some(body) = trimmed.strip_prefix("## ") {
            self.end_handoff();
            self.heading(n, body.trim());
            return;
        }
        if let Some(body) = trimmed.strip_prefix("# ") {
            self.end_handoff();
            let body = body.trim();
            let (slug, heading_path, title, ok, invalid) = split_node_heading(body);
            if ok {
                self.node_heading(n, &slug, &heading_path, &title);
                return;
            }
            if invalid {
                self.errorf(
                    n,
                    format!(
                        "invalid node path {}: use lowercase-kebab segments like core/import",
                        go_quote(&heading_path)
                    ),
                );
                return;
            }
            // A document title without a colon carries no state.
            self.in_meta = false;
            return;
        }
        if trimmed.starts_with('#') {
            self.end_handoff();
            self.errorf(n, "unsupported heading: cards use \"## slug: title\", Rules \"#### rule slug: Statement\", Conflicts \"#### conflict slug: Request\"".to_string());
            return;
        }

        if self.fact.is_some() {
            self.fact_line(n, trimmed);
            return;
        }
        if self.rule.is_some() {
            self.rule_line(n, raw, trimmed);
            return;
        }
        if self.conflict.is_some() {
            self.conflict_line(n, trimmed);
            return;
        }
        if self.decision.is_some() {
            self.decision_line(n, raw, trimmed);
            return;
        }

        if self.in_handoff && self.card.is_some() && self.handoff_line(n, trimmed) {
            return;
        }

        if self.card.is_none() {
            if self.node.is_some() {
                self.node_line(n, raw, trimmed);
                return;
            }
            if TASK_LINE.is_match(trimmed) || BAD_TASK_LINE.is_match(trimmed) {
                self.errorf(
                    n,
                    "task outside a card: move under a \"## slug: title\"".to_string(),
                );
            } else if trimmed.starts_with("? ") {
                self.errorf(
                    n,
                    "question outside a card: move under a \"## slug: title\"".to_string(),
                );
            } else if trimmed.starts_with("! ") {
                self.errorf(
                    n,
                    "invariant outside a card: move under a \"## slug: title\"".to_string(),
                );
            } else if trimmed.starts_with("handoff:") {
                self.errorf(
                    n,
                    "handoff outside a card: move under a \"## slug: title\"".to_string(),
                );
            } else if COORD_LINE.is_match(trimmed) {
                self.errorf(
                    n,
                    "claim outside a card: move under a \"## slug: title\"".to_string(),
                );
            } else if CONFIRM_LINE.is_match(trimmed) {
                self.errorf(
                    n,
                    "confirm outside a card: move under a \"## slug: title\"".to_string(),
                );
            }
            return;
        }

        if trimmed.is_empty() {
            self.in_meta = false;
            if self.card_section.is_some() {
                self.section.push(String::new());
            } else {
                self.intent.push(String::new());
            }
            return;
        }

        if let Some(m) = CARD_SECTION_KEY.captures(trimmed) {
            self.flush_card_section();
            let key = m.get(1).expect("group 1").as_str().to_string();
            let value = m.get(2).expect("group 2").as_str().trim().to_string();
            self.card_section = Some(key);
            self.in_meta = false;
            if value.starts_with("```") {
                self.in_fence = true;
            }
            if !value.is_empty() {
                self.section.push(value);
            }
            return;
        }

        if self.in_meta {
            if COORD_LINE.is_match(trimmed) || CONFIRM_LINE.is_match(trimmed) {
                self.in_meta = false;
            } else if META_KEY.is_match(trimmed) {
                self.meta(n, trimmed);
                return;
            } else {
                self.in_meta = false;
            }
        }

        if self.card_section.is_some()
            && !trimmed.starts_with("handoff:")
            && !COORD_LINE.is_match(trimmed)
            && !CONFIRM_LINE.is_match(trimmed)
            && !TASK_LINE.is_match(trimmed)
            && !BAD_TASK_LINE.is_match(trimmed)
            && !trimmed.starts_with("? ")
            && !trimmed.starts_with("! ")
        {
            if trimmed.starts_with("```") {
                self.in_fence = true;
            }
            self.section.push(raw.to_string());
            return;
        }
        self.flush_card_section();

        if trimmed.starts_with("handoff:") {
            self.start_handoff(n, trimmed);
        } else if let Some(m) = COORD_LINE.captures(trimmed) {
            let action = m.get(1).expect("group 1").as_str().to_string();
            let value = m.get(2).expect("group 2").as_str().trim().to_string();
            self.coord(n, &action, &value);
        } else if let Some(m) = CONFIRM_LINE.captures(trimmed) {
            let value = m.get(1).expect("group 1").as_str().trim().to_string();
            self.confirm(n, &value);
        } else if TASK_LINE.is_match(trimmed) {
            self.task(n, trimmed);
        } else if BAD_TASK_LINE.is_match(trimmed) {
            self.errorf(
                n,
                "malformed checkbox: use - [ ] open, - [x] done, or - [-] dropped".to_string(),
            );
        } else if let Some(body) = trimmed.strip_prefix("? ") {
            self.question(n, body.trim());
        } else if let Some(body) = trimmed.strip_prefix("! ") {
            self.invariant(n, body.trim());
        } else {
            if trimmed.starts_with("```") {
                self.in_fence = true;
            }
            self.intent.push(raw.to_string());
        }
    }

    fn heading(&mut self, n: i32, body: &str) {
        self.close_card();
        self.close_node();
        self.close_decision();
        self.close_fact();
        self.close_rule();
        self.close_conflict();
        let (slug, title) = match body.split_once(':') {
            Some((slug, title)) => (slug.trim().to_string(), title.trim().to_string()),
            None => (body.to_string(), String::new()),
        };
        if !work::valid_slug(&slug) {
            self.errorf(
                n,
                format!(
                    "invalid slug {}: use lowercase-kebab like git-import",
                    go_quote(&slug)
                ),
            );
        }
        match self.slug_lines.get(&slug).copied() {
            Some(prev) => self.errorf(
                n,
                format!(
                    "duplicate slug {} (also line {prev}): merge the two cards",
                    go_quote(&slug)
                ),
            ),
            None => {
                self.slug_lines.insert(slug.clone(), n);
            }
        }
        self.card = Some(CardNode {
            line: n,
            slug,
            title,
            ..CardNode::default()
        });
        self.in_meta = true;
        self.intent.clear();
    }

    fn meta(&mut self, n: i32, line: &str) {
        for part in META_SPLITTER.split(line) {
            let found = META_KEY.captures(part.trim());
            let Some(m) = found else {
                self.errorf(
                    n,
                    format!("malformed metadata {}: use key: value", go_quote(part)),
                );
                continue;
            };
            let key = m.get(1).expect("group 1").as_str().to_string();
            let mut value = m.get(2).expect("group 2").as_str().trim().to_string();
            if key == COORD_CLAIM || key == COORD_DONE || key == COORD_RELEASE {
                self.coord(n, &key, &value);
                continue;
            }
            if key == "handoff" {
                self.start_handoff(n, "handoff:");
                if !value.is_empty() {
                    self.errorf(
                        n,
                        "handoff block starts with \"handoff:\" on its own line".to_string(),
                    );
                }
                continue;
            }
            match key.as_str() {
                "stage" => {
                    value = value.to_lowercase();
                    if !work::valid_stage(&value) {
                        self.errorf(
                            n,
                            format!(
                                "unknown stage {}: use {}",
                                go_quote(&value),
                                work::STAGES.join("|")
                            ),
                        );
                    }
                }
                "status" => {
                    value = value.to_lowercase();
                    if !work::valid_card_status(&value) {
                        self.errorf(
                            n,
                            format!("unknown status {}: use active|removed", go_quote(&value)),
                        );
                    }
                }
                "kind" => {
                    value = value.to_lowercase();
                    if !work::valid_kind(&value) {
                        self.errorf(
                            n,
                            format!(
                                "unknown kind {}: use {}|{}",
                                go_quote(&value),
                                work::KIND_FEATURE,
                                work::KIND_ARCHITECTURE
                            ),
                        );
                    }
                }
                "milestone" => {
                    value = value.to_lowercase();
                    if !value.is_empty() && !work::valid_slug(&value) {
                        self.errorf(
                            n,
                            format!(
                                "invalid milestone {}: use a milestone slug from polterware query milestones",
                                go_quote(&value)
                            ),
                        );
                    }
                }
                "owner" | "under" | "decisions" => {}
                _ => {
                    self.errorf(
                        n,
                        format!(
                            "unknown key {}: use {}",
                            go_quote(&key),
                            META_KEYS.join(", ")
                        ),
                    );
                    continue;
                }
            }
            if let Some(card) = self.card.as_mut() {
                card.meta.push(MetaEntry::new(key, value, n));
            }
        }
    }

    /// Records a coordination line; `value` lists task numbers (T2, T3).
    fn coord(&mut self, n: i32, action: &str, value: &str) {
        self.in_meta = false;
        let tasks = split_list(value);
        if tasks.is_empty() {
            self.errorf(n, format!("{action} needs a task: write {action}: T2"));
            return;
        }
        for task in tasks {
            let found = COORD_TASK.captures(&task);
            let Some(m) = found else {
                self.errorf(
                    n,
                    format!(
                        "invalid task {} in {action}: use T<n> like T2",
                        go_quote(&task)
                    ),
                );
                continue;
            };
            let number = m
                .get(1)
                .expect("group 1")
                .as_str()
                .parse::<i32>()
                .unwrap_or(0);
            if let Some(card) = self.card.as_mut() {
                card.coordination.push(CoordNode {
                    line: n,
                    action: action.to_string(),
                    number,
                });
            }
        }
    }

    /// Reads one `confirm:` line: `! <criterion>` or `rule <slug or id>`.
    fn confirm(&mut self, n: i32, value: &str) {
        self.in_meta = false;
        let value = value.trim();
        let (kind, target) = if let Some(rest) = value.strip_prefix('!') {
            (CONFIRM_INVARIANT, rest.trim())
        } else if value == "rule" || value.starts_with("rule ") {
            (CONFIRM_RULE, value["rule".len()..].trim())
        } else {
            self.errorf(
                n,
                "confirm needs `! <criterion>` or `rule <slug or id>`".to_string(),
            );
            return;
        };
        if target.is_empty() {
            self.errorf(
                n,
                "confirm without a target: write `confirm: ! <criterion>` or `confirm: rule <slug or id>`"
                    .to_string(),
            );
            return;
        }
        if let Some(card) = self.card.as_mut() {
            card.confirmations.push(ConfirmNode {
                line: n,
                kind: kind.to_string(),
                target: target.to_string(),
            });
        }
    }

    fn start_handoff(&mut self, n: i32, trimmed: &str) {
        self.in_meta = false;
        let mut value = "";
        if let Some(rest) = trimmed.strip_prefix("handoff:") {
            value = rest.trim();
        }
        if self
            .card
            .as_ref()
            .is_some_and(|card| card.handoff.is_some())
        {
            let slug = self
                .card
                .as_ref()
                .map(|card| card.slug.clone())
                .unwrap_or_default();
            self.errorf(
                n,
                format!(
                    "duplicate handoff in {}: one handoff block per card",
                    go_quote(&slug)
                ),
            );
            self.in_handoff = true;
            return;
        }
        if !value.is_empty() {
            self.errorf(
                n,
                "handoff block starts with \"handoff:\" on its own line".to_string(),
            );
        }
        if let Some(card) = self.card.as_mut() {
            card.handoff = Some(HandoffNode {
                line: n,
                ..HandoffNode::default()
            });
        }
        self.in_handoff = true;
    }

    /// Consumes a line inside a handoff block. Reports false when the line ends
    /// the block and the caller should parse it as a normal line.
    fn handoff_line(&mut self, n: i32, trimmed: &str) -> bool {
        if trimmed.is_empty()
            || TASK_LINE.is_match(trimmed)
            || BAD_TASK_LINE.is_match(trimmed)
            || trimmed.starts_with("? ")
            || trimmed.starts_with("! ")
            || trimmed.starts_with("```")
        {
            self.end_handoff();
            return false;
        }
        let Some(m) = META_KEY.captures(trimmed) else {
            self.end_handoff();
            return false;
        };
        let key = m.get(1).expect("group 1").as_str().to_string();
        let value = m.get(2).expect("group 2").as_str().trim().to_string();
        let mut unknown = false;
        if let Some(handoff) = self.card.as_mut().and_then(|card| card.handoff.as_mut()) {
            match key.as_str() {
                "branch" => handoff.branch = value,
                "agent" => handoff.agent = value,
                "goal" => handoff.goal = value,
                "done" => handoff.done = value,
                "in_progress" => handoff.in_progress = value,
                "next" => handoff.next = value,
                "touched" => handoff.touched = split_list(&value),
                _ => unknown = true,
            }
        }
        if unknown {
            self.errorf(
                n,
                format!(
                    "unknown handoff key {}: use branch, agent, goal, done, in_progress, next, touched",
                    go_quote(&key)
                ),
            );
        }
        true
    }

    fn end_handoff(&mut self) {
        if !self.in_handoff {
            return;
        }
        self.in_handoff = false;
        let Some((line, missing)) = self
            .card
            .as_ref()
            .and_then(|card| card.handoff.as_ref())
            .map(|handoff| (handoff.line, handoff.branch.trim().is_empty()))
        else {
            return;
        };
        if missing {
            self.errorf(
                line,
                "handoff needs branch: add \"branch: <name>\"".to_string(),
            );
        }
    }

    fn task(&mut self, n: i32, line: &str) {
        let Some(m) = TASK_LINE.captures(line) else {
            return;
        };
        let mut state = work::TASK_OPEN;
        match m.get(1).expect("group 1").as_str() {
            "x" | "X" => state = work::TASK_DONE,
            "-" => state = work::TASK_DROPPED,
            _ => {}
        }
        let mut body = m.get(2).expect("group 2").as_str().trim().to_string();
        let mut node = TaskNode {
            line: n,
            state: state.to_string(),
            ..TaskNode::default()
        };
        if let Some(nm) = TASK_NUMBER.captures(&body) {
            node.number = nm
                .get(1)
                .expect("group 1")
                .as_str()
                .parse::<i32>()
                .unwrap_or(0);
            body = nm.get(2).expect("group 2").as_str().trim().to_string();
            let dups: Vec<i32> = self
                .card
                .as_ref()
                .map(|card| {
                    card.tasks
                        .iter()
                        .filter(|task| task.number == node.number)
                        .map(|task| task.line)
                        .collect()
                })
                .unwrap_or_default();
            for prev in dups {
                self.errorf(
                    n,
                    format!(
                        "duplicate task T{} (also line {prev}): renumber or merge",
                        node.number
                    ),
                );
            }
        }
        // `[repo: api]` may come before or after the trailing `scope`.
        body = self.task_repo(n, &body, &mut node);
        if let Some(sm) = TRAILING_SCOPE.captures(&body) {
            let rest = sm.get(1).expect("group 1").as_str().trim().to_string();
            let raw = sm.get(2).expect("group 2").as_str().to_string();
            body = rest;
            let (repo, glob) = work::split_scope(&raw);
            node.scope = glob;
            if !valid_pattern(&node.scope) || node.scope.is_empty() {
                self.errorf(
                    n,
                    format!(
                        "invalid scope {}: use a path glob like src/auth/* or api:src/auth/*",
                        go_quote(raw.trim())
                    ),
                );
            }
            if !repo.is_empty() {
                if !node.repo.is_empty() && node.repo != repo {
                    self.errorf(
                        n,
                        format!(
                            "task repo {} and scope repo {} differ: keep one",
                            go_quote(&node.repo),
                            go_quote(&repo)
                        ),
                    );
                }
                node.repo = repo;
            }
        }
        body = self.task_repo(n, &body, &mut node);
        if body.is_empty() {
            self.errorf(
                n,
                "task without a title: write it after the checkbox".to_string(),
            );
        }
        node.title = body;
        if let Some(card) = self.card.as_mut() {
            card.tasks.push(node);
        }
    }

    /// Strips a trailing `[repo: name]` from a task body.
    fn task_repo(&mut self, n: i32, body: &str, node: &mut TaskNode) -> String {
        let Some(m) = TRAILING_REPO.captures(body) else {
            return body.to_string();
        };
        let repo = m.get(2).expect("group 2").as_str().trim().to_string();
        if !work::valid_slug(&repo) {
            self.errorf(
                n,
                format!(
                    "invalid repo {}: use a registered checkout name like api",
                    go_quote(&repo)
                ),
            );
        } else if !node.repo.is_empty() && node.repo != repo {
            self.errorf(
                n,
                format!(
                    "task repo {} and scope repo {} differ: keep one",
                    go_quote(&repo),
                    go_quote(&node.repo)
                ),
            );
        } else {
            node.repo = repo;
        }
        m.get(1).expect("group 1").as_str().trim().to_string()
    }

    fn question(&mut self, n: i32, body: &str) {
        let mut node = QuestionNode {
            line: n,
            ..QuestionNode::default()
        };
        // The answer comes last: `? Text [blocking] (a | b) -> a`.
        let mut body = body.to_string();
        if let Some(i) = body.rfind(" -> ") {
            node.answer = body[i + 4..].trim().to_string();
            body = body[..i].trim().to_string();
            if node.answer.is_empty() {
                self.errorf(n, "question answer is empty: write it after ->".to_string());
            }
        } else if let Some(head) = body.strip_suffix(" ->") {
            self.errorf(n, "question answer is empty: write it after ->".to_string());
            body = head.trim().to_string();
        }
        // Tags may follow the options too: `? Text (a | b) [removed]`.
        let (stripped, tags) = self.tags(n, &body, "blocking or removed", &["blocking", "removed"]);
        body = stripped;
        for tag in tags {
            match tag.as_str() {
                "blocking" => node.blocking = true,
                "removed" => node.removed = true,
                _ => {}
            }
        }
        if let Some(om) = TRAILING_OPTS.captures(&body) {
            let rest = om.get(1).expect("group 1").as_str().trim().to_string();
            let options: Vec<String> = om
                .get(2)
                .expect("group 2")
                .as_str()
                .split('|')
                .map(str::trim)
                .filter(|option| !option.is_empty())
                .map(str::to_string)
                .collect();
            body = rest;
            node.options = options;
        }
        let (stripped, tags) = self.tags(n, &body, "blocking or removed", &["blocking", "removed"]);
        body = stripped;
        for tag in tags {
            match tag.as_str() {
                "blocking" => node.blocking = true,
                "removed" => node.removed = true,
                _ => {}
            }
        }
        if body.is_empty() {
            self.errorf(n, "question without text: write it after ?".to_string());
        }
        if node.removed && !node.answer.is_empty() {
            self.errorf(
                n,
                "question is both answered and removed: keep one".to_string(),
            );
        }
        node.text = body;
        if let Some(card) = self.card.as_mut() {
            card.questions.push(node);
        }
    }

    fn invariant(&mut self, n: i32, body: &str) {
        let mut node = InvariantNode {
            line: n,
            ..InvariantNode::default()
        };
        let (body, tags) = self.tags(n, body, "removed", &["removed"]);
        if tags.iter().any(|tag| tag == "removed") {
            node.removed = true;
        }
        if body.is_empty() {
            self.errorf(n, "invariant without text: write it after !".to_string());
        }
        node.text = body;
        if let Some(card) = self.card.as_mut() {
            card.invariants.push(node);
        }
    }

    /// Strips trailing `[tag]` markers, reporting unknown ones.
    fn tags(
        &mut self,
        n: i32,
        body: &str,
        allowed: &str,
        accept: &[&str],
    ) -> (String, Vec<String>) {
        let mut body = body.to_string();
        let mut tags = Vec::new();
        while let Some(m) = TRAILING_TAG.captures(&body) {
            let raw = m.get(2).expect("group 2").as_str().to_string();
            let tag = raw.to_lowercase();
            if !accept.contains(&tag.as_str()) {
                self.errorf(n, format!("unknown tag [{raw}]: use {allowed}"));
            }
            tags.push(tag);
            body = m.get(1).expect("group 1").as_str().trim().to_string();
        }
        (body, tags)
    }

    /// Stores the section being read on the card, if any.
    fn flush_card_section(&mut self) {
        let Some(key) = self.card_section.take() else {
            return;
        };
        let body = self.section.join("\n").trim().to_string();
        let body = collapse_blank_lines(&body);
        if let Some(card) = self.card.as_mut() {
            match key.as_str() {
                "behavior" => {
                    card.behavior = body;
                    card.behavior_set = true;
                }
                "edge-cases" => {
                    card.edge_cases = body;
                    card.edge_cases_set = true;
                }
                "plan" => {
                    card.plan = body;
                    card.plan_set = true;
                }
                _ => {}
            }
        }
        self.section.clear();
    }

    fn close_card(&mut self) {
        if self.card.is_none() {
            return;
        }
        self.end_handoff();
        self.flush_card_section();
        let mut card = self.card.take().expect("checked");
        if self.in_fence {
            self.errorf(
                card.line,
                format!(
                    "unclosed code fence in card {}: close it with ```",
                    go_quote(&card.slug)
                ),
            );
            self.in_fence = false;
        }
        let intent = self.intent.join("\n").trim().to_string();
        if !intent.is_empty() {
            card.intent = collapse_blank_lines(&intent);
            card.intent_set = true;
        }
        self.doc.cards.push(card);
        self.intent.clear();
        self.in_meta = false;
    }

    fn node_heading(&mut self, n: i32, slug: &str, heading_path: &str, title: &str) {
        self.close_card();
        self.close_node();
        self.close_decision();
        self.close_fact();
        self.close_rule();
        self.close_conflict();
        match self.path_lines.get(heading_path).copied() {
            Some(prev) => self.errorf(
                n,
                format!(
                    "duplicate path {} (also line {prev}): merge the two nodes",
                    go_quote(heading_path)
                ),
            ),
            None => {
                self.path_lines.insert(heading_path.to_string(), n);
            }
        }
        self.node = Some(StructureNode {
            line: n,
            slug: slug.to_string(),
            path: heading_path.to_string(),
            title: title.to_string(),
            ..StructureNode::default()
        });
        self.in_meta = true;
        self.intent.clear();
    }

    fn node_line(&mut self, n: i32, raw: &str, trimmed: &str) {
        if trimmed.is_empty() {
            self.in_meta = false;
            self.intent.push(String::new());
            return;
        }
        if self.in_meta {
            if META_KEY.is_match(trimmed) {
                self.node_meta(n, trimmed);
                return;
            }
            self.in_meta = false;
        }
        if TASK_LINE.is_match(trimmed) || BAD_TASK_LINE.is_match(trimmed) {
            self.errorf(
                n,
                "task outside a card: move under a \"## slug: title\"".to_string(),
            );
        } else if trimmed.starts_with("? ") {
            self.errorf(
                n,
                "question outside a card: move under a \"## slug: title\"".to_string(),
            );
        } else if trimmed.starts_with("handoff:") {
            self.errorf(
                n,
                "handoff outside a card: move under a \"## slug: title\"".to_string(),
            );
        } else if COORD_LINE.is_match(trimmed) {
            self.errorf(
                n,
                "claim outside a card: move under a \"## slug: title\"".to_string(),
            );
        } else if let Some(body) = trimmed.strip_prefix("! ") {
            self.node_invariant(n, body.trim());
        } else {
            if trimmed.starts_with("```") {
                self.in_fence = true;
            }
            self.intent.push(raw.to_string());
        }
    }

    fn node_meta(&mut self, n: i32, line: &str) {
        for part in META_SPLITTER.split(line) {
            let found = META_KEY.captures(part.trim());
            let Some(m) = found else {
                self.errorf(
                    n,
                    format!("malformed metadata {}: use key: value", go_quote(part)),
                );
                continue;
            };
            let key = m.get(1).expect("group 1").as_str().to_string();
            let mut value = m.get(2).expect("group 2").as_str().trim().to_string();
            let mut append = true;
            match key.as_str() {
                "level" => {
                    value = value.to_lowercase();
                    if !structure::valid_level(&value) {
                        self.errorf(
                            n,
                            format!(
                                "unknown level {}: use {}",
                                go_quote(&value),
                                structure::LEVELS.join("|")
                            ),
                        );
                    }
                }
                "kind" => {
                    // A component may be several kinds at once:
                    // `kind: service, database`.
                    let kinds = structure::list_values(&value.to_lowercase());
                    for kind in &kinds {
                        if !structure::valid_component_kind(kind) {
                            self.errorf(
                                n,
                                format!(
                                    "unknown kind {}: use one of {}",
                                    go_quote(kind),
                                    structure::COMPONENT_KINDS.join(", ")
                                ),
                            );
                        }
                    }
                    value = structure::join_values(&kinds);
                }
                "tech" => {
                    // And use several technologies: `tech: Go, SQLite`.
                    let techs = structure::list_values(&value);
                    for tech in &techs {
                        if tech.chars().count() > MAX_TECH_RUNES {
                            self.errorf(
                                n,
                                format!(
                                    "tech is too long: keep each under {MAX_TECH_RUNES} characters, like Go or Postgres"
                                ),
                            );
                        }
                    }
                    value = structure::join_values(&techs);
                }
                "status" => {
                    value = value.to_lowercase();
                    if !work::valid_card_status(&value) {
                        self.errorf(
                            n,
                            format!("unknown status {}: use active|removed", go_quote(&value)),
                        );
                    }
                }
                "repo" => {
                    if !value.is_empty() && !work::valid_slug(&value) {
                        self.errorf(
                            n,
                            format!(
                                "invalid repo {}: use a registered checkout name like api",
                                go_quote(&value)
                            ),
                        );
                    }
                }
                "dir" => {
                    if !value.starts_with('/') && !value.starts_with('~') {
                        self.errorf(
                            n,
                            format!(
                                "invalid dir {}: use the absolute folder of a local clone, like ~/code/api",
                                go_quote(&value)
                            ),
                        );
                    }
                }
                "path" => {
                    let (folder, ok) = clean_folder(&value);
                    if !ok {
                        self.errorf(
                            n,
                            format!(
                                "invalid path {}: use a folder inside the checkout like src/auth",
                                go_quote(&value)
                            ),
                        );
                    }
                    value = folder;
                }
                "migrating" => {
                    match parse_migration(&value) {
                        None => self.errorf(
                            n,
                            format!(
                                "invalid migrating {}: write migrating: old design -> new design, or leave it empty to clear",
                                go_quote(&value)
                            ),
                        ),
                        Some((from, to)) => {
                            if !from.is_empty() {
                                value = format!("{from} -> {to}");
                            }
                        }
                    }
                }
                "deploys" => match super::deploy_spec::parse_deploys(&value) {
                    Err(err) => self.errorf(n, err),
                    Ok(specs) => value = super::deploy_spec::format_deploys(&specs),
                },
                "under" | "decisions" => {}
                "uses" => {
                    self.errorf(
                        n,
                        "uses: is retired: draw how nodes relate on a canvas (Structure › Draw as canvas)"
                            .to_string(),
                    );
                    append = false;
                }
                _ => {
                    self.errorf(
                        n,
                        format!(
                            "unknown key {}: use {}",
                            go_quote(&key),
                            NODE_META_KEYS.join(", ")
                        ),
                    );
                    append = false;
                }
            }
            if append && let Some(node) = self.node.as_mut() {
                node.meta.push(MetaEntry::new(key, value, n));
            }
        }
    }

    fn node_invariant(&mut self, n: i32, body: &str) {
        let mut node = InvariantNode {
            line: n,
            ..InvariantNode::default()
        };
        let (body, tags) = self.tags(n, body, "removed", &["removed"]);
        if tags.iter().any(|tag| tag == "removed") {
            node.removed = true;
        }
        if body.is_empty() {
            self.errorf(n, "invariant without text: write it after !".to_string());
        }
        node.text = body;
        if let Some(structure_node) = self.node.as_mut() {
            structure_node.invariants.push(node);
        }
    }

    fn close_node(&mut self) {
        if self.node.is_none() {
            return;
        }
        let mut node = self.node.take().expect("checked");
        if self.in_fence {
            self.errorf(
                node.line,
                format!(
                    "unclosed code fence in node {}: close it with ```",
                    go_quote(&node.path)
                ),
            );
            self.in_fence = false;
        }
        let intent = self.intent.join("\n").trim().to_string();
        if !intent.is_empty() {
            node.intent = collapse_blank_lines(&intent);
            node.intent_set = true;
        }
        self.doc.nodes.push(node);
        self.intent.clear();
        self.in_meta = false;
    }

    fn decision_heading(&mut self, n: i32, body: &str) {
        self.close_card();
        self.close_node();
        self.close_decision();
        self.close_fact();
        self.close_rule();
        self.close_conflict();
        let (reference, title) = match body.split_once(':') {
            Some((reference, title)) => (reference.trim().to_string(), title.trim().to_string()),
            None => (body.to_string(), String::new()),
        };
        if reference.is_empty() {
            self.errorf(
                n,
                "decision needs a reference: write \"### Decision-0007: title\" or \"### new: title\""
                    .to_string(),
            );
        }
        let key = reference.to_lowercase();
        match self.dec_refs.get(&key).copied() {
            Some(prev) if key != DECISION_NEW_REF => self.errorf(
                n,
                format!(
                    "duplicate decision {} (also line {prev}): merge the two blocks",
                    go_quote(&reference)
                ),
            ),
            _ => {
                self.dec_refs.insert(key, n);
            }
        }
        self.decision = Some(DecisionNode {
            line: n,
            reference,
            title,
            ..DecisionNode::default()
        });
        self.in_meta = true;
        self.intent.clear();
    }

    fn decision_line(&mut self, n: i32, raw: &str, trimmed: &str) {
        if trimmed.is_empty() {
            self.in_meta = false;
            self.intent.push(String::new());
            return;
        }
        if self.in_meta {
            if META_KEY.is_match(trimmed) {
                self.decision_meta(n, trimmed);
                return;
            }
            self.in_meta = false;
        }
        if trimmed.starts_with("```") {
            self.in_fence = true;
        }
        self.intent.push(raw.to_string());
    }

    fn decision_meta(&mut self, n: i32, line: &str) {
        for part in META_SPLITTER.split(line) {
            let found = META_KEY.captures(part.trim());
            let Some(m) = found else {
                self.errorf(
                    n,
                    format!("malformed metadata {}: use key: value", go_quote(part)),
                );
                continue;
            };
            let key = m.get(1).expect("group 1").as_str().to_string();
            let mut value = m.get(2).expect("group 2").as_str().trim().to_string();
            match key.as_str() {
                "status" => {
                    value = value.to_lowercase();
                    if !valid_decision_state(&value) {
                        self.errorf(
                            n,
                            format!(
                                "unknown decision status {}: use {}",
                                go_quote(&value),
                                DECISION_STATES.join("|")
                            ),
                        );
                    }
                }
                "level" => {
                    value = value.to_lowercase();
                    if !valid_decision_level(&value) {
                        self.errorf(
                            n,
                            format!(
                                "unknown decision level {}: use decision|principle",
                                go_quote(&value)
                            ),
                        );
                    }
                }
                "supersedes" | "trigger" | "reason" | "scope" => {}
                _ => {
                    self.errorf(
                        n,
                        format!(
                            "unknown key {}: use {}",
                            go_quote(&key),
                            DECISION_META_KEYS.join(", ")
                        ),
                    );
                    continue;
                }
            }
            if let Some(node) = self.decision.as_mut() {
                node.meta.push(MetaEntry::new(key, value, n));
            }
        }
    }

    fn close_decision(&mut self) {
        if self.decision.is_none() {
            return;
        }
        let mut node = self.decision.take().expect("checked");
        if self.in_fence {
            self.errorf(
                node.line,
                format!(
                    "unclosed code fence in decision {}: close it with ```",
                    go_quote(&node.reference)
                ),
            );
            self.in_fence = false;
        }
        let body = self.intent.join("\n").trim().to_string();
        if !body.is_empty() {
            node.body = collapse_blank_lines(&body);
            node.body_set = true;
        }
        self.doc.decisions.push(node);
        self.intent.clear();
        self.in_meta = false;
    }

    fn fact_heading(&mut self, n: i32, body: &str) {
        self.close_card();
        self.close_node();
        self.close_decision();
        self.close_fact();
        self.close_rule();
        self.close_conflict();
        let (reference, statement) = match body.split_once(':') {
            Some((reference, statement)) => {
                (reference.trim().to_string(), statement.trim().to_string())
            }
            None => (body.trim().to_string(), String::new()),
        };
        if !work::valid_slug(&reference) {
            self.errorf(
                n,
                format!(
                    "invalid fact reference {}: use a lowercase-kebab slug or an existing fact id",
                    go_quote(&reference)
                ),
            );
        }
        if let Some(prev) = self.fact_refs.get(&reference).copied() {
            self.errorf(
                n,
                format!(
                    "duplicate fact {} (also line {prev}): merge the two blocks",
                    go_quote(&reference)
                ),
            );
        }
        self.fact_refs.insert(reference.clone(), n);
        self.fact = Some(FactNode {
            line: n,
            reference,
            statement,
            ..FactNode::default()
        });
        self.in_meta = true;
    }

    fn fact_line(&mut self, n: i32, line: &str) {
        if line.is_empty() {
            return;
        }
        for part in META_SPLITTER.split(line) {
            let found = META_KEY.captures(part.trim());
            let Some(m) = found else {
                self.errorf(
                    n,
                    "Facts use a heading statement and kind: or scope: metadata".to_string(),
                );
                continue;
            };
            let key = m.get(1).expect("group 1").as_str().to_string();
            let mut value = m.get(2).expect("group 2").as_str().trim().to_string();
            match key.as_str() {
                "kind" => {
                    value = value.to_lowercase();
                    if !canon::valid_truth_class(&value) {
                        self.errorf(
                            n,
                            format!(
                                "unknown fact kind {}: use declared|inferred|verified",
                                go_quote(&value)
                            ),
                        );
                    }
                }
                "scope" => {}
                "evidence" => {
                    self.errorf(
                        n,
                        "evidence: is retired: a Fact is a statement with its scope".to_string(),
                    );
                    continue;
                }
                _ => {
                    self.errorf(
                        n,
                        format!("unknown fact key {}: use kind or scope", go_quote(&key)),
                    );
                    continue;
                }
            }
            if let Some(node) = self.fact.as_mut() {
                node.meta.push(MetaEntry::new(key, value, n));
            }
        }
    }

    fn close_fact(&mut self) {
        if self.fact.is_none() {
            return;
        }
        let node = self.fact.take().expect("checked");
        self.doc.facts.push(node);
        self.intent.clear();
        self.in_meta = false;
    }

    fn rule_heading(&mut self, n: i32, body: &str) {
        self.close_card();
        self.close_node();
        self.close_decision();
        self.close_fact();
        self.close_rule();
        self.close_conflict();
        let (reference, statement) = match body.split_once(':') {
            Some((reference, statement)) => {
                (reference.trim().to_string(), statement.trim().to_string())
            }
            None => (body.trim().to_string(), String::new()),
        };
        if !work::valid_slug(&reference) && !valid_rule_id(&reference) {
            self.errorf(
                n,
                format!(
                    "invalid rule reference {}: use a lowercase-kebab slug or an existing rule id",
                    go_quote(&reference)
                ),
            );
        }
        if let Some(prev) = self.rule_refs.get(&reference).copied() {
            self.errorf(
                n,
                format!(
                    "duplicate rule {} (also line {prev}): merge the two blocks",
                    go_quote(&reference)
                ),
            );
        }
        self.rule_refs.insert(reference.clone(), n);
        self.rule = Some(RuleNode {
            line: n,
            reference,
            statement,
            ..RuleNode::default()
        });
        self.in_meta = true;
    }

    fn rule_line(&mut self, n: i32, raw: &str, line: &str) {
        if line.is_empty() {
            if self.rule.as_ref().is_some_and(|rule| rule.why_set) {
                self.intent.push(String::new());
            }
            self.in_meta = false;
            return;
        }
        if self.in_meta {
            if let Some(m) = RULE_META_KEY.captures(line) {
                let key = m.get(1).expect("group 1").as_str().to_string();
                let mut value = m.get(2).expect("group 2").as_str().trim().to_string();
                match key.as_str() {
                    "scope" => {}
                    "why" => {
                        if let Some(node) = self.rule.as_mut() {
                            node.why_set = true;
                        }
                        self.in_meta = false;
                        if value.starts_with("```") {
                            self.in_fence = true;
                        }
                        if !value.is_empty() {
                            self.intent.push(value);
                        }
                        return;
                    }
                    "status" => {
                        value = value.to_lowercase();
                        if !work::valid_rule_status(&value) {
                            self.errorf(
                                n,
                                format!(
                                    "unknown rule status {}: use {}",
                                    go_quote(&value),
                                    work::RULE_STATUSES.join("|")
                                ),
                            );
                            return;
                        }
                    }
                    "replaced-by" => {
                        if !work::valid_slug(&value) && !valid_rule_id(&value) {
                            self.errorf(
                                n,
                                format!(
                                    "invalid replaced-by {}: use the Rule's lowercase-kebab slug or its id",
                                    go_quote(&value)
                                ),
                            );
                            return;
                        }
                    }
                    _ => {
                        self.errorf(
                            n,
                            format!(
                                "unknown rule key {}: use {}",
                                go_quote(&key),
                                RULE_META_KEYS.join(", ")
                            ),
                        );
                        return;
                    }
                }
                if let Some(node) = self.rule.as_mut() {
                    node.meta.push(MetaEntry::new(key, value, n));
                }
                return;
            }
            self.in_meta = false;
        }
        if self.rule.as_ref().is_some_and(|rule| rule.why_set) {
            if line.starts_with("```") {
                self.in_fence = true;
            }
            self.intent.push(raw.to_string());
            return;
        }
        self.errorf(
            n,
            "Rules use a heading statement and scope:, status:, replaced-by:, or why: metadata"
                .to_string(),
        );
    }

    fn close_rule(&mut self) {
        if self.rule.is_none() {
            return;
        }
        let mut node = self.rule.take().expect("checked");
        if self.in_fence {
            self.errorf(
                node.line,
                format!(
                    "unclosed code fence in rule {}: close it with ```",
                    go_quote(&node.reference)
                ),
            );
            self.in_fence = false;
        }
        let why = self.intent.join("\n").trim().to_string();
        if !why.is_empty() {
            node.why = collapse_blank_lines(&why);
        }
        self.doc.rules.push(node);
        self.intent.clear();
        self.in_meta = false;
    }

    fn conflict_heading(&mut self, n: i32, body: &str) {
        self.close_card();
        self.close_node();
        self.close_decision();
        self.close_fact();
        self.close_rule();
        self.close_conflict();
        let (reference, request) = match body.split_once(':') {
            Some((reference, request)) => {
                (reference.trim().to_string(), request.trim().to_string())
            }
            None => (body.trim().to_string(), String::new()),
        };
        if !work::valid_slug(&reference) {
            self.errorf(
                n,
                format!(
                    "invalid conflict slug {}: use lowercase-kebab",
                    go_quote(&reference)
                ),
            );
        }
        self.conflict = Some(ConflictNode {
            line: n,
            reference,
            request,
            ..ConflictNode::default()
        });
        self.in_meta = true;
    }

    fn conflict_line(&mut self, n: i32, line: &str) {
        if line.is_empty() {
            return;
        }
        let Some(m) = META_KEY.captures(line) else {
            self.errorf(
                n,
                "Conflicts use a heading request and rule:, decision:, card:, options:, answer:, or status: lines"
                    .to_string(),
            );
            return;
        };
        let key = m.get(1).expect("group 1").as_str().to_string();
        let mut value = m.get(2).expect("group 2").as_str().trim().to_string();
        if !CONFLICT_META_KEYS.contains(&key.as_str()) {
            self.errorf(
                n,
                format!(
                    "unknown conflict key {}: use {}",
                    go_quote(&key),
                    CONFLICT_META_KEYS.join(", ")
                ),
            );
            return;
        }
        if key == "status" {
            value = value.to_lowercase();
            if value != work::CONFLICT_OPEN && value != work::CONFLICT_WITHDRAWN {
                self.errorf(
                    n,
                    format!(
                        "unknown conflict status {}: use open or withdrawn (a person decides with answer:)",
                        go_quote(&value)
                    ),
                );
                return;
            }
        }
        if let Some(node) = self.conflict.as_mut() {
            node.meta.push(MetaEntry::new(key, value, n));
        }
    }

    fn close_conflict(&mut self) {
        if self.conflict.is_none() {
            return;
        }
        let node = self.conflict.take().expect("checked");
        self.doc.conflicts.push(node);
        self.intent.clear();
        self.in_meta = false;
    }
}

/// Splits `# path: Title`. `ok` is a node heading; `invalid` is a colon whose
/// path is not kebab segments. A line with no colon is a title.
fn split_node_heading(body: &str) -> (String, String, String, bool, bool) {
    let Some(idx) = body.find(':') else {
        return (String::new(), String::new(), String::new(), false, false);
    };
    let heading_path = body[..idx].trim().to_string();
    let title = body[idx + 1..].trim().to_string();
    let parts: Vec<&str> = heading_path.split('/').collect();
    if heading_path.is_empty() || parts.is_empty() {
        return (String::new(), heading_path, title, false, true);
    }
    for part in &parts {
        if !work::valid_slug(part) {
            return (String::new(), heading_path, title, false, true);
        }
    }
    (
        parts[parts.len() - 1].to_string(),
        heading_path,
        title,
        true,
        false,
    )
}

/// Collapses runs of blank lines and drops trailing spaces.
fn collapse_blank_lines(text: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut blank = false;
    for line in text.split('\n') {
        let line = line.trim_end_matches([' ', '\t']).to_string();
        if line.is_empty() {
            if blank {
                continue;
            }
            blank = true;
        } else {
            blank = false;
        }
        out.push(line);
    }
    out.join("\n")
}

#[cfg(test)]
pub(crate) const SAMPLE: &str = r#"---
polter: 1
summary: Plan git import
rationale: Slice 5 needs cards
---

# Roadmap

## git-metadata-import: Git metadata import
stage: plan · owner: erick · decisions: Decision-0007

Import HEAD, branch and recent commits as evidence.

- [ ] T1 Commit evidence upsert `backend/domains/canon/*`
- [X] Stamp revision on observed facts
? Include merge commits? [blocking] (yes | no | first-parent)
! No network calls during scan
"#;

/// Reports whether a diagnostic on `line` carries `fragment`.
#[cfg(test)]
pub(crate) fn has_diag(diags: &[Diagnostic], line: i32, fragment: &str) -> bool {
    diags
        .iter()
        .any(|d| d.line == line && d.message.contains(fragment))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn must_decode(text: &str) -> Document {
        let (doc, diags) = decode(text);
        assert!(!has_errors(&diags), "decode {text:?}: {diags:?}");
        doc
    }

    #[test]
    fn decodes_the_sample() {
        let (doc, diags) = decode(SAMPLE);
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
        assert_eq!(doc.summary, "Plan git import", "summary");
        assert_eq!(doc.cards.len(), 1, "one card");
        let card = &doc.cards[0];
        assert_eq!(card.slug, "git-metadata-import", "slug");
        assert_eq!(card.title, "Git metadata import", "title");
        assert_eq!(card.meta_value("stage"), Some("plan"), "stage");
        assert_eq!(
            card.intent, "Import HEAD, branch and recent commits as evidence.",
            "intent"
        );
        assert_eq!(card.tasks.len(), 2, "two tasks");
        assert_eq!(card.tasks[0].number, 1, "first task number");
        assert_eq!(
            card.tasks[0].scope, "backend/domains/canon/*",
            "first task scope"
        );
        assert_eq!(card.tasks[1].state, work::TASK_DONE, "[X] should be done");
        let q = &card.questions[0];
        assert!(q.blocking, "the question blocks");
        assert_eq!(q.options.len(), 3, "three options");
        assert_eq!(q.text, "Include merge commits?", "question text");
        assert_eq!(
            card.invariants[0].text, "No network calls during scan",
            "invariant"
        );
    }

    #[test]
    fn reports_diagnostics() {
        let text = "---\npolter: 1\n---\n- [ ] orphan\n## Bad Slug: x\n## ok: One\nstage: doing\n- [] broken\n## ok: Two\n";
        let (_, diags) = decode(text);
        for (line, fragment) in [
            (4, "task outside a card"),
            (5, "invalid slug"),
            (7, "unknown stage"),
            (8, "malformed checkbox"),
            (9, "duplicate slug"),
        ] {
            assert!(
                has_diag(&diags, line, fragment),
                "missing line {line} {fragment:?} in {diags:?}"
            );
        }
    }

    #[test]
    fn reports_a_missing_header() {
        let (_, diags) = decode("## a: A\n");
        assert!(has_diag(&diags, 1, "missing version header"), "{diags:?}");
    }

    #[test]
    fn decodes_handoffs() {
        let text = r#"---
polter: 1
summary: Handoff for git import
rationale: Session handoff
---

## git-metadata-import: Git metadata import
stage: build

handoff:
branch: feat/git-import
agent: codex
goal: import HEAD as evidence
done: commit upsert
in_progress: stamp revision
next: wire import
touched: backend/import.go, docs/spec.md

- [ ] T1 Wire import
? Include merge commits? [blocking]
"#;
        let doc = must_decode(text);
        let handoff = doc.cards[0].handoff.as_ref().expect("a handoff block");
        assert_eq!(handoff.branch, "feat/git-import", "branch");
        assert_eq!(handoff.agent, "codex", "agent");
        assert_eq!(handoff.in_progress, "stamp revision", "in progress");
        assert_eq!(handoff.touched.len(), 2, "two touched files");
        assert_eq!(handoff.touched[1], "docs/spec.md", "second touched file");
    }

    #[test]
    fn handoff_diagnostics() {
        let (_, diags) = decode("---\npolter: 1\n---\n## a: A\nhandoff:\ngoal: missing branch\n");
        assert!(has_diag(&diags, 5, "handoff needs branch"), "{diags:?}");
        let (_, diags) =
            decode("---\npolter: 1\n---\n## a: A\nhandoff:\nbranch: feat\nowner: erick\n");
        assert!(has_diag(&diags, 7, "unknown handoff key"), "{diags:?}");
        let (_, diags) = decode("---\npolter: 1\n---\nhandoff:\nbranch: feat\n");
        assert!(has_diag(&diags, 4, "handoff outside a card"), "{diags:?}");
    }

    #[test]
    fn decodes_structure_nodes() {
        let text = r#"---
polter: 1
---
# core: Core
level: domain

The core.

! No network calls during scan

# core/import: Import
level: component
under: core

## git-import: Git import
under: core/import
"#;
        let doc = must_decode(text);
        assert_eq!(doc.nodes.len(), 2, "two nodes");
        assert_eq!(doc.nodes[0].slug, "core", "first slug");
        assert_eq!(doc.nodes[0].path, "core", "first path");
        assert_eq!(doc.nodes[1].meta_value("level"), Some("component"), "level");
        assert_eq!(doc.nodes[0].invariants.len(), 1, "one invariant");
        assert_eq!(doc.cards.len(), 1, "one card");
        assert_eq!(doc.cards[0].slug, "git-import", "card slug");
    }

    #[test]
    fn rejects_unknown_kinds_and_long_tech() {
        let (_, diags) =
            decode("---\npolter: 1\n---\n# app/x: X\nlevel: component\nkind: spaceship\n");
        assert!(
            has_diag(&diags, 6, "unknown kind \"spaceship\""),
            "{diags:?}"
        );
        let text = format!(
            "---\npolter: 1\n---\n# app/x: X\nlevel: component\ntech: {}\n",
            "x".repeat(MAX_TECH_RUNES + 1)
        );
        let (_, diags) = decode(&text);
        assert!(has_diag(&diags, 6, "tech is too long"), "{diags:?}");
    }

    #[test]
    fn takes_several_kinds_and_technologies() {
        let doc = must_decode(
            "---\npolter: 1\n---\n# data/api: API\nlevel: component\nkind: Service, database, service\ntech: Go, SQLite, Go\n",
        );
        assert_eq!(
            doc.nodes[0].meta_value("kind"),
            Some("service, database"),
            "kinds are lowercased, deduplicated, and joined"
        );
        assert_eq!(
            doc.nodes[0].meta_value("tech"),
            Some("Go, SQLite"),
            "technologies keep their case and are deduplicated"
        );
    }

    #[test]
    fn decodes_facts_rules_and_conflicts() {
        let text = r#"---
polter: 1
---

#### local-authority: One local core owns SQLite.
kind: declared · scope: core

#### rule local-only: Never call the network from the core.
scope: core, core/import
status: active

#### conflict cache-in-core: Add a Redis cache to the core
rule: local-only
options: Keep it local | Allow Redis
"#;
        let doc = must_decode(text);
        assert_eq!(doc.facts.len(), 1, "one fact");
        assert_eq!(doc.facts[0].reference, "local-authority", "fact ref");
        assert_eq!(doc.facts[0].meta_value("scope"), Some("core"), "fact scope");
        assert_eq!(doc.rules.len(), 1, "one rule");
        assert_eq!(
            doc.rules[0].statement,
            "Never call the network from the core."
        );
        assert_eq!(doc.conflicts.len(), 1, "one conflict");
        assert_eq!(doc.conflicts[0].request, "Add a Redis cache to the core");
        assert_eq!(doc.conflicts[0].meta_value("rule"), Some("local-only"));
    }

    #[test]
    fn retired_decision_and_fact_blocks_point_at_rules() {
        let (_, diags) = decode("---\npolter: 1\n---\n#### decision 7: Local authority\n");
        assert!(has_diag(&diags, 4, "decisions are Rules now"), "{diags:?}");
        let (doc, diags) = decode("---\npolter: 1\n---\n#### old-fact: The store is local.\n");
        assert!(!has_errors(&diags), "{diags:?}");
        let (_, diags) = crate::domains::polter::plan::plan(&doc, &work::Snapshot::default());
        assert!(
            diags
                .iter()
                .any(|diagnostic| diagnostic.message.contains("facts are Rules now")),
            "{diags:?}"
        );
    }

    #[test]
    fn rejects_retired_fact_evidence() {
        let (_, diags) = decode("---\npolter: 1\n---\n#### f: Statement.\nevidence: a commit\n");
        assert!(has_diag(&diags, 5, "evidence: is retired"), "{diags:?}");
    }

    #[test]
    fn decodes_coordination_lines() {
        let doc = must_decode("---\npolter: 1\n---\n## a: A\nclaim: T2\nrelease: T3\n");
        let card = &doc.cards[0];
        assert_eq!(card.coordination.len(), 2, "two coordination lines");
        assert_eq!(card.coordination[0].action, COORD_CLAIM);
        assert_eq!(card.coordination[0].number, 2);
        assert_eq!(card.coordination[1].action, COORD_RELEASE);
    }

    #[test]
    fn validates_task_scopes() {
        let (_, diags) = decode("---\npolter: 1\n---\n## a: A\n- [ ] T1 Fix `src/[`\n");
        assert!(has_diag(&diags, 5, "invalid scope"), "{diags:?}");
        let (_, diags) = decode("---\npolter: 1\n---\n## a: A\n- [ ] T1 Fix `api:src/auth/*`\n");
        assert!(!has_errors(&diags), "{diags:?}");
    }

    #[test]
    fn decodes_questions_answers_and_tags() {
        let doc = must_decode(
            "---\npolter: 1\n---\n## a: A\n? Keep it? (yes | no) [removed]\n? Ship it? -> yes\n",
        );
        let card = &doc.cards[0];
        assert!(card.questions[0].removed, "the tag after the options");
        assert_eq!(card.questions[0].options.len(), 2, "two options");
        assert_eq!(card.questions[1].answer, "yes", "the answer closes it");
        assert_eq!(card.questions[1].text, "Ship it?", "the answer is not text");
    }

    #[test]
    fn decodes_decisions_with_metadata_and_a_body() {
        let doc = must_decode(
            "---\npolter: 1\n---\n### new: Local SQLite authority\nstatus: active · trigger: multi-user sync\n\nThe body.\n",
        );
        let decision = &doc.decisions[0];
        assert_eq!(decision.reference, DECISION_NEW_REF);
        assert_eq!(decision.title, "Local SQLite authority");
        assert_eq!(decision.meta_value("status"), Some("active"));
        assert_eq!(decision.body, "The body.");
        assert!(decision.body_set);
    }
}
