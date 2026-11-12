//! Port of `backend/providers/seed/knowledge.go`.
//!
//! The pinned Rules (the old principles) and the two docs that name the
//! repository documents defining Polterware.

use serde_json::{Map, json};
use time::OffsetDateTime;

use crate::domains::{knowledge, work};
use crate::providers::sqlite;

use super::PROJECT_ID;

struct Principle {
    id: &'static str,
    title: &'static str,
    summary: &'static str,
    body: &'static str,
}

/// Principles are Rules with no scope: they apply to the whole project.
const PRINCIPLES: [Principle; 5] = [
    Principle {
        id: "decision-principle-local-first",
        title: "Local-first truth",
        summary: "Project state must stay useful on one machine.",
        body: "Project state must remain useful on one machine without requiring a hosted account.",
    },
    Principle {
        id: "decision-principle-honest-boundaries",
        title: "Honest product boundaries",
        summary: "Never describe planned work as shipped.",
        body: "Do not describe planned capabilities as shipped. Post-wedge depth (Change Gates, Web App, Cloud) is direction, not behavior.",
    },
    Principle {
        id: "decision-principle-provenance",
        title: "Provenance over inference",
        summary: "Declared facts win over inferred ones.",
        body: "Declared and verified facts take precedence over inferred knowledge. Inference never silently becomes project truth.",
    },
    Principle {
        id: "decision-principle-one-authority",
        title: "One authority per database",
        summary: "Only the local core opens SQLite.",
        body: "Only the local core opens SQLite; every other surface is a client.",
    },
    Principle {
        id: "decision-principle-shared-interface",
        title: "Opaque, shared interface",
        summary: "Product UI uses the shared design system.",
        body: "Product UI uses the shared design system: opaque surfaces, token colors and radii, one icon set.",
    },
];

struct Doc {
    id: &'static str,
    title: &'static str,
    path: &'static str,
    summary: &'static str,
}

/// Docs are the repository documents that define Polterware. The store holds a
/// pointer and a summary; the file stays the source.
const DOCS: [Doc; 2] = [
    Doc {
        id: "doc-future",
        title: "Future work",
        path: "docs/future.md",
        summary: "Planned direction that is not implemented: Cloud, Jev, enrichment, research.",
    },
    Doc {
        id: "doc-design",
        title: "Desktop design",
        path: "DESIGN.md",
        summary: "The visual and interaction contract for the Desktop.",
    },
];

pub(super) fn seed_knowledge(store: &sqlite::Store) -> sqlite::Result<()> {
    let at = OffsetDateTime::now_utc();
    seed_rules(store)?;
    let mut items: Vec<knowledge::Item> = Vec::new();
    for doc in DOCS {
        items.push(knowledge::Item {
            id: doc.id.to_string(),
            project_id: PROJECT_ID.to_string(),
            title: doc.title.to_string(),
            kind: knowledge::KIND_DOC.to_string(),
            doc_type: knowledge::DOC_TYPE_DOC.to_string(),
            body: format!("{}\n\nSource: {}", doc.summary, doc.path),
            summary: doc.summary.to_string(),
            tags: vec!["polterware".to_string()],
            owner: "Platform".to_string(),
            status: "published".to_string(),
            extra_json: must_json(&serde_json::json!({ "path": doc.path })),
            created_at: at,
            updated_at: at,
            ..knowledge::Item::default()
        });
    }
    for item in items {
        store.save_knowledge(item)?;
    }
    Ok(())
}

/// Seeds the project-wide Rules: one Rule per old principle, with the summary
/// as its statement and the body as its why.
fn seed_rules(store: &sqlite::Store) -> sqlite::Result<()> {
    let mut ops: Vec<work::Op> = Vec::with_capacity(PRINCIPLES.len());
    for principle in PRINCIPLES {
        let mut fields = Map::new();
        fields.insert("statement".into(), json!(principle.body));
        fields.insert("why".into(), json!(principle.summary));
        ops.push(work::Op {
            entity: work::ENTITY_RULE.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: principle.id.to_string(),
            card_slug: principle.title.to_string(),
            label: principle.title.to_string(),
            fields,
            ..work::Op::default()
        });
    }
    store.apply_work_ops(PROJECT_ID, "", "seed-rules", &ops)?;
    Ok(())
}

fn must_json(value: &serde_json::Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
}
