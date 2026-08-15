# Tasks: Access Invitation Detail

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/

**Tests**: Risk-proportional per Constitution XII — pure helpers for invitation status tones/labels; composition source contract for DetailPanel LAW; desktop `detail-fixture-coverage` for Pending + Expired completeness; keep `detail-targets` green. Do not mark complete while required tests fail for the changed surface. Agents.md: do not run `pnpm dev` / full build unless the user asks.

**Organization**: Tasks grouped by user story (US1–US4) for independent implementation and testing.

**Constitution / guardrails**:

- DetailPanel LAW only — no second inspector/page/drawer for invitations
- Dependency direction IV: `feature-access` → shell mocks / design-system / state
- SSoT XIV: single `InvitationDetail` composition; no dual inline detail
- Security VI: mock toasts; no secrets; no real SMTP
- Honesty: never claim live email delivery
- Parallel ownership: NEVER edit approval/process/arch view/components or agent/policy/audit/session/member/team detail modules
- DESIGN: StatusChip for status; no PrimaryCell descriptions; no Actions columns

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: US1–US4 maps to [spec.md](./spec.md) user stories
- Include exact file paths in every task description

## Path Conventions (this feature)

- Access feature: `packages/features/access/src/`
- Invitation detail: `packages/features/access/src/invitation-detail.tsx`
- Helpers: `packages/features/access/src/invitation-detail-helpers.ts`
- Tests: `packages/features/access/src/invitation-detail.test.ts`
- Access index: `packages/features/access/src/index.tsx` (invitation mount only)
- Shell mocks: `packages/features/shell/src/workspace-mocks/{types,data}.ts` (invitation fields only)
- Desktop fixture gate: `apps/desktop/src/renderer/detail-fixture-coverage.test.ts` (invitation asserts only)
- Contracts: `specs/024-invitation-detail/contracts/invitation-detail-ui.md`

**FORBIDDEN paths**: `approval-detail*`, process detail, architecture view/components, `agent-detail*`, `policy-detail*`, `audit-detail*`, `agent-session-detail*`, `member-detail*`, `team-detail*`

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

- [x] T004 Extend `WorkspaceInvitation` with optional `teamId`, `invitedAt`, `environments`, `message` in `packages/features/shell/src/workspace-mocks/types.ts`
- [x] T005 Enrich `INVITATION_MOCKS` (Pending `i1` complete; Pending `i2`; Expired `i3` complete; optional Cancelled `i4`) in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T006 [P] Implement pure invitation status label/tone helpers in `packages/features/access/src/invitation-detail-helpers.ts`
- [x] T007 [P] Keep UI contract SSOT current in `specs/024-invitation-detail/contracts/invitation-detail-ui.md`
- [x] T008 [P] Keep data model SSOT current in `specs/024-invitation-detail/data-model.md`
- [x] T009 [P] Add helper unit tests in `packages/features/access/src/invitation-detail.test.ts`
- [x] T010 Expand invitation completeness assertions in `apps/desktop/src/renderer/detail-fixture-coverage.test.ts` (invitation fields only)

**Checkpoint**: Fixture invitation model + helpers ready; stories can compose DetailPanel independently

---

## Phase 3: User Story 1 - Inspect a project invitation (Priority: P1) 🎯 MVP

**Goal**: Operator activates an Invitations row and inspects full invitation posture in the global DetailPanel.

**Independent Test**: Access → People → activate any fixture invitation → DetailPanel shows email/icon + Overview + Assignment + Provenance without a second page.

### Tests for User Story 1

- [x] T011 [P] [US1] Add composition source-contract assertions (sections layout, StatusChip, Overview/Assignment/Provenance, no PrimaryCell `description=`) in `packages/features/access/src/invitation-detail.test.ts`
- [x] T012 [P] [US1] Assert Access wires `InvitationDetail` for `entityKind === "invitation"` via source checks in `packages/features/access/src/invitation-detail.test.ts`

### Implementation for User Story 1

