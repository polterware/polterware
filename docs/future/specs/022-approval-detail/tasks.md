# Tasks: Inbox Approval Detail

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/

**Tests**: Risk-proportional per Constitution XII — pure helpers for risk/status/requester-kind tones; composition source contract for DetailPanel LAW; desktop `detail-fixture-coverage` for approval completeness. Do not mark complete while required tests fail for the changed surface. Agents.md: do not run `pnpm dev` / full build unless the user asks.

**Organization**: Tasks grouped by user story (US1–US4) for independent implementation and testing.

**Constitution / guardrails**:

- DetailPanel LAW only — no second inspector/page/drawer for approvals
- Dependency direction IV: `feature-shell` → design-system / state
- SSoT XIV: single `ApprovalDetail` composition; no dual thin inline detail
- Security VI: mock/session decide is not privilege escalation
- Honesty: never claim live policy engine or remote apply
- Parallel ownership: NEVER edit process-detail, architecture view/components, invitation-detail, access agent/policy/audit modules, or reimplement run-detail
- DESIGN: StatusChip for risk and status; English UI strings

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: US1–US4 maps to [spec.md](./spec.md) user stories
- Include exact file paths in every task description

## Path Conventions (this feature)

- Shell feature: `packages/features/shell/src/`
- Approval detail: `packages/features/shell/src/approval-detail.tsx`
- Helpers: `packages/features/shell/src/approval-detail-helpers.ts`
- Tests: `packages/features/shell/src/approval-detail.test.ts`
- Inbox host: `packages/features/shell/src/inbox-page.tsx`
- Shell mocks: `packages/features/shell/src/workspace-mocks/{types,data,index}.ts` (approval fields only)
- Shell exports: `packages/features/shell/src/index.tsx` (export ApprovalDetail if needed)
- Desktop fixture gate: `apps/desktop/src/renderer/detail-fixture-coverage.test.ts` (approval asserts only)
- Contracts: `specs/022-approval-detail/contracts/approval-detail-ui.md`

**FORBIDDEN paths**: process-detail*, architecture view/components detail*, invitation-detail*, agent-detail*, policy-detail*, audit-detail*, run-detail.tsx (link only)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm feature package surface and test runner for shell

- [x] T001 Confirm feature ownership stays in `packages/features/shell` (no new scenic package) per [plan.md](./plan.md)
- [x] T002 [P] Confirm Vitest config for shell package in `packages/features/shell/vitest.config.ts`
- [x] T003 [P] Confirm shell package test script + `vitest` devDependency in `packages/features/shell/package.json`
- [x] T004 Point active feature dir to `specs/022-approval-detail` in `.specify/feature.json`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Data model + helpers + contract SSOT that all stories consume

**⚠️ CRITICAL**: User story UI work depends on these fixtures/helpers

- [x] T005 Confirm `WorkspaceApproval` fields cover requester kind, risk, decision trail, related entities in `packages/features/shell/src/workspace-mocks/types.ts`
- [x] T006 Ensure `APPROVAL_MOCKS` has pending + approved + rejected, ≥1 High risk, ≥1 agent-requested, decided trail on non-pending in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T007 [P] Implement pure risk/status/requester-kind label + tone helpers in `packages/features/shell/src/approval-detail-helpers.ts`
- [x] T008 [P] Keep UI contract SSOT current in `specs/022-approval-detail/contracts/approval-detail-ui.md`
- [x] T009 [P] Keep data model SSOT current in `specs/022-approval-detail/data-model.md`
- [x] T010 [P] Add helper unit tests in `packages/features/shell/src/approval-detail.test.ts`
- [x] T011 Expand approval completeness assertions in `apps/desktop/src/renderer/detail-fixture-coverage.test.ts` (approval fields only)

**Checkpoint**: Fixture approval model + helpers ready; stories can compose DetailPanel independently

---

## Phase 3: User Story 1 - Inspect a pending approval (Priority: P1) 🎯 MVP

**Goal**: Operator activates a Pending row and inspects full request context in the global DetailPanel.

**Independent Test**: Inbox → Pending → activate any pending fixture → DetailPanel shows title/icon + Overview StatusChips + Request context + Justification without a second page.

### Tests for User Story 1

- [x] T012 [P] [US1] Add composition source-contract assertions (sections layout, StatusChip, PageLoading/PageError, Request context, Justification) in `packages/features/shell/src/approval-detail.test.ts`
- [x] T013 [P] [US1] Assert Inbox wires `ApprovalDetail` (not only inline body) via source checks in `packages/features/shell/src/approval-detail.test.ts`

### Implementation for User Story 1

- [x] T014 [US1] Create `ApprovalDetail` composition with loading/error/not-found/ready branches in `packages/features/shell/src/approval-detail.tsx`
- [x] T015 [US1] Render Overview metrics (risk StatusChip, status StatusChip, environment) in `packages/features/shell/src/approval-detail.tsx`
- [x] T016 [US1] Render Request context metadata (requester, kind, component/node/resource, policy, when, run id) in `packages/features/shell/src/approval-detail.tsx`
- [x] T017 [US1] Render Justification body section in `packages/features/shell/src/approval-detail.tsx`
- [x] T018 [US1] Header domain icon + request title in `packages/features/shell/src/approval-detail.tsx`
- [x] T019 [US1] Replace inline approval detail block with `ApprovalDetail` mount in `packages/features/shell/src/inbox-page.tsx`
- [x] T020 [US1] Export `ApprovalDetail` (and helpers if public) from `packages/features/shell/src/index.tsx` if other packages need them

