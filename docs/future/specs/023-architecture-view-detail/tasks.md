# Tasks: Architecture View Detail

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/

**Tests**: Risk-proportional per Constitution XII — pure membership helpers; composition source contract for DetailPanel LAW; desktop `detail-fixture-coverage` for enriched views; architecture package tests green for touched surface. Agents.md: do not run `pnpm dev` / full build unless the user asks.

**Organization**: Tasks grouped by user story (US1–US4) for independent implementation and testing.

**Constitution / guardrails**:

- DetailPanel LAW only — no second inspector/page for views
- Open reuses canvas activation; no second canvas host
- Membership never invented from nodeCount alone
- Parallelism: DO NOT touch Components tab, process/approval/invitation, deep canvas engine rewrite
- English UI strings

## Format: `[ID] [P?] [Story] Description`

## Path Conventions (this feature)

- Architecture feature: `packages/features/architecture/src/`
- View detail: `packages/features/architecture/src/view-detail.tsx`
- Helpers: `packages/features/architecture/src/view-detail-helpers.ts`
- Tests: `packages/features/architecture/src/view-detail.test.ts`
- Fixtures: `packages/features/architecture/src/extra-fixtures.ts`
- Architecture index: `packages/features/architecture/src/index.tsx`
- Desktop fixture gate: `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`
- Contracts: `specs/023-architecture-view-detail/contracts/architecture-view-detail-ui.md`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm feature package surface and test runner for architecture

- [x] T001 Confirm feature ownership stays in `packages/features/architecture` (no new scenic package) per [plan.md](./plan.md)
- [x] T002 [P] Confirm Vitest config for architecture package in `packages/features/architecture/vitest.config.ts`
- [x] T003 [P] Confirm architecture package test script in `packages/features/architecture/package.json`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Data model + helpers + fixture enrichment that all stories consume

**⚠️ CRITICAL**: User story UI work depends on these fixtures/helpers

- [x] T004 Extend `ArchViewCard` with optional `owner`, `environments`, `tags`, `purpose`, `members`, `notes`, `filterSummary`, `dataSource` and `ArchViewMember` type in `packages/features/architecture/src/extra-fixtures.ts`
- [x] T005 Enrich ≥2 primary VIEW_FIXTURES (System view + Payment flow) with owner/environments/tags and explicit members; leave ≥1 sparse view without members for unavailable path in `packages/features/architecture/src/extra-fixtures.ts`
- [x] T006 [P] Implement pure membership helpers (`hasMembershipList`, empty/unavailable copy) in `packages/features/architecture/src/view-detail-helpers.ts`
- [x] T007 [P] Keep UI contract SSOT current in `specs/023-architecture-view-detail/contracts/architecture-view-detail-ui.md`
- [x] T008 [P] Keep data model SSOT current in `specs/023-architecture-view-detail/data-model.md`
- [x] T009 [P] Add helper unit tests in `packages/features/architecture/src/view-detail.test.ts`
- [x] T010 Expand architecture view completeness assertions in `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`

**Checkpoint**: Fixture view model + helpers ready; stories can compose DetailPanel independently

---

## Phase 3: User Story 1 - Inspect a saved view (Priority: P1) 🎯 MVP

**Goal**: Operator activates Views row and inspects full view posture in global DetailPanel.

**Independent Test**: Architecture → Views → activate any fixture → DetailPanel shows title/icon + overview beyond one-line summary.

### Tests for User Story 1

- [x] T011 [P] [US1] Add composition source-contract assertions (sections layout, metrics, Open/Edit, Overview) in `packages/features/architecture/src/view-detail.test.ts`
- [x] T012 [P] [US1] Assert Architecture wires `ArchitectureViewDetail` for `entityKind === "arch-view"` via source checks in `packages/features/architecture/src/view-detail.test.ts`

### Implementation for User Story 1

- [x] T013 [US1] Create `ArchitectureViewDetail` composition with loading/error/not-found/ready branches in `packages/features/architecture/src/view-detail.tsx`
- [x] T014 [US1] Render overview metrics (node count, updated) + Overview metadata (description, owner, environments, tags/purpose) in `packages/features/architecture/src/view-detail.tsx`
- [x] T015 [US1] Handle empty description with honest empty copy in `packages/features/architecture/src/view-detail.tsx`
- [x] T016 [US1] Replace inline arch-view DetailPanel block with `ArchitectureViewDetail` mount (including not-found when target active but missing) in `packages/features/architecture/src/index.tsx`

