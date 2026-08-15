# Tasks: Resources Nodes Detail

**Input**: Design documents from `specs/014-nodes-detail/`

**Prerequisites**: [plan.md](./plan.md) (required), [spec.md](./spec.md) (user stories), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/node-detail-ui.md](./contracts/node-detail-ui.md), [quickstart.md](./quickstart.md)

**Tests**: Risk-proportional per Constitution XII (status helpers unit tests, fixture richness, composition contract, desktop detail-fixture-coverage). Spec FR-015/FR-018 and plan explicitly require tests.

**Organization**: Tasks grouped by user story for independent implementation and testing.

**Constitution**: Dependency direction (IV), contracts-first UI contract (V), security no secrets (VI), single NodeDetail SSOT (XIV), extend existing package only (XIII).

**Implementation status**: Core mock production-ready UI complete. Residual T040/T042 done; open only T036 (live loading), T041 (operator visual), T043 (future registry guardrail).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: User story label (US1–US5)
- Include exact file paths in descriptions

## Path Conventions

- Feature owner: `packages/features/resources/src/`
- Desktop coverage: `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`
- No new packages; no backend/polter-node protocol in this feature

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm feature package surface and design docs for node detail work

- [x] T001 Confirm feature package layout and existing Nodes list entry points in `packages/features/resources/src/index.tsx` and `packages/features/resources/src/nodes-index.tsx`
- [x] T002 [P] Confirm design-system DetailPanel / DataTable / StatusChip / HardDrive availability via `@polterware/frontend-design-system`
- [x] T003 [P] Freeze UI contract at `specs/014-nodes-detail/contracts/node-detail-ui.md` and data model at `specs/014-nodes-detail/data-model.md`
- [x] T004 [P] Point active feature at `specs/014-nodes-detail` via `.specify/feature.json`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Model, status helpers, and fixture SSOT that ALL user stories share

**⚠️ CRITICAL**: User story UI work depends on these types/helpers/fixtures

- [x] T005 Expand `NodeHealthStatus` and `TargetNode` fields (`nodeId`, `connection`, `healthNote`, capability objects, `upgradeAvailable`) in `packages/features/resources/src/node-types.ts`
- [x] T006 [P] Implement status helpers (`nodeStatusLabel`, `nodeStatusTone`, `isNodeTerminalAvailable`, `isNodeDispatchAvailable`, `capabilityRows`, `resolveNodeId`, connection/result tones) in `packages/features/resources/src/node-status.ts`
- [x] T007 Enrich `NODE_FIXTURES` with rich online, degraded, draining, offline, upgrading, revoked, and sparse unknown shapes (all `dataSource: "fixture"`) in `packages/features/resources/src/node-types.ts`
- [x] T008 [P] Wire list StatusChip mapping through helpers and sortable columns in `packages/features/resources/src/nodes-index.tsx`
- [x] T009 Export node types/helpers/`NodeDetail` from `packages/features/resources/src/index.tsx`
- [x] T010 Include `src/node-detail.test.ts` in package Vitest config `packages/features/resources/vitest.config.ts`

**Checkpoint**: Foundation ready — NodeDetail composition can implement all stories against fixtures

---

## Phase 3: User Story 1 - Inspect node identity and health (Priority: P1) 🎯 MVP

**Goal**: Operator activates a Nodes row and sees global DetailPanel with display name, domain icon, health StatusChip, last heartbeat, version, capability count, and identity honesty (node id ≠ name).

**Independent Test**: Resources → Nodes → select `nitro` and `legacy-node`; metrics differ by StatusChip tone; identity shows distinct node id; Source is fixture.

### Tests for User Story 1

- [x] T011 [P] [US1] Unit tests for `nodeStatusLabel` / `nodeStatusTone` / `resolveNodeId` in `packages/features/resources/src/node-detail.test.ts`
- [x] T012 [P] [US1] Fixture contract test for rich online + offline honesty in `packages/features/resources/src/node-detail.test.ts`
- [x] T013 [P] [US1] Expand desktop coverage assertions for node identity/machine/connection in `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`

### Implementation for User Story 1

- [x] T014 [US1] Harden `NodeDetail` loading/error/not-found + header HardDrive + metrics (Status · Last heartbeat · Version · Capabilities) in `packages/features/resources/src/node-detail.tsx`
- [x] T015 [US1] Render Identity section (node id mono, display name, hostname, location, OS, arch, IP, status chip, Source honesty) in `packages/features/resources/src/node-detail.tsx`
- [x] T016 [US1] Wire Nodes collection target `scopeId: resources:nodes`, `entityKind: node` and detail mount in `packages/features/resources/src/index.tsx`

**Checkpoint**: US1 independently testable — list activate opens panel with health metrics + identity

---

## Phase 4: User Story 2 - Connection, machine, and identity security (Priority: P1)

