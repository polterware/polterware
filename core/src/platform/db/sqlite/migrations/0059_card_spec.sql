-- Migration 0059: the card's spec and plan.
-- behavior is what the feature does, edge_cases the cases it must handle, and
-- plan the technical plan (architecture, APIs, and files touched). The
-- contract's acceptance criteria stay in work_invariants.

ALTER TABLE work_cards ADD COLUMN behavior TEXT NOT NULL DEFAULT '';
ALTER TABLE work_cards ADD COLUMN edge_cases TEXT NOT NULL DEFAULT '';
ALTER TABLE work_cards ADD COLUMN plan TEXT NOT NULL DEFAULT '';
