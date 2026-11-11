-- Extra local Git checkouts for one project. The project root_path stays the
-- primary checkpoint (file import, status, handoff). Each row is another
-- checkout scanned on its own.

CREATE TABLE IF NOT EXISTS project_repos (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  path TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_project_repos_name ON project_repos(project_id, name);
CREATE UNIQUE INDEX IF NOT EXISTS idx_project_repos_path ON project_repos(project_id, path);
