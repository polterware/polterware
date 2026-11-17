//! The demo project's docs and Rules.
//!
//! The demo docs and Rules.

use serde_json::{Value, json};
use time::{Duration, Month};

use crate::domains::knowledge;
use crate::providers::sqlite::{self, Store};

use super::seed::{PROJECT_ID, fixed_utc};

pub(super) fn seed_knowledge(store: &Store) -> sqlite::Result<()> {
    let docs_updated = fixed_utc(2026, Month::September, 17, 22, 0, 0);
    let runbook_updated = fixed_utc(2026, Month::September, 14, 12, 0, 0);
    let items = [
        knowledge::Item {
            id: "doc-architecture".to_string(),
            project_id: PROJECT_ID.to_string(),
            title: "System architecture".to_string(),
            kind: "doc".to_string(),
            doc_type: "doc".to_string(),
            body: "This document describes the local project topology represented on the Architecture canvas.\n\n## Boundaries\n\nComponents, resources, and operational workflows stay scoped to this project.".to_string(),
            tags: vec!["architecture".to_string(), "local".to_string()],
            summary: "How the local project components fit together.".to_string(),
            owner: "Platform".to_string(),
            status: "published".to_string(),
            extra_json: must_json(json!({
                "author": "Project maintainer",
                "componentIds": ["n-gw", "n-pg"],
                "componentNames": ["API Gateway", "PostgreSQL"],
            })),
            created_at: docs_updated,
            updated_at: docs_updated,
            archived_at: None,
        },
        knowledge::Item {
            id: "doc-database".to_string(),
            project_id: PROJECT_ID.to_string(),
            title: "Database conventions".to_string(),
            kind: "doc".to_string(),
            doc_type: "doc".to_string(),
            body: "Use expand-and-contract migrations and validate backups before maintenance.\n\n## Naming\n\n- Tables: `snake_case` plural\n- Indexes: `idx_<table>_<columns>`".to_string(),
            tags: vec!["database".to_string(), "conventions".to_string()],
            summary: "Naming, migrations, and backup windows.".to_string(),
            owner: "Platform".to_string(),
            status: "draft".to_string(),
            extra_json: must_json(json!({
                "author": "Project maintainer",
                "componentIds": ["n-pg"],
                "componentNames": ["PostgreSQL"],
            })),
            created_at: docs_updated - Duration::hours(72),
            updated_at: docs_updated - Duration::hours(72),
            archived_at: None,
        },
        knowledge::Item {
            id: "rb-service".to_string(),
            project_id: PROJECT_ID.to_string(),
            title: "Service recovery".to_string(),
            kind: "doc".to_string(),
            doc_type: "doc".to_string(),
            body: "## Objective\n\nRestore a local service after a failed health check.\n\n## Prerequisites\n\n- Confirm the affected service\n- Open a visual terminal session\n\n## Steps\n\n1. Inspect the process record\n2. Review the restart command\n3. Record the expected result\n\n## Commands\n\n```bash\nsystemctl status example-api\nsystemctl restart example-api\n```\n\n## Rollback\n\nRestore the previous service version if the health check remains degraded.".to_string(),
            tags: vec![
                "runbook".to_string(),
                "api".to_string(),
                "recovery".to_string(),
            ],
            summary: "Restore a local service after a failed health check.".to_string(),
            owner: "Platform".to_string(),
            status: "published".to_string(),
            extra_json: must_json(json!({
                "commands": ["systemctl status example-api", "systemctl restart example-api"],
                "componentId": "n-gw",
                "componentName": "API Gateway",
                "lastValidated": "2026-09-14T12:00:00Z",
                "prerequisites": ["Confirm the affected service", "Open a visual terminal session"],
                "rollback": "Restore the previous service version if the health check remains degraded.",
                "service": "API",
                "severity": "high",
                "steps": ["Inspect the process record", "Review the restart command", "Record the expected result"],
            })),
            created_at: runbook_updated,
            updated_at: runbook_updated,
            archived_at: None,
        },
        knowledge::Item {
            id: "rb-database".to_string(),
            project_id: PROJECT_ID.to_string(),
            title: "Database failover".to_string(),
            kind: "doc".to_string(),
            doc_type: "doc".to_string(),
            body: "## Objective\n\nReview a failover sequence for the primary database.\n\n## Prerequisites\n\n- Verify the standby is healthy\n- Pause local writes\n\n## Steps\n\n1. Review replication state\n2. Promote the standby\n3. Validate application health\n\n## Commands\n\n```bash\npg_isready\npsql -c 'select now();'\n```\n\n## Rollback\n\nRestore the previous primary reference if validation fails.".to_string(),
            tags: vec![
                "runbook".to_string(),
                "database".to_string(),
                "failover".to_string(),
            ],
            summary: "Review a failover sequence for the primary database.".to_string(),
            owner: "Platform".to_string(),
            status: "published".to_string(),
            extra_json: must_json(json!({
                "commands": ["pg_isready", "psql -c 'select now();'"],
                "componentId": "n-pg",
                "componentName": "PostgreSQL",
                "lastValidated": "2026-09-06T12:00:00Z",
                "prerequisites": ["Verify the standby is healthy", "Pause local writes"],
                "rollback": "Restore the previous primary reference if validation fails.",
                "service": "Database",
                "severity": "critical",
                "steps": ["Review replication state", "Promote the standby", "Validate application health"],
            })),
            created_at: runbook_updated - Duration::days(8),
            updated_at: runbook_updated - Duration::days(8),
            archived_at: None,
        },
    ];
    for item in items {
        store.save_knowledge(item)?;
    }
    seed_rules(store)?;
    Ok(())
}

