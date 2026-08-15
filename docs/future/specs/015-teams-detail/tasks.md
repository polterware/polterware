# Tasks: Access Teams Detail

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/

**Tests**: Risk-proportional per Constitution XII — pure helpers for team status / presence / policy / result / resource-kind tones; composition source contract for DetailPanel LAW; desktop `detail-fixture-coverage` for Platform Team completeness + count consistency; keep `detail-targets` green. Do not mark complete while required tests fail for the changed surface. Agents.md: do not run `pnpm dev` / full build unless the user asks.

**Organization**: Tasks grouped by user story (US1–US4) for independent implementation and testing.

**Constitution / guardrails**:

- DetailPanel LAW only — no second inspector/page/drawer for teams
- Dependency direction IV: `feature-access` → shell mocks / design-system / state
- SSoT XIV: single `TeamDetail` composition; no dual inline detail
- Security VI: policy/membership tables display-only; mock toasts; no secrets/tokens
- Honesty: mock mutations never claim live core enforcement
- DESIGN: StatusChip for status; DataTable for collections; no PrimaryCell descriptions; no Actions columns

**Implementation note**: Core mock delivery is already landed on the working tree. Tasks use `[x]` when the artifact exists and tests cover it; remaining open work is polish/validation only.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: US1–US4 maps to [spec.md](./spec.md) user stories
- Include exact file paths in every task description

## Path Conventions (this feature)

- Access feature: `packages/features/access/src/`
- Team detail: `packages/features/access/src/team-detail.tsx`
- Helpers: `packages/features/access/src/team-detail-helpers.ts`
- Access index: `packages/features/access/src/index.tsx`
- Detail targets: `packages/features/access/src/detail-targets.ts`
- Shell mocks: `packages/features/shell/src/workspace-mocks/{types,data,index}.ts`
- Desktop fixture gate: `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`
- Contracts: `specs/015-teams-detail/contracts/teams-detail-ui.md`
- Related People detail (bidirectional nav): `packages/features/access/src/member-detail.tsx`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm feature package surface and test runner for access (reused from people detail)

- [x] T001 Confirm feature ownership stays in `packages/features/access` (no new scenic package) per [plan.md](./plan.md)
- [x] T002 [P] Confirm Vitest config for access package in `packages/features/access/vitest.config.ts` (node env, `src/**/*.{test,spec}.ts`)
- [x] T003 [P] Confirm access package test script + `vitest` devDependency in `packages/features/access/package.json`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Data model + helpers + contract SSOT that all stories consume

**⚠️ CRITICAL**: User story UI work depends on these fixtures/helpers

- [x] T004 Extend `WorkspaceTeam` with detail fields (`description`, `purpose?`, `status`, `leadId?`, `leadName`, `environments`, `escalation`, `schedule?`, `createdAt`, `memberRoster`, `ownedResources`, `policies`, `activity`, `notes?`) in `packages/features/shell/src/workspace-mocks/types.ts`
- [x] T005 Add nested types `WorkspaceTeamMember`, `WorkspaceTeamOwnedResource`, `WorkspaceTeamPolicyLink`, `WorkspaceTeamActivityEvent` in `packages/features/shell/src/workspace-mocks/types.ts`
- [x] T006 Enrich `TEAM_MOCKS` (Platform `t1`, Payments `t2`, Identity `t3`) with complete detail collections and count consistency (`members === memberRoster.length`, `owned === ownedResources.length`) in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T007 [P] Implement pure team status/presence/policy/result/resource-kind helpers in `packages/features/access/src/team-detail-helpers.ts`
- [x] T008 [P] Keep UI contract SSOT current in `specs/015-teams-detail/contracts/teams-detail-ui.md`
- [x] T009 [P] Keep data model SSOT current in `specs/015-teams-detail/data-model.md`
- [x] T010 [P] Add helper unit tests in `packages/features/access/src/team-detail.test.ts`
- [x] T011 Expand Platform Team completeness assertions in `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`

**Checkpoint**: Fixture team model + helpers ready; stories can compose DetailPanel independently

---

## Phase 3: User Story 1 - Inspect a project team (Priority: P1) 🎯 MVP

