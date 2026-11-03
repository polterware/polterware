-- A deploy links to the platform's own project, app, or service: its stable id
-- and its dashboard page. The latest deploy check per link is kept apart, so a
-- link edit never rewrites it.

ALTER TABLE node_deployments ADD COLUMN target_id TEXT NOT NULL DEFAULT '';
ALTER TABLE node_deployments ADD COLUMN target_url TEXT NOT NULL DEFAULT '';

CREATE TABLE IF NOT EXISTS deploy_checks (
  deployment_id TEXT PRIMARY KEY REFERENCES node_deployments(id) ON DELETE CASCADE,
  state TEXT NOT NULL,
  summary TEXT NOT NULL DEFAULT '',
  url TEXT NOT NULL DEFAULT '',
  checked_at TEXT NOT NULL
);
