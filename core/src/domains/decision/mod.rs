//! Port of `backend/domains/decision`.
//!
//! The Decision lifecycle: states, legal transitions, and the supersede chain.
//! It is pure (no store); the codec and the store share it.

use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

pub const STATE_PROPOSED: &str = "proposed";
pub const STATE_ACTIVE: &str = "active";
pub const STATE_CHALLENGED: &str = "challenged";
pub const STATE_SUPERSEDED: &str = "superseded";

/// The lifecycle states in display order.
pub const STATES: [&str; 4] = [
    STATE_ACTIVE,
    STATE_CHALLENGED,
    STATE_PROPOSED,
    STATE_SUPERSEDED,
];

/// The errors the lifecycle returns, one per Go `fmt.Errorf` site.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("a decision cannot supersede itself")]
    SupersedesItself,
    #[error("unknown decision {0:?}")]
    UnknownDecision(String),
    #[error("{0} is already superseded: point at its successor")]
    AlreadySuperseded(String),
    #[error("supersede cycle between {0} and {1}")]
    SupersedeCycle(String, String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Maps a stored Decision status to a lifecycle state. Older rows use
/// "accepted" for what is now active; anything unknown reads as proposed.
pub fn normalize(status: &str) -> &'static str {
    match status.trim().to_lowercase().as_str() {
        "accepted" | STATE_ACTIVE => STATE_ACTIVE,
        STATE_CHALLENGED => STATE_CHALLENGED,
        STATE_SUPERSEDED => STATE_SUPERSEDED,
        STATE_PROPOSED => STATE_PROPOSED,
        _ => STATE_PROPOSED,
    }
}

/// Reports whether `s` is a lifecycle state.
pub fn valid_state(s: &str) -> bool {
    matches!(
        s,
        STATE_PROPOSED | STATE_ACTIVE | STATE_CHALLENGED | STATE_SUPERSEDED
    )
}

/// Reports whether a decision in this status constrains work: active and
/// challenged decisions still govern until a review resolves them.
pub fn binding(status: &str) -> bool {
    matches!(normalize(status), STATE_ACTIVE | STATE_CHALLENGED)
}

/// Reports whether `from` → `to` is a legal lifecycle move.
/// A move to the same state is a no-op and legal.
pub fn can_transition(from: &str, to: &str) -> bool {
    let (from, to) = (normalize(from), normalize(to));
    if from == to {
        return true;
    }
    match from {
        STATE_PROPOSED => to == STATE_ACTIVE,
        STATE_ACTIVE => to == STATE_CHALLENGED || to == STATE_SUPERSEDED,
        STATE_CHALLENGED => to == STATE_ACTIVE || to == STATE_SUPERSEDED,
        _ => false,
    }
}

/// Levels are read on input only. A decision with no contexts is a principle:
/// it applies to the whole project and is pinned above the others. `level:
/// principle` clears a decision's contexts; `level: decision` changes nothing.
pub const LEVEL_DECISION: &str = "decision";
pub const LEVEL_PRINCIPLE: &str = "principle";

/// Reports whether `level` is a decision level.
pub fn valid_level(level: &str) -> bool {
    let level = level.trim().to_lowercase();
    level == LEVEL_DECISION || level == LEVEL_PRINCIPLE
}

/// The lifecycle data kept in a knowledge item's `extra_json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meta {
    #[serde(
        default,
        rename = "supersededBy",
        skip_serializing_if = "String::is_empty"
    )]
    pub superseded_by: String,
    #[serde(
        default,
        rename = "reviewTrigger",
        skip_serializing_if = "String::is_empty"
    )]
    pub review_trigger: String,
    #[serde(
        default,
        rename = "statusReason",
        skip_serializing_if = "String::is_empty"
    )]
    pub status_reason: String,
}

/// Reads lifecycle data from `extra_json`; other keys are ignored.
pub fn parse_meta(extra_json: &str) -> Meta {
    serde_json::from_str(extra_json).unwrap_or_default()
}

