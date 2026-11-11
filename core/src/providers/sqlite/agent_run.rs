//! Port of `backend/providers/sqlite/agent_run.go`.
//!
//! AgentRuns: their headers, the ordered steps that reconstruct a run, and the
//! justified change history (B9) the Unified Change History reads.

use std::result::Result as StdResult;

use rusqlite::Row;
use rusqlite::params;
use rusqlite::params_from_iter;
use rusqlite::types::Value;
use time::OffsetDateTime;
use time::UtcOffset;
use time::format_description::well_known::Rfc3339;

use crate::domains::agentrun;

use super::*;

impl super::Store {
    /// Inserts a run, filling in its id, creation time, status, and actor kind.
    pub fn create_agent_run(&self, mut run: agentrun::Run) -> Result<()> {
        if run.id.is_empty() {
            run.id = agentrun::new_run_id();
        }
        if run.created_at == OffsetDateTime::UNIX_EPOCH {
            run.created_at = OffsetDateTime::now_utc();
        }
        if run.status.is_empty() {
            run.status = agentrun::STATUS_OPEN.to_string();
        }
        if run.actor_kind.is_empty() {
            run.actor_kind = agentrun::ACTOR_KIND_AGENT.to_string();
        }
        let completed = run.completed_at.map(format_rfc3339);
        self.db.execute(
            r#"INSERT INTO agent_runs(id,project_id,task,agent_label,actor_kind,status,created_at,completed_at)
		 VALUES(?,?,?,?,?,?,?,?)"#,
            params![
                &run.id,
                &run.project_id,
                &run.task,
                &run.agent_label,
                &run.actor_kind,
                &run.status,
                format_rfc3339(run.created_at),
                completed,
            ],
        )?;
        Ok(())
    }

