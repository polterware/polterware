# ADR 0004: Product positioning and wedge

- Status: Accepted
- Date: 2026-09-22

## Context

Deep market research (2026-09-22) concluded that Polterware's earlier platform ambition (architecture canvas as product, Operations/control plane, Terminal, remote nodes, generic memory/RAG) is oversized and poorly differentiated. The durable opportunity is verifiable project context plus change/drift verification for coding agents.

The Desktop Electron/Solid application and Go `polterware-core` already ship local surfaces that include Operations, Terminal, and Nodes. Those surfaces remain in the current codebase until a dedicated future kill branch removes them. They must not define the product thesis or receive further expansion.

## Decision

1. **Category:** Project Context & Change Intelligence for AI Coding.
2. **Stack:** Desktop Electron + Solid, Go `polterware-core`, SQLite. Do not rewrite the core in Rust for the wedge.
3. **Wedge MUST:** Polterware Project Model with Facts/Evidence and strong provenance (`declared` / `observed` / `inferred` / `verified`); deterministic import; Context Compiler; stable MCP reads (`context.get`, `entity.get`, `evidence.get`, `drift.check`); narrow Drift Checker; AgentRun audit; audited MCP writes (required `summary` / `rationale` / `agent_run_id`); CLI (`init` / `scan` / `context` / `drift`); Unified Change History (Agents → History for human + agent justified model changes).
4. **Explicitly not the product:** IDP/software catalog, Mem0-style personal/shared memory as positioning, architecture canvas as thesis, generic RAG/GraphRAG marketing, Operations/control plane as thesis.
5. **Kill list (scheduled for removal in a future version on a dedicated branch):** Operations (Runs, Workflows, Schedules as product surfaces), Terminal as a product surface, Nodes/remote execution/process control as product. Do not expand these. Code removal happens only on that future kill branch.
6. **Integrations KEEP:** evidence/read path and intermediate Connect UI (PTY only inside Connect). Not a provider console or ops surface.
7. **Gates:** Understanding / Change / Assurance Gates may appear later. Do not use “Anti-Vibe” as public branding.

Living architecture and the research guide live in the Polterware Notion workspace. Local authority remains: `docs/spec.md` for shipped behavior, this ADR and `docs/product-vision.md` for direction.

## Consequences

- New work prioritizes the wedge over condemned surfaces.
- `docs/spec.md` continues to describe shipped Ops/Terminal/Nodes until removal, labeled as scheduled for removal.
- Agents and humans must not present condemned surfaces as the product story.
- Canvas may remain a human inspector of the model; it is not the wedge or the category.

See also: [ADR 0005](0005-post-wedge-cloud-and-enrichment-boundary.md) (post-wedge Cloud and enrichment boundary).
