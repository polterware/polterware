-- Slice 11: architecture evolution.
-- migration_from / migration_to record a node that is moving from one design
-- to another; both empty means the node is not migrating.
-- work_cards.kind marks an Architecture Change (kind = 'architecture').
-- change_ops.node_id ties invariant ops to their node so the Atlas can be
-- replayed at a past point.

ALTER TABLE atlas_nodes ADD COLUMN migration_from TEXT NOT NULL DEFAULT '';
ALTER TABLE atlas_nodes ADD COLUMN migration_to TEXT NOT NULL DEFAULT '';
ALTER TABLE work_cards ADD COLUMN kind TEXT NOT NULL DEFAULT 'feature' CHECK (kind IN ('feature', 'architecture'));
ALTER TABLE change_ops ADD COLUMN node_id TEXT NOT NULL DEFAULT '';

UPDATE change_ops
   SET node_id = COALESCE((SELECT node_id FROM atlas_invariants WHERE id = change_ops.entity_id), '')
 WHERE entity = 'node' AND entity_id IN (SELECT id FROM atlas_invariants);
