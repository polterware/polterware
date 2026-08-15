# Tasks: Access Agents Detail

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/

**Tests**: Risk-proportional per Constitution XII — pure helpers for agent status / autonomy / capability effect / tool kind / session status / result tones; composition source contract for DetailPanel LAW; desktop `detail-fixture-coverage` for Deployment Agent completeness; keep `detail-targets` green. Do not mark complete while required tests fail for the changed surface. Agents.md: do not run `pnpm dev` / full build unless the user asks.

**Organization**: Tasks grouped by user story (US1–US4) for independent implementation and testing.

**Constitution / guardrails**:

- DetailPanel LAW only — no second inspector/page/drawer for agents
- Dependency direction IV: `feature-access` → shell mocks / design-system / state
- SSoT XIV: single `AgentDetail` composition; no dual inline detail
- Security VI: capability/tool tables display-only; mock toasts; no secrets/tokens
- Honesty: mock mutations never claim live core / IdentityService
- Parallel ownership: NEVER edit policy/audit/session detail files or non-agent mock arrays (except read sessions)
- DESIGN: StatusChip for status; DataTable for collections; no PrimaryCell descriptions; no Actions columns

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: US1–US4 maps to [spec.md](./spec.md) user stories
- Include exact file paths in every task description

## Path Conventions (this feature)

- Access feature: `packages/features/access/src/`
- Agent detail: `packages/features/access/src/agent-detail.tsx`
- Helpers: `packages/features/access/src/agent-detail-helpers.ts`
- Access index: `packages/features/access/src/index.tsx` (agent mount only)
- Shell mocks: `packages/features/shell/src/workspace-mocks/{types,data,index}.ts` (agent fields only)
- Desktop fixture gate: `apps/desktop/src/renderer/detail-fixture-coverage.test.ts` (agent asserts only)
- Contracts: `specs/017-agents-detail/contracts/agents-detail-ui.md`

**FORBIDDEN paths**: `policy-detail*`, `audit-detail*`, `agent-session-detail*`, `member-detail*`, `team-detail*`, `POLICY_MOCKS`, `AUDIT_EVENT_MOCKS`, rewrite of `AGENT_SESSION_MOCKS`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm feature package surface and test runner for access

- [x] T001 Confirm feature ownership stays in `packages/features/access` (no new scenic package) per [plan.md](./plan.md)
- [x] T002 [P] Confirm Vitest config for access package in `packages/features/access/vitest.config.ts`
- [x] T003 [P] Confirm access package test script + `vitest` devDependency in `packages/features/access/package.json`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Data model + helpers + contract SSOT that all stories consume

**⚠️ CRITICAL**: User story UI work depends on these fixtures/helpers

- [x] T004 Extend `WorkspaceAgent` with structured detail fields and nested types (`WorkspaceAgentCapability`, `WorkspaceAgentTool`, `WorkspaceAgentActivityEvent`; optional `description`, `createdAt`, `activity`, `notes`) in `packages/features/shell/src/workspace-mocks/types.ts`
- [x] T005 Export new agent nested types from `packages/features/shell/src/workspace-mocks/index.ts` and `packages/features/shell/src/index.tsx` if re-exported
- [x] T006 Enrich `AGENT_MOCKS` (Deployment `a1` complete; Architecture `a2`; GitHub Sync `a3`; Linear Bridge `a4` Paused) with structured capabilities/tools and optional activity/notes in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T007 [P] Implement pure agent status/autonomy/effect/tool-kind/session/result helpers in `packages/features/access/src/agent-detail-helpers.ts`
- [x] T008 [P] Keep UI contract SSOT current in `specs/017-agents-detail/contracts/agents-detail-ui.md`
- [x] T009 [P] Keep data model SSOT current in `specs/017-agents-detail/data-model.md`
- [x] T010 [P] Add helper unit tests in `packages/features/access/src/agent-detail.test.ts`
- [x] T011 Expand Deployment Agent completeness assertions in `apps/desktop/src/renderer/detail-fixture-coverage.test.ts` (agent fields only)

**Checkpoint**: Fixture agent model + helpers ready; stories can compose DetailPanel independently

---

## Phase 3: User Story 1 - Inspect a project agent (Priority: P1) 🎯 MVP

**Goal**: Operator activates an Agents row and inspects full agent posture in the global DetailPanel.

**Independent Test**: Access → Agents → activate any fixture agent → DetailPanel shows name/icon + Overview + structured collections without a second page.

### Tests for User Story 1

- [x] T012 [P] [US1] Add composition source-contract assertions (sections layout, aria-labels, empty copy, no PrimaryCell `description=`) in `packages/features/access/src/agent-detail.test.ts`
- [x] T013 [P] [US1] Assert Access wires `AgentDetail` for `entityKind === "agent"` via source checks in `packages/features/access/src/agent-detail.test.ts`

### Implementation for User Story 1

- [x] T014 [US1] Create `AgentDetail` composition with loading/error/not-found/ready branches in `packages/features/access/src/agent-detail.tsx`
- [x] T015 [US1] Render metrics (Status StatusChip, Autonomy, Scope, Environments count) in `packages/features/access/src/agent-detail.tsx`
- [x] T016 [US1] Render Overview metadata (type, status, autonomy, owner, scope, last activity, optional description/created) in `packages/features/access/src/agent-detail.tsx`
- [x] T017 [US1] Render Capabilities DataTable (name, effect StatusChip) in `packages/features/access/src/agent-detail.tsx`
- [x] T018 [US1] Render Tools DataTable (name, kind) in `packages/features/access/src/agent-detail.tsx`
- [x] T019 [US1] Render Environments list section in `packages/features/access/src/agent-detail.tsx`
- [x] T020 [US1] Render Linked operations collection (ids → onOpenOperation) in `packages/features/access/src/agent-detail.tsx`
- [x] T021 [US1] Render Recent sessions DataTable from `props.sessions` with empty state in `packages/features/access/src/agent-detail.tsx`
- [x] T022 [US1] Render optional Activity DataTable and Notes when present in `packages/features/access/src/agent-detail.tsx`
- [x] T023 [US1] Replace inline agent detail block with `AgentDetail` mount for `access:agents` + `agent` in `packages/features/access/src/index.tsx` only

