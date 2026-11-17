//! Deriving the items that need a person from data the project already has.
//!
//! Derives the items that need a person from data the project already has: open
//! Conflicts (a request against a Rule), open questions (kind "decision"),
//! and verification checklists. Items are never stored; only a person's
//! disposition is, keyed by the item's stable key.

pub mod text;

pub use text::*;

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use time::OffsetDateTime;

pub const KIND_CONFLICT: &str = "conflict";
pub const KIND_DECISION: &str = "decision";
pub const KIND_CONFIRM: &str = "confirm";

pub const STATE_OPEN: &str = "open";
pub const STATE_DEFERRED: &str = "deferred";

pub const ACTION_DECIDE: &str = "decide";
pub const ACTION_DELEGATE: &str = "delegate";
pub const ACTION_DEFER: &str = "defer";
pub const ACTION_IGNORE: &str = "ignore";

/// The actions each kind accepts, primary first.
pub static KIND_ACTIONS: [(&str, &[&str]); 3] = [
    (KIND_CONFLICT, &[ACTION_DECIDE, ACTION_DEFER]),
    (
        KIND_DECISION,
        &[ACTION_DECIDE, ACTION_DELEGATE, ACTION_DEFER],
    ),
    (KIND_CONFIRM, &[ACTION_DEFER, ACTION_IGNORE]),
];

/// The actions one kind accepts (Go's `KindActions[kind]`).
pub fn kind_actions(kind: &str) -> &'static [&'static str] {
    KIND_ACTIONS
        .iter()
        .find(|(known, _)| *known == kind)
        .map_or(&[], |(_, actions)| *actions)
}

/// Sorts items of equal blocking by kind.
fn kind_order(kind: &str) -> i32 {
    match kind {
        KIND_CONFLICT => 0,
        KIND_DECISION => 2,
        KIND_CONFIRM => 3,
        _ => 0,
    }
}

/// The one error this package returns.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("unknown action {0:?}: use decide, delegate, defer, or ignore")]
    UnknownAction(String),
}

/// Reports whether the item accepts `action`.
pub fn valid_action(item: &Item, action: &str) -> bool {
    item.actions.iter().any(|known| known == action)
}

/// Reports whether `action` is stored as a disposition. Decide writes the
/// answer on the question instead.
pub fn recorded(action: &str) -> bool {
    action != ACTION_DECIDE
}

/// Reports whether a disposition hands the item to agents through
/// `polterware status` instead of leaving it for a person.
pub fn routed(action: &str) -> bool {
    action == ACTION_DELEGATE
}

/// An open question on an active card.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Question {
    pub id: String,
    pub card_slug: String,
    pub text: String,
    pub blocking: bool,
    pub options: Vec<String>,
}

/// An open request that goes against a Rule. `against` names it
/// ("Rule: …"); `details` carry the parts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Conflict {
    pub slug: String,
    pub request: String,
    pub against: String,
    pub card_slug: String,
    pub options: Vec<String>,
    pub details: BTreeMap<String, String>,
}

/// The latest thing a person did with one item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disposition {
    pub key: String,
    pub action: String,
    pub actor: String,
    pub note: String,
    pub created_at: OffsetDateTime,
}

impl Default for Disposition {
    fn default() -> Self {
        Self {
            key: String::new(),
            action: String::new(),
            actor: String::new(),
            note: String::new(),
            created_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

/// One thing that needs a person.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub key: String,
    pub kind: String,
    pub blocking: bool,
    pub title: String,
    pub detail: String,
    pub options: Vec<String>,
    pub where_: String,
    pub source: String,
    pub state: String,
    pub actions: Vec<String>,
    /// `card_slug` and `question_text` let Decide write a question's answer, and
    /// `conflict_slug` a Conflict's.
    pub card_slug: String,
    pub question_text: String,
    pub conflict_slug: String,
    /// The item's structured details.
    pub details: BTreeMap<String, String>,
}

/// An item a person handed to agents, with the action taken.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoutedItem {
    pub item: Item,
    pub action: String,
}

/// The Inbox of one project: open items first (blocking first), then deferred
/// ones, plus the items routed to agents.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Result {
    pub items: Vec<Item>,
    pub routed: Vec<RoutedItem>,
}

