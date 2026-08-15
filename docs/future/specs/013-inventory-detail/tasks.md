# Tasks: Inventory Resource Detail

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/

**Tests**: Risk-proportional per Constitution XII — pure helpers for status/provenance/source; fixture contract invariants; desktop `detail-fixture-coverage` for rich inventory completeness. Do not mark complete while required tests fail for the changed surface. Agents.md: do not run `pnpm dev` / full build unless the user asks.

**Organization**: Tasks grouped by user story (US1–US5) for independent implementation and testing.

**Constitution / guardrails**:

- DetailPanel LAW only — no second inspector/page/drawer for inventory resources
- Dependency direction IV: `feature-resources` → design-system / state
- SSoT XIV: single `ResourceDetail` composition; no dual page+panel
- Security VI: no secrets in inventory fixtures; mock toasts; external hand-off only
- Honesty: mock mutations never claim live discovery/core inventory authority
- DESIGN: StatusChip for status; DataTable for collections; no Actions columns; no Card chrome in operational sections
- Lifecycle: stale/missing ≠ deleted (`docs/spec.md` §111)

**Implementation note**: Core mock delivery is already landed on the working tree. Tasks use `[x]` when the artifact exists and tests cover it; remaining open work is polish/validation/optional hand-off wiring.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: US1–US5 maps to [spec.md](./spec.md) user stories
- Include exact file paths in every task description

## Path Conventions (this feature)

- Resources feature: `packages/features/resources/src/`
- Types / fixtures: `packages/features/resources/src/{types,fixtures}.ts`
- Status helpers: `packages/features/resources/src/resource-status.ts`
- Detail composition: `packages/features/resources/src/resource-detail.tsx`
- Inventory list: `packages/features/resources/src/resources-index.tsx`
- Feature entry: `packages/features/resources/src/index.tsx`
- Map helpers: `packages/features/resources/src/map.ts`
- Desktop fixture gate: `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`
- Contracts: `specs/013-inventory-detail/contracts/inventory-detail-ui.md`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm feature package surface and unit test runner for resources inventory detail

- [x] T001 Confirm feature ownership stays in `packages/features/resources` (no new scenic package) per [plan.md](./plan.md)
- [x] T002 [P] Configure Vitest for pure-TS inventory tests in `packages/features/resources/vitest.config.ts` (node env; include filter/map/provisioning/resource-status)
- [x] T003 [P] Wire resources package test script + `vitest` devDependency in `packages/features/resources/package.json`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Data model + helpers + contract SSOT that all stories consume

**⚠️ CRITICAL**: User story UI work depends on these types/fixtures/helpers

- [x] T004 Extend `Resource` / `ResourceStatus` with lifecycle (`stale`/`missing`), `provenance`, identity ids, `lifecycleNote`, `capabilities`, `metrics`, `externalUrl`, lastSeen/discovered in `packages/features/resources/src/types.ts`
- [x] T005 Enrich `RESOURCE_FIXTURES` with rich (`res-pg`, `res-polter-api`), non-healthy (`degraded`/`stale`/`missing`/`syncing`/`unknown`), and sparse rows in `packages/features/resources/src/fixtures.ts`
- [x] T006 [P] Implement pure status/provenance/source helpers in `packages/features/resources/src/resource-status.ts`
- [x] T007 [P] Keep UI contract SSOT current in `specs/013-inventory-detail/contracts/inventory-detail-ui.md`
- [x] T008 [P] Add helper + fixture contract unit tests in `packages/features/resources/src/resource-status.test.ts`
- [x] T009 Expand inventory richness assertions in `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`
- [x] T010 Align `statusTone` re-export and exports surface in `packages/features/resources/src/map.ts` and `packages/features/resources/src/index.tsx`
- [x] T011 Use shared status helpers for Inventory table StatusChip in `packages/features/resources/src/resources-index.tsx`

**Checkpoint**: Fixture resource model + helpers ready; stories can compose DetailPanel independently

---

## Phase 3: User Story 1 - Inspect inventory identity and health (Priority: P1) 🎯 MVP

**Goal**: Operator activates an Inventory row and inspects identity + lifecycle health metrics in the global DetailPanel without a second page.

