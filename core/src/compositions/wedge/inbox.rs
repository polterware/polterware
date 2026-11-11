#[allow(unused_imports)] // The package-wide glob; every name this file uses is explicit.
use super::*;

use std::collections::BTreeMap;

use time::OffsetDateTime;

use crate::domains::inbox;
use crate::domains::polter;
use crate::domains::work;
use crate::providers::sqlite as storepkg;

use super::brief::helpers;

/// Means the id matches no current Inbox item.
pub const ERR_UNKNOWN_INBOX_ITEM: &str = "unknown inbox item: list them with polterware inbox";

/// A caller mistake: an action the item does not take, or a missing or
/// malformed answer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct InboxInputError {
    pub message: String,
}

/// The id matched no current Inbox item; `errors.Is` against
/// [`ERR_UNKNOWN_INBOX_ITEM`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{ERR_UNKNOWN_INBOX_ITEM} ({0})")]
pub struct UnknownInboxItem(pub String);

fn inbox_input(message: impl Into<String>) -> InboxInputError {
    InboxInputError {
        message: message.into(),
    }
}

/// Derives the project's Inbox. It only reads, so a read never emits a change
/// event.
pub fn list_inbox(store: &storepkg::Store, project_id: &str) -> anyhow::Result<inbox::Result> {
    let snapshot = store.load_work_snapshot(project_id)?;
    list_inbox_for(store, &snapshot)
}

pub(super) fn list_inbox_for(
    store: &storepkg::Store,
    snapshot: &work::Snapshot,
) -> anyhow::Result<inbox::Result> {
    let mut conflicts: Vec<inbox::Conflict> = Vec::new();
    for conflict in &snapshot.conflicts {
        if conflict.state != work::CONFLICT_OPEN {
            continue;
        }
        let (rule, decision_title) = polter::conflict_against(conflict, snapshot);
        let mut details = BTreeMap::from([("conflict".to_string(), conflict.slug.clone())]);
        let mut against: Vec<String> = Vec::new();
        if !rule.id.is_empty() {
            details.insert("rule_id".to_string(), rule.id.clone());
            details.insert("rule".to_string(), rule.statement.clone());
            against.push(format!("Rule: {}", rule.statement));
        }
        if !decision_title.is_empty() {
            details.insert("decision_id".to_string(), conflict.decision.clone());
            details.insert("decision".to_string(), decision_title.clone());
            against.push(format!("Decision: {decision_title}"));
        }
        conflicts.push(inbox::Conflict {
            slug: conflict.slug.clone(),
            request: conflict.request.clone(),
            against: against.join("; "),
            card_slug: conflict.card_slug.clone(),
            options: conflict.options.clone(),
            details,
        });
    }
    let mut questions: Vec<inbox::Question> = Vec::new();
    for card in &snapshot.cards {
        if card.status != work::CARD_ACTIVE {
            continue;
        }
        for question in &card.questions {
            if question.state != work::QUESTION_OPEN {
                continue;
            }
            questions.push(inbox::Question {
                id: question.id.clone(),
                card_slug: card.slug.clone(),
                text: question.text.clone(),
                blocking: question.blocking,
                options: question.options.clone(),
            });
        }
    }
    let links = store.list_node_deploy_links(&snapshot.project_id)?;
    let connections = store.list_connections(&snapshot.project_id)?;
    let probes: Vec<inbox::ProviderConnection> = connections
        .iter()
        .map(|connection| inbox::ProviderConnection {
            provider_id: connection.provider_id.clone(),
            environment_id: connection.environment_id.clone(),
            status: connection.status.clone(),
        })
        .collect();
    let dispositions = store.list_inbox_dispositions(&snapshot.project_id)?;
    let mut deploys = inbox::failed_deploys(&links);
    deploys.extend(inbox::unconnected_deploys(&links, &probes));
    Ok(inbox::derive(
        &conflicts,
        &questions,
        &deploys,
        &dispositions,
    ))
}

/// One action on one Inbox item.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResolveInboxInput {
    pub project_id: String,
    pub item_id: String,
    pub action: String,
    /// The decision for `decide`.
    pub answer: String,
    /// Who acted: the session agent label or "desktop".
    pub actor: String,
    /// The actor kind, tool, and run for the decide write.
    pub apply: ApplyInput,
}

/// What the action wrote.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResolveInboxResult {
    pub item: inbox::Item,
    pub action: String,
    /// Set for decide, which writes the answer as Polter text.
    pub apply: Option<ApplyResult>,
}

