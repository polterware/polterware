-- Atlas node dependencies: uses: lists the node paths a node depends on.
-- The map draws each entry as an edge between the two nodes.
ALTER TABLE atlas_nodes ADD COLUMN uses_json TEXT NOT NULL DEFAULT '[]';
