//! Port of `backend/providers/sqlite/work.go`.
//!
//! The Work snapshot — every card with its tasks, questions, invariants,
//! claims, and handoffs, plus the rows read beside them — and the writer that
//! applies a planned batch of ops in one transaction, recording one change per
//! op.

use std::collections::{HashMap, HashSet};

use rusqlite::types::{ToSqlOutput, ValueRef};
use rusqlite::{Connection, Params, Row, ToSql, params, params_from_iter};
use serde_json::{Map, Value};
use time::OffsetDateTime;

use crate::domains::{structure, work};
use crate::platform::ids;

use super::commit_links::reindex_work_card_fts;
use super::conflicts::apply_conflict_op;
use super::rules::apply_rule_op;
use super::*;

/// The cards of the project the tasks, questions, and invariants belong to.
const CARDS_SCOPE: &str = "card_id IN (SELECT id FROM work_cards WHERE project_id=?)";

/// One value bound into a column: the port of the `any` `columnValue` returns.
/// A JSON array becomes its marshaled text and a boolean becomes SQLite's 1 and
/// 0.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ColumnValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
}

impl ColumnValue {
    pub(crate) fn text(value: impl Into<String>) -> ColumnValue {
        ColumnValue::Text(value.into())
    }

    /// Reports whether the value is the empty text, which an empty milestone
    /// turns into `NULL`.
    fn is_empty_text(&self) -> bool {
        matches!(self, ColumnValue::Text(text) if text.is_empty())
    }
}

impl ToSql for ColumnValue {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(match self {
            ColumnValue::Null => ToSqlOutput::Borrowed(ValueRef::Null),
            ColumnValue::Integer(value) => ToSqlOutput::Borrowed(ValueRef::Integer(*value)),
            ColumnValue::Real(value) => ToSqlOutput::Borrowed(ValueRef::Real(*value)),
            ColumnValue::Text(value) => ToSqlOutput::Borrowed(ValueRef::Text(value.as_bytes())),
        })
    }
}

/// Converts one planned field value into the column value to bind. A JSON array
/// is stored as its marshaled text and a boolean as 1 or 0, like Go's
/// `columnValue`.
pub(crate) fn column_value(value: &Value) -> Result<ColumnValue> {
    Ok(match value {
        Value::Array(_) => ColumnValue::Text(go_json(value)),
        Value::Bool(flag) => ColumnValue::Integer(i64::from(*flag)),
        Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                ColumnValue::Integer(integer)
            } else if let Some(real) = number.as_f64() {
                ColumnValue::Real(real)
            } else {
                ColumnValue::Null
            }
        }
        Value::String(text) => ColumnValue::Text(text.clone()),
        Value::Null => ColumnValue::Null,
        Value::Object(_) => return Err(Error::message("unsupported column value")),
    })
}