**Independent Test**: Resources → Inventory → select `prod-postgres` and a stale/degraded fixture → DetailPanel shows name/icon + StatusChip metrics + lifecycle note when present.

### Tests for User Story 1 (risk-proportional)

- [x] T012 [P] [US1] Unit-map lifecycle tones/labels (including stale→degraded, missing→down, syncing) in `packages/features/resources/src/resource-status.test.ts`
- [x] T013 [P] [US1] Assert rich primary fixture has status/region/owner and non-healthy set includes stale/missing/degraded in `packages/features/resources/src/resource-status.test.ts` and `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`

### Implementation for User Story 1

- [x] T014 [US1] Elevate `ResourceDetail` with loading/error/not-found/ready branches using `PageLoading`/`PageError` in `packages/features/resources/src/resource-detail.tsx`
- [x] T015 [US1] Render header with `ResourceTableIcon` + single-line name in `packages/features/resources/src/resource-detail.tsx`
- [x] T016 [US1] Render summary metrics (Status StatusChip, Provider, Environment, Last sync/Never) in `packages/features/resources/src/resource-detail.tsx`
- [x] T017 [US1] Render Lifecycle section when `lifecycleNote` present in `packages/features/resources/src/resource-detail.tsx`
- [x] T018 [US1] Keep activation `scopeId: resources:inventory` + `entityKind: resource` wiring in `packages/features/resources/src/index.tsx` (no second inspector)
- [x] T019 [US1] Mount elevated `ResourceDetail` from `ResourcesView` when inventory resource target is active in `packages/features/resources/src/index.tsx`

**Checkpoint**: US1 MVP — identity + health inspectable for fixture inventory rows

---

## Phase 4: User Story 2 - Understand provenance and provider context (Priority: P1)

**Goal**: Operator sees provenance, provider kind, external/provider ids, source honesty, and external console hand-off separate from annotations.

**Independent Test**: Open discovered RDS fixture vs manual sparse fixture → provenance/ids/source differ; Open provider is external-only.

### Tests for User Story 2

- [x] T020 [P] [US2] Assert provenance/source labels and rich fixture externalKey/provenance in `packages/features/resources/src/resource-status.test.ts`
- [x] T021 [P] [US2] Assert desktop coverage requires provenance + externalKey on primary inventory fixture in `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`

### Implementation for User Story 2

- [x] T022 [US2] Render Overview metadata (type, provider kind, region, owner, provenance, last seen, discovered, source honesty) in `packages/features/resources/src/resource-detail.tsx`
- [x] T023 [US2] Render Identity metadata (resource id, external key, provider resource id, mono) in `packages/features/resources/src/resource-detail.tsx`
- [x] T024 [US2] Render Annotations section (notes + tags + survive-refresh copy) separate from provider identity in `packages/features/resources/src/resource-detail.tsx`
- [x] T025 [US2] Implement Open provider header action from `externalUrl` / external quick action href via `window.open` (honest toast when missing) in `packages/features/resources/src/resource-detail.tsx`
- [x] T026 [US2] Author discovered vs manual fixture provenance examples in `packages/features/resources/src/fixtures.ts`

**Checkpoint**: US2 provenance and provider context inspectable without console clone

---

## Phase 5: User Story 3 - Review linked architecture and activity (Priority: P1)

**Goal**: Linked architecture and activity DataTables provide system context with honest empties and optional architecture hand-off.

**Independent Test**: Open `prod-postgres`/`polter-api` with links+activity; open sparse row → empty tables, no fake nodes.

### Tests for User Story 3

- [x] T027 [P] [US3] Assert rich fixtures include linkedArchitecture + activity arrays in `packages/features/resources/src/resource-status.test.ts` and `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`

### Implementation for User Story 3

- [x] T028 [US3] Render Linked architecture DataTable (name, type, domain icons; no Actions column; empty honest) in `packages/features/resources/src/resource-detail.tsx`
- [x] T029 [US3] Render Activity DataTable (event, when; no Actions column; empty honest) in `packages/features/resources/src/resource-detail.tsx`
- [x] T030 [US3] Activate architecture row via optional `onOpenArchitecture` or mock-honest toast in `packages/features/resources/src/resource-detail.tsx`
- [x] T031 [US3] Populate linkedArchitecture/activity on rich fixtures and leave sparse empty in `packages/features/resources/src/fixtures.ts`

