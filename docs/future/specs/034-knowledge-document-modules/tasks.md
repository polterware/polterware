# Tasks: Knowledge Document Detail Modules

**Input**: Design documents from `specs/034-knowledge-document-modules/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Helper unit + composition contracts for three kinds (Constitution XII). Mark complete only when package tests pass.

## Phase 1: Setup

- [x] T001 Add vitest config and package test script in `packages/features/knowledge/`
- [x] T002 [P] Create `knowledge-document-helpers.ts` with status/severity tones and property builders
- [x] T003 [P] Create shared `knowledge-document-shell.tsx` (document layout + maximize format dock)

## Phase 2: Kind modules (US1 + US2)

- [x] T004 [US1] Implement `document-detail.tsx` with enriched Doc properties + Archive action
- [x] T005 [US1] Implement `decision-detail.tsx` with status chip + alternatives/consequences reflection
- [x] T006 [US1] Implement `runbook-detail.tsx` with severity chip + steps/commands summary
- [x] T007 [US2] Honest not-found paths for missing document/decision/runbook targets
- [x] T008 [US1] Slim `index.tsx` to orchestration; wire kind detail modules; remove inline detail trees

## Phase 3: Tests (US3)

- [x] T009 [P] [US3] `document-detail.test.ts` composition + helper coverage
- [x] T010 [P] [US3] `decision-detail.test.ts` composition + helper coverage
- [x] T011 [P] [US3] `runbook-detail.test.ts` composition + helper coverage
- [x] T012 Run `pnpm --filter @polterware/feature-knowledge test` and fix failures

## Phase 4: Rubric gate

- [x] T013 Self-check acceptance rubric in spec.md (all green)
- [x] T014 Confirm no Spec Kit package / overview / activity / audit / architecture edits

## Dependency graph

```text
T001 → T002/T003 → T004/T005/T006 → T007 → T008 → T009/T010/T011 → T012 → T013/T014
```
