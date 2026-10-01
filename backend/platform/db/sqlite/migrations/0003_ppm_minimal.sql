-- Minimal Polterware Project Model registry.
-- Canvas body_json and the environments table remain content SSOT;
-- these tables project typed entity refs with provenance.

CREATE TABLE IF NOT EXISTS ppm_entities (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  kind TEXT NOT NULL CHECK (kind IN ('component', 'environment')),
  ref_kind TEXT NOT NULL CHECK (ref_kind IN ('architecture_node', 'environment')),
  ref_id TEXT NOT NULL,
  title TEXT NOT NULL,
  truth_class TEXT NOT NULL DEFAULT 'declared'
    CHECK (truth_class IN ('declared', 'observed', 'operational', 'inferred')),
  source TEXT NOT NULL DEFAULT 'polterware',
  actor TEXT NOT NULL DEFAULT 'system',
  observed_at TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (project_id, ref_kind, ref_id)
);
CREATE INDEX IF NOT EXISTS idx_ppm_entities_project_kind ON ppm_entities(project_id, kind);

CREATE TABLE IF NOT EXISTS ppm_relationships (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  subject_entity_id TEXT NOT NULL REFERENCES ppm_entities(id) ON DELETE CASCADE,
  predicate TEXT NOT NULL CHECK (predicate IN ('DOCUMENTS', 'OPERATES', 'RELATED_TO', 'DEFINES')),
  object_entity_id TEXT NOT NULL REFERENCES ppm_entities(id) ON DELETE CASCADE,
  truth_class TEXT NOT NULL DEFAULT 'declared'
    CHECK (truth_class IN ('declared', 'observed', 'operational', 'inferred')),
  source TEXT NOT NULL DEFAULT 'polterware',
  actor TEXT NOT NULL DEFAULT 'system',
  observed_at TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE (project_id, subject_entity_id, predicate, object_entity_id)
);
CREATE INDEX IF NOT EXISTS idx_ppm_relationships_project ON ppm_relationships(project_id);
CREATE INDEX IF NOT EXISTS idx_ppm_relationships_subject ON ppm_relationships(subject_entity_id);
CREATE INDEX IF NOT EXISTS idx_ppm_relationships_object ON ppm_relationships(object_entity_id);