**Goal**: Operator activates a Teams row and inspects full team posture in the global DetailPanel (identity, overview/ops, environments, members, owned resources, policies, activity).

**Independent Test**: Access → People → Teams table → activate any fixture team → DetailPanel shows name/icon + Overview + structured collections without a second page.

### Tests for User Story 1 (risk-proportional)

- [x] T012 [P] [US1] Add composition source-contract assertions (sections layout, aria-labels, empty copy, no PrimaryCell `description=`) in `packages/features/access/src/team-detail.test.ts`
- [x] T013 [P] [US1] Assert Access wires `TeamDetail` for `entityKind === "team"` via source checks in `packages/features/access/src/team-detail.test.ts`
- [x] T014 [P] [US1] Keep People/Teams target registration green in `packages/features/access/src/detail-targets.test.ts`

### Implementation for User Story 1

- [x] T015 [US1] Create `TeamDetail` composition with loading/error/not-found/ready branches in `packages/features/access/src/team-detail.tsx`
- [x] T016 [US1] Render metrics (Status StatusChip, Members count, Owned resources count, On-call) in `packages/features/access/src/team-detail.tsx`
- [x] T017 [US1] Render Overview metadata (description, purpose, lead, on-call, escalation, schedule, status, created, updated) in `packages/features/access/src/team-detail.tsx`
- [x] T018 [US1] Render Environments list section in `packages/features/access/src/team-detail.tsx`
- [x] T019 [US1] Render Members DataTable (name, role, presence StatusChip) in `packages/features/access/src/team-detail.tsx`
- [x] T020 [US1] Render Owned resources DataTable (name, kind, environment) in `packages/features/access/src/team-detail.tsx`
- [x] T021 [US1] Render Policies DataTable (name, enforcement, status StatusChip) in `packages/features/access/src/team-detail.tsx`
- [x] T022 [US1] Render Activity DataTable (title, when, result StatusChip) in `packages/features/access/src/team-detail.tsx`
- [x] T023 [US1] Render optional Notes section when `notes` present in `packages/features/access/src/team-detail.tsx`
- [x] T024 [US1] Replace inline team detail stub with `TeamDetail` mount for `access:people` + `team` in `packages/features/access/src/index.tsx`
- [x] T025 [US1] Keep Teams list columns (`members`, `owned`, `onCall`, `updated`) reading enriched fixtures in `packages/features/access/src/index.tsx`

**Checkpoint**: US1 MVP — full inspection surface for fixture teams

---

## Phase 4: User Story 2 - Govern team from detail (Priority: P1)

**Goal**: Header mock governance actions with honest toast feedback; no second inspector.

**Independent Test**: Open team detail → Edit team / Add member / Archive team → mock toasts; panel stays on team.

### Tests for User Story 2

- [x] T026 [P] [US2] Assert header action labels exist in `packages/features/access/src/team-detail.tsx` via composition contract in `packages/features/access/src/team-detail.test.ts`
- [x] T027 [P] [US2] Assert Access wires mock toast handlers (`onEditTeam`, `onAddMember`, `onArchiveTeam`) in `packages/features/access/src/index.tsx` via source checks in `packages/features/access/src/team-detail.test.ts`

### Implementation for User Story 2

- [x] T028 [US2] Add DetailPanel header actions Edit team / Add member / Archive team (danger) in `packages/features/access/src/team-detail.tsx`
- [x] T029 [US2] Wire mock `pushToast` handlers for edit/add/archive in `packages/features/access/src/index.tsx`
- [x] T030 [US2] Keep toast copy explicitly labeled `(mock)` in `packages/features/access/src/index.tsx`

**Checkpoint**: US2 mock governance actions honest and reachable

---

## Phase 5: User Story 3 - Navigate between team members and people detail (Priority: P2)

**Goal**: Bidirectional navigation: team roster → member detail; member Teams table → team detail.

**Independent Test**: Platform Team → activate Erick (`m1`) → MemberDetail; member Teams → Platform Team → TeamDetail.

### Tests for User Story 3

