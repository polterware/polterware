# Research: Knowledge Document Detail Modules

**Date**: 2026-08-12 · **Feature**: 034-knowledge-document-modules

## Current state

- `packages/features/knowledge/src/index.tsx` (~750 lines) owns:
  - Tabs Docs · Decisions · Runbooks
  - DataTable collections + filters + create dialog
  - Inline detail trees for all three kinds
  - Shared `DocumentProperties` + `DocumentCanvas` (document layout, maximize-only toolbar)
- Shell mocks already rich:
  - Docs: status, owner, author, updated, environment, tags, components
  - Decisions: status, alternatives[], consequences[], bodyMarkdown ADR SSOT, components
  - Runbooks: severity, steps[], commands[], component/operation links
- Gaps vs peer details:
  - Monolithic composition (no dedicated modules / tests)
  - Decision alternatives/consequences not surfaced as property summaries
  - Runbook steps/commands not summarized in properties
  - No honest not-found when active target id missing
  - Package has `test: echo 'no tests yet'` — no vitest

## Decisions

### D1 — Module split

**Decision**: Shared shell + three kind detail modules + pure helpers.  
**Rationale**: Matches Access agent-detail pattern; keeps document chrome DRY; allows per-kind composition tests.  
**Rejected**: Single `knowledge-document-detail.tsx` with internal switch only (acceptable fallback but harder isolation tests); leave inline (fails rubric).

### D2 — Alternatives/consequences SSOT

**Decision**: Markdown body remains SSOT; properties show short summary (e.g. alternative count + first line / “N alternatives”) derived from structured fields.  
**Rationale**: Spec allows props **or** markdown SSOT; structured fields already seed body via `adrMarkdown`.  
**Rejected**: Duplicating full alternative text only in properties (noise, dual edit risk).

### D3 — Runbook steps/commands

**Decision**: Properties show counts (e.g. “5 steps”, “3 commands”) from arrays; body keeps full procedure. No live execution.  
**Rationale**: FR-005 summary without inventing ops runtime.  
**Rejected**: Embedding a fake step runner UI.

### D4 — Not-found

**Decision**: When scope is `knowledge:{docs|decisions|runbooks}` and entityKind matches but entity is undefined while a target is active, render kind-specific not-found detail.  
**Rationale**: Peer details (AgentDetail notFound) set the honesty bar.  
**Rejected**: Blank Show when={entity} only.

### D5 — Format dock

**Decision**: Preserve existing `createDetailPanelSelector(mode) === "maximized"` gate in shared shell.  
**Rationale**: Explicit preserve requirement.  
**Rejected**: Always-on toolbar or redesign.

### D6 — Spec Kit

**Decision**: Zero file touch outside Knowledge + this specs folder.  
**Rationale**: Parallel agent + product separation (Knowledge vs Spec Kit).

## References

- `docs/spec.md` §8.7 Knowledge
- AGENTS.md Entity detail + document layout law
- Peer: `packages/features/access/src/agent-detail.tsx` + tests
- Peer: `specs/026-architecture-node-detail/`
