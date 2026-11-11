//! Port of `backend/providers/demoseed/facts.go`.

use time::{Duration, Month};

use crate::domains::canon;
use crate::providers::sqlite::{self, Store};

use super::seed::{PROJECT_ID, fixed_utc};

/// Records the demo project's facts.
pub(super) fn seed_facts(store: &Store) -> sqlite::Result<()> {
    let observed = fixed_utc(2026, Month::September, 18, 10, 30, 0);
    let facts = [
        canon::Fact {
            id: "fact-pnpm".to_string(),
            statement: "The workspace uses pnpm.".to_string(),
            truth_class: canon::TRUTH_DECLARED.to_string(),
            source: "package.json".to_string(),
            ..canon::Fact::default()
        },
        canon::Fact {
            id: "fact-gpui".to_string(),
            statement: "The Desktop is a GPUI app.".to_string(),
            truth_class: canon::TRUTH_DECLARED.to_string(),
            source: "app/Cargo.toml".to_string(),
            ..canon::Fact::default()
        },
        canon::Fact {
            id: "fact-sqlite".to_string(),
            statement: "Project state lives in a local SQLite database.".to_string(),
            truth_class: canon::TRUTH_DECLARED.to_string(),
            source: "go.mod".to_string(),
            ..canon::Fact::default()
        },
        canon::Fact {
            id: "fact-commit-citations".to_string(),
            statement: "Card commits are read from the commits that cite a task.".to_string(),
            truth_class: canon::TRUTH_VERIFIED.to_string(),
            source: "go test".to_string(),
            actor: "claude-code".to_string(),
            ..canon::Fact::default()
        },
        canon::Fact {
            id: "fact-no-network".to_string(),
            statement: "Imports never call the network.".to_string(),
            truth_class: canon::TRUTH_DECLARED.to_string(),
            source: "Decision 0002".to_string(),
            ..canon::Fact::default()
        },
        canon::Fact {
            id: "fact-redis-sessions".to_string(),
            statement: "Sessions are cached in Redis.".to_string(),
            truth_class: canon::TRUTH_DECLARED.to_string(),
            source: "doc-architecture".to_string(),
            ..canon::Fact::default()
        },
        canon::Fact {
            id: "fact-webhook-queue".to_string(),
            statement: "Webhook retries will probably run through the event queue.".to_string(),
            truth_class: canon::TRUTH_INFERRED.to_string(),
            source: "billing-webhooks".to_string(),
            ..canon::Fact::default()
        },
    ];
    for (index, mut fact) in facts.into_iter().enumerate() {
        let at = observed + Duration::minutes(index as i64);
        fact.project_id = PROJECT_ID.to_string();
        fact.observed_at = at;
        fact.created_at = at;
        fact.updated_at = at;
        if fact.actor.is_empty() {
            fact.actor = "system".to_string();
        }
        store.upsert_canon_fact(&fact)?;
    }
    Ok(())
}
