CREATE TABLE IF NOT EXISTS projects (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  root_path TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'active',
  archived_at TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS environments (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  kind TEXT NOT NULL,
  protection TEXT NOT NULL DEFAULT 'standard',
  is_default INTEGER NOT NULL DEFAULT 0,
  description TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS project_settings (
  project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
  description TEXT NOT NULL DEFAULT '',
  default_environment_id TEXT REFERENCES environments(id) ON DELETE SET NULL,
  repository TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS resources (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  environment_id TEXT REFERENCES environments(id) ON DELETE SET NULL,
  name TEXT NOT NULL,
  type TEXT NOT NULL,
  external_key TEXT,
  source TEXT NOT NULL,
  provider TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'unknown',
  region TEXT NOT NULL DEFAULT '',
  address TEXT NOT NULL DEFAULT '',
  tags_json TEXT NOT NULL DEFAULT '[]',
  description TEXT NOT NULL DEFAULT ''
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_resources_external
  ON resources(project_id, external_key) WHERE external_key IS NOT NULL AND external_key != '';

CREATE TABLE IF NOT EXISTS nodes (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  environment_id TEXT REFERENCES environments(id) ON DELETE SET NULL,
  name TEXT NOT NULL,
  platform TEXT NOT NULL DEFAULT '',
  shell TEXT NOT NULL DEFAULT 'bash',
  status TEXT NOT NULL DEFAULT 'unknown',
  address TEXT NOT NULL DEFAULT '',
  cpu TEXT NOT NULL DEFAULT '',
  memory TEXT NOT NULL DEFAULT '',
  last_seen_at TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_nodes_project ON nodes(project_id);

CREATE TABLE IF NOT EXISTS architecture_layouts (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL UNIQUE REFERENCES projects(id) ON DELETE CASCADE,
  revision INTEGER NOT NULL,
  body_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS architecture_views (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  owner TEXT NOT NULL DEFAULT '',
  purpose TEXT NOT NULL DEFAULT '',
  notes TEXT NOT NULL DEFAULT '',
  filter_summary TEXT NOT NULL DEFAULT '',
  environments_json TEXT NOT NULL DEFAULT '[]',
  tags_json TEXT NOT NULL DEFAULT '[]',
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_architecture_views_project ON architecture_views(project_id);

CREATE TABLE IF NOT EXISTS architecture_view_members (
  view_id TEXT NOT NULL REFERENCES architecture_views(id) ON DELETE CASCADE,
  node_id TEXT NOT NULL,
  name TEXT NOT NULL DEFAULT '',
  type TEXT NOT NULL DEFAULT '',
  layer TEXT NOT NULL DEFAULT '',
  PRIMARY KEY (view_id, node_id)
);

CREATE TABLE IF NOT EXISTS knowledge_items (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('doc', 'runbook', 'spec', 'plan', 'tasks')),
  tags_json TEXT NOT NULL DEFAULT '[]',
  summary TEXT NOT NULL DEFAULT '',
  owner TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT '',
  extra_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_knowledge_project_kind ON knowledge_items(project_id, kind);

CREATE TABLE IF NOT EXISTS operations (
  id TEXT PRIMARY KEY,
  correlation_id TEXT NOT NULL,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  environment_id TEXT REFERENCES environments(id) ON DELETE SET NULL,
  type TEXT NOT NULL,
  status TEXT NOT NULL,
  result_summary TEXT NOT NULL DEFAULT '',
  error_message TEXT NOT NULL DEFAULT '',
  params_json TEXT NOT NULL DEFAULT '{}',
  idempotency_key TEXT,
  requested_at TEXT NOT NULL,
  started_at TEXT,
  finished_at TEXT,
  title TEXT NOT NULL DEFAULT '',
  target TEXT NOT NULL DEFAULT '',
  command TEXT NOT NULL DEFAULT '',
  steps_json TEXT NOT NULL DEFAULT '[]'
);
CREATE INDEX IF NOT EXISTS idx_operations_project_requested ON operations(project_id, requested_at DESC);

CREATE TABLE IF NOT EXISTS workflows (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  status TEXT NOT NULL,
  steps_json TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  trigger TEXT NOT NULL DEFAULT 'manual',
  last_run_at TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS operation_schedules (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  environment_id TEXT REFERENCES environments(id) ON DELETE SET NULL,
  name TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL,
  workflow_id TEXT NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
  expression TEXT NOT NULL,
  timezone TEXT NOT NULL DEFAULT 'UTC',
  next_run_at TEXT NOT NULL DEFAULT '',
  last_run_at TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_operation_schedules_project ON operation_schedules(project_id);

CREATE TABLE IF NOT EXISTS integration_connections (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  provider_id TEXT NOT NULL,
  status TEXT NOT NULL,
  label TEXT NOT NULL,
  environment_id TEXT REFERENCES environments(id) ON DELETE SET NULL,
  last_sync_at TEXT NOT NULL DEFAULT '',
  summary TEXT NOT NULL DEFAULT '',
  cli_command TEXT NOT NULL DEFAULT '',
  sync_interval TEXT NOT NULL DEFAULT '',
  freshness TEXT NOT NULL DEFAULT '',
  error_message TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS integration_resources (
  id TEXT PRIMARY KEY,
  connection_id TEXT NOT NULL REFERENCES integration_connections(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  kind TEXT NOT NULL,
  external_id TEXT NOT NULL DEFAULT '',
  environment_id TEXT REFERENCES environments(id) ON DELETE SET NULL,
  status TEXT NOT NULL DEFAULT '',
  inventory_resource_id TEXT REFERENCES resources(id) ON DELETE SET NULL
);
CREATE INDEX IF NOT EXISTS idx_integration_resources_connection ON integration_resources(connection_id);

CREATE TABLE IF NOT EXISTS integration_sync_runs (
  id TEXT PRIMARY KEY,
  connection_id TEXT NOT NULL REFERENCES integration_connections(id) ON DELETE CASCADE,
  started_at TEXT NOT NULL,
  finished_at TEXT NOT NULL DEFAULT '',
  result TEXT NOT NULL,
  imported INTEGER NOT NULL DEFAULT 0,
  changed INTEGER NOT NULL DEFAULT 0,
  message TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_integration_sync_runs_connection ON integration_sync_runs(connection_id);

CREATE TABLE IF NOT EXISTS process_sessions (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  environment_id TEXT REFERENCES environments(id) ON DELETE SET NULL,
  command TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  name TEXT NOT NULL DEFAULT '',
  node_id TEXT REFERENCES nodes(id) ON DELETE SET NULL,
  pid INTEGER,
  started_at TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS log_chunks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  operation_id TEXT NOT NULL,
  sequence INTEGER NOT NULL,
  body TEXT NOT NULL,
  truncated INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS store_health (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  quarantined INTEGER NOT NULL DEFAULT 0,
  message TEXT NOT NULL DEFAULT ''
);
INSERT OR IGNORE INTO store_health (id, quarantined, message) VALUES (1, 0, '');
