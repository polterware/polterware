# Wedge validation sequence

Status: Priority locked after Phase A cleanup

Authority: [ADR 0004](../adr/0004-product-positioning-and-wedge.md), [`docs/product-vision.md`](../product-vision.md) v3. Do not treat incomplete wedge work as shipped in [`docs/spec.md`](../spec.md).

## Next implementation slice

**B7 — Audited MCP writes**

B6 (Minimal AgentRun audit) is implemented: MCP `agent_run.*` tools record `context → task → change → evidence → drift`; `context.get` / `drift.check` accept optional `agent_run_id`; Desktop Agents → History lists runs. See [`minimal-agent-run-audit.md`](minimal-agent-run-audit.md). This slice adds narrow semantic MCP write tools that require `summary`, `rationale`, and `agent_run_id`.

## Full order (do not skip foundations)

1. B1 Facts / Evidence / provenance — **done (core)**
2. B2 Deterministic import (MVP sources) — **done (core + Desktop)**
3. B3 Context Compiler → `context.get` shape (reasons + evidence) — **done (core + Desktop preview)**
4. B4 Stable MCP reads (`context.get`, `entity.get`, `evidence.get`, `drift.check`) — **done (core + skill)**
5. B5 Narrow Drift Checker — **done (core + skill)**
6. B6 Minimal AgentRun audit — **done (core + Desktop History + skill)**
7. B7 Audited MCP writes (narrow semantic mutations + required change envelope)
8. B8 CLI `polterware init | scan | context | drift`

### B7 — Audited MCP writes

After AgentRun exists, agents may mutate the project model through narrow semantic write tools (for example knowledge upsert and fact/evidence link). Writes are not a mirror of every Desktop surface.

Every mutation MUST include `summary`, `rationale`, and `agent_run_id`; the core rejects the call if any are missing. Mutations are versioned and attributable via AgentRun (`context → task → change → evidence → drift`).

Out of scope for B7: Settings, shell chrome, Integrations Connect auth, and generic CRUD of all UI entities.

Kill branch for Ops / Terminal / Nodes is documented in [`kill-ops-terminal-nodes.md`](kill-ops-terminal-nodes.md) and may run in parallel after Phase A freeze; it does not block wedge validation.
