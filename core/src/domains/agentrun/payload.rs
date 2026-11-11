//! Port of `backend/domains/agentrun/payload.go`.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::domains::context::Package;

use super::{Error, KIND_CHANGE, KIND_CONTEXT, Result, Step, new_step_id};

/// A light snapshot of a compiled context package.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPayload {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub query: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub entity_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub item_ids: Vec<String>,
}

/// A change envelope. Declared-only appends omit `tool`/`subject_ids`; audited
/// MCP writes (B7) set them after a real mutation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangePayload {
    pub summary: String,
    pub rationale: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tool: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subject_ids: Vec<String>,
    /// Links the step to its `change_ops` rows (Polter apply).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub batch_id: String,
}

pub fn marshal_payload<T: Serialize + ?Sized>(
    value: &T,
) -> std::result::Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

pub fn context_from_package(pkg: &Package) -> ContextPayload {
    let item_ids: Vec<String> = pkg
        .items
        .iter()
        .filter(|item| !item.id.is_empty())
        .map(|item| item.id.clone())
        .collect();
    ContextPayload {
        query: pkg.query.clone(),
        entity_id: pkg.entity_id.clone(),
        item_ids,
    }
}

pub fn context_step_summary(pkg: &Package) -> String {
    format!(
        "context package: {} items, {} tokens",
        pkg.items.len(),
        pkg.tokens_used
    )
}

pub fn new_change_step(
    run_id: &str,
    summary: &str,
    rationale: &str,
    paths: &[String],
    now: Option<OffsetDateTime>,
) -> Result<Step> {
    new_mutation_change_step(run_id, summary, rationale, "", &[], paths, now)
}

/// Builds a change step, optionally attributing a write tool and subject ids.
pub fn new_mutation_change_step(
    run_id: &str,
    summary: &str,
    rationale: &str,
    tool: &str,
    subject_ids: &[String],
    paths: &[String],
    now: Option<OffsetDateTime>,
) -> Result<Step> {
    new_batch_change_step(
        run_id,
        summary,
        rationale,
        tool,
        "",
        subject_ids,
        paths,
        now,
    )
}

/// `new_mutation_change_step` plus the `change_ops` batch id.
#[allow(clippy::too_many_arguments)]
pub fn new_batch_change_step(
    run_id: &str,
    summary: &str,
    rationale: &str,
    tool: &str,
    batch_id: &str,
    subject_ids: &[String],
    paths: &[String],
    now: Option<OffsetDateTime>,
) -> Result<Step> {
    // The audited write already checked the rationale for its actor (a person
    // may leave it out); every step still names its change.
    if summary.trim().is_empty() {
        return Err(Error::ChangeIncomplete);
    }
    let payload = marshal_payload(&ChangePayload {
        summary: summary.trim().to_string(),
        rationale: rationale.trim().to_string(),
        paths: paths.to_vec(),
        tool: tool.trim().to_string(),
        subject_ids: subject_ids.to_vec(),
        batch_id: batch_id.trim().to_string(),
    })
    .map_err(|err| Error::Json(err.to_string()))?;
    Ok(new_step(run_id, KIND_CHANGE, summary.trim(), &payload, now))
}

pub fn new_context_step(run_id: &str, pkg: &Package, now: Option<OffsetDateTime>) -> Result<Step> {
    let payload =
        marshal_payload(&context_from_package(pkg)).map_err(|err| Error::Json(err.to_string()))?;
    Ok(new_step(
        run_id,
        KIND_CONTEXT,
        &context_step_summary(pkg),
        &payload,
        now,
    ))
}

fn new_step(
    run_id: &str,
    kind: &str,
    summary: &str,
    payload_json: &str,
    now: Option<OffsetDateTime>,
) -> Step {
    let created_at = match now {
        Some(now) => now.to_offset(time::UtcOffset::UTC),
        None => OffsetDateTime::now_utc(),
    };
    Step {
        id: new_step_id(),
        agent_run_id: run_id.to_string(),
        kind: kind.to_string(),
        summary: summary.to_string(),
        payload_json: if payload_json.is_empty() {
            "{}".to_string()
        } else {
            payload_json.to_string()
        },
        created_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::context::Item;
    use pretty_assertions::assert_eq;

    #[test]
    fn change_step_requires_the_envelope() {
        assert_eq!(
            new_change_step("arun-1", "", "why", &[], None).unwrap_err(),
            Error::ChangeIncomplete
        );
        let step = new_change_step(
            "arun-1",
            "Add DEPENDS_ON",
            "Matches Decision",
            &["docs/decision/0004.md".to_string()],
            None,
        )
        .expect("change step");
        assert_eq!(step.kind, KIND_CHANGE);
        assert!(step.payload_json.contains("rationale"), "{step:?}");
        assert_eq!(
            step.payload_json,
            r#"{"summary":"Add DEPENDS_ON","rationale":"Matches Decision","paths":["docs/decision/0004.md"]}"#
        );
    }

    #[test]
    fn mutation_change_step_includes_the_tool() {
        let step = new_mutation_change_step(
            "arun-1",
            "Add fact",
            "From Decision",
            "fact.upsert",
            &["fact-1".to_string()],
            &["docs/decision/0004.md".to_string()],
            None,
        )
        .expect("mutation step");
        assert!(
            step.payload_json.contains(r#""tool":"fact.upsert""#),
            "missing tool in payload: {}",
            step.payload_json
        );
        assert!(
            step.payload_json.contains("fact-1"),
            "missing subject id: {}",
            step.payload_json
        );
    }

    #[test]
    fn batch_change_step_carries_the_batch_id() {
        let step = new_batch_change_step(
            "arun-1",
            "Apply Polter",
            "Batch",
            "polter.apply",
            "batch-1",
            &[],
            &[],
            None,
        )
        .expect("batch step");
        assert_eq!(
            step.payload_json,
            r#"{"summary":"Apply Polter","rationale":"Batch","tool":"polter.apply","batch_id":"batch-1"}"#
        );
    }

    #[test]
    fn context_payload_snapshots_the_package() {
        let pkg = Package {
            query: "sqlite".to_string(),
            entity_id: "ent-1".to_string(),
            tokens_used: 40,
            items: vec![
                Item {
                    id: "item-1".to_string(),
                    ..Item::default()
                },
                Item {
                    id: "item-2".to_string(),
                    ..Item::default()
                },
                Item::default(),
            ],
            ..Package::default()
        };
        let step = new_context_step("arun-1", &pkg, None).expect("context step");
        assert_eq!(step.kind, KIND_CONTEXT);
        assert!(step.payload_json.contains("item-1"), "{step:?}");
        assert_eq!(step.summary, "context package: 3 items, 40 tokens");
        assert_eq!(
            step.payload_json,
            r#"{"query":"sqlite","entity_id":"ent-1","item_ids":["item-1","item-2"]}"#
        );
    }
}
