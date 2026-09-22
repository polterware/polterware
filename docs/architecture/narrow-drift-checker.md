# Narrow Drift Checker (B5)

Status: Shipped for `polterware-core mcp` `drift.check`

Authority: [ADR 0004](../adr/0004-product-positioning-and-wedge.md), [`wedge-validation-sequence.md`](wedge-validation-sequence.md), [`stable-mcp-reads.md`](stable-mcp-reads.md), [`docs/product-vision.md`](../product-vision.md) v3.

## Purpose

Make disagreement visible between declared intent and observed project state. Drift is a **finding**, never an automatic overwrite.

## Prerequisites

- `SyncProjectModel` projects architecture canvas nodes and edges into `ppm_entities` / `ppm_relationships` (`source = architecture_layout`). Canvas edge `evidence: false` → relationship `truth_class = inferred`; otherwise `declared`.
- Canvas types map to PPM predicates: dependency-like → `DEPENDS_ON`, call-like → `CALLS`, else `RELATED_TO`.

## Finding types

| `type` | Rule |
|--------|------|
| `dependency_added` | Fresh import parse produces a stable fact id not present in `ppm_facts` with `source` prefix `import:` |
| `dependency_removed` | Stored `import:*` fact id is absent from the current parse |
| `edge_decision_conflict` | `DEPENDS_ON` / `CALLS` relationship with `truth_class = inferred` involves an entity linked to an ADR with `status = accepted` |
| `declared_observed_conflict` | Same non-empty `entity_id` has declared and observed facts whose statements share a topic but differ in value |

`checker` is always `"narrow"` when this implementation responds. Optional `entity_id` filters findings whose `subject_ids` (or linked fact/relationship ids) involve that entity.

Import dependency diff runs only when the project `root_path` exists and is readable. Otherwise that rule is skipped (no false “removed everything” flood).

## Limits

- Does not interpret ADR markdown; “active decision” means accepted ADR + `knowledge_entity_links`.
- Does not map manifest dependencies onto canvas nodes.
- Does not prune stale import facts on re-import; `dependency_removed` remains the honest signal until a later prune.
- No Desktop Drift surface; no CLI `polterware drift` (B8). AgentRun attribution for drift steps is B6 (`agent_run_id` on `drift.check`); see [`minimal-agent-run-audit.md`](minimal-agent-run-audit.md).

## Code

- Domain: `backend/domains/drift`
- MCP: `backend/providers/mcp/drift_check.go`
- Layout edge parse: `backend/domains/ppm/layout.go` (`ParseLayoutEdges`)
