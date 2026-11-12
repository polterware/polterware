//! Deriving the items that need a person from data the project already has.
//!
//! Derives the items that need a person from data the project already has: open
//! Conflicts (a request against a Rule or Decision), open questions (Decision),
//! deploys whose latest check failed (Deploy), and deploy links whose provider
//! CLI is not connected (Setup). Items are never stored; only a person's
//! disposition is, keyed by the item's stable key.

pub mod text;

pub use text::*;

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use crate::domains::integrations::provider_name;

pub const KIND_CONFLICT: &str = "conflict";
pub const KIND_DECISION: &str = "decision";
pub const KIND_CONFIRM: &str = "confirm";
pub const KIND_SETUP: &str = "setup";
pub const KIND_DEPLOY: &str = "deploy";

pub const STATE_OPEN: &str = "open";
pub const STATE_DEFERRED: &str = "deferred";

pub const ACTION_DECIDE: &str = "decide";
pub const ACTION_DELEGATE: &str = "delegate";
pub const ACTION_DEFER: &str = "defer";
pub const ACTION_IGNORE: &str = "ignore";

/// The actions each kind accepts, primary first.
pub static KIND_ACTIONS: [(&str, &[&str]); 5] = [
    (KIND_CONFLICT, &[ACTION_DECIDE, ACTION_DEFER]),
    (
        KIND_DECISION,
        &[ACTION_DECIDE, ACTION_DELEGATE, ACTION_DEFER],
    ),
    (KIND_CONFIRM, &[ACTION_DEFER, ACTION_IGNORE]),
    (KIND_SETUP, &[ACTION_DEFER, ACTION_IGNORE]),
    (KIND_DEPLOY, &[ACTION_DEFER, ACTION_IGNORE]),
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
        KIND_DEPLOY => 1,
        KIND_DECISION => 2,
        KIND_CONFIRM => 3,
        KIND_SETUP => 4,
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

/// An open request that goes against a Rule or a Decision. `against` names it
/// ("Rule: …" or "Decision: …"); `details` carry the parts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Conflict {
    pub slug: String,
    pub request: String,
    pub against: String,
    pub card_slug: String,
    pub options: Vec<String>,
    pub details: BTreeMap<String, String>,
}

/// One node deployment. A Setup item is derived when its provider CLI is not
/// connected, and a Deploy item when its latest check failed (`failed`, set by
/// `failed_deploys`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeployLink {
    pub id: String,
    pub node_id: String,
    pub node_path: String,
    pub node_title: String,
    pub environment_id: String,
    pub environment: String,
    pub provider: String,
    pub resource: String,
    pub target_id: String,
    pub url: String,
    /// The latest check: `state` is ok, building, failed, stopped, or unknown.
    pub state: String,
    pub summary: String,
    pub state_url: String,
    pub checked_at: String,
    pub failed: bool,
}

/// One stored CLI connection. Status "connected" matches a successful probe. An
/// empty `environment_id` covers every environment.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderConnection {
    pub provider_id: String,
    pub environment_id: String,
    pub status: String,
}

/// The probe status stored when a provider CLI reports an authenticated
/// session.
pub const PROVIDER_STATUS_CONNECTED: &str = "connected";

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
    /// The item's facts, such as a setup item's provider.
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

/// The stable key of a Setup item for one node in one environment. It does not
/// include the provider or resource, so a relink stays the same item.
pub fn deploy_key(node_id: &str, environment_id: &str) -> String {
    format!("setup:deploy:{node_id}:{environment_id}")
}

/// Keeps links whose provider CLI is not connected for that environment. A
/// connection with an empty environment covers every environment.
pub fn unconnected_deploys(
    links: &[DeployLink],
    connections: &[ProviderConnection],
) -> Vec<DeployLink> {
    links
        .iter()
        .filter(|link| !provider_connected(&link.provider, &link.environment_id, connections))
        .cloned()
        .collect()
}

/// Keeps the links whose latest check failed, marked failed.
pub fn failed_deploys(links: &[DeployLink]) -> Vec<DeployLink> {
    links
        .iter()
        .filter(|link| link.state == "failed")
        .map(|link| DeployLink {
            failed: true,
            ..link.clone()
        })
        .collect()
}

/// Names one failure: a later failing deploy (another `state_url`) is a new
/// item, so dismissing one failure does not hide the next.
pub fn failed_deploy_key(link: &DeployLink) -> String {
    format!(
        "deploy:{}:{}",
        link.id,
        short_id(&format!("{}\u{0}{}", link.state_url, link.summary))
    )
}

