# Tasks: Overview Operational Command Center

**Input**: Design documents from `specs/032-overview-operational/`  
**Prerequisites**: plan.md, spec.md, contracts/overview-ui.md

**Tests**: Pure helpers unit tests + UI contract source checks; shell package vitest must actually pass for changed tests.

**Constitution**: Renderer fixture only; shell overview boundary; no Activity rewrite.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [x] T001 Create feature docs under `specs/032-overview-operational/` and set `.specify/feature.json` → `specs/032-overview-operational`

---

## Phase 2: Foundational (model + fixtures)

- [ ] T002 Expand pure helpers in `packages/features/shell/src/overview-model.ts` (system health StatusChip presentation, integration health aggregate, dataSource label/message, OVERVIEW_SHORTCUTS contract)
- [ ] T003 [P] Extend `OverviewSummary` / `buildOverviewSummary` as needed for integration counts; keep empty/offline fixture paths honest
- [ ] T004 [P] Expand `packages/features/shell/src/overview-model.test.ts` for new helpers + keep existing filter/segment coverage green

---

## Phase 3: User Story 1 + 5 — Health, context, honesty (P1)

**Goal**: StatusChip on ≥2 health surfaces; project/env context; fixture/empty/offline honesty

- [ ] T005 [US1][US5] Elevate `overview-cards.tsx` — SystemHealth StatusChip; nodes/integrations health StatusChip surfaces; health-event StatusChip; no new fake history charts
- [ ] T006 [US1][US5] Elevate `overview-page.tsx` — context header (project, env, health), dataSource banner/footer, wire integrations + dataSource props

---

## Phase 4: User Story 2 + 3 — Attention, recent ops/changes (P1)

- [ ] T007 [US2] Needs attention: pending count + View inbox → `inbox`; row open approval; empty pending honesty
- [ ] T008 [US3] Recent changes + Operations in progress hand-offs; run status StatusChip; empty honesty; no Activity page edits

---

## Phase 5: User Story 4 — Shortcuts (P1)

- [ ] T009 [US4] Shortcuts strip: Architecture, Resources, Operations, Inbox, Terminal via `onQuickAction`
- [ ] T010 [US4] Ensure App.tsx maps `terminal` quick-action to Terminal section if missing

---

## Phase 6: Polish & verification

- [ ] T011 Export new helpers from index if part of public API
- [ ] T012 Run shell overview tests; confirm rejection rubric green
- [ ] T013 Confirm activity-page, knowledge, spec-kit, audit-detail untouched

---

## Dependencies

- T001 → T002–T004 → T005–T006 → T007–T008 → T009–T010 → T011–T013
- T003/T004 can parallel after T002 API sketched

## Parallel opportunities

- T003 and T004 after model helpers
- T007–T009 after cards/page structure exists

## Implementation strategy

MVP = T002–T009 (model + page cohesion + StatusChip + shortcuts + honesty). Verify with T012–T013.
