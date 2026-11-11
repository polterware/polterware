-- Auto routes on a difficulty classifier (Jev, OpenAI Decisions, or a CLI
-- model) instead of keyword features: a run and its routing decision keep the
-- classifier, its P(easy), P(normal), P(hard), and the difficulty d.

ALTER TABLE dispatch_runs ADD COLUMN classifier TEXT NOT NULL DEFAULT '';
ALTER TABLE dispatch_runs ADD COLUMN difficulty REAL;
ALTER TABLE dispatch_runs ADD COLUMN difficulty_probs_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE dispatch_runs DROP COLUMN feature_schema;
ALTER TABLE dispatch_runs DROP COLUMN classify_features_json;

ALTER TABLE dispatch_route_decisions ADD COLUMN classifier TEXT NOT NULL DEFAULT '';
ALTER TABLE dispatch_route_decisions ADD COLUMN probs_json TEXT NOT NULL DEFAULT '[]';
ALTER TABLE dispatch_route_decisions DROP COLUMN feature_schema;
ALTER TABLE dispatch_route_decisions DROP COLUMN features_json;