**Goal**: Operator sees connection model (outbound), machine facts, health notes, and revoked/draining posture without secrets.

**Independent Test**: `nitro` shows machine + connection; `node-revoked` / `node-staging` show explicit lifecycle and no secret material.

### Tests for User Story 2

- [x] T017 [P] [US2] Assert connection/modelNote and healthNote presence on lifecycle fixtures in `packages/features/resources/src/node-detail.test.ts`
- [x] T018 [P] [US2] Composition contract asserts Source fixture honesty and DetailPanel Metadata usage in `packages/features/resources/src/node-detail.test.ts`

### Implementation for User Story 2

- [x] T019 [US2] Render Machine section (CPU/memory/storage/arch) when facts present in `packages/features/resources/src/node-detail.tsx`
- [x] T020 [US2] Render Connection & health section (connection chip, last connected, model note, health note, upgrade available, agent version) in `packages/features/resources/src/node-detail.tsx`
- [x] T021 [US2] Ensure fixtures never include cert/secret fields in `packages/features/resources/src/node-types.ts`

**Checkpoint**: US2 independently testable — connection/machine/security honesty without dual detail host

---

## Phase 5: User Story 3 - Capabilities, assigned resources, recent operations (Priority: P1)

**Goal**: DataTables for capabilities (with authorization when modeled), assigned resources, and recent operations with honest empties and no Actions columns.

**Independent Test**: `nitro` full tables; sparse `node-sparse` empty states; no fabricated rows.

### Tests for User Story 3

- [x] T022 [P] [US3] Unit tests for `capabilityRows` string vs object normalization in `packages/features/resources/src/node-detail.test.ts`
- [x] T023 [P] [US3] Composition contract asserts aria-labels and empty copy for three tables in `packages/features/resources/src/node-detail.test.ts`

### Implementation for User Story 3

- [x] T024 [US3] Capabilities DataTable (name + Authorization chip) without Actions column in `packages/features/resources/src/node-detail.tsx`
- [x] T025 [US3] Assigned resources DataTable (name/type/status chip + domain icon) in `packages/features/resources/src/node-detail.tsx`
- [x] T026 [US3] Recent operations DataTable (command/result chip/actor/when) in `packages/features/resources/src/node-detail.tsx`

**Checkpoint**: US3 independently testable — tables + empties match contract

---

## Phase 6: User Story 4 - Honest capability-gated actions (Priority: P1)

**Goal**: Open terminal / Run command only when health allows; never claim live dispatch ACK.

**Independent Test**: `nitro` shows both actions; offline/revoked/draining/upgrading hide them; callbacks remain App-owned mocks.

### Tests for User Story 4

- [x] T027 [P] [US4] Unit tests for `isNodeTerminalAvailable` / `isNodeDispatchAvailable` across health states in `packages/features/resources/src/node-detail.test.ts`
- [x] T028 [P] [US4] Composition contract asserts Open terminal / Run command labels in `packages/features/resources/src/node-detail.test.ts`

### Implementation for User Story 4

- [x] T029 [US4] Gate header `DetailPanel.Action` visibility with terminal/dispatch helpers in `packages/features/resources/src/node-detail.tsx`
- [x] T030 [US4] Wire `onTerminal` / `onRunCommand` callbacks from `packages/features/resources/src/index.tsx` without inventing live node protocol success

**Checkpoint**: US4 independently testable — action gates match §28 revoked/offline invariants

---

## Phase 7: User Story 5 - Sparse and lifecycle edge states (Priority: P2)

**Goal**: Sparse, lifecycle, loading, and not-found states never blank the panel.

**Independent Test**: Activate `edge-unknown`, degraded/upgrading fixtures; force not-found target; loading branch present.

### Tests for User Story 5

- [x] T031 [P] [US5] Fixture coverage for degraded/draining/upgrading/revoked/unknown set in `packages/features/resources/src/node-detail.test.ts`
- [x] T032 [P] [US5] Desktop coverage asserts multi-lifecycle presence in `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`

### Implementation for User Story 5

- [x] T033 [US5] Loading / not-found / error branches with PageLoading / PageError in `packages/features/resources/src/node-detail.tsx`
- [x] T034 [US5] notFound wiring when Nodes scope has missing entity in `packages/features/resources/src/index.tsx`
- [x] T035 [US5] Sparse fixture `node-sparse` with honest — / empty tables in `packages/features/resources/src/node-types.ts`
- [ ] T036 [US5] Optional: pass explicit `loading` prop from Resources host when nodes catalog gains real async path in `packages/features/resources/src/index.tsx` (**deferred** — fixtures-only path has no real async; do not invent fake loading)

**Checkpoint**: US5 edge states honest; optional loading prop remains for live swap

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Convergence, docs, residual product polish