- [x] T031 [P] [US3] Assert Platform roster includes person id `m1` in fixture coverage `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`
- [x] T032 [P] [US3] Assert `onOpenMember` wiring exists for team detail in `packages/features/access/src/index.tsx` via `packages/features/access/src/team-detail.test.ts`
- [x] T033 [P] [US3] Assert existing People → Team path (`onOpenTeam`) remains in `packages/features/access/src/member-detail.tsx` / `packages/features/access/src/index.tsx`

### Implementation for User Story 3

- [x] T034 [US3] Activate member roster row to call `onOpenMember(member.id)` in `packages/features/access/src/team-detail.tsx`
- [x] T035 [US3] Wire `onOpenMember={(id) => activate(id, "member")}` from `packages/features/access/src/index.tsx`
- [x] T036 [US3] Author Platform roster with at least one resolvable People id (`m1`) in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T037 [US3] Keep People `teamIds` → filtered `TEAM_MOCKS` + `onOpenTeam` path intact in `packages/features/access/src/index.tsx` / `member-detail.tsx`

**Checkpoint**: US3 bidirectional nav for known person/team pairs

---

## Phase 6: User Story 4 - Honest empty, missing, and partial states (Priority: P2)

**Goal**: Not-found, empty collections, and mock feedback stay honest; team detail only for `entityKind === "team"`.

**Independent Test**: Unknown team id → Team not found; Identity empty policies → empty table copy; member/invitation activation does not mount TeamDetail.

### Tests for User Story 4

- [x] T038 [P] [US4] Assert empty-state copy strings present in composition contract `packages/features/access/src/team-detail.test.ts`
- [x] T039 [P] [US4] Assert not-found/loading PageError/PageLoading branches in `packages/features/access/src/team-detail.tsx` via contract tests

### Implementation for User Story 4

- [x] T040 [US4] Implement loading / error / not-found branches with `PageLoading` / `PageError` in `packages/features/access/src/team-detail.tsx`
- [x] T041 [US4] Always render Members/Owned resources/Policies/Activity sections with `DataTable.Empty` (not hide sections) in `packages/features/access/src/team-detail.tsx`
- [x] T042 [US4] Author Identity Team (`t3`) with empty `policies` array for empty-state demo in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T043 [US4] Pass `notFound={!team()}` when team entityKind is active but id missing in `packages/features/access/src/index.tsx`
- [x] T044 [US4] Gate `TeamDetail` mount strictly on `scopeId === "access:people" && entityKind === "team"` in `packages/features/access/src/index.tsx`

**Checkpoint**: US4 honest empty/missing/mock states

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Convergence, residual UX polish, validation

- [x] T045 [P] Remove thin inline team stub and unused imports from `packages/features/access/src/index.tsx` after extraction
- [x] T046 [P] Ensure no PrimaryCell `description=` / forbidden metadata in `packages/features/access/src/team-detail.tsx`
- [x] T047 [P] Ensure DitherIcon tones for resources use valid design-system tones in `packages/features/access/src/team-detail-helpers.ts`
- [x] T048 Run access package tests: `cd packages/features/access && pnpm test` (13 passed including team-detail, 2026-08-12)
- [x] T049 [P] Run desktop fixture coverage: `cd apps/desktop && ./node_modules/.bin/vitest run --config vitest.ui.config.ts src/renderer/detail-fixture-coverage.test.ts` (1 passed, 2026-08-12)
- [x] T050 [P] Optional manual quickstart S1–S5 validation per [quickstart.md](./quickstart.md) — **skipped** (Agents.md: no `pnpm dev` unless operator asks; automated gates cover MVP)
- [x] T051 Update feature status Draft → Implemented in `specs/015-teams-detail/spec.md` after validation
- [x] T052 Convergence check: code ↔ spec ↔ plan ↔ tasks ↔ contract (Governance §6) — TeamDetail wired; WorkspaceTeam fields + TEAM_MOCKS rich; fixture-coverage Platform complete; no PrimaryCell description; mock toasts; sections layout
- [x] T053 [P] Optional jsdom DetailPanel.Root smoke — **deferred** (source contract + fixture coverage sufficient for mock stage; add only if team wants DOM coverage)
- [x] T054 [P] Confirm `.specify/feature.json` points at `specs/015-teams-detail` when working this feature

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies
- **Foundational (Phase 2)**: Depends on Setup — **blocks** all user stories (fixtures + helpers)
- **US1 (Phase 3)**: Depends on Foundational — **MVP**
- **US2 (Phase 4)**: Depends on US1 `TeamDetail` shell (header lives on same component)
- **US3 (Phase 5)**: Depends on US1 Members table + Access activation; reuses People detail path from `012-people-detail`
- **US4 (Phase 6)**: Can parallel with US2/US3 after US1 empty/not-found scaffolding
- **Polish (Phase 7)**: After desired stories complete

