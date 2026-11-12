-- Migration 0060: card confirmations.
-- One row per acceptance criterion or Rule a person or an agent confirmed for a
-- card. verify lists a confirmed item as pass and the rest as manual, and an
-- unconfirmed card in the verify stage raises an Inbox item.

CREATE TABLE card_confirmations (
  card_id TEXT NOT NULL REFERENCES work_cards(id) ON DELETE CASCADE,
  kind TEXT NOT NULL CHECK (kind IN ('invariant', 'rule')),
  target_id TEXT NOT NULL,
  confirmed_by TEXT NOT NULL DEFAULT '',
  confirmed_at TEXT NOT NULL,
  PRIMARY KEY (card_id, kind, target_id)
);
CREATE INDEX idx_card_confirmations_card ON card_confirmations(card_id);

-- change_ops records confirmation writes too.
CREATE TABLE change_ops_new (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  agent_run_id TEXT NOT NULL DEFAULT '',
  batch_id TEXT NOT NULL,
  entity TEXT NOT NULL CHECK (entity IN ('card', 'task', 'question', 'invariant', 'handoff', 'claim', 'node', 'decision', 'canvas', 'milestone', 'fact', 'rule', 'conflict', 'confirmation')),
  entity_id TEXT NOT NULL,
  op TEXT NOT NULL CHECK (op IN ('create', 'update', 'remove')),
  data_json TEXT NOT NULL DEFAULT '{}',
  node_id TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL
);
INSERT INTO change_ops_new(id, project_id, agent_run_id, batch_id, entity, entity_id, op, data_json, node_id, created_at)
  SELECT id, project_id, agent_run_id, batch_id, entity, entity_id, op, data_json, node_id, created_at FROM change_ops;
DROP TABLE change_ops;
ALTER TABLE change_ops_new RENAME TO change_ops;
CREATE INDEX IF NOT EXISTS idx_change_ops_project ON change_ops(project_id, id);
CREATE INDEX IF NOT EXISTS idx_change_ops_batch ON change_ops(batch_id);
CREATE INDEX IF NOT EXISTS idx_change_ops_entity ON change_ops(entity_id);