/// Seeds the demo Rules: the three lifecycle Rules and the three project-wide
/// Rules, with their rationale as the why.
fn seed_rules(store: &Store) -> sqlite::Result<()> {
    struct RuleSpec {
        id: &'static str,
        statement: &'static str,
        why: &'static str,
        status: &'static str,
    }
    let rules = [
        RuleSpec {
            id: "decision-sqlite-local",
            statement: "Persist project truth in a local SQLite database owned by polterware-core.",
            why: "## Status\n\nAccepted\n\n## Context\n\nThe desktop needs durable local project state without a hosted account directory.\n\n## Rule\n\nPersist project truth in a local SQLite database owned by polterware-core.\n\n## Consequences\n\n- Desktop remains useful offline.\n- Cloud sync is optional later and must not replace local ownership.",
            status: "active",
        },
        RuleSpec {
            id: "decision-deterministic-import",
            statement: "Import only a fixed set of project files and Git metadata, without network calls or model inference.",
            why: "## Status\n\nAccepted\n\n## Context\n\nImported data must be reproducible and explainable.\n\n## Rule\n\nImport only a fixed set of project files and Git metadata, without network calls or model inference.\n\n## Consequences\n\n- The same checkout always yields the same output.\n- Anything outside the MVP file set needs an explicit source.",
            status: "active",
        },
        RuleSpec {
            id: "decision-bounded-context",
            statement: "Compile context per session with a token budget, active invariants first.",
            why: "## Status\n\nProposed\n\n## Context\n\nAgents lose accuracy when handed the whole project at once.\n\n## Rule\n\nCompile context per session with a token budget, active invariants first.\n\n## Consequences\n\n- Sessions start faster and cheaper.\n- Budget tuning needs measurement on large projects.",
            status: "proposed",
        },
        RuleSpec {
            id: "decision-principle-local-first",
            statement: "Project state must remain useful on one machine without requiring a hosted account.",
            why: "Project state must stay useful on one machine.",
            status: "active",
        },
        RuleSpec {
            id: "decision-principle-honest-boundaries",
            statement: "Do not describe planned capabilities as shipped.",
            why: "Never describe planned work as shipped.",
            status: "active",
        },
        RuleSpec {
            id: "decision-principle-provenance",
            statement: "Declared and verified statements take precedence over inferred knowledge.",
            why: "Declared statements win over inferred ones.",
            status: "active",
        },
    ];
    let ops: Vec<crate::domains::work::Op> = rules
        .iter()
        .map(|rule| {
            let mut fields = serde_json::Map::new();
            fields.insert(
                "statement".into(),
                Value::String(rule.statement.to_string()),
            );
            fields.insert("why".into(), Value::String(rule.why.to_string()));
            fields.insert("status".into(), Value::String(rule.status.to_string()));
            crate::domains::work::Op {
                entity: crate::domains::work::ENTITY_RULE.to_string(),
                kind: crate::domains::work::OP_CREATE.to_string(),
                entity_id: rule.id.to_string(),
                card_slug: rule.id.to_string(),
                label: rule.statement.to_string(),
                fields,
                ..crate::domains::work::Op::default()
            }
        })
        .collect();
    store.apply_work_ops(PROJECT_ID, "", "seed-rules", &ops)?;
    Ok(())
}

fn must_json(value: Value) -> String {
    go_json_marshal(&value)
}

/// Go's `json.Marshal`: map keys sorted (serde_json sorts them too), and `<`,
/// `>`, `&`, and the two line separators escaped. They only ever appear inside
/// JSON strings, so the rewrite is safe.
fn go_json_marshal(value: &Value) -> String {
    value
        .to_string()
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}