/// Applies one action. Decide writes `? text -> answer` through the Polter
/// apply path, and every other action appends a disposition.
pub fn resolve_inbox(
    store: &storepkg::Store,
    input: &ResolveInboxInput,
) -> anyhow::Result<ResolveInboxResult> {
    let current = list_inbox(store, &input.project_id)?;
    let Some(item) = current.find(&input.item_id) else {
        return Err(UnknownInboxItem(input.item_id.trim().to_string()).into());
    };
    if !inbox::valid_action(&item, &input.action) {
        let labels: Vec<String> = item
            .actions
            .iter()
            .map(|action| inbox::action_label(action))
            .collect();
        return Err(inbox_input(format!(
            "cannot {} a {} item: use {}",
            inbox::action_label(&input.action),
            item.kind,
            labels.join(", ")
        ))
        .into());
    }
    let mut result = ResolveInboxResult {
        item: item.clone(),
        action: input.action.clone(),
        ..ResolveInboxResult::default()
    };
    let actor = input.actor.trim().to_string();

    if input.action == inbox::ACTION_DECIDE {
        let text = decide_text(&item, &input.answer)?;
        let mut apply = input.apply.clone();
        apply.project_id = input.project_id.clone();
        apply.text = text;
        let (applied, outcome) = helpers::apply_polter(store, &apply);
        result.apply = Some(applied);
        return match outcome {
            Ok(()) => Ok(result),
            Err(err) => Err(err),
        };
    }
    store.insert_inbox_disposition(
        &input.project_id,
        inbox::Disposition {
            key: item.key.clone(),
            action: input.action.clone(),
            actor,
            note: String::new(),
            created_at: OffsetDateTime::now_utc(),
        },
    )?;
    Ok(result)
}

