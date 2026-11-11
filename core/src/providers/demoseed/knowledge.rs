//! Port of `backend/providers/demoseed/knowledge.go`.
//!
//! The demo docs, Decisions, and principles, and their links to the
//! environment entities the project model syncs.

use serde_json::{Value, json};
use time::{Duration, Month, OffsetDateTime};

use crate::domains::canon;
use crate::domains::knowledge;
use crate::providers::sqlite::{self, Store};

use super::seed::{ENV_PRODUCTION_ID, PROJECT_ID, fixed_utc};

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
                "environment": "All",
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
                "environment": "All",
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
            body: "## Objective\n\nRestore a local service after a failed health check.\n\n## Prerequisites\n\n- Confirm the affected environment\n- Open a visual terminal session\n\n## Steps\n\n1. Inspect the process record\n2. Review the restart command\n3. Record the expected result\n\n## Commands\n\n```bash\nsystemctl status example-api\nsystemctl restart example-api\n```\n\n## Rollback\n\nRestore the previous service version if the health check remains degraded.".to_string(),
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
                "prerequisites": ["Confirm the affected environment", "Open a visual terminal session"],
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
        knowledge::Item {
            id: "decision-sqlite-local".to_string(),
            project_id: PROJECT_ID.to_string(),
            title: "Decision 0001: Local SQLite authority".to_string(),
            kind: "decision".to_string(),
            doc_type: String::new(),
            body: "## Status\n\nAccepted\n\n## Context\n\nThe desktop needs durable local project state without a hosted account directory.\n\n## Decision\n\nPersist project truth in a local SQLite database owned by polterware-core.\n\n## Consequences\n\n- Desktop remains useful offline.\n- Cloud sync is optional later and must not replace local ownership.".to_string(),
            tags: vec!["storage".to_string()],
            summary: "Keep local SQLite as the desktop authority for project data.".to_string(),
            owner: "Platform".to_string(),
            status: "active".to_string(),
            extra_json: must_json(json!({
                "decisionStatus": "active",
            })),
            created_at: docs_updated - Duration::hours(48),
            updated_at: docs_updated - Duration::hours(48),
            archived_at: None,
        },
        knowledge::Item {
            id: "decision-deterministic-import".to_string(),
            project_id: PROJECT_ID.to_string(),
            title: "Decision 0002: Deterministic import".to_string(),
            kind: "decision".to_string(),
            doc_type: String::new(),
            body: "## Status\n\nAccepted\n\n## Context\n\nFacts imported from the repository must be reproducible and explainable.\n\n## Decision\n\nImport only a fixed set of project files and Git metadata, without network calls or model inference.\n\n## Consequences\n\n- The same checkout always yields the same facts.\n- Anything outside the MVP file set needs an explicit source.".to_string(),
            tags: vec!["import".to_string()],
            summary: "Import facts from a fixed file set with no inference.".to_string(),
            owner: "Platform".to_string(),
            status: "active".to_string(),
            extra_json: must_json(json!({
                "decisionStatus": "active",
            })),
            created_at: docs_updated - Duration::hours(36),
            updated_at: docs_updated - Duration::hours(36),
            archived_at: None,
        },
        knowledge::Item {
            id: "decision-bounded-context".to_string(),
            project_id: PROJECT_ID.to_string(),
            title: "Decision 0003: Bounded agent context".to_string(),
            kind: "decision".to_string(),
            doc_type: String::new(),
            body: "## Status\n\nProposed\n\n## Context\n\nAgents lose accuracy when handed the whole project at once.\n\n## Decision\n\nCompile context per session with a token budget, active invariants first.\n\n## Consequences\n\n- Sessions start faster and cheaper.\n- Budget tuning needs measurement on large projects.".to_string(),
            tags: vec!["agents".to_string()],
            summary: "Give agents a budgeted context instead of the whole project.".to_string(),
            owner: "Agents".to_string(),
            status: "proposed".to_string(),
            extra_json: must_json(json!({
                "decisionStatus": "proposed",
            })),
            created_at: docs_updated - Duration::hours(12),
            updated_at: docs_updated - Duration::hours(12),
            archived_at: None,
        },
        principle_item(
            "decision-principle-local-first",
            "Local-first truth",
            "Project state must stay useful on one machine.",
            "Project state must remain useful on one machine without requiring a hosted account.",
            docs_updated - Duration::hours(48),
        ),
        principle_item(
            "decision-principle-honest-boundaries",
            "Honest product boundaries",
            "Never describe planned work as shipped.",
            "Do not describe planned capabilities as shipped.",
            docs_updated - Duration::hours(48),
        ),
        principle_item(
            "decision-principle-provenance",
            "Provenance over inference",
            "Declared facts win over inferred ones.",
            "Declared and verified facts take precedence over inferred knowledge.",
            docs_updated - Duration::hours(48),
        ),
    ];
    for item in items {
        store.save_knowledge(item)?;
    }
    Ok(())
}

pub(super) fn seed_knowledge_links(store: &Store) -> sqlite::Result<()> {
    struct LinkSpec {
        knowledge_id: &'static str,
        ref_kind: &'static str,
        ref_id: &'static str,
        predicate: &'static str,
    }
    // Links to environment entities (which are synced by SyncProjectModel)
    let specs = [
        LinkSpec {
            knowledge_id: "doc-database",
            ref_kind: canon::REF_ENVIRONMENT,
            ref_id: ENV_PRODUCTION_ID,
            predicate: canon::PRED_DOCUMENTS,
        },
        LinkSpec {
            knowledge_id: "rb-service",
            ref_kind: canon::REF_ENVIRONMENT,
            ref_id: ENV_PRODUCTION_ID,
            predicate: canon::PRED_OPERATES,
        },
        LinkSpec {
            knowledge_id: "rb-database",
            ref_kind: canon::REF_ENVIRONMENT,
            ref_id: ENV_PRODUCTION_ID,
            predicate: canon::PRED_OPERATES,
        },
        LinkSpec {
            knowledge_id: "decision-sqlite-local",
            ref_kind: canon::REF_ENVIRONMENT,
            ref_id: ENV_PRODUCTION_ID,
            predicate: canon::PRED_RELATED_TO,
        },
    ];
    for spec in specs {
        let Ok(entity) = store.get_canon_entity_by_ref(PROJECT_ID, spec.ref_kind, spec.ref_id)
        else {
            continue;
        };
        let link = knowledge::new_entity_link(
            PROJECT_ID,
            spec.knowledge_id,
            &entity.id,
            spec.predicate,
            canon::TRUTH_DECLARED,
            "seed",
            "system",
        )
        .map_err(|err| sqlite::Error::message(err.to_string()))?;
        store.save_knowledge_link(&link)?;
    }
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

fn principle_item(
    id: &str,
    title: &str,
    summary: &str,
    body: &str,
    at: OffsetDateTime,
) -> knowledge::Item {
    knowledge::Item {
        id: id.to_string(),
        project_id: PROJECT_ID.to_string(),
        title: title.to_string(),
        kind: "decision".to_string(),
        body: format!("## Decision\n\n{body}"),
        summary: summary.to_string(),
        owner: "Platform".to_string(),
        status: "active".to_string(),
        extra_json: must_json(json!({"level": "principle"})),
        created_at: at - Duration::hours(48),
        updated_at: at,
        ..knowledge::Item::default()
    }
}