### User Story Dependencies

| Story | Depends on | Independently testable? |
| --- | --- | --- |
| **US1** Inspect team | Phase 2 | Yes — full read-only inspection |
| **US2** Govern from detail | US1 component shell | Yes — action toasts without backend |
| **US3** Bidirectional nav | US1 Members table + existing MemberDetail | Yes — activate known person/team pair |
| **US4** Honest states | US1 state branches | Yes — not-found/empty without other stories |

### Within Each User Story

1. Risk-proportional tests (where new rules)
2. Data/helpers if story-local
3. UI composition
4. AccessView wiring
5. Checkpoint validation

### Parallel Opportunities

```text
Phase 2: T007 || T008 || T009 || T010  (helpers / contract / data-model / helper tests)
Phase 3: T012 || T013 || T014          (tests)
Phase 4: T026 || T027                  (action contracts)
Phase 5: T031 || T032 || T033          (nav assertions)
Phase 6: T038 || T039                  (empty/not-found contracts)
Phase 7: T048 then T049; T050 optional manual; T053 optional
```

After Foundational:
- Dev A: US1 → US2
- Dev B: US3 nav wiring once Members table exists
- Dev C: US4 empty/not-found polish once TeamDetail exists

---

## Parallel Example: User Story 1

```bash
# Tests first (parallel):
Task: T012 composition source-contract in packages/features/access/src/team-detail.test.ts
Task: T013 Access wiring source checks in packages/features/access/src/team-detail.test.ts
Task: T014 detail-targets remain green in packages/features/access/src/detail-targets.test.ts

# Then sequential composition build in team-detail.tsx:
Task: T015 shell branches → T016 metrics → T017 overview → T018–T023 sections
Task: T024–T025 AccessView wiring
```

---

## Parallel Example: User Story 3

```bash
# Assertions parallel:
Task: T031 Platform roster includes m1 in apps/desktop/src/renderer/detail-fixture-coverage.test.ts
Task: T032 onOpenMember wiring source check in packages/features/access/src/team-detail.test.ts
Task: T033 onOpenTeam path remains in member-detail / index

# Implementation:
Task: T034 roster activate in team-detail.tsx
Task: T035 AccessView onOpenMember → activate(member)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 Setup  
2. Phase 2 Foundational (fixtures + helpers)  
3. Phase 3 US1 inspection surface  
4. **STOP and VALIDATE** — open Access → People → Teams → activate Platform Team  
5. Demo mock inspection  

### Incremental Delivery

1. Setup + Foundational → model ready  
2. US1 → inspect team posture (MVP)  
3. US2 → mock governance actions  
4. US3 → bidirectional member ↔ team nav  
5. US4 → empty/not-found honesty  
6. Polish → tests + convergence  

### Residual status (closed 2026-08-12)

| ID | Outcome |
| --- | --- |
| T048–T049 | Automated gates green (access 13/13, fixture coverage 1/1) |
| T050 | Skipped — no `pnpm dev` without operator request |
| T051–T052 | Spec **Implemented** + convergence static pass |
| T053 | Deferred — source/fixture contracts sufficient for mock stage |
| T054 | `.specify/feature.json` → `specs/015-teams-detail` |

---

## Notes

- [P] = different files / no incomplete deps
- Mock stage only — no live membership CRUD, no real archive/edit persistence, no ConnectRPC teams
- Agents / invitations keep separate entityKinds; roles matrix remains 007
- People detail bidirectional path is a dependency, not rewritten here
- English UI strings; Portuguese chat only
- Commit after logical groups if the user requests commits
- Feature directory is `specs/015-teams-detail` even if git branch stays `refactor/new`