- [x] T013 [US1] Create `InvitationDetail` composition with loading/error/not-found/ready branches in `packages/features/access/src/invitation-detail.tsx`
- [x] T014 [US1] Render metrics (Status StatusChip, Expires, Role, Team) in `packages/features/access/src/invitation-detail.tsx`
- [x] T015 [US1] Render Overview metadata (status, expires, role, team) in `packages/features/access/src/invitation-detail.tsx`
- [x] T016 [US1] Render Assignment metadata (role, team, optional environments) in `packages/features/access/src/invitation-detail.tsx`
- [x] T017 [US1] Render Provenance (invitedBy, optional invitedAt) in `packages/features/access/src/invitation-detail.tsx`
- [x] T018 [US1] Replace inline invitation detail block with `InvitationDetail` mount for `access:people` + `invitation` in `packages/features/access/src/index.tsx` only

**Checkpoint**: US1 MVP — full inspection surface for fixture invitations

---

## Phase 4: User Story 2 - Govern invitation lifecycle from detail (Priority: P1)

**Goal**: Header mock lifecycle actions with honest toast feedback.

**Independent Test**: Pending → Resend + Cancel; Expired → Re-invite; toasts include mock honesty.

- [x] T019 [US2] Render status-appropriate DetailPanel.Actions (Pending: Resend + Cancel danger; Expired: Re-invite; Cancelled: none) in `packages/features/access/src/invitation-detail.tsx`
- [x] T020 [US2] Wire mock toast handlers from Access (`onResend` / `onCancel` / `onReInvite`) in `packages/features/access/src/index.tsx`
- [x] T021 [P] [US2] Assert action labels and danger Cancel in `packages/features/access/src/invitation-detail.test.ts`

**Checkpoint**: Lifecycle actions honest and status-gated

---

## Phase 5: User Story 3 - Relate invitation to team (Priority: P2)

**Goal**: Optional open-team hand-off; optional message; no fabricated member profile.

- [x] T022 [US3] Optional Open team action when `teamId` resolvable via `onOpenTeam` in `packages/features/access/src/invitation-detail.tsx`
- [x] T023 [US3] Optional Message/Notes section when fixture-backed in `packages/features/access/src/invitation-detail.tsx`
- [x] T024 [US3] Wire `onOpenTeam` to activate team entity in `packages/features/access/src/index.tsx`
- [x] T025 [P] [US3] Assert no member profile invention and message/open-team contract strings in `packages/features/access/src/invitation-detail.test.ts`

**Checkpoint**: Team hand-off + message without member fabrication

---

## Phase 6: User Story 4 - Honest empty / missing states (Priority: P2)

**Goal**: loading / not-found / sparse optional fields remain honest.

- [x] T026 [US4] Ensure mount condition is `entityKind === "invitation"` (not only when fixture found) in `packages/features/access/src/index.tsx`
- [x] T027 [US4] PageLoading + PageError / not-found copy in `packages/features/access/src/invitation-detail.tsx`
- [x] T028 [P] [US4] Assert loading/not-found strings in `packages/features/access/src/invitation-detail.test.ts`

**Checkpoint**: Honest edge states

---

## Phase 7: Polish & Cross-Cutting

- [x] T029 Run `pnpm test` in `packages/features/access` and fix failures for invitation surface only
- [x] T030 Confirm rejection rubric green (dedicated module, StatusChip, honest states, tests, actions, fixtures, no SMTP claim, no sibling rewrites)
- [x] T031 Mark completed tasks in this file

---

## Dependency Graph

```text
T001–T003 (setup)
    → T004–T010 (foundational)
        → T011–T018 (US1)
            → T019–T021 (US2)
            → T022–T025 (US3)
            → T026–T028 (US4)
                → T029–T031 (polish)
```

## Parallel opportunities

- T006/T007/T008/T009 after T004
- T011/T012 with implementation once composition file exists
- T021/T025/T028 test asserts after composition settles

## Implementation strategy

1. Land fixtures + helpers first so composition has real data.
2. Extract InvitationDetail and swap Access wiring (remove inline).
3. Status actions + team/message + not-found mount condition.
4. Green access tests + fixture coverage + rubric.
