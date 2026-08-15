# Tasks: Access People Detail

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/

**Tests**: Risk-proportional per Constitution XII — pure helpers for presence/effect/result tones; composition source contract for DetailPanel LAW; desktop `detail-fixture-coverage` for Admin person completeness; keep `detail-targets` green. Do not mark complete while required tests fail for the changed surface. Agents.md: do not run `pnpm dev` / full build unless the user asks.

**Organization**: Tasks grouped by user story (US1–US4) for independent implementation and testing.

**Constitution / guardrails**:

- DetailPanel LAW only — no second inspector/page/drawer for members
- Dependency direction IV: `feature-access` → shell mocks / design-system / state
- SSoT XIV: single `MemberDetail` composition; no dual inline detail
- Security VI: permissions table display-only; mock toasts; no secrets/tokens
- Honesty: mock mutations never claim live core enforcement
- DESIGN: StatusChip for status; DataTable for collections; no PrimaryCell descriptions; no Actions columns

**Implementation note**: Core mock delivery is already landed on the working tree. Tasks use `[x]` when the artifact exists and tests cover it; remaining open work is polish/validation only.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: US1–US4 maps to [spec.md](./spec.md) user stories
- Include exact file paths in every task description

## Path Conventions (this feature)

- Access feature: `packages/features/access/src/`
- Member detail: `packages/features/access/src/member-detail.tsx`
- Helpers: `packages/features/access/src/member-detail-helpers.ts`
- Access index: `packages/features/access/src/index.tsx`
- Detail targets: `packages/features/access/src/detail-targets.ts`
- Shell mocks: `packages/features/shell/src/workspace-mocks/{types,data,index}.ts`
- Desktop fixture gate: `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`
- Contracts: `specs/012-people-detail/contracts/people-detail-ui.md`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm feature package surface and test runner for access

- [x] T001 Confirm feature ownership stays in `packages/features/access` (no new scenic package) per [plan.md](./plan.md)
- [x] T002 [P] Add Vitest config for access package in `packages/features/access/vitest.config.ts` (node env, `src/**/*.{test,spec}.ts`)
- [x] T003 [P] Wire access package test script + `vitest` devDependency in `packages/features/access/package.json`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Data model + helpers + contract SSOT that all stories consume

**⚠️ CRITICAL**: User story UI work depends on these fixtures/helpers

- [x] T004 Extend `WorkspacePerson` with detail fields (`joinedAt`, `invitedBy?`, `authMethod`, `mfaEnabled`, `title?`, `notes?`, `permissions`, `sessions`, `activity`) in `packages/features/shell/src/workspace-mocks/types.ts`
- [x] T005 Enrich `PEOPLE_MOCKS` (Admin `m1`, Operator `m2`, Viewer `m3`) with complete detail collections in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T006 [P] Implement pure presence/effect/result helpers in `packages/features/access/src/member-detail-helpers.ts`
- [x] T007 [P] Keep UI contract SSOT current in `specs/012-people-detail/contracts/people-detail-ui.md`
- [x] T008 [P] Add helper unit tests in `packages/features/access/src/member-detail.test.ts`
- [x] T009 Expand Admin people completeness assertions in `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`

**Checkpoint**: Fixture person model + helpers ready; stories can compose DetailPanel independently

---

## Phase 3: User Story 1 - Inspect a project member (Priority: P1) 🎯 MVP

**Goal**: Operator activates a People row and inspects full membership posture in the global DetailPanel (identity, overview, environments, permissions, teams, sessions, activity).

**Independent Test**: Access → People → activate any fixture person → DetailPanel shows name/icon + Overview + structured collections without a second page.

### Tests for User Story 1 (risk-proportional)

