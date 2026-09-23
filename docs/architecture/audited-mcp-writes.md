# Audited MCP writes (B7)

Status: Shipped for `polterware-core mcp` semantic write tools

Authority: [ADR 0004](../adr/0004-product-positioning-and-wedge.md), [`wedge-validation-sequence.md`](wedge-validation-sequence.md), [`minimal-agent-run-audit.md`](minimal-agent-run-audit.md), [`docs/product-vision.md`](../product-vision.md) v3.

## Purpose

Agents mutate the project model through **narrow semantic MCP write tools**. Every mutation requires a change envelope (`summary`, `rationale`, `agent_run_id`), runs against an open AgentRun, persists the real PPM/Knowledge change, and appends an attributable `change` step.

Writes are not a mirror of every Desktop surface.

## Envelope

Every write tool requires:

| Field | Rule |
|-------|------|
| `summary` | Non-empty |
| `rationale` | Non-empty |
| `agent_run_id` | Non-empty; run must exist and be `open` |
| `project_id` | Must match the AgentRun’s `project_id` |

The core rejects the call if any envelope field is missing or the run is not open / project-mismatched.

Actor attribution on persisted rows: `actor = "agent"`, `source = "mcp"` (callers cannot override actor to `system` for verified-bypass).

## Tools

| Tool | Mutation |
|------|----------|
| `fact.upsert` | Upsert a PPM fact (`statement`, optional `id` / `truth_class` / `entity_id`) |
| `evidence.upsert` | Upsert PPM evidence (`kind`, optional `id` / `evidence_summary` / `path`; content summary defaults to envelope `summary`) |
| `evidence.link` | Link existing evidence to a fact (`fact_id`, `evidence_id`) |
| `knowledge.upsert` | Upsert a knowledge item (`title`, `kind`, optional `id` / `body` / `tags`) |
| `knowledge.link` | Link knowledge to a PPM entity (`knowledge_id`, `entity_id`, `predicate`) |

Each successful write appends a `change` step whose payload includes `summary`, `rationale`, `tool`, and `subject_ids` (mutated record ids). Optional `paths` may be passed through when the agent supplies them.

`agent_run.append` with `kind: "change"` remains a **declared-only** envelope (no PPM mutation). Prefer the write tools when the model should change.

## Suggested flow

1. `agent_run.start`
2. `context.get` with `agent_run_id`
3. One or more audited write tools (same `agent_run_id`)
4. `agent_run.append` evidence ids if needed
5. `drift.check` with `agent_run_id`
6. `agent_run.complete`

## Out of scope

- Settings, shell chrome, Integrations Connect auth
- Generic entity/relationship/resource CRUD and `SyncProjectModel`
- Unified Change History UI depth beyond the Agents → History feed (see B9)

Desktop Agents → Facts exposes the same fact/evidence semantic mutations for humans (`UpsertFact` / `UpsertEvidence` / `LinkEvidenceToFact` with change envelope). It is not a full PPM CRUD surface.

## Code

- Domain envelope: `backend/domains/agentrun` (`ValidateAuditedWrite`, mutation change payload)
- MCP: `backend/providers/mcp/writes.go`, `backend/providers/mcp/audited.go`
- Desktop: `packages/features/settings/src/ppm-facts-panel.tsx`
- Store: existing `UpsertPpmFact`, `UpsertPpmEvidence`, `LinkPpmEvidenceToFact`, `SaveKnowledge`, `SaveKnowledgeLink`
