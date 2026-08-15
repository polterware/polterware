# Tasks: Terminal Sessions UI (Mock)

**Input**: Design documents from `specs/031-terminal-sessions-ui/`  
**Prerequisites**: plan.md, spec.md, contracts/terminal-sessions-ui.md

**Tests**: Pure helpers unit tests + UI contract source checks; package vitest must pass for new/changed tests.

**Constitution**: Renderer-only mock; no SSH/PTY/main spawn; shell package boundary only.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [x] T001 Create feature docs under `specs/031-terminal-sessions-ui/` (spec, plan, tasks, checklist, contracts) and set `.specify/feature.json` → `specs/031-terminal-sessions-ui`

---

## Phase 2: Foundational (session model + fixtures)

- [x] T002 Define session status model and pure helpers in `packages/features/shell/src/terminal-session-model.ts` (status tone/label, create mock session, close helper, unusable feedback message, dataSource label)
- [x] T003 [P] Update `packages/features/shell/src/workspace-mocks/types.ts` `TerminalSessionMock` to active|idle|closed + shell, startedAt, correlationId, dataSource
- [x] T004 [P] Migrate `packages/features/shell/src/workspace-mocks/data.ts` `TERMINAL_SESSION_MOCKS` to new shape; add offline/revoked targets in `packages/features/shell/src/fixtures.ts`
- [x] T005 [P] Unit tests in `packages/features/shell/src/terminal-session-model.test.ts`; expand `packages/features/shell/src/types.test.ts` if needed for usability contract

---

## Phase 3: User Story 2 — Target picker honesty (P1)

**Goal**: StatusChip health; unusable blocked with feedback  

- [x] T006 [US2] Elevate `packages/features/shell/src/target-picker.tsx` — StatusChip via terminalTargetStatusTone/Label; empty no-targets; unusable rows disabled or blocked with reason; usable rows call onSelect

---

## Phase 4: User Story 1 + 3 — Terminal page sessions (P1)

**Goal**: Professional switcher, create/close, empty  

- [x] T007 [US1] Elevate `packages/features/shell/src/terminal-page.tsx` — session StatusChip, target/shell/startedAt chrome, mono correlation/session id, fixture label
- [x] T008 [US2][US3] Wire new-session mock through usability gate + toast feedback; close session mock; empty no-sessions / no-targets copy

---

## Phase 5: User Story 4 — Side panel audit (P1)

- [x] T009 [US4] Elevate `packages/features/shell/src/terminal-side-panel.tsx` — StatusChip target health, session/correlation mono when known, mock-honest connection copy (no real-host claim)

---

## Phase 6: Polish & verification

- [x] T010 Export helpers from `packages/features/shell/src/index.tsx` if public API needed
- [x] T011 Run shell package tests (`pnpm --filter @polterware/feature-shell test` or vitest config); confirm rejection rubric green
- [x] T012 Confirm no SSH/PTY/main-process code introduced in owned files

---

## Dependencies

- T001 → T002–T005 → T006 → T007–T008 → T009 → T010–T012
- T003/T004/T005 can parallel after T002 shape is defined

## Parallel opportunities

- T003, T004, T005 after model API sketched
- T006 can start once helpers + fixtures exist
- T009 can parallel T007 after model exists if side panel only needs fixtures

## Implementation strategy

MVP = T002–T008 (page + picker + helpers + tests). Side panel (T009) completes audit rubric. Verify with T011–T012.
