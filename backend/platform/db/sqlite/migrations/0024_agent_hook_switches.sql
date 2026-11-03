-- Slice 13: per-hook switches for the Claude Code plugin. hooks.json stays
-- fixed; each `polterware hook <name>` reads its switch here. A missing row
-- means the hook is on.
CREATE TABLE IF NOT EXISTS agent_hook_switches (
  name TEXT PRIMARY KEY,
  enabled INTEGER NOT NULL DEFAULT 1,
  updated_at TEXT NOT NULL
);
