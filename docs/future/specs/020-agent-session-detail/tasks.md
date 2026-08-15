# Tasks: Access Agent Session Detail

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/

**Tests**: Risk-proportional per Constitution XII — pure helpers for session/step status tones and terminable gate; composition source contract for DetailPanel LAW; desktop `detail-fixture-coverage` for multi-status + rich primary session; keep `detail-targets` green. Do not mark complete while required tests fail for the changed surface. Agents.md: do not run `pnpm dev` / full build unless the user asks.

**Organization**: Tasks grouped by user story (US1–US4) for independent implementation and testing.

**Constitution / guardrails**:

- DetailPanel LAW only — no second inspector/page/drawer for sessions
- Dependency direction IV: `feature-access` → shell mocks / design-system / charts / state
- SSoT XIV: single `AgentSessionDetail` composition; no dual inline detail
- Security VI: no secrets; terminate mock-only; UI not enforcement
- Parallelism: DO NOT touch `agent-detail.tsx`, `policy-detail*`, `audit-detail*`
- DESIGN: StatusChip for status; DataTable for steps; no PrimaryCell descriptions; no Actions columns

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: US1–US4 maps to [spec.md](./spec.md) user stories
- Include exact file paths in every task description

## Path Conventions (this feature)

- Access feature: `packages/features/access/src/`
- Session detail: `packages/features/access/src/agent-session-detail.tsx`
- Helpers: `packages/features/access/src/agent-session-detail-helpers.ts`
- Tests: `packages/features/access/src/agent-session-detail.test.ts`
- Access index: `packages/features/access/src/index.tsx`
- Detail targets: `packages/features/access/src/detail-targets.ts`
- Shell mocks: `packages/features/shell/src/workspace-mocks/{types,data,index}.ts`
- Desktop fixture gate: `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`
- Contracts: `specs/020-agent-session-detail/contracts/agent-session-detail-ui.md`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm feature package surface and test runner for access

- [x] T001 Confirm feature ownership stays in `packages/features/access` (no new scenic package) per [plan.md](./plan.md)
- [x] T002 [P] Confirm Vitest config for access package in `packages/features/access/vitest.config.ts`
- [x] T003 [P] Confirm access package test script + charts dependency in `packages/features/access/package.json`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Data model + helpers + contract SSOT that all stories consume

**⚠️ CRITICAL**: User story UI work depends on these fixtures/helpers

- [x] T004 Extend `WorkspaceAgentSessionStep` with `Failed`/`Skipped` status and optional `tool`, `action`, `startedAt`, `finishedAt`, `duration` in `packages/features/shell/src/workspace-mocks/types.ts`
- [x] T005 Extend `WorkspaceAgentSession` with `endedAt`, `duration`, `waitingReason`, `approvalId`, `approvalStatus`, `relatedOperationIds`, `relatedAuditIds`, `correlationId`, `actorLabel`, `actorKind`, `dataSource` in `packages/features/shell/src/workspace-mocks/types.ts`
- [x] T006 Enrich `AGENT_SESSION_MOCKS` for Running/Completed/Waiting and add Failed fixture with rich steps in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T007 [P] Implement pure session/step status + terminable + step-count helpers in `packages/features/access/src/agent-session-detail-helpers.ts`
- [x] T008 [P] Keep UI contract SSOT current in `specs/020-agent-session-detail/contracts/agent-session-detail-ui.md`
- [x] T009 [P] Keep data model SSOT current in `specs/020-agent-session-detail/data-model.md`
- [x] T010 [P] Add helper unit tests in `packages/features/access/src/agent-session-detail.test.ts`
- [x] T011 Expand agent session completeness assertions in `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`

**Checkpoint**: Fixture session model + helpers ready; stories can compose DetailPanel independently

---

## Phase 3: User Story 1 - Inspect a session (Priority: P1) 🎯 MVP

**Goal**: Operator activates Recent sessions row and inspects full session posture in global DetailPanel.

**Independent Test**: Access → Agents → Recent sessions → activate any fixture → DetailPanel shows title/icon + overview + steps without a second page.

### Tests for User Story 1

- [x] T012 [P] [US1] Add composition source-contract assertions (sections layout, StatusChip, DataTable steps, DitherSegmentedBar, empty copy) in `packages/features/access/src/agent-session-detail.test.ts`
- [x] T013 [P] [US1] Assert Access wires `AgentSessionDetail` for `entityKind === "agent-session"` via source checks in `packages/features/access/src/agent-session-detail.test.ts`
- [x] T014 [P] [US1] Keep Agents target registration green in `packages/features/access/src/detail-targets.test.ts`

### Implementation for User Story 1