- [x] T010 [P] [US1] Add composition source-contract assertions (sections layout, aria-labels, empty copy, no PrimaryCell `description=`) in `packages/features/access/src/member-detail.test.ts`
- [x] T011 [P] [US1] Assert Access wires `MemberDetail` for `entityKind === "member"` via source checks in `packages/features/access/src/member-detail.test.ts`
- [x] T012 [P] [US1] Keep People target registration green in `packages/features/access/src/detail-targets.test.ts`

### Implementation for User Story 1

- [x] T013 [US1] Create `MemberDetail` composition with loading/error/not-found/ready branches in `packages/features/access/src/member-detail.tsx`
- [x] T014 [US1] Render metrics (Presence StatusChip, Role, Teams count, Environments count) in `packages/features/access/src/member-detail.tsx`
- [x] T015 [US1] Render Overview metadata (email, title, role, presence, auth, MFA, joined, invitedBy, last activity/seen) in `packages/features/access/src/member-detail.tsx`
- [x] T016 [US1] Render Environments list section in `packages/features/access/src/member-detail.tsx`
- [x] T017 [US1] Render Permissions DataTable (capability + effect StatusChip) in `packages/features/access/src/member-detail.tsx`
- [x] T018 [US1] Render Teams DataTable (filtered teams + on-call) in `packages/features/access/src/member-detail.tsx`
- [x] T019 [US1] Render Sessions DataTable (device, location, last active, current StatusChip) in `packages/features/access/src/member-detail.tsx`
- [x] T020 [US1] Render Activity DataTable (title, when, result StatusChip) in `packages/features/access/src/member-detail.tsx`
- [x] T021 [US1] Render optional Notes section when `notes` present in `packages/features/access/src/member-detail.tsx`
- [x] T022 [US1] Replace inline member detail with `MemberDetail` mount for `access:people` + `member` in `packages/features/access/src/index.tsx`
- [x] T023 [US1] Pass filtered `TEAM_MOCKS` by `member.teamIds` into `MemberDetail` from `packages/features/access/src/index.tsx`

**Checkpoint**: US1 MVP — full inspection surface for fixture members

---

## Phase 4: User Story 2 - Govern membership from detail (Priority: P1)

**Goal**: Header (and session row) mock governance actions with honest toast feedback; no second inspector.

**Independent Test**: Open member detail → Change role / Add to team / Revoke access / revoke non-current session → mock toasts; panel stays on member.

### Tests for User Story 2

- [x] T024 [P] [US2] Assert header action labels exist in `packages/features/access/src/member-detail.tsx` via composition contract in `packages/features/access/src/member-detail.test.ts`
- [x] T025 [P] [US2] Assert Access wires mock toast handlers (`onChangeRole`, `onAddToTeam`, `onRevokeAccess`, `onRevokeSession`) in `packages/features/access/src/index.tsx` via source checks in `packages/features/access/src/member-detail.test.ts`

### Implementation for User Story 2

- [x] T026 [US2] Add DetailPanel header actions Change role / Add to team / Revoke access (danger) in `packages/features/access/src/member-detail.tsx`
- [x] T027 [US2] Wire mock `pushToast` handlers for role/team/revoke in `packages/features/access/src/index.tsx`
- [x] T028 [US2] Activate non-current session row to call `onRevokeSession`; leave current session without revoke in `packages/features/access/src/member-detail.tsx`
- [x] T029 [US2] Wire session revoke toast and team row → `activate(id, "team")` in `packages/features/access/src/index.tsx`

**Checkpoint**: US2 mock governance actions honest and reachable

---

## Phase 5: User Story 3 - Understand permissions and auth posture (Priority: P2)

**Goal**: Admin / Operator / Viewer fixtures show distinct capability effects plus auth method and MFA posture.

**Independent Test**: Open m1/m2/m3 → permissions effects differ by role; auth + MFA visible; sessions or empty state present.

### Tests for User Story 3