impl Result {
    /// Counts items still waiting for a person (deferred excluded).
    pub fn open(&self) -> usize {
        self.items
            .iter()
            .filter(|item| item.state == STATE_OPEN)
            .count()
    }

    /// Counts open items that block.
    pub fn blocking(&self) -> usize {
        self.items
            .iter()
            .filter(|item| item.state == STATE_OPEN && item.blocking)
            .count()
    }

    /// Returns the item or routed item whose short id or key is `ref_`.
    pub fn find(&self, ref_: &str) -> Option<Item> {
        let ref_ = ref_.trim();
        self.items
            .iter()
            .chain(self.routed.iter().map(|routed| &routed.item))
            .find(|item| item.id == ref_ || item.key == ref_)
            .cloned()
    }
}

/// The id people type: "i" and eight hex digits of the key hash.
pub fn short_id(key: &str) -> String {
    let sum = Sha256::digest(key.as_bytes());
    format!("i{}", hex::encode(&sum[..4]))
}

/// A card in the verify stage whose manual checks are still unconfirmed: each
/// acceptance criterion and each Rule the card must hold, listed as one item.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Checklist {
    pub card_slug: String,
    pub card_title: String,
    pub items: Vec<String>,
}

/// Builds the Inbox from its sources and the latest disposition per key.
pub fn derive(
    conflicts: &[Conflict],
    questions: &[Question],
    dispositions: &[Disposition],
) -> Result {
    derive_with(conflicts, questions, &[], dispositions)
}

/// [`derive`] with the verify-stage cards whose manual checks are still
/// unconfirmed, one item each.
pub fn derive_with(
    conflicts: &[Conflict],
    questions: &[Question],
    checklists: &[Checklist],
    dispositions: &[Disposition],
) -> Result {
    let mut latest: BTreeMap<&str, &Disposition> = BTreeMap::new();
    for disposition in dispositions {
        match latest.get(disposition.key.as_str()) {
            Some(current) if current.created_at > disposition.created_at => {}
            _ => {
                latest.insert(disposition.key.as_str(), disposition);
            }
        }
    }

    let mut items: Vec<Item> = Vec::new();
    for conflict in conflicts {
        let key = format!("conflict:{}", conflict.slug);
        items.push(Item {
            id: short_id(&key),
            key,
            kind: KIND_CONFLICT.to_string(),
            blocking: true,
            title: conflict.request.clone(),
            detail: conflict.against.clone(),
            options: conflict.options.clone(),
            where_: conflict.card_slug.clone(),
            card_slug: conflict.card_slug.clone(),
            conflict_slug: conflict.slug.clone(),
            details: conflict.details.clone(),
            ..Item::default()
        });
    }
    for question in questions {
        let key = format!("decision:{}", question.id);
        items.push(Item {
            id: short_id(&key),
            key,
            kind: KIND_DECISION.to_string(),
            blocking: question.blocking,
            title: question.text.clone(),
            options: question.options.clone(),
            where_: question.card_slug.clone(),
            card_slug: question.card_slug.clone(),
            question_text: question.text.clone(),
            ..Item::default()
        });
    }
    for checklist in checklists {
        if checklist.items.is_empty() {
            continue;
        }
        let key = format!("confirm:{}", checklist.card_slug);
        let count = checklist.items.len();
        let title = if checklist.card_title.is_empty() {
            checklist.card_slug.clone()
        } else {
            checklist.card_title.clone()
        };
        items.push(Item {
            id: short_id(&key),
            key,
            kind: KIND_CONFIRM.to_string(),
            title: format!("Confirm {count} check(s) for {title}"),
            detail: checklist.items.join("; "),
            where_: checklist.card_slug.clone(),
            card_slug: checklist.card_slug.clone(),
            ..Item::default()
        });
    }
    let mut result = Result::default();
    for mut item in items {
        item.state = STATE_OPEN.to_string();
        if item.actions.is_empty() {
            item.actions = kind_actions(&item.kind)
                .iter()
                .map(|action| (*action).to_string())
                .collect();
        }
        if let Some(disposition) = latest.get(item.key.as_str()) {
            if disposition.action == ACTION_DEFER {
                item.state = STATE_DEFERRED.to_string();
            } else if routed(&disposition.action) {
                result.routed.push(RoutedItem {
                    item,
                    action: disposition.action.clone(),
                });
                continue;
            } else if disposition.action == ACTION_IGNORE {
                continue;
            }
        }
        result.items.push(item);
    }
    result.items.sort_by(|a, b| {
        let (a_open, b_open) = (a.state == STATE_OPEN, b.state == STATE_OPEN);
        if a_open != b_open {
            return b_open.cmp(&a_open);
        }
        if a.blocking != b.blocking {
            return b.blocking.cmp(&a.blocking);
        }
        kind_order(&a.kind).cmp(&kind_order(&b.kind))
    });
    result
}