    pub fn get_agent_run(&self, id: &str) -> Result<agentrun::Run> {
        let found = self.db.query_row(
            r#"SELECT id,project_id,task,agent_label,actor_kind,status,created_at,completed_at
		 FROM agent_runs WHERE id=?"#,
            params![id],
            scan_run,
        );
        match found {
            Ok(run) => Ok(run),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(run_error(agentrun::Error::UnknownRun))
            }
            Err(err) => Err(err.into()),
        }
    }

    pub fn update_agent_run(&self, run: agentrun::Run) -> Result<()> {
        let completed = run.completed_at.map(format_rfc3339);
        let updated = self.db.execute(
            r#"UPDATE agent_runs SET task=?, agent_label=?, actor_kind=?, status=?, completed_at=? WHERE id=?"#,
            params![
                &run.task,
                &run.agent_label,
                &run.actor_kind,
                &run.status,
                completed,
                &run.id,
            ],
        )?;
        if updated == 0 {
            return Err(run_error(agentrun::Error::UnknownRun));
        }
        Ok(())
    }

    pub fn append_agent_run_step(&self, mut step: agentrun::Step) -> Result<()> {
        if step.id.is_empty() {
            step.id = agentrun::new_step_id();
        }
        if step.created_at == OffsetDateTime::UNIX_EPOCH {
            step.created_at = OffsetDateTime::now_utc();
        }
        if step.payload_json.is_empty() {
            step.payload_json = "{}".to_string();
        }
        self.db.execute(
            r#"INSERT INTO agent_run_steps(id,agent_run_id,kind,summary,payload_json,created_at)
		 VALUES(?,?,?,?,?,?)"#,
            params![
                &step.id,
                &step.agent_run_id,
                &step.kind,
                &step.summary,
                &step.payload_json,
                format_rfc3339_nano(step.created_at),
            ],
        )?;
        Ok(())
    }

    pub fn list_agent_run_steps(&self, run_id: &str) -> Result<Vec<agentrun::Step>> {
        let mut stmt = self.db.prepare(
            r#"SELECT id,agent_run_id,kind,summary,payload_json,created_at
		 FROM agent_run_steps WHERE agent_run_id=? ORDER BY created_at ASC, rowid ASC"#,
        )?;
        let mut rows = stmt.query(params![run_id])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(agentrun::Step {
                id: row.get(0)?,
                agent_run_id: row.get(1)?,
                kind: row.get(2)?,
                summary: row.get(3)?,
                payload_json: row.get(4)?,
                created_at: parse_agent_run_time(&row.get::<_, String>(5)?),
            });
        }
        Ok(out)
    }

    pub fn get_agent_run_record(&self, id: &str) -> Result<agentrun::Record> {
        let run = self.get_agent_run(id)?;
        let steps = self.list_agent_run_steps(id)?;
        Ok(agentrun::Record { run, steps })
    }

    pub fn list_agent_runs(&self, project_id: &str, limit: i64) -> Result<Vec<agentrun::Run>> {
        let limit = if limit <= 0 || limit > 200 { 50 } else { limit };
        let mut stmt = self.db.prepare(
            r#"SELECT id,project_id,task,agent_label,actor_kind,status,created_at,completed_at
		 FROM agent_runs WHERE project_id=? ORDER BY created_at DESC LIMIT ?"#,
        )?;
        let mut rows = stmt.query(params![project_id, limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(scan_run(row)?);
        }
        Ok(out)
    }

    /// Returns change steps for a project, newest first, with an optional actor
    /// filter.
    pub fn list_changes(
        &self,
        project_id: &str,
        actor_kind: &str,
        limit: i64,
    ) -> Result<Vec<ChangeRow>> {
        self.query_changes(project_id, actor_kind, "", change_limit(limit))
    }

    /// Returns every change step for a project, newest first.
    pub fn list_all_changes(&self, project_id: &str, actor_kind: &str) -> Result<Vec<ChangeRow>> {
        self.query_changes(project_id, actor_kind, "", 0)
    }

    /// Returns the change steps that name `subject_id` among their subjects,
    /// newest first: the history of one card, Decision, doc, fact, or node.
    pub fn list_subject_changes(
        &self,
        project_id: &str,
        actor_kind: &str,
        subject_id: &str,
        limit: i64,
    ) -> Result<Vec<ChangeRow>> {
        self.query_changes(project_id, actor_kind, subject_id, change_limit(limit))
    }

    fn query_changes(
        &self,
        project_id: &str,
        actor_kind: &str,
        subject_id: &str,
        limit: i64,
    ) -> Result<Vec<ChangeRow>> {
        let mut query = String::from(
            r#"
		SELECT s.id, s.agent_run_id, r.project_id, r.actor_kind, r.agent_label, r.status, r.task,
		       s.summary, s.payload_json, s.created_at
		FROM agent_run_steps s
		INNER JOIN agent_runs r ON r.id = s.agent_run_id
		WHERE r.project_id = ? AND s.kind = 'change'"#,
        );
        let mut args: Vec<Value> = vec![Value::Text(project_id.to_string())];
        for pattern in HISTORY_HIDDEN_TOOLS {
            query.push_str(" AND COALESCE(json_extract(s.payload_json, '$.tool'), '') NOT LIKE ?");
            args.push(Value::Text(pattern.to_string()));
        }
        if !actor_kind.is_empty() {
            query.push_str(" AND r.actor_kind = ?");
            args.push(Value::Text(actor_kind.to_string()));
        }
        if !subject_id.is_empty() {
            query.push_str(
                " AND EXISTS (SELECT 1 FROM json_each(s.payload_json, '$.subject_ids') WHERE value = ?)",
            );
            args.push(Value::Text(subject_id.to_string()));
        }
        query.push_str(" ORDER BY s.created_at DESC, s.rowid DESC");
        if limit > 0 {
            query.push_str(" LIMIT ?");
            args.push(Value::Integer(limit));
        }

        let mut stmt = self.db.prepare(&query)?;
        let mut rows = stmt.query(params_from_iter(args))?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let mut actor_kind: String = row.get(3)?;
            if actor_kind.is_empty() {
                actor_kind = agentrun::ACTOR_KIND_AGENT.to_string();
            }
            out.push(ChangeRow {
                step_id: row.get(0)?,
                agent_run_id: row.get(1)?,
                project_id: row.get(2)?,
                actor_kind,
                agent_label: row.get(4)?,
                run_status: row.get(5)?,
                run_task: row.get(6)?,
                summary: row.get(7)?,
                payload_json: row.get(8)?,
                created_at: parse_agent_run_time(&row.get::<_, String>(9)?),
            });
        }
        Ok(out)
    }

    /// Loads the run, ensures it is open, and appends the step `build` returns.
    pub fn append_agent_run_step_if_open(
        &self,
        run_id: &str,
        build: impl FnOnce(&agentrun::Run) -> StdResult<agentrun::Step, agentrun::Error>,
    ) -> Result<()> {
        let run = self.get_agent_run(run_id)?;
        agentrun::can_append(&run).map_err(run_error)?;
        let step = build(&run).map_err(run_error)?;
        self.append_agent_run_step(step)
    }
}

