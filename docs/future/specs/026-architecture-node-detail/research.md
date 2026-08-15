# Research: Architecture Node Detail Polish

**Date**: 2026-08-12 | **Feature**: 026-architecture-node-detail

## Current state

- `ArchitectureNodeDetail` exists (~163 lines): loading/error/not-found, header actions Open on canvas + Link resource, Overview as Metadata raw strings, Connected DataTable with raw Proven/Unproven text, docs + activity tables.
- Zero StatusChip usage in this file.
- Zero dedicated composition tests.
- Domain already has `node-lifecycle.ts` with `nodeStatusLabel`, `nodeLifecycleTone`, `linkStatusLabel`, `linkStatusTone` used by Components table.
- Host wiring in `index.tsx` maps document nodes → `ArchitectureNodeView` and fixtures via `fixtureNodeDetail`; Components + canvas activate `arch-node`; Views use separate `ArchitectureViewDetail`.

## Decisions

| Decision | Choice | Rationale |
| --- | --- | --- |
| Lifecycle/link tones | Reuse `domain/node-lifecycle` | SSOT already honest (planned/linked ≠ healthy) |
| Evidence chips | New helpers Proven/Unproven + tones | No existing evidence helper; keep local to detail helpers |
| Header icon | `NodeIcon` from kind/type | Matches Components PrimaryCell pattern |
| Overview layout | Metrics for chip axes + Metadata for dense facts | Align with Resources node-detail density |
| Tests | Source composition contract + pure helpers | Matches 014-nodes-detail / view-detail pattern without Electron |
| Host rewrite | Avoid structural index rewrite | Props already sufficient; enrich fixtures + optional mapping fields if missing |

## Alternatives rejected

- Rewrite canvas or place flow for “richer detail” — out of scope and parallel-risk.
- Claim done because file exists — fails delivery rubric explicitly.
- Hand-rolled status pills — forbidden by AGENTS/DESIGN StatusChip law.

## Open items

None blocking. Proceed to implementation.