**Checkpoint**: US3 architecture + activity context available from inventory detail

---

## Phase 6: User Story 4 - Act only through honest capabilities (Priority: P1)

**Goal**: Header + capabilities + quick actions are capability-gated; disabled reasons always present; mock refresh/edit never claim live provider mutation.

**Independent Test**: On `prod-postgres`, Open provider enabled; health check disabled with reason; Edit opens form path; Refresh toast is mock-honest.

### Tests for User Story 4

- [x] T032 [P] [US4] Assert every disabled quick action has `disabledReason` and every unavailable capability has `reason` across fixtures in `packages/features/resources/src/resource-status.test.ts`
- [x] T033 [P] [US4] Assert desktop coverage requires disabled quick action + unavailable capability on rich fixture in `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`

### Implementation for User Story 4

- [x] T034 [US4] Render Capabilities DataTable (available/unavailable StatusChip + reason) in `packages/features/resources/src/resource-detail.tsx`
- [x] T035 [US4] Render Quick actions list with disabled controls + reason (title/aria/helper) in `packages/features/resources/src/resource-detail.tsx`
- [x] T036 [US4] Add header Refresh inventory mock toast (or `onRefresh` callback) and Edit primary action in `packages/features/resources/src/resource-detail.tsx`
- [x] T037 [US4] Keep Edit wired to existing resource form dialog path via `onEdit` from `packages/features/resources/src/index.tsx`
- [x] T038 [US4] Author capability + quick-action fixtures including disabled operation paths in `packages/features/resources/src/fixtures.ts`
- [x] T039 [US4] Ensure mock toast copy labels mock / not live discovery in `packages/features/resources/src/resource-detail.tsx`

**Checkpoint**: US4 capability-gated actions honest and reachable

---

## Phase 7: User Story 5 - Sparse and degraded inventory without scenic emptiness (Priority: P2)

**Goal**: Sparse rows, non-healthy statuses, and loading/not-found remain usable chrome with honest empties.

**Independent Test**: Activate sparse/manual/syncing fixtures and missing target states → header + sections + empty copy; StatusChip syncing not healthy.

### Tests for User Story 5

- [x] T040 [P] [US5] Assert fixture set covers degraded/stale/missing/syncing/unknown and lifecycle notes for missing/stale in `packages/features/resources/src/resource-status.test.ts`
- [x] T041 [P] [US5] Assert all fixtures remain `dataSource: "fixture"` in `packages/features/resources/src/resource-status.test.ts`

### Implementation for User Story 5

- [x] T042 [US5] Ensure optional Metrics section only renders when `metrics` present in `packages/features/resources/src/resource-detail.tsx`
- [x] T043 [US5] Keep empty architecture/activity/capabilities/actions copy honest (no invented rows) in `packages/features/resources/src/resource-detail.tsx`
- [x] T044 [US5] Keep loading/error/not-found panel structure in `packages/features/resources/src/resource-detail.tsx`
- [x] T045 [US5] Author sparse + syncing + unknown/manual + missing fixtures in `packages/features/resources/src/fixtures.ts`
- [x] T046 [US5] Pass loading/error/notFound props from `ResourcesView` into `ResourceDetail` in `packages/features/resources/src/index.tsx`

**Checkpoint**: US5 degraded/sparse honesty without scenic emptiness

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Residual wiring, composition contracts, validation, convergence

