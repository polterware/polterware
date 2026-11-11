-- Slice 3a: relational Work model (cards, tasks, questions, invariants) written
-- through the Polter codec, plus the append-only change operation log.

CREATE TABLE IF NOT EXISTS work_cards (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  slug TEXT NOT NULL,
  title TEXT NOT NULL,
  intent TEXT NOT NULL DEFAULT '',
  stage TEXT NOT NULL DEFAULT 'shape'
    CHECK (stage IN ('shape', 'plan', 'build', 'verify', 'delivered')),
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'removed')),
  owner TEXT NOT NULL DEFAULT '',
  under TEXT NOT NULL DEFAULT '',
  decisions_json TEXT NOT NULL DEFAULT '[]',
  position INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (project_id, slug)
);
CREATE INDEX IF NOT EXISTS idx_work_cards_project ON work_cards(project_id, position);

CREATE TABLE IF NOT EXISTS work_tasks (
  id TEXT PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES work_cards(id) ON DELETE CASCADE,
  number INTEGER NOT NULL,
  title TEXT NOT NULL,
  state TEXT NOT NULL DEFAULT 'open' CHECK (state IN ('open', 'done', 'dropped')),
  scope TEXT NOT NULL DEFAULT '',
  position INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (card_id, number)
);
CREATE INDEX IF NOT EXISTS idx_work_tasks_card ON work_tasks(card_id, position);

CREATE TABLE IF NOT EXISTS work_questions (
  id TEXT PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES work_cards(id) ON DELETE CASCADE,
  text TEXT NOT NULL,
  blocking INTEGER NOT NULL DEFAULT 0,
  options_json TEXT NOT NULL DEFAULT '[]',
  state TEXT NOT NULL DEFAULT 'open' CHECK (state IN ('open', 'answered', 'removed')),
  answer TEXT NOT NULL DEFAULT '',
  position INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_work_questions_card ON work_questions(card_id, position);

CREATE TABLE IF NOT EXISTS work_invariants (
  id TEXT PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES work_cards(id) ON DELETE CASCADE,
  text TEXT NOT NULL,
  state TEXT NOT NULL DEFAULT 'active' CHECK (state IN ('active', 'removed')),
  position INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_work_invariants_card ON work_invariants(card_id, position);

-- Append-only operation log: one row per entity mutation, grouped by batch
-- (one apply call). This is the audit trail and the future Cloud sync contract.
CREATE TABLE IF NOT EXISTS change_ops (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  agent_run_id TEXT NOT NULL DEFAULT '',
  batch_id TEXT NOT NULL,
  entity TEXT NOT NULL CHECK (entity IN ('card', 'task', 'question', 'invariant')),
  entity_id TEXT NOT NULL,
  op TEXT NOT NULL CHECK (op IN ('create', 'update', 'remove')),
  data_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_change_ops_project ON change_ops(project_id, id);
CREATE INDEX IF NOT EXISTS idx_change_ops_batch ON change_ops(batch_id);
CREATE INDEX IF NOT EXISTS idx_change_ops_entity ON change_ops(entity_id);