/// The CLI spelling of an action ("update_intent" → "update-intent").
pub fn action_label(action: &str) -> String {
    action.replace('_', "-")
}

/// Reads a CLI action spelling.
pub fn parse_action(text: &str) -> std::result::Result<String, Error> {
    let action = text.trim().to_lowercase().replace('-', "_");
    if KIND_ACTIONS
        .iter()
        .any(|(_, actions)| actions.contains(&action.as_str()))
    {
        return Ok(action);
    }
    Err(Error::UnknownAction(text.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use time::macros::datetime;

    fn question(id: &str, text: &str) -> Question {
        Question {
            id: id.to_string(),
            card_slug: "git-import".to_string(),
            text: text.to_string(),
            ..Question::default()
        }
    }

    #[test]
    fn derive_sources_and_order() {
        let questions = vec![
            question("q1", "How many commits back?"),
            Question {
                blocking: true,
                options: vec!["yes".to_string(), "no".to_string()],
                ..question("q2", "Include merge commits?")
            },
        ];
        let result = derive(&[], &questions, &[]);

        let order: Vec<String> = result
            .items
            .iter()
            .map(|item| format!("{}:{}", item.kind, item.title))
            .collect();
        assert_eq!(
            order,
            vec![
                "decision:Include merge commits?",
                "decision:How many commits back?",
            ]
        );
        assert_eq!(result.open(), 2);
        assert_eq!(result.blocking(), 1);
        assert_eq!(result.items[0].actions, vec!["decide", "delegate", "defer"]);
        let id = &result.items[0].id;
        assert_eq!(id.len(), 9, "short id = {id:?}");
        assert_eq!(id.as_bytes()[0], b'i');
    }

    #[test]
    fn derive_lists_one_checklist_item_per_verify_card() {
        let checklists = vec![
            Checklist {
                card_slug: "git-import".to_string(),
                card_title: "Git import".to_string(),
                items: vec![
                    "! No network calls".to_string(),
                    "rule local-only: Never call the network (from core)".to_string(),
                ],
            },
            Checklist {
                card_slug: "confirmed".to_string(),
                card_title: "Confirmed".to_string(),
                ..Checklist::default()
            },
        ];
        let result = derive_with(&[], &[], &checklists, &[]);
        assert_eq!(
            result.items.len(),
            1,
            "only the unconfirmed card: {result:?}"
        );
        let item = &result.items[0];
        assert_eq!(item.kind, KIND_CONFIRM);
        assert_eq!(item.key, "confirm:git-import");
        assert_eq!(item.title, "Confirm 2 check(s) for Git import");
        assert_eq!(
            item.detail,
            "! No network calls; rule local-only: Never call the network (from core)"
        );
        assert_eq!(item.where_, "git-import");
        assert_eq!(item.card_slug, "git-import");
        assert_eq!(item.actions, vec![ACTION_DEFER, ACTION_IGNORE]);
        assert!(!item.blocking, "a confirmation is not a failure");
        assert_eq!(result.open(), 1);

        let dispositions = vec![Disposition {
            key: "confirm:git-import".to_string(),
            action: ACTION_DEFER.to_string(),
            created_at: datetime!(2026-11-12 12:00:00 UTC),
            ..Disposition::default()
        }];
        let result = derive_with(&[], &[], &checklists, &dispositions);
        assert_eq!(result.items[0].state, STATE_DEFERRED);
        assert_eq!(result.open(), 0, "a deferred item waits for later");
    }

    #[test]
    fn derive_dispositions() {
        let questions = vec![
            question("q1", "Defer me?"),
            Question {
                blocking: true,
                ..question("q2", "Delegate me?")
            },
        ];
        let t0 = datetime!(2026-09-28 12:00:00 UTC);
        let dispositions = vec![
            Disposition {
                key: "decision:q1".to_string(),
                action: ACTION_DEFER.to_string(),
                created_at: t0,
                ..Disposition::default()
            },
            Disposition {
                key: "decision:q2".to_string(),
                action: ACTION_DEFER.to_string(),
                created_at: t0,
                ..Disposition::default()
            },
            Disposition {
                key: "decision:q2".to_string(),
                action: ACTION_DELEGATE.to_string(),
                created_at: t0 + time::Duration::minutes(1),
                ..Disposition::default()
            },
            Disposition {
                key: "decision:gone".to_string(),
                action: ACTION_IGNORE.to_string(),
                created_at: t0,
                ..Disposition::default()
            },
        ];

        let result = derive(&[], &questions, &dispositions);

        assert_eq!(result.items.len(), 1, "items = {:?}", result.items);
        assert_eq!(result.items[0].state, STATE_DEFERRED);
        assert_eq!(result.open(), 0);
        assert_eq!(result.blocking(), 0);
        assert_eq!(result.routed.len(), 1);
        assert_eq!(result.routed[0].action, ACTION_DELEGATE);
        assert_eq!(result.routed[0].item.title, "Delegate me?");
        assert!(
            result.find(&result.routed[0].item.id).is_some(),
            "find should also match routed items"
        );
    }

    #[test]
    fn conflicts_carry_what_they_go_against() {
        let conflict = Conflict {
            slug: "drop-columns".to_string(),
            request: "Drop the legacy columns".to_string(),
            against: "Rule: migrations are additive".to_string(),
            card_slug: "schema-cleanup".to_string(),
            options: vec!["additive only".to_string(), "drop it".to_string()],
            details: BTreeMap::from([("rule".to_string(), "migrations-additive".to_string())]),
        };
        let result = derive(&[conflict], &[], &[]);
        assert_eq!(result.items.len(), 1);
        let item = &result.items[0];
        assert_eq!(item.kind, KIND_CONFLICT);
        assert!(item.blocking);
        assert_eq!(item.title, "Drop the legacy columns");
        assert_eq!(item.detail, "Rule: migrations are additive");
        assert_eq!(item.where_, "schema-cleanup");
        assert_eq!(item.conflict_slug, "drop-columns");
        assert_eq!(item.actions, vec!["decide", "defer"]);
        assert_eq!(item.details["rule"], "migrations-additive");
        assert_eq!(item.key, "conflict:drop-columns");
        assert_eq!(item.id, short_id("conflict:drop-columns"));
    }

    #[test]
    fn parse_action_reads_cli_spellings() {
        assert_eq!(parse_action("defer").expect("defer"), ACTION_DEFER);
        assert_eq!(parse_action(" Defer ").expect("defer"), ACTION_DEFER);
        assert!(
            parse_action("reopen").is_err(),
            "reopen went with commit drift"
        );
        assert!(parse_action("fix-code").is_err(), "fix-code is retired");
        assert!(
            parse_action("approve").is_err(),
            "unknown action should fail"
        );
        assert_eq!(
            parse_action("approve").unwrap_err(),
            Error::UnknownAction("approve".to_string())
        );
        assert_eq!(
            parse_action("approve").unwrap_err().to_string(),
            "unknown action \"approve\": use decide, delegate, defer, or ignore"
        );
        assert_eq!(action_label("update_intent"), "update-intent");
        assert!(!valid_action(
            &Item {
                kind: KIND_CONFIRM.to_string(),
                actions: kind_actions(KIND_CONFIRM)
                    .iter()
                    .map(|action| (*action).to_string())
                    .collect(),
                ..Item::default()
            },
            ACTION_DECIDE
        ));
        assert!(recorded(ACTION_DEFER));
        assert!(!recorded(ACTION_DECIDE));
        assert!(routed(ACTION_DELEGATE));
        assert!(!routed(ACTION_DEFER));
    }
}