- [x] T047 [P] Optional: wire `onOpenArchitecture` from `packages/features/resources/src/index.tsx` (and/or `apps/desktop/src/renderer/App.tsx`) to Architecture node activation when product hand-off is desired; keep toast fallback
- [x] T048 [P] Optional: add composition source-contract tests (sections titles, StatusChip usage, empty copy, no Actions columns) in `packages/features/resources/src/resource-detail.test.ts` following access/run detail patterns
- [x] T049 Run resources package tests: `pnpm --filter @polterware/feature-resources test`
- [x] T050 [P] Run desktop fixture coverage: `cd apps/desktop && pnpm exec vitest run --config vitest.ui.config.ts src/renderer/detail-fixture-coverage.test.ts`
- [x] T051 [P] Optional manual quickstart scenarios A–F per [quickstart.md](./quickstart.md) — deferred (no operator-approved `pnpm dev`); automated quickstart gates covered by T049/T050
- [x] T052 Update feature status Draft → Implemented in `specs/013-inventory-detail/spec.md` after validation
- [x] T053 Convergence check: code ↔ spec ↔ plan ↔ tasks ↔ contract (Governance §6)
- [x] T054 [P] Confirm no provider-branch layout (`if provider ===`) in `packages/features/resources/src/resource-detail.tsx` structure
- [x] T055 [P] Optional: pass-through `onRefresh` from App if shell wants coordinated mock refresh toasts; default in-detail toast remains valid

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies
- **Foundational (Phase 2)**: Depends on Setup — **blocks** all user stories (types + fixtures + helpers)
- **US1 (Phase 3)**: Depends on Foundational — **MVP**
- **US2 (Phase 4)**: Depends on US1 DetailPanel shell (same composition)
- **US3 (Phase 5)**: Can parallel with US2 after US1 shell exists
- **US4 (Phase 6)**: Can parallel with US2/US3 after US1 header shell exists
- **US5 (Phase 7)**: Can parallel with US2–US4 after US1 empty/not-found scaffolding
- **Polish (Phase 8)**: After desired stories complete

### User Story Dependencies

| Story | Depends on | Independently testable? |
| --- | --- | --- |
| **US1** Identity + health | Phase 2 | Yes — open any inventory row metrics/lifecycle |
| **US2** Provenance + provider | US1 shell | Yes — Overview/Identity/Annotations/Open provider |
| **US3** Architecture + activity | US1 shell | Yes — tables + empty states |
| **US4** Capabilities + actions | US1 shell | Yes — disabled reasons + mock toasts |
| **US5** Sparse/degraded honesty | US1 state branches | Yes — sparse/non-healthy fixtures |

### Within Each User Story

- Risk-proportional tests first where new rules land
- Types/fixtures before or with UI that consumes them
- Story complete only when constitution gates and required checks hold

### Parallel Opportunities

```text
After Phase 2:
  US2 Overview/Identity  ||  US3 Architecture/Activity  ||  US4 Capabilities/Actions
US5 fixtures + empty honesty can track alongside US2–US4
Polish T047/T048/T050/T054/T055 can run in parallel after stories land
```

---

## Parallel Example: User Story 1

```bash
# After foundational fixtures/helpers:
# In parallel — pure tests:
Task: T012 lifecycle tone/label unit tests
Task: T013 rich fixture coverage assertions

# Then sequential UI on resource-detail.tsx:
Task: T014 loading/error/not-found branches
Task: T015–T017 header + metrics + lifecycle
Task: T018–T019 ResourcesView mount/wiring
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup  
2. Complete Phase 2: Foundational (CRITICAL)  
3. Complete Phase 3: User Story 1  
4. **STOP and VALIDATE**: open Inventory → any row → metrics + status  
5. Demo MVP if ready  

### Incremental Delivery

1. Setup + Foundational → model ready  
2. US1 → identity/health MVP  
3. US2 → provenance/provider context  
4. US3 → architecture/activity context  
5. US4 → capability-gated actions  
6. US5 → sparse/degraded honesty  
7. Polish → optional hand-off + validation  

### Current tree status

Most of Phases 1–7 are **already implemented** on the working tree from the specify/implement session. Remaining executable work is primarily **Phase 8** (optional architecture wire, optional composition tests, formal validation, status flip, convergence).

---

## Notes

- [P] tasks = different files, no incomplete dependencies
- [Story] labels map to US1–US5 in [spec.md](./spec.md)
- Do not claim live discovery/sync or polter-core inventory authority
- Do not redesign Nodes/Processes details in this feature
- Do not invent a second inventory detail composition
- Commit after logical groups when the operator requests commits
- Implementation ends only after code ↔ spec ↔ plan ↔ tasks convergence
