-- Slice 8: multi-repository nodes and tasks.
-- repo is a project_repos name; empty means the project root checkout.
-- folder is a directory inside that checkout. It is not atlas_nodes.path,
-- which stays the node identity.

ALTER TABLE atlas_nodes ADD COLUMN repo TEXT NOT NULL DEFAULT '';
ALTER TABLE atlas_nodes ADD COLUMN folder TEXT NOT NULL DEFAULT '';
ALTER TABLE work_tasks ADD COLUMN repo TEXT NOT NULL DEFAULT '';
