# Tasks: Integrations Secondary Flows

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md)

**Prerequisites**: plan.md, spec.md

**Tests**: Risk-proportional pure helpers (sync, conflict, wizard steps). No `pnpm dev` unless operator asks.

**Organization**: Shared helpers → US1 wizard → US2 webhooks → US3 sync → US4 conflict → wiring → rubric.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [x] T001 Create Spec Kit dir `specs/029-integrations-secondary-flows/` and point `.specify/feature.json`
- [x] T002 Confirm touch list: only `packages/features/integrations/src/{setup-wizard,webhooks,sync-center,conflict-resolve,index}*` + helpers/tests

## Phase 2: Foundational helpers

- [x] T003 [P] Add `setup-wizard-helpers.ts` (steps order, canProceed, finish payload) + `setup-wizard-helpers.test.ts`
- [x] T004 [P] Add `sync-status.ts` (health tone/label, outcome tone/label, stale detection) + `sync-status.test.ts`
- [x] T005 [P] Expand `conflict.test.ts` for `keep-local` | `keep-remote` | `merge` via `isValidConflictChoice`
- [x] T006 [P] Add `webhook-status.ts` (status tone/label) used by webhooks list

## Phase 3: US1 Setup wizard

- [x] T007 [US1] Rewrite `setup-wizard.tsx` to four steps with StatusChip progress, Field controls, cancel, test connection, finish
- [x] T008 [US1] Wire `index.tsx` onFinish to Connected + activate matching integration detail when fixture exists

## Phase 4: US2 Webhooks

- [x] T009 [US2] Elevate `webhooks.tsx`: DataTable columns destination/events/status/last delivery; StatusChip; empty; row detail; create mock A11yDialog

## Phase 5: US3 Sync center

- [x] T010 [US3] Elevate `sync-center.tsx`: overall last success/fail StatusChip, stale, Sync now mock, history/queue StatusChip, empty states, Card for summary (no hand-rolled chrome)

## Phase 6: US4 Conflict resolve

- [x] T011 [US4] Elevate `conflict-resolve.tsx`: keep-local/keep-remote/merge, Card compare panels, StatusChip unresolved, no silent overwrite
- [x] T012 [US4] Ensure sync-center Conflicts tab opens resolver and removes resolved mock rows

## Phase 7: Verify

- [x] T013 Run vitest for integrations package helpers/tests
- [x] T014 Rubric self-check; confirm `integration-detail.tsx` diff is none or minimal non-rewrite

## Parallel opportunities

T003–T006 parallel; T007–T011 after helpers; T008 after T007.