/// Writes `meta` into `extra_json`, keeping unrelated keys. Empty meta fields
/// remove their key.
pub fn merge_meta(extra_json: &str, meta: &Meta) -> String {
    // A JSON null leaves Go's map nil, which marshals back as null; anything
    // that is not an object starts empty.
    let mut fields: Option<Map<String, Value>> = match serde_json::from_str(extra_json) {
        Ok(Value::Object(fields)) => Some(fields),
        Ok(Value::Null) => None,
        _ => Some(Map::new()),
    };
    set_meta_field(&mut fields, "supersededBy", &meta.superseded_by);
    set_meta_field(&mut fields, "reviewTrigger", &meta.review_trigger);
    set_meta_field(&mut fields, "statusReason", &meta.status_reason);
    // The level merged into contexts; a stale key is dropped on the next write.
    if let Some(fields) = &mut fields {
        fields.remove("level");
    }
    match fields {
        Some(fields) => go_json_marshal(&normalize_numbers(&Value::Object(fields))),
        None => "null".to_string(),
    }
}

fn set_meta_field(fields: &mut Option<Map<String, Value>>, key: &str, value: &str) {
    if value.is_empty() {
        if let Some(fields) = fields {
            fields.remove(key);
        }
        return;
    }
    fields
        .get_or_insert_with(Map::new)
        .insert(key.to_string(), Value::String(value.to_string()));
}

/// Go reads JSON numbers as float64 and writes an integral one without a
/// fraction (`1.0` becomes `1`).
fn normalize_numbers(value: &Value) -> Value {
    match value {
        Value::Number(number) => match number.as_f64() {
            Some(float) if float.fract() == 0.0 && float.abs() < 9.0e15 => match number.as_i64() {
                Some(_) => value.clone(),
                None => Value::from(float as i64),
            },
            _ => value.clone(),
        },
        Value::Array(items) => Value::Array(items.iter().map(normalize_numbers).collect()),
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(key, item)| (key.clone(), normalize_numbers(item)))
                .collect(),
        ),
        _ => value.clone(),
    }
}

/// One Decision as the lifecycle sees it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Decision {
    pub id: String,
    pub title: String,
    pub status: String,
    pub body: String,
    /// The paths of the Structure nodes the decision applies to, sorted.
    /// None means the whole project: the decision is a principle.
    pub scopes: Vec<String>,
    pub meta: Meta,
}

impl Decision {
    /// Reports whether the decision applies to the whole project.
    pub fn is_principle(&self) -> bool {
        self.scopes.is_empty()
    }
}

/// Reads a decision number from a reference ("Decision-0008") or from the start
/// of a numbered title ("Decision 0008: Title"). It also reads the legacy
/// prefixes older stores and agents used (see the pattern). The number must
/// open the value and end it or meet a separator, so free text never matches.
static NUMBERED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^[\t\n\f\r ]*(?:adr|verdict|decision)[\t\n\f\r \-_]*0*([0-9]+)[\t\n\f\r ]*(?:[:.\-–—]|$)")
        .expect("the numbered reference pattern")
});

/// The "Decision 0008: " prefix of a numbered title.
static NUMBER_PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^[\t\n\f\r ]*(?:adr|verdict|decision)[\t\n\f\r \-_]*[0-9]+[\t\n\f\r ]*[:.\-–—]?[\t\n\f\r ]*")
        .expect("the numbered title prefix pattern")
});

/// Reads the decision number from a reference or a numbered title.
pub fn number(value: &str) -> Option<i64> {
    let found = NUMBERED.captures(value)?;
    found.get(1)?.as_str().parse().ok()
}

/// Renders the reference agents write for a decision number: Decision-0008.
/// Go calls this `Ref`, which is a keyword in Rust.
pub fn decision_ref(number: i64) -> String {
    format!("Decision-{number:04}")
}

/// Drops the "Decision 0008: " prefix from a numbered title.
pub fn strip_number(title: &str) -> String {
    NUMBER_PREFIX.replace(title, "").trim().to_string()
}

/// Returns the decision that `reference` names by id, exact title, or Decision
/// number.
pub fn find(decisions: &[Decision], reference: &str) -> Option<Decision> {
    let reference = reference.trim();
    if reference.is_empty() {
        return None;
    }
    for decision in decisions {
        if equal_fold(&decision.id, reference) || equal_fold(&decision.title, reference) {
            return Some(decision.clone());
        }
    }
    if let Some(number) = number(reference) {
        for decision in decisions {
            if number_of_title(decision) == Some(number) {
                return Some(decision.clone());
            }
        }
    }
    None
}