**Checkpoint**: US1 MVP — full inspection surface for fixture agents

---

## Phase 4: User Story 2 - Govern agent from detail (Priority: P1)

**Goal**: Header mock governance actions with honest toast feedback; no second inspector.

**Independent Test**: Open agent detail → Pause/Resume / Edit permissions / Revoke → mock toasts; panel stays on agent.

### Tests for User Story 2

- [x] T024 [P] [US2] Assert header action labels exist in composition contract in `packages/features/access/src/agent-detail.test.ts`
- [x] T025 [P] [US2] Assert Access wires mock toast handlers in `packages/features/access/src/index.tsx` via source checks in `packages/features/access/src/agent-detail.test.ts`

### Implementation for User Story 2

- [x] T026 [US2] Add DetailPanel header actions Pause/Resume, Edit permissions, Revoke (danger) in `packages/features/access/src/agent-detail.tsx`
- [x] T027 [US2] Wire mock `pushToast` handlers for pause/resume, edit permissions, revoke in `packages/features/access/src/index.tsx`
- [x] T028 [US2] Keep toast copy explicitly labeled `(mock)` in `packages/features/access/src/index.tsx`

**Checkpoint**: US2 mock governance actions honest and reachable

---

## Phase 5: User Story 3 - Navigate to sessions and operations (Priority: P2)

**Goal**: Session and operation hand-offs via callbacks only; no session detail UI rewrite.

**Independent Test**: Deployment Agent → Open recent session / session row → agent-session activation; operation id → onOpenOperation.

### Tests for User Story 3

- [x] T029 [P] [US3] Assert Deployment agent has session link coverage in `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`
- [x] T030 [P] [US3] Assert `onOpenSession` / `onOpenOperation` wiring exists in `packages/features/access/src/index.tsx` via `packages/features/access/src/agent-detail.test.ts`

### Implementation for User Story 3

- [x] T031 [US3] Pass filtered `AGENT_SESSION_MOCKS` by agent id into `AgentDetail` sessions prop from `packages/features/access/src/index.tsx`
- [x] T032 [US3] Activate session row to call `onOpenSession(session.id)` in `packages/features/access/src/agent-detail.tsx`
- [x] T033 [US3] Wire `onOpenSession={(id) => activate(id, "agent-session")}` from `packages/features/access/src/index.tsx`
- [x] T034 [US3] Wire Open recent session header action to first/most recent related session in `packages/features/access/src/agent-detail.tsx`
- [x] T035 [US3] Wire operation row activate → `onOpenOperation` and host `props.onOpenOperation` in `packages/features/access/src/index.tsx`

**Checkpoint**: US3 hand-offs without sibling UI ownership

---

## Phase 6: User Story 4 - Honest empty, missing, and partial states (Priority: P2)

**Goal**: Not-found, empty collections, and mock feedback stay honest; agent detail only for `entityKind === "agent"`.

**Independent Test**: Unknown agent id → Agent not found; empty operations → empty copy; session activation does not mount AgentDetail.

### Tests for User Story 4

- [x] T036 [P] [US4] Assert PageLoading / PageError / not-found copy in composition contract in `packages/features/access/src/agent-detail.test.ts`
- [x] T037 [P] [US4] Assert empty collection copy strings present in `packages/features/access/src/agent-detail.tsx` via tests
- [x] T038 [P] [US4] Assert mount condition uses agent entityKind (not bare truthy agent only if it would clash) in `packages/features/access/src/index.tsx` via tests

### Implementation for User Story 4

- [x] T039 [US4] Implement loading / error / not-found branches with PageLoading / PageError in `packages/features/access/src/agent-detail.tsx`
- [x] T040 [US4] Empty states for capabilities, tools, environments, operations, sessions, activity in `packages/features/access/src/agent-detail.tsx`
- [x] T041 [US4] Mount `AgentDetail` for agent entityKind with `notFound={!agent()}` when scope is agents+agent in `packages/features/access/src/index.tsx`

**Checkpoint**: US4 honest states complete

---

## Phase 7: Polish & Cross-Cutting

- [x] T042 [P] Re-export helpers from `agent-detail.tsx` if matching member/team pattern
- [x] T043 Run `pnpm test` in `packages/features/access` and fix failures for agent-detail only
- [x] T044 Confirm no forbidden sibling files modified (`policy-detail*`, `audit-detail*`, `agent-session-detail*`)
- [x] T045 Mark Spec Kit status Implemented in `specs/017-agents-detail/spec.md` when green

---

## Dependencies

```text
Phase 1 → Phase 2 → Phase 3 (US1) → Phase 4 (US2)
                              ↘ Phase 5 (US3)
                              ↘ Phase 6 (US4)
Phase 3–6 → Phase 7
```

## Implementation Strategy

1. Land fixtures + helpers first (T004–T011).
2. Build full `AgentDetail` composition (T014–T022).
3. Wire Access mount + toasts + callbacks (T023, T027–T028, T031–T035, T041).
4. Lock with tests and fixture coverage (T010–T013, T024–T025, T029–T030, T036–T038, T043).

**MVP**: Phase 1–3 (inspect) + Phase 4 (govern) delivers production-looking agent detail; US3/US4 complete the hand-off and honesty bar.