- [x] T015 [US1] Create `AgentSessionDetail` composition with loading/error/not-found/ready branches in `packages/features/access/src/agent-session-detail.tsx`
- [x] T016 [US1] Render overview metrics (status StatusChip, environment, started, duration/ended) in `packages/features/access/src/agent-session-detail.tsx`
- [x] T017 [US1] Render Objective section in `packages/features/access/src/agent-session-detail.tsx`
- [x] T018 [US1] Render Agent identity section (resolved name + id) in `packages/features/access/src/agent-session-detail.tsx`
- [x] T019 [US1] Render step progress metric + `DitherSegmentedBar` in `packages/features/access/src/agent-session-detail.tsx`
- [x] T020 [US1] Render Steps DataTable (label, status chip, tool/action, timing) with empty state in `packages/features/access/src/agent-session-detail.tsx`
- [x] T021 [US1] Render Context / identity honesty (session id, correlation, data source, actor, environment) in `packages/features/access/src/agent-session-detail.tsx`
- [x] T022 [US1] Replace inline `Show when={agentSession()}` block with `AgentSessionDetail` mount in `packages/features/access/src/index.tsx`

**Checkpoint**: US1 MVP — full inspection surface for fixture sessions

---

## Phase 4: User Story 2 - Waiting / governance / related refs (Priority: P1)

**Goal**: Waiting reason, approval linkage, related operation/audit ids, multi-status honesty.

### Tests for User Story 2

- [x] T023 [P] [US2] Assert waiting/approval section copy and related metadata presence in composition contract in `packages/features/access/src/agent-session-detail.test.ts`
- [x] T024 [P] [US2] Assert fixtures cover Waiting with reason/approval and Failed with failed steps in `packages/features/access/src/agent-session-detail.test.ts` and/or desktop coverage

### Implementation for User Story 2

- [x] T025 [US2] Render Waiting / approval section when status Waiting or waitingReason/approval fields present in `packages/features/access/src/agent-session-detail.tsx`
- [x] T026 [US2] Render Related operations / audit ids metadata when present in `packages/features/access/src/agent-session-detail.tsx`
- [x] T027 [US2] Ensure Failed segment included in progress bar when failed steps exist in `packages/features/access/src/agent-session-detail.tsx`

**Checkpoint**: US2 governance and related refs honest

---

## Phase 5: User Story 3 - Header actions (Priority: P2)

**Goal**: Terminate (gated), Open agent, optional Open related operation with mock/host honesty.

### Tests for User Story 3

- [x] T028 [P] [US3] Assert Terminate/Open agent labels and terminable helper in `packages/features/access/src/agent-session-detail.test.ts`
- [x] T029 [P] [US3] Assert Access wires mock toast terminate + onOpenAgent/onOpenOperation in `packages/features/access/src/index.tsx` via source checks

### Implementation for User Story 3

- [x] T030 [US3] Add header actions Terminate (danger, Running/Waiting only), Open agent, optional Open related operation in `packages/features/access/src/agent-session-detail.tsx`
- [x] T031 [US3] Wire handlers in `packages/features/access/src/index.tsx` with `(mock)` toast for terminate and callbacks for agent/operation
- [x] T032 [US3] Keep toast copy explicitly labeled `(mock)` for terminate in `packages/features/access/src/index.tsx`

**Checkpoint**: US3 actions honest and reachable

---

## Phase 6: User Story 4 - Edge states (Priority: P3)

**Goal**: Loading, not-found, empty steps, missing agent name fallback.

### Tests for User Story 4

- [x] T033 [P] [US4] Assert PageLoading / PageError / not-found / empty steps copy in composition contract in `packages/features/access/src/agent-session-detail.test.ts`

### Implementation for User Story 4

- [x] T034 [US4] Ensure loading/error/not-found branches complete in `packages/features/access/src/agent-session-detail.tsx`
- [x] T035 [US4] Fallback agent name when lookup missing in `packages/features/access/src/agent-session-detail.tsx` / helpers
- [x] T036 [US4] Wire `notFound={!agentSession()}` when target is agent-session in `packages/features/access/src/index.tsx`

**Checkpoint**: US4 edge honesty complete

---

## Phase 7: Polish & Cross-Cutting

- [x] T037 Confirm no modifications to `agent-detail.tsx` / policy-detail / audit-detail for this feature (boundary check)
- [x] T038 [P] Run `pnpm --filter @polterware/feature-access test` and fix failures
- [x] T039 [P] Run desktop `detail-fixture-coverage` vitest for session asserts and fix failures
- [x] T040 Mark [spec.md](./spec.md) status Implemented when validation green

---

## Dependencies & Execution Order

1. Phase 1 Setup  
2. Phase 2 Foundational (blocks UI stories)  
3. Phase 3 US1 MVP  
4. Phase 4 US2 (can start after T015 skeleton)  
5. Phase 5 US3 (after header skeleton)  
6. Phase 6 US4 (can parallel with US2 after loading branches exist)  
7. Phase 7 Polish  

**Parallel opportunities**: T002/T003; T007–T010; composition tests vs implementation once files exist; T038/T039 after code complete.

## Implementation Strategy

1. Enrich types + fixtures first (unblocks tests and UI).  
2. Ship helpers + tests.  
3. Build `AgentSessionDetail` to Run-detail quality.  
4. Thin-wire Access index only for agent-session Show.  
5. Expand fixture coverage; run package + desktop focused tests.  

## Notes

- Do not claim live agent runtime.  
- Do not redesign Agents identity table beyond necessary wiring.  
- Chat with Erick in Portuguese; code/UI/docs English.
