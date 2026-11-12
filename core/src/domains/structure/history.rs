//! A Structure node's changes over time.

use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value};
use time::{
    Date, Duration, OffsetDateTime, format_description::well_known::Rfc3339,
    macros::format_description,
};

use super::{Error, NODE_ACTIVE, Node, Result, Row, decorate, deployments_from};

/// One `change_ops` JSON object: a diff of the columns an op changed.
pub type Fields = Map<String, Value>;

/// One `change_ops` row that touched a Structure node or invariant.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryOp {
    pub seq: i64,
    pub batch_id: String,
    pub agent_run_id: String,
    pub kind: String,
    pub entity_id: String,
    /// Set for invariant ops; empty for node ops.
    pub node_id: String,
    pub fields: Fields,
    pub created_at: OffsetDateTime,
}

impl Default for HistoryOp {
    fn default() -> Self {
        Self {
            seq: 0,
            batch_id: String::new(),
            agent_run_id: String::new(),
            kind: String::new(),
            entity_id: String::new(),
            node_id: String::new(),
            fields: Fields::new(),
            created_at: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

/// One batch that touched the Structure.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryEntry {
    pub batch_id: String,
    pub at: OffsetDateTime,
    pub agent_run_id: String,
    pub node_ops: usize,
    pub summary: String,
}

impl Default for HistoryEntry {
    fn default() -> Self {
        Self {
            batch_id: String::new(),
            at: OffsetDateTime::UNIX_EPOCH,
            agent_run_id: String::new(),
            node_ops: 0,
            summary: String::new(),
        }
    }
}

/// Groups ops into batches, oldest first.
pub fn history(ops: &[HistoryOp]) -> Vec<HistoryEntry> {
    let mut entries: Vec<HistoryEntry> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for op in ops {
        let i = match index.get(op.batch_id.as_str()).copied() {
            Some(i) => i,
            None => {
                let i = entries.len();
                index.insert(op.batch_id.clone(), i);
                entries.push(HistoryEntry {
                    batch_id: op.batch_id.clone(),
                    at: op.created_at,
                    agent_run_id: op.agent_run_id.clone(),
                    ..HistoryEntry::default()
                });
                i
            }
        };
        entries[i].node_ops += 1;
        if entries[i].summary.is_empty() && op.node_id.is_empty() {
            entries[i].summary = first_string(&op.fields, &["path", "title", "slug"]);
        }
    }
    entries
}

/// Returns how many leading ops belong to the point at: a change batch id (that
/// batch is included), an RFC3339 time, or a YYYY-MM-DD day (the whole day is
/// included, in UTC). `ops` must be ordered oldest first.
pub fn cut(ops: &[HistoryOp], at: &str) -> Result<usize> {
    let at = at.trim();
    if at.is_empty() {
        return Ok(ops.len());
    }
    let mut last = None;
    for (i, op) in ops.iter().enumerate() {
        if op.batch_id == at {
            last = Some(i);
        }
    }
    if let Some(last) = last {
        return Ok(last + 1);
    }
    let limit = parse_point(at)?;
    let mut n = 0;
    for op in ops {
        if op.created_at > limit {
            break;
        }
        n += 1;
    }
    Ok(n)
}

/// Reads an RFC3339 time or a YYYY-MM-DD day (end of that day, UTC).
pub fn parse_point(at: &str) -> Result<OffsetDateTime> {
    if let Ok(point) = OffsetDateTime::parse(at, &Rfc3339) {
        return Ok(point);
    }
    if let Ok(day) = Date::parse(at, format_description!("[year]-[month]-[day]")) {
        return Ok(day.midnight().assume_utc() + Duration::hours(24) - Duration::seconds(1));
    }
    Err(Error::UnknownPoint(at.to_string()))
}

/// Rebuilds the nodes as they were after `ops` (oldest first). Ops are diffs: a
/// create carries every column, an update only the changed ones. The result is
/// ordered by position; removed nodes stay, marked `NODE_REMOVED`.
pub fn replay(project_id: &str, ops: &[HistoryOp]) -> Vec<Node> {
    let mut nodes: HashMap<String, Node> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut invariant_owner: HashSet<String> = HashSet::new();
    for op in ops {
        if !op.node_id.is_empty() {
            // A node invariant op from before Rules; Rules are not replayed.
            invariant_owner.insert(op.entity_id.clone());
            continue;
        }
        if invariant_owner.contains(&op.entity_id) {
            continue;
        }
        if !nodes.contains_key(&op.entity_id) {
            // Invariant ops written before migration 0023 carry no node id;
            // they have no path either, so they are skipped here.
            if op.kind != "create" || first_string(&op.fields, &["path"]).is_empty() {
                continue;
            }
            nodes.insert(
                op.entity_id.clone(),
                Node {
                    id: op.entity_id.clone(),
                    project_id: project_id.to_string(),
                    status: NODE_ACTIVE.to_string(),
                    ..Node::default()
                },
            );
            order.push(op.entity_id.clone());
        }
        if let Some(node) = nodes.get_mut(&op.entity_id) {
            apply_node_fields(node, &op.fields);
        }
    }
    let mut result: Vec<Node> = order
        .iter()
        .filter_map(|id| nodes.get(id).cloned())
        .collect();
    result.sort_by_key(|node| node.position);
    result
}

fn apply_node_fields(node: &mut Node, fields: &Fields) {
    for (key, value) in fields {
        let text = match value {
            Value::String(text) => text.as_str(),
            _ => "",
        };
        match key.as_str() {
            "parent_id" => node.parent_id = text.to_string(),
            "level" => node.level = text.to_string(),
            "slug" => node.slug = text.to_string(),
            "path" => node.path = text.to_string(),
            "title" => node.title = text.to_string(),
            "intent" => node.intent = text.to_string(),
            "status" => node.status = text.to_string(),
            "repo" => node.repo = text.to_string(),
            "folder" => node.folder = text.to_string(),
            "kind" => node.kind = text.to_string(),
            "tech" => node.tech = text.to_string(),
            "migration_from" => node.migration_from = text.to_string(),
            "migration_to" => node.migration_to = text.to_string(),
            "position" => {
                if let Some(position) = number(value) {
                    node.position = position;
                }
            }
            "decisions" => node.decisions = string_list(value),
            "deploys" => {
                if let Some(deploys) = deployments_from(value) {
                    node.deploys = deploys;
                }
            }
            _ => {}
        }
    }
}

fn number(value: &Value) -> Option<i32> {
    match value {
        Value::Number(value) => value.as_f64().map(|value| value as i32),
        _ => None,
    }
}

fn string_list(value: &Value) -> Vec<String> {
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

fn first_string(fields: &Fields, keys: &[&str]) -> String {
    for key in keys {
        if let Some(Value::String(text)) = fields.get(*key)
            && !text.is_empty()
        {
            return text.clone();
        }
    }
    String::new()
}

/// Wraps replayed nodes as rows with no derived status or counts. Removed nodes
/// are left out. Child counts are still computed.
pub fn historical(nodes: &[Node]) -> Vec<Row> {
    let mut rows = decorate(nodes, &[]);
    for row in &mut rows {
        row.derived = String::new();
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use time::macros::datetime;

    fn fields(value: Value) -> Fields {
        value.as_object().expect("an object of fields").clone()
    }

    fn history_ops() -> Vec<HistoryOp> {
        let t0 = datetime!(2026-09-01 10:00:00 UTC);
        vec![
            HistoryOp {
                seq: 1,
                batch_id: "b1".to_string(),
                kind: "create".to_string(),
                entity_id: "n1".to_string(),
                created_at: t0,
                fields: fields(json!({
                    "path": "core", "slug": "core", "title": "Core", "level": "domain", "status": "active",
                    "position": 0, "decisions": ["Decision-0001"],
                })),
                ..HistoryOp::default()
            },
            HistoryOp {
                seq: 2,
                batch_id: "b1".to_string(),
                kind: "create".to_string(),
                entity_id: "i1".to_string(),
                node_id: "n1".to_string(),
                created_at: t0,
                fields: fields(json!({"text": "No network", "state": "active", "position": 0})),
                ..HistoryOp::default()
            },
            HistoryOp {
                seq: 3,
                batch_id: "b2".to_string(),
                kind: "create".to_string(),
                entity_id: "n2".to_string(),
                created_at: t0 + Duration::hours(24),
                fields: fields(json!({
                    "path": "core/import", "slug": "import", "title": "Import", "level": "component",
                    "status": "active", "parent_id": "n1", "position": 1,
                })),
                ..HistoryOp::default()
            },
            HistoryOp {
                seq: 4,
                batch_id: "b3".to_string(),
                kind: "update".to_string(),
                entity_id: "n1".to_string(),
                created_at: t0 + Duration::hours(48),
                fields: fields(json!({"migration_from": "REST", "migration_to": "Connect"})),
                ..HistoryOp::default()
            },
            HistoryOp {
                seq: 5,
                batch_id: "b3".to_string(),
                kind: "update".to_string(),
                entity_id: "i1".to_string(),
                node_id: "n1".to_string(),
                created_at: t0 + Duration::hours(48),
                fields: fields(json!({"state": "removed"})),
                ..HistoryOp::default()
            },
            HistoryOp {
                seq: 6,
                batch_id: "b4".to_string(),
                kind: "remove".to_string(),
                entity_id: "n2".to_string(),
                created_at: t0 + Duration::hours(72),
                fields: fields(json!({"status": "removed"})),
                ..HistoryOp::default()
            },
        ]
    }

    #[test]
    fn replay_at_batch() {
        let ops = history_ops();
        let cut_at = cut(&ops, "b2").expect("b2 is a batch id");
        assert_eq!(cut_at, 3, "Cut(b2) = {cut_at}");
        let nodes = replay("p", &ops[..cut_at]);
        assert!(
            nodes.len() == 2 && nodes[0].path == "core" && nodes[1].path == "core/import",
            "nodes = {nodes:?}"
        );
        assert!(!nodes[0].migrating(), "core migrates before b3");
        let decisions = &nodes[0].decisions;
        assert!(
            decisions.len() == 1 && decisions[0] == "Decision-0001",
            "decisions = {decisions:?}"
        );
    }

    #[test]
    fn replay_latest_matches_final_state() {
        let ops = history_ops();
        let nodes = replay("p", &ops);
        let rows = historical(&nodes);
        assert!(
            rows.len() == 1 && rows[0].node.path == "core",
            "rows = {rows:?} (removed node must be left out)"
        );
        assert!(
            rows[0].node.migration_from == "REST" && rows[0].node.migration_to == "Connect",
            "migration = {:?} -> {:?}",
            rows[0].node.migration_from,
            rows[0].node.migration_to
        );
        assert!(rows[0].derived.is_empty(), "row = {:?}", rows[0]);
    }

    #[test]
    fn cut_by_day() {
        let ops = history_ops();
        let cut_at = cut(&ops, "2026-09-02").expect("a day is a known point");
        assert_eq!(cut_at, 3, "Cut(day) = {cut_at}");
        assert!(
            cut(&ops, "yesterday").is_err(),
            "expected an error for an unknown point"
        );
    }

    #[test]
    fn history_groups_batches() {
        let entries = history(&history_ops());
        assert!(
            entries.len() == 4 && entries[0].node_ops == 2 && entries[0].summary == "core",
            "entries = {entries:?}"
        );
    }

    #[test]
    fn replay_kind_and_tech() {
        let t0 = datetime!(2026-09-01 10:00:00 UTC);
        let ops = [
            HistoryOp {
                seq: 1,
                batch_id: "b1".to_string(),
                kind: "create".to_string(),
                entity_id: "n1".to_string(),
                created_at: t0,
                fields: fields(json!({
                    "path": "db", "slug": "db", "title": "DB", "level": "component", "status": "active",
                    "kind": "database", "tech": "Postgres",
                })),
                ..HistoryOp::default()
            },
            HistoryOp {
                seq: 2,
                batch_id: "b2".to_string(),
                kind: "update".to_string(),
                entity_id: "n1".to_string(),
                created_at: t0 + Duration::hours(1),
                fields: fields(json!({"tech": "SQLite"})),
                ..HistoryOp::default()
            },
        ];
        let at_b1 = replay("p", &ops[..1]);
        assert!(
            at_b1[0].kind == "database" && at_b1[0].tech == "Postgres",
            "at b1 = {at_b1:?}"
        );
        let latest = replay("p", &ops);
        assert!(
            latest[0].kind == "database" && latest[0].tech == "SQLite",
            "latest = {latest:?}"
        );
    }
}
