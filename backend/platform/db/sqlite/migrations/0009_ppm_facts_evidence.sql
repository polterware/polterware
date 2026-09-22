-- PPM Facts / Evidence with provenance (wedge B1).
-- Entity truth_class may still use operational; facts use verified instead.

CREATE TABLE IF NOT EXISTS ppm_facts (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  statement TEXT NOT NULL,
  truth_class TEXT NOT NULL DEFAULT 'declared'
    CHECK (truth_class IN ('declared', 'observed', 'inferred', 'verified')),
  source TEXT NOT NULL DEFAULT 'polterware',
  actor TEXT NOT NULL DEFAULT 'system',
  observed_at TEXT NOT NULL,
  entity_id TEXT REFERENCES ppm_entities(id) ON DELETE SET NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ppm_facts_project ON ppm_facts(project_id);
CREATE INDEX IF NOT EXISTS idx_ppm_facts_entity ON ppm_facts(entity_id);

CREATE TABLE IF NOT EXISTS ppm_evidence (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  summary TEXT NOT NULL DEFAULT '',
  source TEXT NOT NULL DEFAULT 'polterware',
  path TEXT NOT NULL DEFAULT '',
  actor TEXT NOT NULL DEFAULT 'system',
  observed_at TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ppm_evidence_project ON ppm_evidence(project_id);

CREATE TABLE IF NOT EXISTS ppm_fact_evidence (
  fact_id TEXT NOT NULL REFERENCES ppm_facts(id) ON DELETE CASCADE,
  evidence_id TEXT NOT NULL REFERENCES ppm_evidence(id) ON DELETE CASCADE,
  created_at TEXT NOT NULL,
  PRIMARY KEY (fact_id, evidence_id)
);
CREATE INDEX IF NOT EXISTS idx_ppm_fact_evidence_evidence ON ppm_fact_evidence(evidence_id);
