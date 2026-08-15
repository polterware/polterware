# Tasks: Resources Process Detail

**Input**: Design documents from `specs/021-process-detail/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/process-detail-ui.md](./contracts/process-detail-ui.md), [quickstart.md](./quickstart.md)

**Tests**: Composition contract, status gates, fixture richness, desktop detail-fixture-coverage (Constitution XII).

**Organization**: Tasks grouped by user story.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [x] T001 Confirm Resources Processes entry + stub ProcessDetail in `packages/features/resources/src/index.tsx` and `process-detail.tsx`
- [x] T002 [P] Freeze UI contract at `specs/021-process-detail/contracts/process-detail-ui.md` and data model
- [x] T003 [P] Point active feature at `specs/021-process-detail` via `.specify/feature.json`

---

## Phase 2: Foundational

- [x] T004 Expand `ProcessRow` optional fields (`uptime`, related refs) in `packages/features/resources/src/node-types.ts`
- [x] T005 [P] Implement process status helpers in `packages/features/resources/src/process-status.ts`
- [x] T006 Enrich `PROCESS_FIXTURES` (full, sparse, Stopped, Unknown) in `packages/features/resources/src/node-types.ts`
- [x] T007 [P] Wire Processes table StatusChip via helpers in `packages/features/resources/src/processes-table.tsx`
- [x] T008 Export helpers from `packages/features/resources/src/index.tsx`
- [x] T009 Include `src/process-detail.test.ts` in `packages/features/resources/vitest.config.ts`

---

## Phase 3: User Story 1 - Inspect process (P1)

- [x] T010 [P] [US1] Unit tests for status labels/tones/gates in `process-detail.test.ts`
- [x] T011 [P] [US1] Fixture full + sparse contract tests in `process-detail.test.ts`
- [x] T012 [US1] Harden ProcessDetail loading/error/not-found + Overview metrics (StatusChip, PID, host, CPU, memory, started/uptime) in `process-detail.tsx`
- [x] T013 [US1] Runtime section (command mono, user, port, dataSource honesty) in `process-detail.tsx`
- [x] T014 [US1] Fix processes detail mount for not-found in `index.tsx`

---

## Phase 4: User Story 2 - Control actions (P1)

- [x] T015 [US2] Status-gated header actions (View logs, Open terminal, Restart, Stop danger) with mock toast honesty in `process-detail.tsx`
- [x] T016 [P] [US2] Composition contract asserts action labels, danger tone, StatusChip, PageLoading/PageError in `process-detail.test.ts`
- [x] T017 [US2] Assert Stop/Restart gates for Stopped/Unknown fixtures in `process-detail.test.ts`

---

## Phase 5: User Story 3 - Host + related (P2)

- [x] T018 [US3] Host context section + optional `onOpenNode` hand-off in `process-detail.tsx`
- [x] T019 [US3] Optional Related section only when fixture fields present in `process-detail.tsx`
- [x] T020 [US3] Wire `onOpenNode` from ResourcesView process branch in `index.tsx`

---

## Phase 6: User Story 4 - Honest states + coverage (P2)

- [x] T021 [US4] Expand desktop process full + sparse coverage in `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`
- [x] T022 [US4] Sparse metrics never invent CPU bar (composition + fixture assertions)

---

## Phase 7: Polish

- [x] T023 Run `pnpm --filter @polterware/feature-resources test` and confirm green
- [x] T024 Rubric self-check against plan acceptance checklist

---

## Dependencies

- Phase 2 blocks all UI stories
- US1 → US2/US3 compose on same file
- US4 coverage after fixtures exist
