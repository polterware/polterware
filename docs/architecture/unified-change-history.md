# Unified Change History (B9)

Status: Shipped (core + Desktop Agents → History; human Desktop semantic writes require the same change envelope as MCP)

Authority: [ADR 0004](../adr/0004-product-positioning-and-wedge.md), [`wedge-validation-sequence.md`](wedge-validation-sequence.md), [`minimal-agent-run-audit.md`](minimal-agent-run-audit.md), [`audited-mcp-writes.md`](audited-mcp-writes.md), [`docs/product-vision.md`](../product-vision.md) v3.

## Purpose

**Agents → History** is one chronological feed of **justified project-model changes** for both `human` and `agent` actors. Same change envelope as audited writes (`summary`, `rationale`, attributable run). Not an enterprise audit / Activity / SIEM surface.

## Depends on

- **B6** — AgentRun chain (`context → task → change → evidence → drift`) and agent-only History list
- **B7** — MCP semantic writes that persist real mutations behind the envelope
- Human Desktop mutations on the **same** semantic write path (`wedge.ExecuteAuditedWrite`) so human rows are honest

## Envelope (human and agent)

Every semantic mutation requires non-empty `summary` and `rationale`.

| Actor | How the run is attached |
|-------|-------------------------|
| `agent` | MCP: open `agent_run_id` required (B7) |
| `human` | Desktop: dialog collects envelope; core auto-starts and completes an `actor_kind=human` run when `agent_run_id` is empty |

Body autosave reuses a session-scoped envelope for the open document so typing does not re-prompt on every debounce.

## In scope (shipped)

- Single Agents → History surface (no new product nav item)
- Feed rows: actor kind, summary, time, linked run; actor filter (`all` / `human` / `agent`)
- Detail pane: envelope (`summary`, `rationale`, tool, subjects) + ordered audit chain; `drift` steps expand typed findings from `payloadJson` (severity, type, summary, subjects)
- `ListChanges` RPC + `actor_kind` on `agent_runs`
- Desktop Knowledge / PPM semantic writes gated by change envelope

## Out of scope

- Settings, shell chrome, Integrations Connect auth
- Generic UI CRUD that is not a PPM / Knowledge / decision mutation
- Enterprise compliance dashboards, Access/Teams/Policies ACL, Change Gates (post-wedge)
- Import scan (`ImportProjectSources`) remains a system path without human dialog in this slice

## Code

- Shared write path: `backend/compositions/wedge/audited.go`
- Store: `ListChanges`, `actor_kind` migration `0013_agent_run_actor_kind.sql`
- Desktop dialog: `packages/features/knowledge/src/change-envelope-dialog.tsx`
- History UI: `packages/features/settings/src/agent-run-history-card.tsx`
- Drift step findings parse: `packages/features/settings/src/drift-step-payload.ts`