/// Marshals a value the way Go's `json.Marshal` does, which escapes `<`, `>`,
/// `&`, and the two line separators inside strings. Map keys sort in both.
pub(crate) fn go_json(value: &Value) -> String {
    value
        .to_string()
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

/// Marshals an op's fields as the `data_json` a `change_ops` row keeps.
pub(crate) fn go_json_fields(fields: &Map<String, Value>) -> String {
    go_json(&Value::Object(fields.clone()))
}

/// Marshals a string list the way Go's `json.Marshal` does.
pub(crate) fn go_json_strings(values: &[String]) -> String {
    go_json(&Value::Array(
        values
            .iter()
            .map(|value| Value::from(value.clone()))
            .collect(),
    ))
}

/// A string field of an op, or the empty string when it is absent or of
/// another type: Go's `op.Fields[key].(string)` in every writer.
pub(crate) fn field_str(fields: &Map<String, Value>, key: &str) -> String {
    field_text(fields, key).unwrap_or_default()
}

/// A string field that was present as a string, like a Go type assertion that
/// reports whether it held.
pub(crate) fn field_text(fields: &Map<String, Value>, key: &str) -> Option<String> {
    match fields.get(key) {
        Some(Value::String(text)) => Some(text.clone()),
        _ => None,
    }
}

/// The column value of an absent field is `NULL`, like indexing a Go map.
pub(crate) fn field_column(fields: &Map<String, Value>, key: &str) -> Result<ColumnValue> {
    column_value(fields.get(key).unwrap_or(&Value::Null))
}

/// Reads a `scope` field: the Structure node paths it names, or nothing when
/// the value is not a list of strings, like Go's `op.Fields["scope"].([]string)`.
pub(crate) fn scope_paths(value: &Value) -> Option<Vec<String>> {
    let Value::Array(items) = value else {
        return None;
    };
    let mut paths = Vec::with_capacity(items.len());
    for item in items {
        paths.push(item.as_str()?.to_string());
    }
    Some(paths)
}

impl super::Store {
    /// Runs `query` and calls `scan` for each row. The Go helper every reader
    /// shares takes one argument; the port takes the statement's parameters.
    pub fn each_row(
        &self,
        query: &str,
        params: impl Params,
        mut scan: impl FnMut(&Row<'_>) -> rusqlite::Result<()>,
    ) -> Result<()> {
        let mut stmt = self.db.prepare(query)?;
        let mut rows = stmt.query(params)?;
        while let Some(row) = rows.next()? {
            scan(row)?;
        }
        Ok(())
    }

    /// Returns every card of a project with its tasks, questions, and
    /// invariants, ordered by position.
    pub fn load_work_snapshot(&self, project_id: &str) -> Result<work::Snapshot> {
        let mut snapshot = work::Snapshot {
            project_id: project_id.to_string(),
            ..work::Snapshot::default()
        };
        for env in self.list_environments(project_id)? {
            snapshot.environments.push(work::NamedEnvironment {
                id: env.id,
                name: env.name,
            });
        }
        let mut index: HashMap<String, usize> = HashMap::new();
        {
            let mut stmt = self.db.prepare(
                r#"SELECT id,slug,title,intent,stage,status,owner,kind,under,decisions_json,
		COALESCE(milestone_id,''),COALESCE((SELECT slug FROM milestones WHERE milestones.id=work_cards.milestone_id),''),
		position,created_at,updated_at
		FROM work_cards WHERE project_id=? ORDER BY position, created_at"#,
            )?;
            let mut rows = stmt.query(params![project_id])?;
            while let Some(row) = rows.next()? {
                let decisions: String = row.get(9)?;
                let created: String = row.get(13)?;
                let updated: String = row.get(14)?;
                let card = work::Card {
                    id: row.get(0)?,
                    project_id: project_id.to_string(),
                    slug: row.get(1)?,
                    title: row.get(2)?,
                    intent: row.get(3)?,
                    stage: row.get(4)?,
                    status: row.get(5)?,
                    owner: row.get(6)?,
                    kind: row.get(7)?,
                    under: row.get(8)?,
                    decisions: serde_json::from_str::<Vec<String>>(&decisions).unwrap_or_default(),
                    milestone_id: row.get(10)?,
                    milestone: row.get(11)?,
                    position: row.get(12)?,
                    created_at: parse_rfc3339(&created),
                    updated_at: parse_rfc3339(&updated),
                    ..work::Card::default()
                };
                index.insert(card.id.clone(), snapshot.cards.len());
                snapshot.cards.push(card);
            }
        }
        snapshot.repos = self.load_repo_names(project_id)?;
        snapshot.milestones = self.list_milestones(project_id)?;
        snapshot.rules = self.load_rules(project_id)?;
        snapshot.conflicts = self.load_conflicts(project_id)?;
        if snapshot.cards.is_empty() {
            snapshot.nodes = self.load_structure_nodes(project_id)?;
            return Ok(snapshot);
        }

        self.each_row(
            &format!(
                r#"SELECT id,card_id,number,title,state,repo,scope,position FROM work_tasks WHERE {CARDS_SCOPE} ORDER BY position, number"#
            ),
            params![project_id],
            |row| {
                let task = work::Task {
                    id: row.get(0)?,
                    card_id: row.get(1)?,
                    number: row.get(2)?,
                    title: row.get(3)?,
                    state: row.get(4)?,
                    repo: row.get(5)?,
                    scope: row.get(6)?,
                    position: row.get(7)?,
                    claim: None,
                };
                if let Some(&position) = index.get(&task.card_id) {
                    snapshot.cards[position].tasks.push(task);
                }
                Ok(())
            },
        )?;
        self.each_row(
            &format!(
                r#"SELECT id,card_id,text,blocking,options_json,state,answer,position FROM work_questions WHERE {CARDS_SCOPE} ORDER BY position"#
            ),
            params![project_id],
            |row| {
                let blocking: i64 = row.get(3)?;
                let options: String = row.get(4)?;
                let question = work::Question {
                    id: row.get(0)?,
                    card_id: row.get(1)?,
                    text: row.get(2)?,
                    blocking: blocking != 0,
                    options: serde_json::from_str::<Vec<String>>(&options).unwrap_or_default(),
                    state: row.get(5)?,
                    answer: row.get(6)?,
                    position: row.get(7)?,
                };
                if let Some(&position) = index.get(&question.card_id) {
                    snapshot.cards[position].questions.push(question);
                }
                Ok(())
            },
        )?;
        self.each_row(
            &format!(
                r#"SELECT id,card_id,text,state,position FROM work_invariants WHERE {CARDS_SCOPE} ORDER BY position"#
            ),
            params![project_id],
            |row| {
                let invariant = work::Invariant {
                    id: row.get(0)?,
                    card_id: row.get(1)?,
                    text: row.get(2)?,
                    state: row.get(3)?,
                    position: row.get(4)?,
                };
                if let Some(&position) = index.get(&invariant.card_id) {
                    snapshot.cards[position].invariants.push(invariant);
                }
                Ok(())
            },
        )?;
        self.load_work_claims(project_id, &mut snapshot.cards)?;
        self.each_row(
            &format!(
                r#"SELECT id,project_id,card_id,branch,goal,done,in_progress,next,touched_json,agent,created_at
		FROM work_handoffs WHERE {CARDS_SCOPE} ORDER BY created_at, id"#
            ),
            params![project_id],
            |row| {
                let touched: String = row.get(8)?;
                let created: String = row.get(10)?;
                let handoff = work::Handoff {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    card_id: row.get(2)?,
                    branch: row.get(3)?,
                    goal: row.get(4)?,
                    done: row.get(5)?,
                    in_progress: row.get(6)?,
                    next: row.get(7)?,
                    touched: serde_json::from_str::<Vec<String>>(&touched).unwrap_or_default(),
                    agent: row.get(9)?,
                    created_at: parse_rfc3339(&created),
                };
                if let Some(&position) = index.get(&handoff.card_id) {
                    snapshot.cards[position].handoffs.push(handoff);
                }
                Ok(())
            },
        )?;
        snapshot.nodes = self.load_structure_nodes(project_id)?;
        Ok(snapshot)
    }

    /// Lists the project's repository names.
    fn load_repo_names(&self, project_id: &str) -> Result<Vec<String>> {
        let mut names = Vec::new();
        self.each_row(
            r#"SELECT name FROM repositories WHERE project_id=? ORDER BY name"#,
            params![project_id],
            |row| {
                names.push(row.get(0)?);
                Ok(())
            },
        )?;
        Ok(names)
    }

    /// Attaches each active claim to its task.
    fn load_work_claims(&self, project_id: &str, cards: &mut [work::Card]) -> Result<()> {
        let mut positions: HashMap<String, (usize, usize)> = HashMap::new();
        for (card_index, card) in cards.iter().enumerate() {
            for (task_index, task) in card.tasks.iter().enumerate() {
                positions.insert(task.id.clone(), (card_index, task_index));
            }
        }
        let mut claims: Vec<work::Claim> = Vec::new();
        self.each_row(
            r#"SELECT id,project_id,task_id,holder,scope,created_at FROM work_claims WHERE project_id=?"#,
            params![project_id],
            |row| {
                let created: String = row.get(5)?;
                claims.push(work::Claim {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    task_id: row.get(2)?,
                    holder: row.get(3)?,
                    scope: row.get(4)?,
                    created_at: parse_rfc3339(&created),
                });
                Ok(())
            },
        )?;
        for claim in claims {
            if let Some(&(card_index, task_index)) = positions.get(&claim.task_id) {
                cards[card_index].tasks[task_index].claim = Some(claim);
            }
        }
        Ok(())
    }

    /// Writes ops in one transaction and appends one `change_ops` row per op.
    /// It returns the ids of the cards touched, in first-touch order.
    pub fn apply_work_ops(
        &self,
        project_id: &str,
        agent_run_id: &str,
        batch_id: &str,
        ops: &[work::Op],
    ) -> Result<Vec<String>> {
        let now = format_rfc3339(OffsetDateTime::now_utc());
        let tx = self.db.unchecked_transaction()?;

        let mut card_ids: Vec<String> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for op in ops {
            apply_work_op(&tx, project_id, op, &now).map_err(|err| {
                Error::message(format!("{} {} {}: {err}", op.kind, op.entity, op.card_slug))
            })?;
            if op.entity == work::ENTITY_NODE && !op.node_id.is_empty() {
                tx.execute(
                    r#"UPDATE structure_nodes SET updated_at=? WHERE id=?"#,
                    params![now, op.node_id],
                )?;
            } else if op.entity != work::ENTITY_CARD
                && op.entity != work::ENTITY_NODE
                && !op.card_id.is_empty()
            {
                tx.execute(
                    r#"UPDATE work_cards SET updated_at=? WHERE id=?"#,
                    params![now, op.card_id],
                )?;
            }
            let data = go_json_fields(&op.fields);
            tx.execute(
                r#"INSERT INTO change_ops(id,project_id,agent_run_id,batch_id,entity,entity_id,op,data_json,node_id,created_at)
			VALUES(?,?,?,?,?,?,?,?,?,?)"#,
                params![
                    ids::new_ulid(),
                    project_id,
                    agent_run_id,
                    batch_id,
                    op.entity,
                    op.entity_id,
                    op.kind,
                    data,
                    op.node_id,
                    now,
                ],
            )?;
            if !op.card_id.is_empty()
                && op.entity != work::ENTITY_NODE
                && seen.insert(op.card_id.clone())
            {
                card_ids.push(op.card_id.clone());
            }
        }
        for card_id in &card_ids {
            reindex_work_card_fts(&tx, card_id)?;
        }
        tx.commit()?;
        Ok(card_ids)
    }

    /// Returns the `change_ops` rows that touched Structure nodes and
    /// invariants, oldest first.
    pub fn list_structure_change_ops(&self, project_id: &str) -> Result<Vec<structure::HistoryOp>> {
        let mut ops = Vec::new();
        self.each_row(
            r#"SELECT rowid,batch_id,agent_run_id,op,entity_id,node_id,data_json,created_at
		FROM change_ops WHERE project_id=? AND entity='node' ORDER BY rowid"#,
            params![project_id],
            |row| {
                let data: String = row.get(6)?;
                let created: String = row.get(7)?;
                ops.push(structure::HistoryOp {
                    seq: row.get(0)?,
                    batch_id: row.get(1)?,
                    agent_run_id: row.get(2)?,
                    kind: row.get(3)?,
                    entity_id: row.get(4)?,
                    node_id: row.get(5)?,
                    fields: serde_json::from_str::<Map<String, Value>>(&data).unwrap_or_default(),
                    created_at: parse_rfc3339(&created),
                });
                Ok(())
            },
        )?;
        Ok(ops)
    }
}

/// Writes one op, dispatching the entity kinds that keep their own writer.
fn apply_work_op(tx: &Connection, project_id: &str, op: &work::Op, now: &str) -> Result<()> {
    match op.entity.as_str() {
        work::ENTITY_CLAIM => return apply_claim_op(tx, project_id, op, now),
        work::ENTITY_NODE => return super::structure::apply_structure_op(tx, project_id, op, now),
        work::ENTITY_RULE => return apply_rule_op(tx, project_id, op, now),
        work::ENTITY_CONFLICT => return apply_conflict_op(tx, project_id, op, now),
        _ => {}
    }
    let Some(table) = work_table(&op.entity) else {
        return Err(Error::message(format!("unknown entity {:?}", op.entity)));
    };
    let columns = work_columns(&op.entity).expect("every table has its columns");
    let mut names: Vec<&'static str> = Vec::new();
    let mut values: Vec<ColumnValue> = Vec::new();
    for (field, value) in &op.fields {
        let mut column: Option<&'static str> = None;
        for &(name, target) in columns {
            if name == field {
                column = Some(target);
                break;
            }
        }
        let Some(column) = column else {
            return Err(Error::message(format!("unknown field {field:?}")));
        };
        let mut converted = column_value(value)?;
        // An empty milestone is the Backlog: NULL keeps the foreign key valid.
        if column == "milestone_id" && converted.is_empty_text() {
            converted = ColumnValue::Null;
        }
        names.push(column);
        values.push(converted);
    }

    if op.kind == work::OP_CREATE {
        if op.entity == work::ENTITY_HANDOFF {
            names.extend(["id", "project_id", "card_id", "created_at"]);
            values.extend([
                ColumnValue::text(op.entity_id.as_str()),
                ColumnValue::text(project_id),
                ColumnValue::text(op.card_id.as_str()),
                ColumnValue::text(now),
            ]);
        } else {
            names.extend(["id", "created_at", "updated_at"]);
            values.extend([
                ColumnValue::text(op.entity_id.as_str()),
                ColumnValue::text(now),
                ColumnValue::text(now),
            ]);
            if op.entity == work::ENTITY_CARD {
                names.push("project_id");
                values.push(ColumnValue::text(project_id));
            } else {
                names.push("card_id");
                values.push(ColumnValue::text(op.card_id.as_str()));
            }
        }
        let mut marks = "?,".repeat(names.len());
        marks.pop();
        let query = format!("INSERT INTO {table}({}) VALUES({marks})", names.join(","));
        tx.execute(&query, params_from_iter(values.iter()))?;
        return Ok(());
    }

    if names.is_empty() {
        return Ok(());
    }
    let mut assignments: Vec<String> = names.iter().map(|name| format!("{name}=?")).collect();
    assignments.push("updated_at=?".to_string());
    values.push(ColumnValue::text(now));
    values.push(ColumnValue::text(op.entity_id.as_str()));
    let query = format!("UPDATE {table} SET {} WHERE id=?", assignments.join(","));
    let affected = tx.execute(&query, params_from_iter(values.iter()))?;
    if affected == 0 {
        return Err(Error::message(format!("record {} not found", op.entity_id)));
    }
    Ok(())
}

/// The table each entity is stored in.
fn work_table(entity: &str) -> Option<&'static str> {
    match entity {
        work::ENTITY_CARD => Some("work_cards"),
        work::ENTITY_TASK => Some("work_tasks"),
        work::ENTITY_QUESTION => Some("work_questions"),
        work::ENTITY_INVARIANT => Some("work_invariants"),
        work::ENTITY_HANDOFF => Some("work_handoffs"),
        _ => None,
    }
}

const CARD_COLUMNS: [(&str, &str); 11] = [
    ("slug", "slug"),
    ("title", "title"),
    ("intent", "intent"),
    ("stage", "stage"),
    ("status", "status"),
    ("owner", "owner"),
    ("kind", "kind"),
    ("under", "under"),
    ("decisions", "decisions_json"),
    ("position", "position"),
    ("milestone", "milestone_id"),
];

const TASK_COLUMNS: [(&str, &str); 6] = [
    ("number", "number"),
    ("title", "title"),
    ("state", "state"),
    ("repo", "repo"),
    ("scope", "scope"),
    ("position", "position"),
];

const QUESTION_COLUMNS: [(&str, &str); 6] = [
    ("text", "text"),
    ("blocking", "blocking"),
    ("options", "options_json"),
    ("state", "state"),
    ("answer", "answer"),
    ("position", "position"),
];

const INVARIANT_COLUMNS: [(&str, &str); 3] = [
    ("text", "text"),
    ("state", "state"),
    ("position", "position"),
];

const HANDOFF_COLUMNS: [(&str, &str); 7] = [
    ("branch", "branch"),
    ("agent", "agent"),
    ("goal", "goal"),
    ("done", "done"),
    ("in_progress", "in_progress"),
    ("next", "next"),
    ("touched", "touched_json"),
];

/// The column map of each entity. A field that is not listed is rejected.
fn work_columns(entity: &str) -> Option<&'static [(&'static str, &'static str)]> {
    match entity {
        work::ENTITY_CARD => Some(&CARD_COLUMNS),
        work::ENTITY_TASK => Some(&TASK_COLUMNS),
        work::ENTITY_QUESTION => Some(&QUESTION_COLUMNS),
        work::ENTITY_INVARIANT => Some(&INVARIANT_COLUMNS),
        work::ENTITY_HANDOFF => Some(&HANDOFF_COLUMNS),
        _ => None,
    }
}

