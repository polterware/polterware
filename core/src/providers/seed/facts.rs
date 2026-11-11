//! Port of `backend/providers/seed/facts.go`.
//!
//! The facts AGENTS.md declares.

use time::{Duration, OffsetDateTime};

use crate::domains::canon;
use crate::providers::sqlite;

use super::PROJECT_ID;

struct SeedFact {
    id: &'static str,
    statement: &'static str,
    source: &'static str,
}

const FACTS: [SeedFact; 6] = [
    SeedFact {
        id: "fact-single-core",
        statement: "Only the local core process opens SQLite.",
        source: "AGENTS.md",
    },
    SeedFact {
        id: "fact-core-daemon",
        statement: "The core is a detached daemon that keeps running until stopped.",
        source: "AGENTS.md",
    },
    SeedFact {
        id: "fact-desktop-core",
        statement: "The Desktop reads and writes only through the core.",
        source: "AGENTS.md",
    },
    SeedFact {
        id: "fact-no-credentials",
        statement: "Provider credentials are never persisted in SQLite.",
        source: "AGENTS.md",
    },
    SeedFact {
        id: "fact-category",
        statement: "The category is Project Context and Change Intelligence for AI Coding.",
        source: "AGENTS.md",
    },
    SeedFact {
        id: "fact-store-truth",
        statement: "The Polterware store is the source of truth; Polter text is input, not storage.",
        source: "AGENTS.md",
    },
];

pub(super) fn seed_facts(store: &sqlite::Store) -> sqlite::Result<()> {
    let at = OffsetDateTime::now_utc();
    for (index, item) in FACTS.iter().enumerate() {
        let when = at + Duration::seconds(index as i64);
        let fact = canon::Fact {
            id: item.id.to_string(),
            project_id: PROJECT_ID.to_string(),
            statement: item.statement.to_string(),
            truth_class: canon::TRUTH_DECLARED.to_string(),
            source: item.source.to_string(),
            actor: "system".to_string(),
            observed_at: when,
            created_at: when,
            updated_at: when,
            ..canon::Fact::default()
        };
        store.upsert_canon_fact(&fact)?;
    }
    Ok(())
}