/// One justified mutation for Unified Change History (B9).
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeRow {
    pub step_id: String,
    pub agent_run_id: String,
    pub project_id: String,
    pub actor_kind: String,
    pub agent_label: String,
    pub run_status: String,
    pub run_task: String,
    pub summary: String,
    pub payload_json: String,
    pub created_at: OffsetDateTime,
}

/// Clamps a history page size. Non-positive and oversized limits fall back to
/// 50, matching [`super::Store::list_changes`].
pub fn change_limit(limit: i64) -> i64 {
    if limit <= 0 || limit > 200 {
        return 50;
    }
    limit
}

/// Write tools whose change steps stay audited (`change_ops` keep the run) but
/// are left out of the history: canvas edits are workspace layout, not project
/// truth, and every autosave would add a row.
const HISTORY_HIDDEN_TOOLS: [&str; 1] = ["desktop.canvas.%"];

/// Reads a run row in `SELECT id,project_id,task,agent_label,actor_kind,status,
/// created_at,completed_at` order, defaulting an empty actor kind to agent.
fn scan_run(row: &Row<'_>) -> rusqlite::Result<agentrun::Run> {
    let created: String = row.get(6)?;
    let completed: Option<String> = row.get(7)?;
    let mut actor_kind: String = row.get(4)?;
    if actor_kind.is_empty() {
        actor_kind = agentrun::ACTOR_KIND_AGENT.to_string();
    }
    Ok(agentrun::Run {
        id: row.get(0)?,
        project_id: row.get(1)?,
        task: row.get(2)?,
        agent_label: row.get(3)?,
        actor_kind,
        status: row.get(5)?,
        created_at: parse_rfc3339(&created),
        completed_at: parse_optional_rfc3339(&completed.unwrap_or_default()),
    })
}

/// Reads a step's stamp: an empty value is Go's zero time, which the port
/// keeps as the Unix epoch.
fn parse_agent_run_time(value: &str) -> OffsetDateTime {
    if value.is_empty() {
        return OffsetDateTime::UNIX_EPOCH;
    }
    parse_rfc3339(value)
}

/// Formats a stamp the way Go's `time.RFC3339Nano` layout does: UTC, with the
/// fraction trimmed of trailing zeros (absent when there is none).
///
/// TODO(dedupe): private copy; the foundation's `format_rfc3339` drops the
/// fraction, and a step's order depends on it.
fn format_rfc3339_nano(value: OffsetDateTime) -> String {
    value
        .to_offset(UtcOffset::UTC)
        .format(&Rfc3339)
        .expect("RFC3339 formatting cannot fail")
}

/// Maps an agent run error to the store's error, keeping Go's message.
fn run_error(err: agentrun::Error) -> Error {
    Error::message(err.to_string())
}

#[cfg(test)]
mod tests {
    use crate::providers::sqlite::test_support::{TempStore, save_project};