- [x] T030 [P] [US3] Assert Admin fixture elevated allows + request-only rows in `packages/features/shell/src/workspace-mocks/data.ts` via `packages/features/access/src/member-detail.test.ts` and `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`
- [x] T031 [P] [US3] Assert Viewer fixture includes deny posture + `mfaEnabled: false` in fixture/source checks in `packages/features/access/src/member-detail.test.ts`

### Implementation for User Story 3

- [x] T032 [US3] Author Admin permission set (manage/ops allow + destructive request-only) in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T033 [US3] Author Operator mixed allow/request-only/deny set in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T034 [US3] Author Viewer read-allow + write deny/request-only set in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T035 [US3] Surface auth method + MFA StatusChip in Overview of `packages/features/access/src/member-detail.tsx`
- [x] T036 [US3] Map effect labels/tones (`Allow`/`Deny`/`Request only`) via helpers in `packages/features/access/src/member-detail-helpers.ts` consumed by `member-detail.tsx`

**Checkpoint**: US3 least-privilege posture inspectable across roles

---

## Phase 6: User Story 4 - Honest empty, missing, and partial states (Priority: P2)

**Goal**: Not-found, empty collections, and mock feedback stay honest; member detail only for `entityKind === "member"`.

**Independent Test**: Unknown member id → Member not found; empty teams/sessions/activity → empty table copy; invitation/team activation does not mount MemberDetail.

### Tests for User Story 4

- [x] T037 [P] [US4] Assert empty-state copy strings present in composition contract `packages/features/access/src/member-detail.test.ts`
- [x] T038 [P] [US4] Assert not-found/loading PageError/PageLoading branches in `packages/features/access/src/member-detail.tsx` via contract tests

### Implementation for User Story 4

- [x] T039 [US4] Implement loading / error / not-found branches with `PageLoading` / `PageError` in `packages/features/access/src/member-detail.tsx`
- [x] T040 [US4] Always render Permissions/Teams/Sessions/Activity sections with `DataTable.Empty` (not hide sections) in `packages/features/access/src/member-detail.tsx`
- [x] T041 [US4] Pass `notFound={!member()}` when member entityKind is active but id missing in `packages/features/access/src/index.tsx`
- [x] T042 [US4] Gate `MemberDetail` mount strictly on `scopeId === "access:people" && entityKind === "member"` in `packages/features/access/src/index.tsx`
- [x] T043 [US4] Keep mock toast copy explicitly labeled `(mock)` in `packages/features/access/src/index.tsx`

**Checkpoint**: US4 honest empty/missing/mock states

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Convergence, residual UX polish, validation

- [x] T044 [P] Remove unused member-only icon imports from `packages/features/access/src/index.tsx` after extraction
- [x] T045 [P] Ensure no PrimaryCell `description=` / forbidden metadata in `packages/features/access/src/member-detail.tsx`
- [x] T046 Run access package tests: `cd packages/features/access && pnpm test` (8 passed, 2026-08-12)
- [x] T047 [P] Run desktop fixture coverage: `cd apps/desktop && ./node_modules/.bin/vitest run --config vitest.ui.config.ts src/renderer/detail-fixture-coverage.test.ts` (1 passed, 2026-08-12)
- [x] T048 [P] Optional manual quickstart S1–S5 validation per [quickstart.md](./quickstart.md) — **skipped** (Agents.md: no `pnpm dev` unless operator asks; automated gates cover MVP)
- [x] T049 Update feature status Draft → Implemented in `specs/012-people-detail/spec.md` after validation
- [x] T050 Convergence check: code ↔ spec ↔ plan ↔ tasks ↔ contract (Governance §6) — MemberDetail wired; WorkspacePerson fields + PEOPLE_MOCKS rich; fixture-coverage Admin complete; no PrimaryCell description; mock toasts; sections layout
- [x] T051 [P] Optional jsdom DetailPanel.Root smoke — **deferred** (source contract + fixture coverage sufficient for mock stage; add only if team wants DOM coverage)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies
- **Foundational (Phase 2)**: Depends on Setup — **blocks** all user stories (fixtures + helpers)
- **US1 (Phase 3)**: Depends on Foundational — **MVP**
- **US2 (Phase 4)**: Depends on US1 `MemberDetail` shell (header lives on same component)
- **US3 (Phase 5)**: Can parallel with US2 after Foundational (fixture content) + needs Permissions/Overview from US1
- **US4 (Phase 6)**: Can parallel with US2/US3 after US1 empty/not-found scaffolding
- **Polish (Phase 7)**: After desired stories complete

