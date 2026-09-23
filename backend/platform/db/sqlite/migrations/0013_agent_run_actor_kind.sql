-- B9: attribute AgentRuns to human vs agent for Unified Change History.

ALTER TABLE agent_runs ADD COLUMN actor_kind TEXT NOT NULL DEFAULT 'agent';

CREATE INDEX IF NOT EXISTS idx_agent_runs_project_actor_created
  ON agent_runs(project_id, actor_kind, created_at DESC);
