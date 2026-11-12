-- Migration 0058: Rules unification.
-- Decisions, principles, and facts become rules.
--
-- Widens rules.status to proposed|active|under_review|replaced|removed,
-- adds rules.why and rules.replaced_by. Moves decisions into rules with
-- lifecycle metadata appended to why. Moves principles into unscoped rules.
-- Moves facts into rules unless an active rule shares the statement (case-insensitive,
-- ignoring trailing period). Drops moved decisions, principles, and facts.

-- 1. Rebuild rules table with why, replaced_by, and widened status CHECK.
CREATE TABLE rules_new (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  slug TEXT,
  statement TEXT NOT NULL,
  why TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('proposed', 'active', 'under_review', 'replaced', 'removed')),
  replaced_by TEXT,
  position INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

INSERT INTO rules_new (id, project_id, slug, statement, why, status, replaced_by, position, created_at, updated_at)
SELECT id, project_id, slug, statement, '', status, NULL, position, created_at, updated_at
FROM rules;

DROP TABLE rules;
ALTER TABLE rules_new RENAME TO rules;

CREATE UNIQUE INDEX idx_rules_slug ON rules(project_id, slug) WHERE slug IS NOT NULL;
CREATE INDEX idx_rules_project ON rules(project_id, position);

DROP TRIGGER IF EXISTS entity_contexts_rule_delete;
CREATE TRIGGER entity_contexts_rule_delete AFTER DELETE ON rules
BEGIN DELETE FROM entity_contexts WHERE project_id = old.project_id AND entity_id = old.id; END;

-- 2. Move decisions (and principles) from knowledge_items into rules.
WITH decision_source AS (
  SELECT
    id,
    project_id,
    COALESCE(title, '') AS title,
    status,
    extra_json,
    created_at,
    updated_at,
    CASE
      WHEN trim(body) = '' THEN ''
      ELSE rtrim(body, char(10) || char(13) || ' ' || char(9))
    END AS b,
    CASE
      WHEN json_valid(extra_json) AND json_extract(extra_json, '$.reviewTrigger') IS NOT NULL
      THEN trim(json_extract(extra_json, '$.reviewTrigger'))
      ELSE ''
    END AS rt,
    CASE
      WHEN json_valid(extra_json) AND json_extract(extra_json, '$.statusReason') IS NOT NULL
      THEN trim(json_extract(extra_json, '$.statusReason'))
      ELSE ''
    END AS sr
  FROM knowledge_items
  WHERE kind = 'decision'
),
decision_p1 AS (
  SELECT
    id,
    project_id,
    title,
    status,
    extra_json,
    created_at,
    updated_at,
    CASE
      WHEN b <> '' AND rt <> '' THEN b || char(10) || char(10) || rt
      WHEN b <> '' THEN b
      ELSE rt
    END AS p1,
    sr
  FROM decision_source
),
decision_rows AS (
  SELECT
    id,
    project_id,
    title,
    CASE
      WHEN p1 <> '' AND sr <> '' THEN p1 || char(10) || char(10) || sr
      WHEN p1 <> '' THEN p1
      ELSE sr
    END AS why,
    CASE
      WHEN lower(trim(status)) IN ('accepted', 'active') THEN 'active'
      WHEN lower(trim(status)) = 'challenged' THEN 'under_review'
      WHEN lower(trim(status)) = 'superseded' THEN 'replaced'
      WHEN lower(trim(status)) = 'proposed' THEN 'proposed'
      ELSE 'removed'
    END AS mapped_status,
    CASE
      WHEN lower(trim(status)) = 'superseded'
       AND json_valid(extra_json)
       AND json_extract(extra_json, '$.supersededBy') IS NOT NULL
       AND trim(json_extract(extra_json, '$.supersededBy')) <> ''
      THEN trim(json_extract(extra_json, '$.supersededBy'))
      ELSE NULL
    END AS replaced_by,
    created_at,
    updated_at,
    ROW_NUMBER() OVER (PARTITION BY project_id ORDER BY created_at, id) AS seq
  FROM decision_p1
)
INSERT INTO rules (id, project_id, slug, statement, why, status, replaced_by, position, created_at, updated_at)
SELECT
  dr.id,
  dr.project_id,
  NULL,
  dr.title,
  dr.why,
  dr.mapped_status,
  dr.replaced_by,
  COALESCE((SELECT MAX(position) FROM rules WHERE project_id = dr.project_id), -1) + dr.seq,
  dr.created_at,
  dr.updated_at
FROM decision_rows dr;

-- 3. Move facts from canon_facts into rules unless an active rule has the same statement.
WITH fact_candidates AS (
  SELECT
    f.id,
    f.project_id,
    f.slug,
    f.statement,
    f.created_at,
    f.updated_at,
    ROW_NUMBER() OVER (
      PARTITION BY f.project_id, lower(rtrim(trim(f.statement), '.'))
      ORDER BY f.created_at, f.id
    ) AS dup_seq
  FROM canon_facts f
  WHERE NOT EXISTS (
    SELECT 1 FROM rules r
    WHERE r.project_id = f.project_id
      AND r.status = 'active'
      AND lower(rtrim(trim(r.statement), '.')) = lower(rtrim(trim(f.statement), '.'))
  )
),
fact_ordered AS (
  SELECT
    fc.id,
    fc.project_id,
    fc.slug,
    fc.statement,
    fc.created_at,
    fc.updated_at,
    ROW_NUMBER() OVER (PARTITION BY fc.project_id ORDER BY fc.created_at, fc.id) AS seq
  FROM fact_candidates fc
  WHERE fc.dup_seq = 1
)
INSERT INTO rules (id, project_id, slug, statement, why, status, replaced_by, position, created_at, updated_at)
SELECT
  fo.id,
  fo.project_id,
  CASE
    WHEN fo.slug IS NOT NULL AND fo.slug <> '' AND NOT EXISTS (SELECT 1 FROM rules r WHERE r.project_id = fo.project_id AND r.slug = fo.slug)
    THEN fo.slug
    ELSE NULL
  END,
  fo.statement,
  '',
  'active',
  NULL,
  COALESCE((SELECT MAX(position) FROM rules WHERE project_id = fo.project_id), -1) + fo.seq,
  fo.created_at,
  fo.updated_at
FROM fact_ordered fo;

-- 4. Delete contexts for facts that were dropped (not migrated into rules).
DELETE FROM entity_contexts
WHERE entity_id IN (
  SELECT f.id FROM canon_facts f
  WHERE NOT EXISTS (
    SELECT 1 FROM rules r WHERE r.id = f.id
  )
);

-- 5. Delete moved decision rows, principles and facts without cascading to entity_contexts.
DROP TRIGGER IF EXISTS entity_contexts_knowledge_delete;
DROP TRIGGER IF EXISTS entity_contexts_fact_delete;

DELETE FROM knowledge_items WHERE kind = 'decision';
DELETE FROM knowledge_fts WHERE kind = 'decision';
DELETE FROM canon_facts;

CREATE TRIGGER entity_contexts_knowledge_delete AFTER DELETE ON knowledge_items
BEGIN DELETE FROM entity_contexts WHERE project_id = old.project_id AND entity_id = old.id; END;

CREATE TRIGGER entity_contexts_fact_delete AFTER DELETE ON canon_facts
BEGIN DELETE FROM entity_contexts WHERE project_id = old.project_id AND entity_id = old.id; END;
