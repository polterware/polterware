-- Dispatch moved to Polterware Dispatch, which keeps its runs in its own
-- database. dispatch_tiers stays so Dispatch's first start can copy the tier
-- table; nothing in Polterware reads or writes it.

DROP TABLE IF EXISTS dispatch_route_decisions;
DROP TABLE IF EXISTS dispatch_run_events;
DROP TABLE IF EXISTS dispatch_worktrees;
DROP TABLE IF EXISTS dispatch_quota_snapshots;
DROP TABLE IF EXISTS dispatch_agents;
DROP TABLE IF EXISTS dispatch_runs;
