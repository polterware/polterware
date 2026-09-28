# ADR 0006: Polter format and relational Work model

- Status: Proposed
- Date: 2026-09-28
- Relates to: [ADR 0004](0004-product-positioning-and-wedge.md), [ADR 0005](0005-post-wedge-cloud-and-enrichment-boundary.md), [`polter-format.md`](../architecture/polter-format.md), [`atlas-grounded-sdd-roadmap.md`](../architecture/atlas-grounded-sdd-roadmap.md)

## Context

Agents create and update project state (roadmaps, Work features, tasks, open questions) through MCP. Creating one Work feature takes about seven calls (`agent_run.start`, five `knowledge.upsert` for Spec/Plan/Tasks/Clarify/Checklist, `agent_run.complete`), so a ten-feature roadmap takes about seventy. Cost comes from the number of calls, tool schemas loaded into every session, and repeated JSON envelopes. The five-document model also keeps tasks and questions as markdown checkboxes inside documents, which the core cannot address, claim, or link to commits.

The Atlas + Grounded SDD direction needs addressable cards, tasks, questions, and invariants, and an operation log that can later become the Cloud sync contract.

## Decision

1. **Polter format is the agent ↔ Polterware language.** A small markdown dialect (frontmatter `polter: 1`, `## slug: Title` cards, `key: value` metadata, `- [ ]` tasks, `?` questions, `!` invariants) is used in both directions: agents write it, Polterware returns it. The grammar is versioned and decoded by a deterministic line-oriented parser in the Go core (`backend/domains/polter`), shared by CLI, MCP, the core, and Cloud.
2. **The Polterware store is the source of truth.** SQLite locally, Cloud for teams. Polter text is never stored as a blob, and `.pw.md` files are optional input, never the truth. Repositories provide evidence in and receive projections out (ADRs, `AGENTS.md` blocks).
3. **Relational Work model.** Cards, tasks, questions, and invariants are typed tables (`work_cards`, `work_tasks`, `work_questions`, `work_invariants`) with ULID ids generated where the change happens. Cards are upserted by slug; tasks keep per-card numbers that are never reused.
4. **Partial writes, explicit removal.** A text only changes what it mentions. Removal is explicit (`status: removed`, `- [-]`, trailing `[removed]`) and rows are never deleted.
5. **Append-only change log.** Every applied operation is one `change_ops` row (entity, id, op, changed fields), grouped by a batch id that the AgentRun change step also carries. Each apply is one transaction and one change envelope (`summary` + `rationale`).
6. **The five-document Work model is discontinued.** Spec/Plan/Tasks/Clarify/Checklist Knowledge items are not migrated or kept in parallel. The Desktop Board moves to the Work tables, and the five-document flow is removed in the same change.
7. **CLI first, MCP next.** `polterware apply`, `lint`, `format`, and `query` ship first. MCP adds `polter.apply`, `polter.query`, and `polter.format` next (slice 3b); existing MCP tools remain until a later change retires them.

## Consequences

- One `polterware apply` call replaces dozens of MCP calls for a roadmap, and re-applying the same text is a no-op.
- Tools lose per-field MCP schema validation; line-numbered lint errors that include the fix replace it.
- The Desktop reads cards through a typed `WorkService` (proto) and writes them by sending Polter text to `WorkService.ApplyPolter`, so the Desktop and agents share one write path.
- Existing local spec features are not carried over when the Board switches; there is no compatibility requirement for earlier data.
- The `change_ops` log is the basis for Cloud replication; its granularity may be revised before Cloud (C0) without changing the grammar.
