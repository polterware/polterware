-- Slice 4: handoff records keyed by Work card and Git branch. Rows are
-- append-only; the latest row for a card and branch is the current handoff.
-- change_ops is rebuilt so its entity check accepts 'handoff' (SQLite cannot
-- alter a CHECK constraint).

CREATE TABLE IF NOT EXISTS work_handoffs (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  card_id TEXT NOT NULL REFERENCES work_cards(id) ON DELETE CASCADE,
  branch TEXT NOT NULL DEFAULT '',
  goal TEXT NOT NULL DEFAULT '',
  done TEXT NOT NULL DEFAULT '',
  in_progress TEXT NOT NULL DEFAULT '',
  next TEXT NOT NULL DEFAULT '',
  touched_json TEXT NOT NULL DEFAULT '[]',
  agent TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_work_handoffs_card ON work_handoffs(card_id, created_at);

CREATE TABLE change_ops_new (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  agent_run_id TEXT NOT NULL DEFAULT '',
  batch_id TEXT NOT NULL,
  entity TEXT NOT NULL CHECK (entity IN ('card', 'task', 'question', 'invariant', 'handoff')),
  entity_id TEXT NOT NULL,
  op TEXT NOT NULL CHECK (op IN ('create', 'update', 'remove')),
  data_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL
);
INSERT INTO change_ops_new(id, project_id, agent_run_id, batch_id, entity, entity_id, op, data_json, created_at)
  SELECT id, project_id, agent_run_id, batch_id, entity, entity_id, op, data_json, created_at FROM change_ops;
DROP TABLE change_ops;
ALTER TABLE change_ops_new RENAME TO change_ops;
CREATE INDEX IF NOT EXISTS idx_change_ops_project ON change_ops(project_id, id);
CREATE INDEX IF NOT EXISTS idx_change_ops_batch ON change_ops(batch_id);
CREATE INDEX IF NOT EXISTS idx_change_ops_entity ON change_ops(entity_id);
