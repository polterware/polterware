-- Minimal AgentRun audit (wedge B6): context → task → change → evidence → drift.

CREATE TABLE IF NOT EXISTS agent_runs (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  task TEXT NOT NULL,
  agent_label TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL CHECK (status IN ('open', 'completed')),
  created_at TEXT NOT NULL,
  completed_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_agent_runs_project_created
  ON agent_runs(project_id, created_at DESC);

CREATE TABLE IF NOT EXISTS agent_run_steps (
  id TEXT PRIMARY KEY,
  agent_run_id TEXT NOT NULL REFERENCES agent_runs(id) ON DELETE CASCADE,
  kind TEXT NOT NULL CHECK (kind IN ('context', 'change', 'evidence', 'drift')),
  summary TEXT NOT NULL DEFAULT '',
  payload_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_agent_run_steps_run_created
  ON agent_run_steps(agent_run_id, created_at ASC);

ALTER TABLE agent_tool_calls ADD COLUMN agent_run_id TEXT NOT NULL DEFAULT '';
CREATE INDEX IF NOT EXISTS idx_agent_tool_calls_run ON agent_tool_calls(agent_run_id);