/// Returns the next free Decision number.
pub fn next_number(decisions: &[Decision]) -> i64 {
    let mut next = 1;
    for decision in decisions {
        if let Some(number) = number_of_title(decision)
            && number >= next
        {
            next = number + 1;
        }
    }
    next
}

/// Renders a numbered Decision title: "Decision 0010: Title".
pub fn title_for(number: i64, title: &str) -> String {
    format!("Decision {number:04}: {}", title.trim())
}

/// Validates that `by` may supersede `target`: distinct, both known to exist,
/// `by` is not itself superseded, and the chain has no cycle.
pub fn check_supersede(decisions: &[Decision], target: &str, by: &str) -> Result<()> {
    if target == by {
        return Err(Error::SupersedesItself);
    }
    let by_id: std::collections::HashMap<&str, &Decision> = decisions
        .iter()
        .map(|decision| (decision.id.as_str(), decision))
        .collect();
    if !by_id.contains_key(target) {
        return Err(Error::UnknownDecision(target.to_string()));
    }
    let Some(newer) = by_id.get(by).copied() else {
        return Err(Error::UnknownDecision(by.to_string()));
    };
    if normalize(&newer.status) == STATE_SUPERSEDED {
        return Err(Error::AlreadySuperseded(newer.title.clone()));
    }
    // Walking from by along superseded_by must never reach target.
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut cur = newer;
    while !cur.meta.superseded_by.is_empty() {
        if cur.meta.superseded_by == target || seen.contains(cur.id.as_str()) {
            return Err(Error::SupersedeCycle(
                newer.title.clone(),
                by_id[target].title.clone(),
            ));
        }
        seen.insert(cur.id.as_str());
        let Some(next) = by_id.get(cur.meta.superseded_by.as_str()).copied() else {
            break;
        };
        cur = next;
    }
    Ok(())
}

/// Returns the ids from `id` forward through `superseded_by` (id first).
pub fn chain(decisions: &[Decision], id: &str) -> Vec<String> {
    let by_id: std::collections::HashMap<&str, &Decision> = decisions
        .iter()
        .map(|decision| (decision.id.as_str(), decision))
        .collect();
    let mut chain = Vec::new();
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut cur = by_id.get(id).copied();
    while let Some(decision) = cur {
        if seen.contains(decision.id.as_str()) {
            break;
        }
        seen.insert(decision.id.as_str());
        chain.push(decision.id.clone());
        if decision.meta.superseded_by.is_empty() {
            break;
        }
        cur = by_id.get(decision.meta.superseded_by.as_str()).copied();
    }
    chain
}

fn number_of_title(decision: &Decision) -> Option<i64> {
    number(&decision.title)
}