    use super::*;

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    #[test]
    fn agent_run_lifecycle() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "proj-a", at);

        let run = agentrun::start("proj-a", "Ship the build", "codex", Some(at)).expect("start");
        store.create_agent_run(run.clone()).expect("create");

        let got = store.get_agent_run(&run.id).expect("get");
        assert_eq!(got.status, agentrun::STATUS_OPEN);
        assert_eq!(got.task, "Ship the build");
        assert_eq!(got.agent_label, "codex");
        assert!(got.completed_at.is_none(), "run = {got:?}");

        let done =
            agentrun::complete(got, Some(at + time::Duration::seconds(1))).expect("complete");
        store.update_agent_run(done).expect("update");
        let got = store.get_agent_run(&run.id).expect("completed run");
        assert_eq!(got.status, agentrun::STATUS_COMPLETED);
        assert!(got.completed_at.is_some(), "completed run = {got:?}");
        assert!(
            agentrun::complete(got, Some(at)).is_err(),
            "a completed run must not complete again"
        );
    }

    #[test]
    fn agent_run_list_is_scoped_to_project() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "proj-a", at);
        save_project(store, "proj-empty", at);

        let before = store.list_agent_runs("proj-a", 50).expect("before");
        let run = agentrun::start("proj-a", "One", "claude", Some(at)).expect("start");
        store.create_agent_run(run).expect("create");
        let after = store.list_agent_runs("proj-a", 50).expect("after");
        assert_eq!(after.len(), before.len() + 1);

        let empty = store.list_agent_runs("proj-empty", 50).expect("other");
        assert!(
            empty.is_empty(),
            "other project must see no runs: {empty:?}"
        );
        let limited = store.list_agent_runs("proj-a", 1).expect("limited");
        assert!(limited.len() <= 1, "limit ignored: {} rows", limited.len());
    }

    #[test]
    fn agent_run_steps_are_ordered_and_validated() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "proj-a", at);
        let run = agentrun::start("proj-a", "Audit", "codex", Some(at)).expect("start");
        store.create_agent_run(run.clone()).expect("create");

        let base = now();
        for (i, kind) in [
            agentrun::KIND_CONTEXT,
            agentrun::KIND_CHANGE,
            agentrun::KIND_CONTEXT,
        ]
        .into_iter()
        .enumerate()
        {
            let step = agentrun::Step {
                agent_run_id: run.id.clone(),
                kind: kind.to_string(),
                summary: kind.to_string(),
                created_at: base + time::Duration::seconds(i as i64),
                ..agentrun::Step::default()
            };
            store.append_agent_run_step(step).expect("append");
        }

        let steps = store.list_agent_run_steps(&run.id).expect("steps");
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].kind, agentrun::KIND_CONTEXT);
        assert_eq!(steps[2].kind, agentrun::KIND_CONTEXT);
        assert_eq!(
            steps[0].payload_json, "{}",
            "empty payload must default to {{}}, got {:?}",
            steps[0].payload_json
        );

        let record = store.get_agent_run_record(&run.id).expect("record");
        assert_eq!(record.steps.len(), 3);
        assert_eq!(record.run.id, run.id);

        assert!(
            store
                .append_agent_run_step(agentrun::Step {
                    agent_run_id: run.id.clone(),
                    kind: "bogus".to_string(),
                    ..agentrun::Step::default()
                })
                .is_err(),
            "unknown step kind must be rejected"
        );
        assert!(
            store
                .append_agent_run_step(agentrun::Step {
                    agent_run_id: "missing".to_string(),
                    kind: agentrun::KIND_CHANGE.to_string(),
                    ..agentrun::Step::default()
                })
                .is_err(),
            "a step for an unknown run must be rejected"
        );
    }

    #[test]
    fn get_agent_run_unknown() {
        let holder = TempStore::new();
        let err = holder
            .store
            .get_agent_run("missing")
            .expect_err("want error for an unknown run");
        assert_eq!(err.to_string(), "unknown agent run");
    }
}