**Checkpoint**: US1 MVP — full inspection surface for fixture approvals

---

## Phase 4: User Story 2 - Decide approve or reject (Priority: P1)

**Goal**: Header Approve/Reject for pending; session-local status + decision trail; no live policy/apply claim.

**Independent Test**: Open pending → Approve/Reject → item leaves pending posture; decided view shows Decision without live apply claim.

### Tests for User Story 2

- [x] T021 [P] [US2] Assert pending header actions Approve + Reject and primary/danger tones in composition contract in `packages/features/shell/src/approval-detail.test.ts`
- [x] T022 [P] [US2] Assert helpers expose `isApprovalPending` / decide gate in `packages/features/shell/src/approval-detail.test.ts`

### Implementation for User Story 2

- [x] T023 [US2] Show DetailPanel.Action Reject + Approve (primary) only when pending in `packages/features/shell/src/approval-detail.tsx`
- [x] T024 [US2] Wire session-local decide (status + decidedBy/decidedAt/decisionNote) from `packages/features/shell/src/inbox-page.tsx` into ApprovalDetail callbacks
- [x] T025 [US2] Keep honesty: no remote apply success copy beyond fixture `runResult` in `packages/features/shell/src/approval-detail.tsx`

**Checkpoint**: US2 mock decide honest and reachable

---

## Phase 5: User Story 3 - Inspect decided approvals and related run (Priority: P2)

**Goal**: Approved/Rejected show Decision trail; optional open-run hand-off; hide live approve/reject.

**Independent Test**: Open approved fixture with decision fields → Decision complete; no Approve/Reject; run id/result visible when present.

### Tests for User Story 3

- [x] T026 [P] [US3] Assert Decision section labels and conditional actions in composition contract in `packages/features/shell/src/approval-detail.test.ts`

### Implementation for User Story 3

- [x] T027 [US3] Render Decision section (decidedBy, decidedAt, decisionNote, optional runResult) for non-pending in `packages/features/shell/src/approval-detail.tsx`
- [x] T028 [US3] Optional Open related run action via `onOpenRun` when `runId` present in `packages/features/shell/src/approval-detail.tsx`
- [x] T029 [US3] Ensure decided fixtures remain rich in `packages/features/shell/src/workspace-mocks/data.ts`

**Checkpoint**: US3 history inspection complete

---

## Phase 6: User Story 4 - Honest empty / missing / partial states (Priority: P2)

**Goal**: loading, not-found, error, sparse related entities do not crash.

**Independent Test**: Force notFound/loading/error props; sparse component/node/resource show honest placeholders.

### Tests for User Story 4

- [x] T030 [P] [US4] Assert PageLoading / PageError / not-found copy in composition contract in `packages/features/shell/src/approval-detail.test.ts`

### Implementation for User Story 4

- [x] T031 [US4] Implement loading / error / not-found branches with PageLoading + PageError in `packages/features/shell/src/approval-detail.tsx`
- [x] T032 [US4] Use em dash / honest empty for missing related entities in Request context in `packages/features/shell/src/approval-detail.tsx`
- [x] T033 [US4] Inbox keeps panel empty when no current target after filter empty in `packages/features/shell/src/inbox-page.tsx`

**Checkpoint**: US4 honest failure modes

---

## Phase 7: Polish & Cross-Cutting

- [x] T034 [P] Re-export helpers used by Inbox table risk tone from `approval-detail-helpers` if shared in `packages/features/shell/src/inbox-page.tsx`
- [x] T035 Run shell vitest for approval-detail + package tests; fix failures
- [x] T036 Mark [spec.md](./spec.md) **Status: Implemented** only after rubric green
- [x] T037 Mark all tasks complete in this file after verification

---

## Dependency Graph

```text
Phase 1 Setup
    ↓
Phase 2 Foundational (types/fixtures/helpers/tests scaffold)
    ↓
Phase 3 US1 Inspect (MVP) ──→ Phase 4 US2 Decide
    ↓                              ↓
Phase 6 US4 Honest states ←── Phase 5 US3 Decision trail
    ↓
Phase 7 Polish
```

## Parallel opportunities

- T002/T003, T007–T011, composition contract tests T012/T013/T021/T026/T030 can run in parallel once files exist.
- Implementation of sections inside approval-detail.tsx is sequential within the same file.

## MVP scope

US1 + foundational helpers + wire from Inbox (T001–T020). Decide + Decision trail complete the product slice for P0.

## Rejection rubric mapping

| Rubric item | Tasks |
| --- | --- |
| Dedicated module imported | T014, T019, T013 |
| Dense detail (~250 lines or helpers) | T007, T014–T018, T023, T027–T028, T031 |
| StatusChip risk + status | T007, T015 |
| PageLoading + PageError/notFound | T014, T031, T030 |
| Composition test | T010, T012, T021, T026, T030 |
| agent-requested + high-risk fixtures | T006, T011 |
| Pending actions / decided Decision | T023–T024, T027 |
| Spec Status Implemented after real work | T036 |
