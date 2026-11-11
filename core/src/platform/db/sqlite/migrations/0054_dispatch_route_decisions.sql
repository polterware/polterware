-- How each new dispatch run's candidates were ordered: the policy, the
-- classifier features, and every candidate with the policy's pass estimate.
-- It holds no prompt. Calibration joins it with the run's Verify verdict.

CREATE TABLE IF NOT EXISTS dispatch_route_decisions (
  run_id TEXT PRIMARY KEY REFERENCES dispatch_runs(id) ON DELETE CASCADE,
  project_id TEXT NOT NULL DEFAULT '',
  policy_version TEXT NOT NULL,
  route TEXT NOT NULL,
  priority TEXT NOT NULL DEFAULT '',
  feature_schema INTEGER NOT NULL DEFAULT 0,
  features_json TEXT NOT NULL DEFAULT '[]',
  difficulty REAL,
  candidates_json TEXT NOT NULL DEFAULT '[]',
  chosen INTEGER NOT NULL DEFAULT 0,
  propensity REAL NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_dispatch_route_decisions_project_created
  ON dispatch_route_decisions(project_id, created_at DESC);