- [x] T037 [P] Design docs complete (`plan.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/node-detail-ui.md`) under `specs/014-nodes-detail/`
- [x] T038 [P] Package tests green: `pnpm --filter @polterware/feature-resources test`
- [x] T039 [P] Desktop fixture coverage green: `apps/desktop` `detail-fixture-coverage.test.ts`
- [x] T040 Align shell Terminal target connected treatment with expanded node health gates (`online`|`degraded` usable; offline/revoked/draining/upgrading not connected) in `packages/features/shell/src/types.ts`, `terminal-page.tsx`, `target-picker.tsx`, `terminal-side-panel.tsx`
- [ ] T041 [P] Manual quickstart scenarios A–F from `specs/014-nodes-detail/quickstart.md` when operator requests visual validation (no agent `pnpm dev` by default)
- [x] T042 Convergence note: mark feature status Implemented in `specs/014-nodes-detail/spec.md` after residual T040
- [ ] T043 No live polter-node dual detail component — when core lands, inject DTOs into existing `NodeDetail` only (XIV guardrail; future work, no dual detail component)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies
- **Foundational (Phase 2)**: Depends on Setup — blocks all user stories
- **US1 (Phase 3)**: Depends on Foundational — **MVP**
- **US2–US4 (Phases 4–6)**: Depend on Foundational; can parallelize after US1 header/metrics shell exists (same file serialization on `node-detail.tsx`)
- **US5 (Phase 7)**: Depends on Foundational; fixtures from T007
- **Polish (Phase 8)**: After desired stories complete

### User Story Dependencies

| Story | Depends on | Notes |
| --- | --- | --- |
| US1 (P1) | Phase 2 | MVP inspect path |
| US2 (P1) | Phase 2 | Same `NodeDetail`; sections independent of tables |
| US3 (P1) | Phase 2 | Tables independent of connection section |
| US4 (P1) | Phase 2 + status helpers | Action gates |
| US5 (P2) | Phase 2 + fixtures | Edge states |

### Within Each User Story

- Tests co-located with helpers/fixtures (already green)
- Model/helpers before composition
- Wire into `ResourcesView` after composition exists

### Parallel Opportunities

- T002/T003/T004 after T001
- T006 parallel with T005 completion of types
- T011/T012/T013 tests parallel
- T017/T018, T022/T023, T027/T028, T031/T032 parallel within story
- T037–T039 polish checks parallel
- **Same-file constraint**: T014–T016, T019–T020, T024–T026, T029, T033 serialize on `node-detail.tsx`

---

## Parallel Example: User Story 1

```bash
# Tests in parallel:
Task: "T011 unit status helpers in packages/features/resources/src/node-detail.test.ts"
Task: "T012 fixture contract in packages/features/resources/src/node-detail.test.ts"
Task: "T013 desktop coverage in apps/desktop/src/renderer/detail-fixture-coverage.test.ts"

# Then composition (serialized on one file):
Task: "T014 metrics + header in packages/features/resources/src/node-detail.tsx"
Task: "T015 Identity section in packages/features/resources/src/node-detail.tsx"
Task: "T016 wire ResourcesView in packages/features/resources/src/index.tsx"
```

---

## Parallel Example: User Story 3

```bash
# After foundation fixtures exist:
Task: "T022 capabilityRows tests"
Task: "T023 table aria-label contract tests"
# Then tables in node-detail.tsx (serialized):
Task: "T024 Capabilities table"
Task: "T025 Assigned resources table"
Task: "T026 Recent operations table"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 Setup  
2. Phase 2 Foundational (types, helpers, fixtures, list chips)  
3. Phase 3 US1 (metrics + identity + panel wire)  
4. **STOP and VALIDATE** via T011–T013 + manual nitro/legacy  

### Incremental Delivery (actual sequence used)

1. Setup + Foundational  
2. US1 MVP inspect  
3. US2 connection/machine  
4. US3 tables  
5. US4 action gates  
6. US5 edges  
7. Polish residual (T040–T042)

### Suggested residual scope (open tasks)

| ID | Priority | Why open |
| --- | --- | --- |
| T036 | Low | Live async loading prop not needed while fixtures-only (still deferred) |
| T040 | Done | Shell Terminal aligned: online\|degraded usable |
| T041 | Operator | Visual quickstart when requested |
| T042 | Done | Spec status → Implemented |
| T043 | Future | Live registry swap guardrail — inject into existing NodeDetail only |

---

## Notes

- [P] = different files / no incomplete deps
- [Story] maps to US1–US5 in `spec.md`
- Do not complete while required package tests fail for changed surface
- No permanent dual node detail page + panel
- No scenic packages; no live protocol claims
- Commit after logical groups when operator requests commits
- Convergence: code ↔ spec ↔ plan ↔ tasks — **core delivery complete**; residual open: T036, T041, T043
