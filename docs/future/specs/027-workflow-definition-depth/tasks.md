# Tasks: Workflow Definition Depth

**Input**: [spec.md](./spec.md), [plan.md](./plan.md), [data-model.md](./data-model.md), [contracts/workflow-builder-ui.md](./contracts/workflow-builder-ui.md)

## Phase 1: Setup

- [x] T001 Confirm feature dir `specs/027-workflow-definition-depth/` and `.specify/feature.json` point here
- [x] T002 [P] Review current `packages/features/operations/src/{types,fixtures,workflow-builder,index}.tsx` boundaries (no run/schedule edits)

## Phase 2: Foundational model

- [x] T003 Enrich `WorkflowDef` / `WorkflowStep` types in `packages/features/operations/src/types.ts` (version, failureStrategy, retry, approval, I/O summaries)
- [x] T004 [P] Add `packages/features/operations/src/workflow-status.ts` (label + tone for Draft/Ready/…)
- [x] T005 [P] Add `packages/features/operations/src/workflow-model.ts` (createStep defaults, cloneSteps, bumpVersion, buildSavedWorkflow, stepConfigFields contract)
- [x] T006 [P] Tests: `workflow-status.test.ts` + `workflow-model.test.ts`

## Phase 3: User Story 1–3 — Builder depth (P1)

- [x] T007 [US1] Enrich `WORKFLOW_FIXTURES` in `fixtures.ts` (≥2 rich: approval + retries; version; failureStrategy; summaries)
- [x] T008 [US1] Upgrade `workflow-builder.tsx` trigger panel: description, failure strategy, I/O summaries, version display
- [x] T009 [US1] Type-specific step config panels with distinct fields for ≥3 types; shared retry block for executable steps
- [x] T010 [US2] StatusChip for workflow status in builder header/metadata; honesty copy for immutable run versions
- [x] T011 [US3] Approval semantics fields + copy (approval ≠ execution); retry respects policy copy

## Phase 4: User Story 4 — Edge + recent runs (P2)

- [x] T012 [US4] Empty pipeline state (ensure present) + not-found workflow chrome in builder
- [x] T013 [US4] Recent runs list/buttons via `recentRuns` + `onOpenRun` props
- [x] T014 [US4] Wire `index.tsx`: workflow StatusChip helpers, pass recent runs from RUN_FIXTURES, onOpenRun, not-found when missing target

## Phase 5: User Story 5 — Tests & polish (P2)

- [x] T015 [US5] `workflow-builder.test.ts` composition/fixture depth contracts (distinct step configs, version bump helper, ≥2 rich fixtures)
- [x] T016 Export new helpers from `index.tsx` as needed
- [x] T017 Run `pnpm --filter @polterware/feature-operations test` and fix failures
- [x] T018 Rubric pass: confirm run-detail/schedule-detail untouched (`git diff --stat`)

## Dependencies

T003 → T005 → T008–T011  
T004 → T010, T014  
T007 → T015  
T012–T014 after builder props exist  

## MVP

T003–T011 + T015–T017 (definition depth + tests). Recent runs (T012–T014) complete full rubric loop.
