//! Agent tool calls, stored.
//!
//! Agent tool calls: the audit row each MCP or Desktop tool call appends and
//! the list the app reads back.

use rusqlite::params;
use time::OffsetDateTime;

use crate::domains::agentaccess;

use super::*;

impl super::Store {
    pub fn append_agent_tool_call(&self, mut call: agentaccess::ToolCall) -> Result<()> {
        if call.id.is_empty() {
            call.id = agentaccess::new_call_id();
        }
        if call.created_at == OffsetDateTime::UNIX_EPOCH {
            call.created_at = OffsetDateTime::now_utc();
        }
        if call.source.is_empty() {
            call.source = "mcp".to_string();
        }
        if call.arguments_json.is_empty() {
            call.arguments_json = "{}".to_string();
        }
        let ok = i64::from(call.ok);
        self.db.execute(
            r#"INSERT INTO agent_tool_calls(id,project_id,agent_run_id,tool_name,arguments_json,result_summary,ok,source,created_at)
		 VALUES(?,?,?,?,?,?,?,?,?)"#,
            params![
                &call.id,
                &call.project_id,
                &call.agent_run_id,
                &call.tool_name,
                &call.arguments_json,
                &call.result_summary,
                ok,
                &call.source,
                format_rfc3339_zone(call.created_at),
            ],
        )?;
        Ok(())
    }

    pub fn list_agent_tool_calls(&self, limit: i64) -> Result<Vec<agentaccess::ToolCall>> {
        let limit = if limit <= 0 || limit > 200 { 50 } else { limit };
        let mut stmt = self.db.prepare(
            r#"SELECT id,project_id,COALESCE(agent_run_id,''),tool_name,arguments_json,result_summary,ok,source,created_at
		 FROM agent_tool_calls ORDER BY created_at DESC LIMIT ?"#,
        )?;
        let mut rows = stmt.query(params![limit])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            let ok_flag: i64 = row.get(6)?;
            let created: String = row.get(8)?;
            out.push(agentaccess::ToolCall {
                id: row.get(0)?,
                project_id: row.get(1)?,
                agent_run_id: row.get(2)?,
                tool_name: row.get(3)?,
                arguments_json: row.get(4)?,
                result_summary: row.get(5)?,
                ok: ok_flag != 0,
                source: row.get(7)?,
                created_at: parse_rfc3339(&created),
            });
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

    use crate::providers::sqlite::test_support::{TempStore, save_project};

    use super::*;

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
            .replace_nanosecond(0)
            .expect("second precision")
    }

    #[test]
    fn tool_calls_round_trip_with_their_defaults() {
        let holder = TempStore::new();
        let store = &holder.store;
        let at = now();
        save_project(store, "proj-a", at);

        store
            .append_agent_tool_call(agentaccess::ToolCall {
                project_id: "proj-a".to_string(),
                tool_name: "polter.query".to_string(),
                created_at: at,
                ..agentaccess::ToolCall::default()
            })
            .expect("first call");
        store
            .append_agent_tool_call(agentaccess::ToolCall {
                id: "acall-explicit".to_string(),
                project_id: "proj-a".to_string(),
                agent_run_id: "arun-1".to_string(),
                tool_name: "polter.apply".to_string(),
                arguments_json: r#"{"text":"card x"}"#.to_string(),
                result_summary: "applied".to_string(),
                ok: true,
                source: "desktop".to_string(),
                created_at: at + time::Duration::seconds(1),
            })
            .expect("second call");

        let calls = store.list_agent_tool_calls(50).expect("calls");
        assert_eq!(calls.len(), 2, "calls = {calls:?}");
        assert_eq!(calls[0].id, "acall-explicit");
        assert_eq!(calls[0].agent_run_id, "arun-1");
        assert_eq!(calls[0].arguments_json, r#"{"text":"card x"}"#);
        assert!(calls[0].ok);
        assert_eq!(calls[0].source, "desktop");

        let defaulted = &calls[1];
        assert!(defaulted.id.starts_with("acall-"), "{defaulted:?}");
        assert_eq!(defaulted.source, "mcp");
        assert_eq!(defaulted.arguments_json, "{}");
        assert!(!defaulted.ok);
        assert_eq!(defaulted.agent_run_id, "");
        assert_eq!(defaulted.created_at, at);

        let limited = store.list_agent_tool_calls(1).expect("limited");
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].id, "acall-explicit");
    }
}
