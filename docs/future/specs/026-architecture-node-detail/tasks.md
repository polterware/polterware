# Tasks: Architecture Node Detail Polish

**Feature**: 026-architecture-node-detail  
**Spec**: [spec.md](./spec.md) · **Plan**: [plan.md](./plan.md)

## Phase 1 — Helpers & fixtures

- [x] T001 Create `node-detail-helpers.ts` — lifecycle/link/evidence labels + tones (reuse `domain/node-lifecycle`)
- [x] T002 Enrich `NODE_DETAIL_FIXTURES` / sparse `fixtureNodeDetail` with lifecycle, linkStatus, canLinkResource as needed

## Phase 2 — Composition

- [x] T003 Polish `node-detail.tsx`: header `NodeIcon`, StatusChip metrics, evidence chips in Connected table, denser Overview, keep docs/activity/loading/error and header actions
- [x] T004 Ensure host mapping still supplies fields for chips (minimal `index.tsx` / types only if required; do not rewrite canvas/views/components-table)

## Phase 3 — Tests & gate

- [x] T005 Add `node-detail.test.ts` (helpers + composition source contract covering rubric)
- [x] T006 Run architecture package vitest; mark rubric green

## Dependency order

T001 → T002 → T003 → T004 → T005 → T006

## Out of scope tasks (do not implement)

- Workflow definition depth, org team, integrations secondary, project settings, terminal sessions
- Canvas engine, place flow, view-detail, components-table rewrites