/// Go's `strings.EqualFold`: a case-insensitive comparison of whole strings.
fn equal_fold(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// Go's `json.Marshal` escapes `<`, `>`, `&`, and the two line separators, and
/// sorts map keys; serde_json sorts them too but leaves those five characters
/// alone. They only ever appear inside JSON strings, so the rewrite is safe.
fn go_json_marshal(value: &Value) -> String {
    value
        .to_string()
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_statuses() {
        let cases = [
            ("accepted", STATE_ACTIVE),
            ("Active", STATE_ACTIVE),
            ("", STATE_PROPOSED),
            ("weird", STATE_PROPOSED),
            ("superseded", STATE_SUPERSEDED),
        ];
        for (input, want) in cases {
            assert_eq!(normalize(input), want, "normalize({input:?})");
        }
    }

    #[test]
    fn can_transition_lifecycle() {
        let ok = [
            ("proposed", "active"),
            ("active", "challenged"),
            ("challenged", "active"),
            ("challenged", "superseded"),
            ("active", "superseded"),
            ("accepted", "active"),
        ];
        for (from, to) in ok {
            assert!(can_transition(from, to), "{from} -> {to} should be legal");
        }
        let bad = [
            ("superseded", "active"),
            ("proposed", "challenged"),
            ("proposed", "superseded"),
            ("active", "proposed"),
        ];
        for (from, to) in bad {
            assert!(
                !can_transition(from, to),
                "{from} -> {to} should be illegal"
            );
        }
    }

    #[test]
    fn meta_round_trip() {
        let extra = merge_meta(
            r#"{"decisionStatus":"active"}"#,
            &Meta {
                superseded_by: "b".to_string(),
                review_trigger: "multi-user".to_string(),
                ..Meta::default()
            },
        );
        let meta = parse_meta(&extra);
        assert!(
            meta.superseded_by == "b" && meta.review_trigger == "multi-user",
            "meta = {meta:?}"
        );
        let extra = merge_meta(&extra, &Meta::default());
        assert_eq!(parse_meta(&extra), Meta::default(), "meta not cleared");
    }

    #[test]
    fn supersede_checks() {
        let ds = [
            Decision {
                id: "a".to_string(),
                title: "Decision 0001: A".to_string(),
                status: "superseded".to_string(),
                meta: Meta {
                    superseded_by: "b".to_string(),
                    ..Meta::default()
                },
                ..Decision::default()
            },
            Decision {
                id: "b".to_string(),
                title: "Decision 0002: B".to_string(),
                status: "active".to_string(),
                ..Decision::default()
            },
            Decision {
                id: "c".to_string(),
                title: "Decision 0003: C".to_string(),
                status: "active".to_string(),
                ..Decision::default()
            },
        ];
        assert!(
            check_supersede(&ds, "b", "b").is_err(),
            "self supersede must fail"
        );
        assert!(
            check_supersede(&ds, "c", "a").is_err(),
            "superseding by a superseded decision must fail"
        );
        assert!(
            check_supersede(&ds, "b", "c").is_ok(),
            "valid supersede failed: {:?}",
            check_supersede(&ds, "b", "c")
        );
        let got = chain(&ds, "a");
        assert!(got.len() == 2 && got[1] == "b", "chain = {got:?}");
        let found = find(&ds, "Decision-2");
        assert!(
            found.as_ref().is_some_and(|decision| decision.id == "b"),
            "find by number = {found:?}"
        );
        assert_eq!(next_number(&ds), 4);
    }

    #[test]
    fn number_reads_refs_and_titles_only() {
        let cases = [
            ("Decision-0008", 8),
            ("decision-8", 8),
            ("Decision 0008", 8),
            ("Decision 0008: Title", 8),
            ("ADR-0008", 8),
            ("Verdict-0008", 8),        // legacy prefix
            ("Verdict 0008: Title", 8), // legacy prefix
        ];
        for (value, want) in cases {
            assert_eq!(number(value), Some(want), "number({value:?})");
        }
        for value in [
            "decision 3 of the team",
            "see Decision-0008",
            "Decisions 4",
            "0008",
        ] {
            assert_eq!(number(value), None, "number({value:?}) must read no number");
        }
        assert_eq!(decision_ref(8), "Decision-0008");
        assert_eq!(strip_number("Decision 0008: Local store"), "Local store");
    }

    #[test]
    fn merge_meta_matches_the_go_marshal() {
        let extra = merge_meta(
            r#"{"a":"<b>&c","n":1.0,"z":[1,"x"],"level":"principle"}"#,
            &Meta {
                superseded_by: "b".to_string(),
                status_reason: "why".to_string(),
                ..Meta::default()
            },
        );
        assert_eq!(
            extra,
            r#"{"a":"\u003cb\u003e\u0026c","n":1,"statusReason":"why","supersededBy":"b","z":[1,"x"]}"#
        );
        assert_eq!(merge_meta("null", &Meta::default()), "null");
        assert_eq!(merge_meta("", &Meta::default()), "{}");
        assert_eq!(
            merge_meta(r#"{"supersededBy":"x"}"#, &Meta::default()),
            "{}"
        );
    }

    #[test]
    fn merge_meta_drops_the_retired_level() {
        assert_eq!(
            merge_meta(r#"{"other":1,"level":"principle"}"#, &Meta::default()),
            r#"{"other":1}"#,
            "the level key stays"
        );
        assert!(Decision::default().is_principle());
        assert!(
            !Decision {
                scopes: vec!["core".to_string()],
                ..Decision::default()
            }
            .is_principle(),
            "a decision is a principle exactly when it has no contexts"
        );
    }
}