fn provider_connected(
    provider: &str,
    environment_id: &str,
    connections: &[ProviderConnection],
) -> bool {
    connections.iter().any(|connection| {
        connection.provider_id == provider
            && connection.status == PROVIDER_STATUS_CONNECTED
            && (connection.environment_id.is_empty() || connection.environment_id == environment_id)
    })
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
    deploys: &[DeployLink],
    dispositions: &[Disposition],
) -> Result {
    derive_with(conflicts, questions, deploys, &[], dispositions)
}

/// [`derive`] with the verify-stage cards whose manual checks are still
/// unconfirmed, one item each.
pub fn derive_with(
    conflicts: &[Conflict],
    questions: &[Question],
    deploys: &[DeployLink],
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
    for link in deploys {
        let mut key = deploy_key(&link.node_id, &link.environment_id);
        let env = if link.environment.is_empty() {
            link.environment_id.clone()
        } else {
            link.environment.clone()
        };
        let where_ = if link.node_path.is_empty() {
            link.node_id.clone()
        } else {
            link.node_path.clone()
        };
        if link.failed {
            key = failed_deploy_key(link);
            let title = if link.node_title.is_empty() {
                where_.clone()
            } else {
                link.node_title.clone()
            };
            let platform = provider_name(&link.provider).to_string();
            let mut details = BTreeMap::from([
                ("provider".to_string(), link.provider.clone()),
                ("platform".to_string(), platform.clone()),
                ("resource".to_string(), link.resource.clone()),
                ("node_id".to_string(), link.node_id.clone()),
            ]);
            if !link.state_url.is_empty() {
                details.insert("url".to_string(), link.state_url.clone());
            }
            if !link.url.is_empty() {
                details.insert("dashboard".to_string(), link.url.clone());
            }
            items.push(Item {
                id: short_id(&key),
                key,
                kind: KIND_DEPLOY.to_string(),
                title: format!(
                    "{} deploy of {} failed on {}",
                    upper_first(&env),
                    title,
                    platform
                ),
                detail: link.summary.clone(),
                where_,
                source: env,
                details,
                ..Item::default()
            });
            continue;
        }
        items.push(Item {
            id: short_id(&key),
            key,
            kind: KIND_SETUP.to_string(),
            title: format!("Connect {} for {} in {}", link.provider, where_, env),
            detail: link.provider.clone(),
            where_,
            source: env,
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

fn upper_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
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

    fn vercel_link() -> DeployLink {
        DeployLink {
            node_id: "n1".to_string(),
            node_path: "app/web".to_string(),
            environment_id: "env-prod".to_string(),
            environment: "production".to_string(),
            provider: "vercel".to_string(),
            ..DeployLink::default()
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
        let result = derive(&[], &questions, &[vercel_link()], &[]);

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
                "setup:Connect vercel for app/web in production",
            ]
        );
        assert_eq!(result.open(), 3);
        assert_eq!(result.blocking(), 1);
        assert_eq!(result.items[0].actions, vec!["decide", "delegate", "defer"]);
        assert_eq!(result.items[2].actions, vec!["defer", "ignore"]);
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
        let result = derive_with(&[], &[], &[], &checklists, &[]);
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
        let result = derive_with(&[], &[], &[], &checklists, &dispositions);
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

        let result = derive(&[], &questions, &[vercel_link()], &dispositions);

        assert_eq!(result.items.len(), 2, "items = {:?}", result.items);
        assert_eq!(result.items[0].kind, KIND_SETUP);
        assert_eq!(result.items[1].state, STATE_DEFERRED);
        assert_eq!(result.open(), 1);
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
    fn derive_unconnected_deploy() {
        let links = vec![
            DeployLink {
                resource: "web".to_string(),
                ..vercel_link()
            },
            DeployLink {
                node_id: "n1".to_string(),
                node_path: "app/web".to_string(),
                environment_id: "env-local".to_string(),
                environment: "local".to_string(),
                provider: "docker".to_string(),
                resource: "api".to_string(),
                ..DeployLink::default()
            },
            DeployLink {
                node_id: "n2".to_string(),
                node_path: "app/api".to_string(),
                environment_id: "env-prod".to_string(),
                environment: "production".to_string(),
                provider: "aws".to_string(),
                resource: "fn".to_string(),
                ..DeployLink::default()
            },
        ];
        let connections = vec![
            ProviderConnection {
                provider_id: "docker".to_string(),
                status: PROVIDER_STATUS_CONNECTED.to_string(),
                ..ProviderConnection::default()
            },
            ProviderConnection {
                provider_id: "aws".to_string(),
                environment_id: "env-local".to_string(),
                status: PROVIDER_STATUS_CONNECTED.to_string(),
            },
            ProviderConnection {
                provider_id: "vercel".to_string(),
                status: "ready".to_string(),
                ..ProviderConnection::default()
            },
        ];
        let pending = unconnected_deploys(&links, &connections);
        assert_eq!(
            pending.len(),
            2,
            "want vercel and aws (other environment, and ready is not connected): {pending:?}"
        );
        let result = derive(&[], &[], &pending, &[]);
        assert_eq!(result.items.len(), 2, "items = {:?}", result.items);
        assert_eq!(result.items[0].kind, KIND_SETUP);
        assert_eq!(result.items[1].kind, KIND_SETUP);
        let key = deploy_key("n1", "env-prod");
        assert_eq!(result.items[0].key, key);
        assert_eq!(
            result.items[0].title,
            "Connect vercel for app/web in production"
        );
        assert_eq!(result.items[0].detail, "vercel");
        assert_eq!(result.items[0].where_, "app/web");
        let again = derive(&[], &[], &pending, &[]);
        assert_eq!(again.items[0].key, result.items[0].key);
        assert_eq!(again.items[0].id, result.items[0].id);
        let hidden = derive(
            &[],
            &[],
            &pending,
            &[Disposition {
                key: key.clone(),
                action: ACTION_IGNORE.to_string(),
                created_at: OffsetDateTime::now_utc(),
                ..Disposition::default()
            }],
        );
        assert_eq!(hidden.items.len(), 1, "dismiss should hide it: {hidden:?}");
        assert_ne!(hidden.items[0].key, key);
        // A connection for the same provider with no environment covers every environment.
        let covered = unconnected_deploys(
            &links,
            &[ProviderConnection {
                provider_id: "vercel".to_string(),
                status: PROVIDER_STATUS_CONNECTED.to_string(),
                ..ProviderConnection::default()
            }],
        );
        assert!(
            covered.iter().all(|link| link.provider != "vercel"),
            "global connection should cover vercel: {covered:?}"
        );
    }

    #[test]
    fn failed_deploys_get_their_own_keys() {
        let failed = DeployLink {
            id: "link-1".to_string(),
            node_id: "n1".to_string(),
            node_path: "app/web".to_string(),
            node_title: String::new(),
            environment_id: "env-prod".to_string(),
            environment: "production".to_string(),
            provider: "vercel".to_string(),
            resource: "web".to_string(),
            state: "failed".to_string(),
            summary: "Build failed".to_string(),
            state_url: "https://vercel.com/acme/web/1".to_string(),
            url: "https://vercel.com/acme/web".to_string(),
            ..DeployLink::default()
        };
        let marked = failed_deploys(std::slice::from_ref(&failed));
        assert_eq!(marked.len(), 1);
        assert!(marked[0].failed);
        let result = derive(&[], &[], &marked, &[]);
        assert_eq!(result.items.len(), 1);
        let item = &result.items[0];
        assert_eq!(item.kind, KIND_DEPLOY);
        assert_eq!(item.title, "Production deploy of app/web failed on Vercel");
        assert_eq!(item.detail, "Build failed");
        assert_eq!(item.where_, "app/web");
        assert_eq!(item.source, "production");
        assert_eq!(item.details["provider"], "vercel");
        assert_eq!(item.details["platform"], "Vercel");
        assert_eq!(item.details["resource"], "web");
        assert_eq!(item.details["node_id"], "n1");
        assert_eq!(item.details["url"], "https://vercel.com/acme/web/1");
        assert_eq!(item.details["dashboard"], "https://vercel.com/acme/web");
        let other = DeployLink {
            state_url: "https://vercel.com/acme/web/2".to_string(),
            ..failed.clone()
        };
        assert_ne!(failed_deploy_key(&failed), failed_deploy_key(&other));
        assert_eq!(
            failed_deploy_key(&failed),
            format!(
                "deploy:link-1:{}",
                short_id("https://vercel.com/acme/web/1\u{0}Build failed")
            )
        );
        // A dismissal hides this failure only.
        let dismissed = derive(
            &[],
            &[],
            &marked,
            &[Disposition {
                key: failed_deploy_key(&failed),
                action: ACTION_IGNORE.to_string(),
                created_at: OffsetDateTime::now_utc(),
                ..Disposition::default()
            }],
        );
        assert!(dismissed.items.is_empty());
        let next = derive(&[], &[], &failed_deploys(&[other]), &[]);
        assert_eq!(next.items.len(), 1);
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
        let result = derive(&[conflict], &[], &[], &[]);
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
                kind: KIND_SETUP.to_string(),
                actions: kind_actions(KIND_SETUP)
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