**Checkpoint**: US1 MVP — full inspection surface for fixture views

---

## Phase 4: User Story 2 - Open / Edit from detail (Priority: P1)

**Goal**: Header Open activates canvas view; Edit opens existing view-edit deep flow.

### Tests for User Story 2

- [x] T017 [P] [US2] Assert Open (primary) and Edit action labels + onOpen/onEdit wiring in composition and index source checks in `packages/features/architecture/src/view-detail.test.ts`

### Implementation for User Story 2

- [x] T018 [US2] Wire header Open primary → existing `setActiveViewId` + canvas tab change via props in `packages/features/architecture/src/view-detail.tsx` and `packages/features/architecture/src/index.tsx`
- [x] T019 [US2] Wire header Edit → existing `setEditViewName` + `deepMode("view-edit")` in `packages/features/architecture/src/index.tsx`

**Checkpoint**: US2 open/edit reuse existing flows

---

## Phase 5: User Story 3 - Membership (Priority: P2)

**Goal**: Structured membership when fixture members exist; honest unavailable when only nodeCount.

### Tests for User Story 3

- [x] T020 [P] [US3] Assert Membership section + DataTable/empty/unavailable copy and no nodeCount→name synthesis in `packages/features/architecture/src/view-detail.test.ts`
- [x] T021 [P] [US3] Assert ≥2 enriched fixtures include members or extended fields in tests / desktop coverage

### Implementation for User Story 3

- [x] T022 [US3] Render Membership DataTable when members present; empty and unavailable states from helpers in `packages/features/architecture/src/view-detail.tsx`
- [x] T023 [US3] Optional Notes / Filters sections when fixture fields present in `packages/features/architecture/src/view-detail.tsx`

**Checkpoint**: US3 membership honesty

---

## Phase 6: User Story 4 - Honest empty / missing states (Priority: P2)

**Goal**: not-found, loading, empty description non-crashing.

### Tests for User Story 4

- [x] T024 [P] [US4] Assert PageLoading / PageError / not-found / empty description strings in composition contract in `packages/features/architecture/src/view-detail.test.ts`

### Implementation for User Story 4

- [x] T025 [US4] Ensure loading and not-found branches use PageLoading/PageError in `packages/features/architecture/src/view-detail.tsx`
- [x] T026 [US4] Ensure index mounts detail for active arch-view even when fixture missing in `packages/features/architecture/src/index.tsx`

**Checkpoint**: US4 honest states

---

## Phase 7: Polish & Cross-Cutting

- [x] T027 [P] Re-export `ArchitectureViewDetail` from `packages/features/architecture/src/index.tsx` if useful for tests/consumers
- [x] T028 Run architecture package tests and desktop detail-fixture-coverage for views; fix failures
- [x] T029 Mark this tasks file complete and confirm rejection rubric green

## Dependencies

```text
Phase 1 → Phase 2 → Phase 3 (US1 MVP) → Phase 4 (US2) → Phase 5 (US3) → Phase 6 (US4) → Phase 7
```

## Parallel opportunities

- T002/T003 after T001
- T006–T009 after T004–T005
- T011/T012 with composition once T013 exists
- T017 with T018–T019
- T020/T021 with T022–T023

## MVP scope

Phases 1–3 (fixture enrichment + ArchitectureViewDetail inspect surface) deliver SC-001/SC-005; Open/Edit and membership complete the full rubric.

## Implementation strategy

1. Enrich fixtures first so coverage gate is honest.
2. Extract composition with full sections before wiring.
3. Wire index with not-found + Open/Edit handlers.
4. Lock with package + desktop tests.

## Rubric (rejection gate)

- [x] Dedicated module (not only inline index)
- [x] Membership section exists (collection OR unavailable copy)
- [x] ≥2 views with fields beyond name/description/nodeCount/updated
- [x] Open and Edit wired to existing flows
- [x] PageLoading/PageError or notFound honest
- [x] view-detail.test.ts exists
- [x] detail-fixture-coverage updated for enriched views
- [x] Never invent member names from nodeCount alone
- [x] Not claiming “already implemented” from thin DetailPanel stub
