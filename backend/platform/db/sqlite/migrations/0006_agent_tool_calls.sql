-- Local MCP / agent tool-call history for Settings → Agents.

CREATE TABLE IF NOT EXISTS agent_tool_calls (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL DEFAULT '',
  tool_name TEXT NOT NULL,
  arguments_json TEXT NOT NULL DEFAULT '{}',
  result_summary TEXT NOT NULL DEFAULT '',
  ok INTEGER NOT NULL DEFAULT 1,
  source TEXT NOT NULL DEFAULT 'mcp',
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_agent_tool_calls_created ON agent_tool_calls(created_at DESC);
