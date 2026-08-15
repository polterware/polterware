# Tasks: Architecture Components Tab

**Input**: Design documents from `/specs/025-architecture-components-tab/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Route/tab normalization + list projection/filter + activation contract. Run package vitest for architecture (and shell if touched).

## Phase 1: Setup

- [x] T001 Create `specs/025-architecture-components-tab/` artifacts and set `.specify/feature.json`

## Phase 2: Foundational

- [x] T002 [P] Add pure `normalizeArchitectureTab` in `packages/features/architecture/src/tab.ts` (keep `components`; retire only dependencies/changes → canvas)
- [x] T003 [P] Extend `packages/features/architecture/src/domain/node-lifecycle.ts` with StatusChip tone (+ link status label/tone helpers if needed)
- [x] T004 [P] Add pure list projection/filter helpers in `packages/features/architecture/src/components-list.ts`

## Phase 3: User Story 1 — Components tab visible & not redirected (P1) 🎯 MVP

**Goal**: Toolbar shows Canvas · Components · Views; `components` stays `components`

- [x] T005 [US1] Update `ARCH_TABS` and tab normalization in `packages/features/architecture/src/index.tsx` to include Components and use `normalizeArchitectureTab`
- [x] T006 [US1] Test: `normalizeArchitectureTab("components") === "components"` in `packages/features/architecture/src/components-tab.test.ts`

## Phase 4: User Story 2 — Canonical DataTable list (P1)

**Goal**: Professional table from same document nodes

- [x] T007 [US2] Implement `packages/features/architecture/src/components-table.tsx` (DataTable, PrimaryCell icon by type, StatusChip lifecycle, type/env/owner-or-link; no Actions)
- [x] T008 [US2] Wire Components panel in `index.tsx` from `doc().nodes` with loading/empty honesty
- [x] T009 [US2] Test: projection ids match document node ids; empty filter yields empty list

## Phase 5: User Story 3 — Row activate → arch-node detail (P1)

**Goal**: Global DetailPanel reuses ArchitectureNodeDetail

- [x] T010 [US3] Register collection `architecture:components` with arch-node targets; resolve detailNode for canvas **or** components scopes
- [x] T011 [US3] Row activate → `activateTarget({ scopeId: "architecture:components", entityKind: "arch-node", entityId })`
- [x] T012 [US3] Optional: “Open on canvas” DetailPanel.Action
- [x] T013 [US3] Test: activation contract shape for components list

## Phase 6: User Story 4 — Filters (P2)

- [x] T014 [US4] FilterPopover for lifecycle / type / environment on Components toolbar actions
- [x] T015 [US4] Test filter helpers reduce row set

## Phase 7: Polish & rubric

- [x] T016 Confirm shell route still keeps components (existing test); adjust only if broken
- [x] T017 Run focused architecture tests for components tab + lifecycle; shell route tests green

## Dependency Graph

```text
T001 → T002/T003/T004 → T005/T006 → T007/T008/T009 → T010–T013 → T014/T015 → T016/T017
```

## MVP

US1 + US2 + US3 (T002–T013) deliver §8.2 restore. US4 is polish filters.