### User Story Dependencies

| Story | Depends on | Independently testable? |
| --- | --- | --- |
| **US1** Inspect member | Phase 2 | Yes — full read-only inspection |
| **US2** Govern from detail | US1 component shell | Yes — action toasts without backend |
| **US3** Permissions/auth posture | Phase 2 fixtures + US1 permissions/overview | Yes — open m1/m2/m3 |
| **US4** Honest states | US1 state branches | Yes — not-found/empty without other stories |

### Within Each User Story

1. Risk-proportional tests (where new rules)
2. Data/helpers if story-local
3. UI composition
4. AccessView wiring
5. Checkpoint validation

### Parallel Opportunities

```text
Phase 2: T006 || T007 || T008  (helpers / contract / helper tests)
Phase 3: T010 || T011 || T012  (tests)
Phase 5: T030 || T031          (fixture assertions)
Phase 6: T037 || T038          (empty/not-found contracts)
Phase 7: T046 then T047; T048 optional manual; T051 optional
```

After Foundational:
- Dev A: US1 → US2
- Dev B: US3 fixture authoring (T032–T034) in parallel with early US1
- Dev C: US4 empty/not-found polish once MemberDetail exists

---

## Parallel Example: User Story 1

```bash
# Tests first (parallel):
Task: T010 composition source-contract in packages/features/access/src/member-detail.test.ts
Task: T011 Access wiring source checks in packages/features/access/src/member-detail.test.ts
Task: T012 detail-targets remain green in packages/features/access/src/detail-targets.test.ts

# Then sequential composition build in member-detail.tsx:
Task: T013 shell branches → T014 metrics → T015 overview → T016–T021 sections
Task: T022–T023 AccessView wiring
```

---

## Parallel Example: User Story 3

```bash
# Fixture authoring can split by person file regions (same file — not [P] together):
Task: T032 Admin permissions in packages/features/shell/src/workspace-mocks/data.ts
# then
Task: T033 Operator permissions
Task: T034 Viewer permissions
# parallel with UI mapping already in helpers:
Task: T036 effect labels in packages/features/access/src/member-detail-helpers.ts
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 Setup  
2. Phase 2 Foundational (fixtures + helpers)  
3. Phase 3 US1 inspection surface  
4. **STOP and VALIDATE** — open Access → People → activate member  
5. Demo mock inspection  

### Incremental Delivery

1. Setup + Foundational → model ready  
2. US1 → inspect posture (MVP)  
3. US2 → mock governance actions  
4. US3 → role posture richness  
5. US4 → empty/not-found honesty  
6. Polish → tests + convergence  

### Residual status (closed 2026-08-12)

| ID | Outcome |
| --- | --- |
| T046–T047 | Automated gates green (access 8/8, fixture coverage 1/1) |
| T048 | Skipped — no `pnpm dev` without operator request |
| T049–T050 | Spec **Implemented** + convergence static pass |
| T051 | Deferred — source/fixture contracts sufficient for mock stage |

---

## Notes

- [P] = different files / no incomplete deps
- Mock stage only — no IdP, no real session kill, no ConnectRPC membership
- Agents / invitations / teams details are out of scope (separate entityKinds)
- Role matrix editor remains 007
- English UI strings; Portuguese chat only
- Commit after logical groups if the user requests commits
- Feature directory is `specs/012-people-detail` even if git branch stays `refactor/new`
