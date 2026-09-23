# Wedge validation sequence

Status: Priority locked after Phase A cleanup

Authority: [ADR 0004](../adr/0004-product-positioning-and-wedge.md), [`docs/product-vision.md`](../product-vision.md) v3. Do not treat incomplete wedge work as shipped in [`docs/spec.md`](../spec.md).

## Next implementation slice

Wedge B1–B9 complete for local validation. Next product work is outside this sequence (or post-wedge gates), not another B-slice foundation.

B9 (Unified Change History) is implemented: Agents → History lists justified `human` + `agent` changes; Desktop semantic Knowledge/PPM writes require `summary` + `rationale`. See [`unified-change-history.md`](unified-change-history.md).

## Full order (do not skip foundations)

1. B1 Facts / Evidence / provenance — **done (core)**
2. B2 Deterministic import (MVP sources) — **done (core + Desktop)**
3. B3 Context Compiler → `context.get` shape (reasons + evidence) — **done (core + Desktop preview)**
4. B4 Stable MCP reads (`context.get`, `entity.get`, `evidence.get`, `drift.check`) — **done (core + skill)**
5. B5 Narrow Drift Checker — **done (core + skill)**
6. B6 Minimal AgentRun audit — **done (core + Desktop History + skill)**
7. B7 Audited MCP writes (narrow semantic mutations + required change envelope) — **done (core + skill)**
8. B8 CLI `polterware init | scan | context | drift` — **done (core CLI + skill)**
9. B9 Unified Change History (Agents → History: human + agent, justified changes) — **done (core + Desktop)**

### B7 — Audited MCP writes

After AgentRun exists, agents may mutate the project model through narrow semantic write tools (for example knowledge upsert and fact/evidence link). Writes are not a mirror of every Desktop surface.

Every mutation MUST include `summary`, `rationale`, and `agent_run_id`; the core rejects the call if any are missing. Mutations are versioned and attributable via AgentRun (`context → task → change → evidence → drift`).

Out of scope for B7: Settings, shell chrome, Integrations Connect auth, and generic CRUD of all UI entities.

See [`audited-mcp-writes.md`](audited-mcp-writes.md).

### B8 — CLI

Local Go binary `polterware` with `init`, `scan`, `context`, and `drift`. In-process SQLite (same default `-db` as `polterware-core`); no session file in this slice. Shares wedge composition with MCP.

See [`cli.md`](cli.md).

### B9 — Unified Change History

**Done.** Agents → History is a chronological feed of justified changes for `human` and `agent` (actor filter, envelope detail, audit chain). Desktop Knowledge/PPM semantic writes share `wedge.ExecuteAuditedWrite` with MCP and require `summary` + `rationale`.

See [`unified-change-history.md`](unified-change-history.md).

Out of scope for B9: Settings, Connect auth, generic UI CRUD, Access/Teams/Policies ACL, Change Gates, SIEM-style compliance.

Kill branch for Ops / Terminal / Nodes is documented in [`kill-ops-terminal-nodes.md`](kill-ops-terminal-nodes.md) and may run in parallel after Phase A freeze; it does not block wedge validation.
