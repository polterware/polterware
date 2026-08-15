# Tasks: Activity Actor Inspection

**Input**: Design documents from `/specs/033-activity-actor-inspection/`  
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/

## Phase 1: Setup

- [x] T001 Confirm feature dir `specs/033-activity-actor-inspection` and `.specify/feature.json` point here
- [x] T002 [P] Inventory current `activity-page.tsx`, `activity-event-icon.tsx`, `ACTIVITY_MOCKS`, App.tsx mount

## Phase 2: Foundational

- [x] T003 Extend `ActivityEvent` with optional `targetId` if open-target needs it (`workspace-mocks/types.ts`)
- [x] T004 [P] Ensure ACTIVITY_MOCKS has ≥1 person, ≥1 agent, ≥1 system with clear titles (`workspace-mocks/data.ts`)
- [x] T005 Create pure helpers `activity-event-helpers.ts` (filter + labels/tones)
- [x] T006 Create detail helpers `activity-event-detail-helpers.ts`

## Phase 3: User Story 1 — Browse + filter (P1)

- [x] T007 [US1] Rewrite Activity table columns: Event, Actor, Actor kind StatusChip, Environment, When — no Actions
- [x] T008 [US1] Wire FilterCategoryPopover for type, actorKind, environment; use pure filter helper; honest empty state
- [x] T009 [US1] Differentiate page copy from Access Audit (operational feed language)

## Phase 4: User Story 2 — Detail inspection (P1)

- [x] T010 [US2] Implement `activity-event-detail.tsx` DetailPanel sections (overview, target, description, honesty states)
- [x] T011 [US2] Register collection `activity:index` / `activity-event`; activate on row; mount detail Show
- [x] T012 [US2] Optional Open target header action via callback when target present
- [x] T013 [US2] Export detail/helpers from `index.tsx` as needed

## Phase 5: User Story 3 — Tests (P2)

- [x] T014 [P] [US3] `activity-event-helpers.test.ts` — filter combinations + empty results
- [x] T015 [P] [US3] `activity-event-detail.test.ts` — composition contract + page wiring + non-Audit fusion
- [x] T016 Run shell vitest for activity tests; mark rubric green

## Phase 6: Polish

- [x] T017 Confirm overview consumers of ACTIVITY_MOCKS still typecheck against optional fields
- [x] T018 Update checklist/rubric notes in spec status to Implemented when green

## Dependencies

- US1 (page/filters) and US2 (detail) share helpers/fixtures; implement foundational first.
- US3 tests after modules exist.

## Parallel opportunities

- T005/T006 helpers parallel
- T014/T015 tests parallel after implementation

## Implementation Strategy

MVP = US1 + US2 (filterable feed + detail). Ship tests in same PR. Do not touch Audit/Overview/Knowledge/Spec-kit.