/// The Polter text that answers a Decision item's question or decides a
/// Conflict. A question repeats `[blocking]` so the write does not change the
/// flag.
pub fn decide_text(item: &inbox::Item, answer: &str) -> Result<String, InboxInputError> {
    let answer = answer.trim();
    if item.kind == inbox::KIND_CONFLICT {
        if answer.is_empty() || answer.contains('\r') || answer.contains('\n') {
            return Err(inbox_input("decide needs a one-line answer"));
        }
        return Ok(polter::conflict_text(
            &item.conflict_slug,
            answer,
            &format!("Decide conflict {}", item.conflict_slug),
            "A person decided the conflict from the Inbox",
        ));
    }
    if item.kind != inbox::KIND_DECISION {
        return Err(inbox_input(
            "only a decision or conflict item takes an answer",
        ));
    }
    if answer.is_empty() {
        return Err(inbox_input("decide needs an answer"));
    }
    if answer.contains('\r') || answer.contains('\n') || answer.contains("->") {
        return Err(inbox_input("answer must be one line without ->"));
    }
    let mut line = format!("? {}", item.question_text);
    if item.blocking {
        line.push_str(" [blocking]");
    }
    Ok(format!(
        "---\npolter: 1\nsummary: Answer an open question on {}: {}\nrationale: A person decided the open question from the Inbox\n---\n\n## {}\n{} -> {}\n",
        item.card_slug, item.question_text, item.card_slug, line, answer
    ))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::super::brief::helpers::{ApplyInput, InitInput};
    use super::*;
    use crate::domains::{agentrun, canon};
    use crate::providers::sqlite::test_support::TempStore;

    const INBOX_ROADMAP: &str = r#"---
polter: 1
summary: Add a card with open questions
rationale: Exercise slice 7
---

## git-import: Git import
stage: plan

? Include merge commits? [blocking] (yes | no | first-parent)
? How many commits back?
"#;

    fn seed(store: &storepkg::Store) -> String {
        let project = helpers::init_project(
            store,
            &InitInput {
                name: "Demo".to_string(),
                ..InitInput::default()
            },
        )
        .expect("init project");
        let (result, outcome) = helpers::apply_polter(
            store,
            &ApplyInput {
                project_id: project.id.clone(),
                text: INBOX_ROADMAP.to_string(),
                auto_start_label: "cli".to_string(),
                ..ApplyInput::default()
            },
        );
        assert!(
            outcome.is_ok(),
            "{:?} {:?}",
            outcome.err(),
            result.diagnostics
        );
        project.id
    }

    fn find_kind(store: &storepkg::Store, project_id: &str, kind: &str) -> Option<inbox::Item> {
        let result = list_inbox(store, project_id).expect("inbox");
        result.items.into_iter().find(|item| item.kind == kind)
    }

    #[test]
    fn inbox_end_to_end() {
        let temp = TempStore::named("work.db");
        let store = &temp.store;
        let project_id = seed(store);
        // A fact raises nothing: only questions, commits, and setup reach the
        // Inbox.
        let declared = canon::new_fact(
            &project_id,
            "Local auth is bearer token",
            canon::TRUTH_DECLARED,
            "mcp",
            "codex",
            "",
        )
        .expect("fact");
        store.upsert_canon_fact(&declared).expect("save fact");

        let listed = list_inbox(store, &project_id).expect("inbox");
        assert_eq!(
            (listed.open(), listed.blocking()),
            (2, 1),
            "items = {:?}",
            listed.items
        );
        let blocking = listed.items[0].clone();
        assert_eq!(blocking.kind, inbox::KIND_DECISION);
        assert_eq!(blocking.title, "Include merge commits?");

        // Status carries the same count and the blocking item.
        let status = inbox::status_lines(&listed);
        assert!(
            status.contains("inbox: 2 open (1 blocking)\n"),
            "status:\n{status}"
        );
        assert!(
            status.contains(&format!(
                "! {} decision Include merge commits?",
                blocking.id
            )),
            "status:\n{status}"
        );

        let resolve = |item_id: &str, action: &str, answer: &str| {
            resolve_inbox(
                store,
                &ResolveInboxInput {
                    project_id: project_id.clone(),
                    item_id: item_id.to_string(),
                    action: action.to_string(),
                    answer: answer.to_string(),
                    actor: "desktop".to_string(),
                    apply: ApplyInput {
                        actor_kind: agentrun::ACTOR_KIND_HUMAN.to_string(),
                        tool: "desktop.inbox.resolve".to_string(),
                        ..ApplyInput::default()
                    },
                },
            )
        };

        // Wrong action for the kind, and a decide without an answer, are caller
        // errors.
        let err =
            resolve(&blocking.id, inbox::ACTION_IGNORE, "").expect_err("ignore on a decision");
        assert!(
            err.downcast_ref::<InboxInputError>().is_some(),
            "ignore on a decision: {err}"
        );
        let err =
            resolve(&blocking.id, inbox::ACTION_DECIDE, "").expect_err("decide without answer");
        assert!(
            err.downcast_ref::<InboxInputError>().is_some(),
            "decide without answer: {err}"
        );
        let err = resolve("i00000000", inbox::ACTION_DEFER, "").expect_err("unknown item");
        assert!(
            err.downcast_ref::<UnknownInboxItem>().is_some(),
            "unknown item: {err}"
        );
        assert_eq!(
            err.to_string(),
            "unknown inbox item: list them with polterware inbox (i00000000)"
        );

        // Decide writes `? text -> answer` through apply; the item closes.
        let decided = resolve(&blocking.id, inbox::ACTION_DECIDE, "first-parent").expect("decide");
        let applied = decided.apply.clone().expect("apply result");
        assert!(applied.applied, "decide: {decided:?}");
        let snapshot = store.load_work_snapshot(&project_id).expect("snapshot");
        let card = snapshot.card("git-import").expect("card").clone();
        let text = polter::encode_card(&card);
        assert!(
            text.contains(
                "? Include merge commits? [blocking] (yes | no | first-parent) -> first-parent\n"
            ),
            "card after decide:\n{text}"
        );

        // Delegate routes the other question to agents through status.
        let other = find_kind(store, &project_id, inbox::KIND_DECISION).expect("second decision");
        resolve(&other.id, inbox::ACTION_DELEGATE, "").expect("delegate");
        let status = inbox::status_lines(&list_inbox(store, &project_id).expect("inbox"));
        assert!(
            status.contains("delegated: How many commits back?"),
            "status after delegate:\n{status}"
        );

        let final_inbox = list_inbox(store, &project_id).expect("inbox");
        assert_eq!(
            (
                final_inbox.open(),
                final_inbox.items.len(),
                final_inbox.routed.len()
            ),
            (0, 0, 1),
            "final inbox = {final_inbox:?}"
        );
        assert_eq!(inbox::list(&final_inbox), "inbox empty\n");
    }

    #[test]
    fn inbox_defer() {
        let temp = TempStore::named("work.db");
        let store = &temp.store;
        let project_id = seed(store);
        let listed = list_inbox(store, &project_id).expect("inbox");
        let blocking = listed.items[0].clone();
        resolve_inbox(
            store,
            &ResolveInboxInput {
                project_id: project_id.clone(),
                item_id: blocking.id.clone(),
                action: inbox::ACTION_DEFER.to_string(),
                actor: "desktop".to_string(),
                ..ResolveInboxInput::default()
            },
        )
        .expect("defer");

        let listed = list_inbox(store, &project_id).expect("inbox");
        assert_eq!(
            (listed.open(), listed.blocking()),
            (1, 0),
            "want 1 and 0 after defer"
        );
        let last = listed.items.last().expect("last item");
        assert_eq!(last.id, blocking.id);
        assert_eq!(last.state, inbox::STATE_DEFERRED);
    }
}
