# Tasks: Access Audit Event Detail

**Input**: Design documents from `/specs/019-audit-detail/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included (explicitly required by feature description + constitution XII).

**Organization**: Phases by user story so each story can be validated independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Feature docs already exist; confirm workspace pointers.

- [x] T001 Confirm `.specify/feature.json` points to `specs/019-audit-detail` and artifact set is present (spec, plan, research, data-model, contracts, quickstart, this tasks file)

---

## Phase 2: Foundational (blocking)

**Purpose**: Enriched audit model + pure helpers required by all stories.

**⚠️ CRITICAL**: No user story work depends on incomplete foundation.

- [x] T002 Extend `WorkspaceAuditEvent` optional fields (`actorId`, `requestId`, `correlationId`, `runId`, `contextNotes`, `contextPayload`) in `packages/features/shell/src/workspace-mocks/types.ts`
- [x] T003 Enrich `AUDIT_EVENT_MOCKS` for completeness gates (policy+approval, Denied, correlation ids, context, no-related) in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T004 [P] Add pure helpers (`resultTone`, `resultLabel`, `auditDetailTitle`, `actorKindLabel`) in `packages/features/access/src/audit-detail-helpers.ts`

**Checkpoint**: Model + helpers ready; detail composition can start.

---

## Phase 3: User Story 1 — Inspect an audit event (Priority: P1) 🎯 MVP

**Goal**: Production DetailPanel for a found audit event with result metrics, actor, action/entity, origin/time.

**Independent Test**: Activate any audit row → DetailPanel shows title, actor icon, StatusChip result, environment, actor, action, entity, origin, time.

### Tests for User Story 1

- [x] T005 [P] [US1] Write helpers + composition contract tests for found-event sections in `packages/features/access/src/audit-detail.test.ts`

### Implementation for User Story 1

- [x] T006 [US1] Implement `AuditDetail` found-event composition (header icon by actorKind, title from action, metrics Result/Environment/Origin/Time, Actor + Action sections, StatusChip) in `packages/features/access/src/audit-detail.tsx`
- [x] T007 [US1] Wire Access → Audit target to `<AuditDetail event={audit()} />` (mount on `access:audit` + `audit-event`, remove thin inline block) in `packages/features/access/src/index.tsx`

**Checkpoint**: Found events open with production sections; collection still virtualized.

---

## Phase 4: User Story 2 — Related controls & correlation (Priority: P1)

**Goal**: Correlation ids, related policy/approval, optional Copy / Open actions (non-mutating only).

**Independent Test**: Open `au2` → correlation + related controls + Copy + Open policy; no Approve/Delete.

### Tests for User Story 2

- [x] T008 [P] [US2] Extend composition contract tests for Copy event id, Open policy/approval, absence of mutation labels, related/correlation section markers in `packages/features/access/src/audit-detail.test.ts`

### Implementation for User Story 2

- [x] T009 [US2] Add Correlation + Related controls sections and optional header actions (`onCopyEventId`, `onOpenPolicy`, `onOpenApproval`) in `packages/features/access/src/audit-detail.tsx`
- [x] T010 [US2] Pass mock handlers from Access index (clipboard/toast copy; `activate`/`onOpenPolicy` for policy; toast for approval) in `packages/features/access/src/index.tsx`

**Checkpoint**: Related governance references are inspectable; trail remains read-only.

---

## Phase 5: User Story 3 — Honest empty / missing states (Priority: P2)

**Goal**: loading, error, not-found, empty context honesty.

**Independent Test**: Unknown id → PageError not-found; event without context → empty Context message.

### Tests for User Story 3

- [x] T011 [P] [US3] Assert PageLoading / PageError / not-found / empty context strings and mount condition (`notFound={!audit()}`) in `packages/features/access/src/audit-detail.test.ts`

### Implementation for User Story 3

- [x] T012 [US3] Implement loading / error / not-found branches and Context empty state (plus muted read-only note) in `packages/features/access/src/audit-detail.tsx`
- [x] T013 [US3] Ensure Access index mounts AuditDetail when target is audit-event even if fixture missing (`notFound={!audit()}`) in `packages/features/access/src/index.tsx`

**Checkpoint**: Honest states for missing and partial data.

---

## Phase 6: Polish & Cross-Cutting

- [x] T014 [P] Expand audit asserts in `apps/desktop/src/renderer/detail-fixture-coverage.test.ts` (policy+approval, Denied, correlation ids, required fields)
- [x] T015 [P] Confirm Access index Audit table filters + `virtualized` remain unchanged (static review) in `packages/features/access/src/index.tsx`
- [x] T016 Run `pnpm --filter @polterware/feature-access test` and fix regressions limited to audit-detail files
- [x] T017 Mark all tasks complete in `specs/019-audit-detail/tasks.md` after green tests

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies
- **Foundational (Phase 2)**: depends on Setup — **BLOCKS** all user stories
- **US1 (Phase 3)**: depends on Foundational — MVP
- **US2 (Phase 4)**: depends on US1 composition existing
- **US3 (Phase 5)**: depends on US1 mount pattern (can extend same file after US1)
- **Polish (Phase 6)**: depends on US1–US3 complete

### User Story Dependencies

- **US1**: Independent after foundation
- **US2**: Builds on US1 composition
- **US3**: Builds on US1 mount/states

### Parallel Opportunities

- T004 helpers parallel with T002/T003 after type shape known (prefer after T002)
- T005 / T008 / T011 test authoring can draft early; run after implementation
- T014 fixture coverage parallel with package tests once fixtures enriched

### Parallel Example: Foundational

```bash
# After T002 type extension:
# T003 fixtures + T004 helpers in parallel
```

---

## Implementation Strategy

### MVP First (User Story 1)

1. Complete Phase 1–2  
2. Complete Phase 3 (US1)  
3. **STOP and validate**: open any audit event, sections present  
4. Continue US2 → US3 → Polish  

### Incremental Delivery

1. Foundation → enriched model  
2. US1 → inspectable detail  
3. US2 → correlation + related + copy/open  
4. US3 → not-found / empty honesty  
5. Fixture coverage + package tests green  

### Notes

- Chat with Erick in Portuguese; code/UI/docs English  
- Do not touch agent-detail / policy-detail / agent-session-detail surfaces  
- Do not invent timeline without multi-step timestamps  

---

## Task Summary

| Phase | Tasks | Story |
| --- | --- | --- |
| Setup | T001 | — |
| Foundational | T002–T004 | — |
| US1 | T005–T007 | Inspect event |
| US2 | T008–T010 | Related + correlation |
| US3 | T011–T013 | Honest states |
| Polish | T014–T017 | Coverage + verify |

**Total**: 17 tasks  
**MVP scope**: T001–T007  
**Suggested parallel**: T003∥T004 after T002; T014∥T016 after implementation