/// Inserts or deletes one `work_claims` row. The planner already arbitrated;
/// the checks here only catch a claim another write committed in between (the
/// `task_id` unique key covers the same task).
fn apply_claim_op(tx: &Connection, project_id: &str, op: &work::Op, now: &str) -> Result<()> {
    let holder = field_str(&op.fields, "holder");
    match op.kind.as_str() {
        work::OP_CREATE => {
            let task_id = field_str(&op.fields, "task_id");
            let scope = field_str(&op.fields, "scope");
            if !scope.is_empty() {
                let mut stmt = tx.prepare(
                    r#"SELECT holder,scope FROM work_claims WHERE project_id=? AND holder<>? AND scope<>''"#,
                )?;
                let mut rows = stmt.query(params![project_id, holder])?;
                while let Some(row) = rows.next()? {
                    let other_holder: String = row.get(0)?;
                    let other_scope: String = row.get(1)?;
                    if work::scopes_overlap(&scope, &other_scope) {
                        return Err(Error::message(format!(
                            "scope {scope} overlaps {other_scope} held by {other_holder}"
                        )));
                    }
                }
            }
            tx.execute(
                r#"INSERT INTO work_claims(id,project_id,task_id,holder,scope,created_at) VALUES(?,?,?,?,?,?)"#,
                params![op.entity_id, project_id, task_id, holder, scope, now],
            )?;
            Ok(())
        }
        work::OP_REMOVE => {
            let removed = tx.execute(
                r#"DELETE FROM work_claims WHERE id=?"#,
                params![op.entity_id],
            )?;
            if removed == 0 {
                return Err(Error::message(format!("claim {} not found", op.entity_id)));
            }
            Ok(())
        }
        _ => Err(Error::message(format!(
            "unsupported claim op {:?}",
            op.kind
        ))),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use time::OffsetDateTime;

    use crate::domains::work;
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    use super::*;

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    fn fields(entries: &[(&str, Value)]) -> work::Fields {
        let mut fields = work::Fields::new();
        for (key, value) in entries {
            fields.insert((*key).to_string(), value.clone());
        }
        fields
    }

    fn card_op(id: &str, slug: &str, stage: &str) -> work::Op {
        work::Op {
            entity: work::ENTITY_CARD.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: id.to_string(),
            card_id: id.to_string(),
            card_slug: slug.to_string(),
            label: slug.to_string(),
            fields: fields(&[
                ("slug", json!(slug)),
                ("title", json!(slug.to_uppercase())),
                ("stage", json!(stage)),
                ("status", json!(work::CARD_ACTIVE)),
                ("decisions", json!(["Decision-0001"])),
                ("position", json!(0)),
            ]),
            ..work::Op::default()
        }
    }

    #[test]
    fn snapshot_reads_cards_with_their_children() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", now());
        let card = card_op("card-a", "a", work::STAGE_BUILD);
        let task = work::Op {
            entity: work::ENTITY_TASK.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: "task-1".to_string(),
            card_id: "card-a".to_string(),
            card_slug: "a".to_string(),
            fields: fields(&[
                ("number", json!(1)),
                ("title", json!("Wire the store")),
                ("state", json!(work::TASK_OPEN)),
                ("position", json!(0)),
            ]),
            ..work::Op::default()
        };
        let question = work::Op {
            entity: work::ENTITY_QUESTION.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: "question-1".to_string(),
            card_id: "card-a".to_string(),
            card_slug: "a".to_string(),
            fields: fields(&[
                ("text", json!("Which store?")),
                ("blocking", json!(true)),
                ("options", json!(["sqlite", "postgres"])),
                ("position", json!(0)),
            ]),
            ..work::Op::default()
        };
        let handoff = work::Op {
            entity: work::ENTITY_HANDOFF.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: "handoff-1".to_string(),
            card_id: "card-a".to_string(),
            card_slug: "a".to_string(),
            fields: fields(&[
                ("branch", json!("main")),
                ("agent", json!("codex")),
                ("goal", json!("port the store")),
                ("touched", json!(["work.go"])),
            ]),
            ..work::Op::default()
        };
        let touched = store
            .apply_work_ops("proj", "", "batch-1", &[card, task, question, handoff])
            .expect("apply");
        assert_eq!(touched, vec!["card-a".to_string()]);

        let snapshot = store.load_work_snapshot("proj").expect("snapshot");
        assert_eq!(snapshot.project_id, "proj");
        assert_eq!(snapshot.cards.len(), 1);
        let card = &snapshot.cards[0];
        assert_eq!(card.slug, "a");
        assert_eq!(card.decisions, vec!["Decision-0001".to_string()]);
        assert_eq!(card.tasks.len(), 1);
        assert_eq!(card.tasks[0].title, "Wire the store");
        assert_eq!(card.questions.len(), 1);
        assert!(card.questions[0].blocking);
        assert_eq!(card.questions[0].options, vec!["sqlite", "postgres"]);
        assert_eq!(card.handoffs.len(), 1);
        assert_eq!(card.handoffs[0].touched, vec!["work.go".to_string()]);
    }

    #[test]
    fn empty_milestone_is_stored_as_null() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", now());
        let mut card = card_op("card-a", "a", work::STAGE_SHAPE);
        card.fields.insert("milestone".to_string(), json!(""));
        store
            .apply_work_ops("proj", "", "batch-1", &[card])
            .expect("apply");
        let stored: Option<String> = store
            .db
            .query_row(
                "SELECT milestone_id FROM work_cards WHERE id='card-a'",
                [],
                |row| row.get(0),
            )
            .expect("milestone");
        assert_eq!(stored, None, "an empty milestone must be the Backlog");
    }

    #[test]
    fn unknown_field_and_record_are_rejected() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", now());
        let mut card = card_op("card-a", "a", work::STAGE_SHAPE);
        card.fields.insert("nope".to_string(), json!("x"));
        let err = store
            .apply_work_ops("proj", "", "batch-1", &[card])
            .expect_err("unknown field");
        assert_eq!(err.to_string(), "create card a: unknown field \"nope\"");

        let update = work::Op {
            entity: work::ENTITY_CARD.to_string(),
            kind: work::OP_UPDATE.to_string(),
            entity_id: "missing".to_string(),
            card_id: "missing".to_string(),
            card_slug: "missing".to_string(),
            fields: fields(&[("title", json!("Title"))]),
            ..work::Op::default()
        };
        let err = store
            .apply_work_ops("proj", "", "batch-2", &[update])
            .expect_err("missing record");
        assert_eq!(
            err.to_string(),
            "update card missing: record missing not found"
        );
        let ops: i64 = store
            .db
            .query_row("SELECT COUNT(*) FROM change_ops", [], |row| row.get(0))
            .expect("ops");
        assert_eq!(ops, 0, "a failed batch records nothing");
    }

    #[test]
    fn overlapping_claims_are_rejected() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", now());
        let card = card_op("card-a", "a", work::STAGE_BUILD);
        let task = |id: &str, number: i32| work::Op {
            entity: work::ENTITY_TASK.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: id.to_string(),
            card_id: "card-a".to_string(),
            card_slug: "a".to_string(),
            fields: fields(&[
                ("number", json!(number)),
                ("title", json!(id)),
                ("state", json!(work::TASK_OPEN)),
            ]),
            ..work::Op::default()
        };
        store
            .apply_work_ops(
                "proj",
                "",
                "batch-0",
                &[card, task("task-1", 1), task("task-2", 2)],
            )
            .expect("seed tasks");
        let claim = |id: &str, task: &str, holder: &str| work::Op {
            entity: work::ENTITY_CLAIM.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: id.to_string(),
            card_id: "card-a".to_string(),
            card_slug: "a".to_string(),
            fields: fields(&[
                ("task_id", json!(task)),
                ("holder", json!(holder)),
                ("scope", json!("src/*")),
            ]),
            ..work::Op::default()
        };
        store
            .apply_work_ops(
                "proj",
                "",
                "batch-1",
                &[claim("claim-1", "task-1", "codex")],
            )
            .expect("first claim");
        let err = store
            .apply_work_ops(
                "proj",
                "",
                "batch-2",
                &[claim("claim-2", "task-2", "claude")],
            )
            .expect_err("overlapping scope");
        assert_eq!(
            err.to_string(),
            "create claim a: scope src/* overlaps src/* held by codex"
        );
    }

    #[test]
    fn structure_change_ops_are_listed() {
        let holder = TempStore::new();
        let store = &holder.store;
        save_project(store, "proj", now());
        let op = work::Op {
            entity: work::ENTITY_NODE.to_string(),
            kind: work::OP_CREATE.to_string(),
            entity_id: "node-1".to_string(),
            card_slug: "app".to_string(),
            node_id: "node-1".to_string(),
            fields: fields(&[
                ("level", json!("component")),
                ("slug", json!("app")),
                ("path", json!("app")),
                ("title", json!("App")),
                ("intent", json!("")),
                ("status", json!("active")),
                ("decisions", json!([])),
                ("position", json!(0)),
                ("repo", json!("")),
                ("folder", json!("")),
                ("kind", json!("service")),
                ("tech", json!("Rust")),
                ("migration_from", json!("")),
                ("migration_to", json!("")),
            ]),
            ..work::Op::default()
        };
        store
            .apply_work_ops("proj", "", "batch-1", &[op])
            .expect("apply node");
        let ops = store.list_structure_change_ops("proj").expect("ops");
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].entity_id, "node-1");
        assert_eq!(ops[0].kind, work::OP_CREATE);
        assert_eq!(ops[0].fields.get("path"), Some(&json!("app")));
    }
}
