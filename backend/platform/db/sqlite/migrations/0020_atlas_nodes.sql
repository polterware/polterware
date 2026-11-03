-- Slice 8: Atlas nodes above Work cards (system, domain, component).
-- Features stay in work_cards. Status is derived on read, not stored.
-- change_ops is rebuilt so its entity check accepts 'node'.

CREATE TABLE IF NOT EXISTS atlas_nodes (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  parent_id TEXT REFERENCES atlas_nodes(id) ON DELETE CASCADE,
  level TEXT NOT NULL CHECK (level IN ('system', 'domain', 'component')),
  slug TEXT NOT NULL,
  path TEXT NOT NULL,
  title TEXT NOT NULL,
  intent TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'removed')),
  decisions_json TEXT NOT NULL DEFAULT '[]',
  position INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (project_id, path)
);
CREATE INDEX IF NOT EXISTS idx_atlas_nodes_project ON atlas_nodes(project_id, position);
CREATE INDEX IF NOT EXISTS idx_atlas_nodes_parent ON atlas_nodes(parent_id);

CREATE TABLE IF NOT EXISTS atlas_invariants (
  id TEXT PRIMARY KEY,
  node_id TEXT NOT NULL REFERENCES atlas_nodes(id) ON DELETE CASCADE,
  text TEXT NOT NULL,
  state TEXT NOT NULL DEFAULT 'active' CHECK (state IN ('active', 'removed')),
  position INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_atlas_invariants_node ON atlas_invariants(node_id, position);

CREATE TABLE change_ops_new (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  agent_run_id TEXT NOT NULL DEFAULT '',
  batch_id TEXT NOT NULL,
  entity TEXT NOT NULL CHECK (entity IN ('card', 'task', 'question', 'invariant', 'handoff', 'claim', 'node')),
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
