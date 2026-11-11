-- Slice 7: Inbox items are derived, never stored. A person's disposition on
-- one item (defer, delegate, ignore, reject, fix code, update intent) is kept
-- here, append-only; the latest row per item key wins.

CREATE TABLE IF NOT EXISTS inbox_dispositions (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  item_key TEXT NOT NULL,
  action TEXT NOT NULL CHECK (action IN ('delegate', 'defer', 'fix_code', 'update_intent', 'ignore', 'reject')),
  actor TEXT NOT NULL DEFAULT '',
  note TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_inbox_dispositions_project ON inbox_dispositions(project_id, item_key, created_at);
