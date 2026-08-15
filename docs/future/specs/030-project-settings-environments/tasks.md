# Tasks: Project Settings Environments

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md, spec.md

**Tests**: Risk-proportional — pure helpers for kind/protection/tab normalize; composition source contracts for table + detail + general/advanced. Agents.md: no `pnpm dev` unless asked.

**Constitution / guardrails**:

- DetailPanel LAW + DataTable LAW + StatusChip LAW + Field/Dialog primitives
- Tabs: General · Environments · Advanced only
- Mock honesty for save / promote / demote / archive
- FORBIDDEN: org-team, integrations secondary, architecture, operations, terminal; reintroduce Access/Integrations into Settings

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [x] T001 Create feature directory `specs/030-project-settings-environments/` with plan artifacts
- [x] T002 Point `.specify/feature.json` to `specs/030-project-settings-environments`
- [x] T003 Add vitest + `@polterware/frontend-state` to `packages/features/settings/package.json` and `vitest.config.ts`

## Phase 2: Foundational

- [x] T004 [P] Add `types.ts` (ProjectEnvironment, kinds, protection, general snapshot)
- [x] T005 [P] Add `fixtures.ts` with `SETTINGS_ENVIRONMENT_MOCKS` (≥3 envs, all protections, rules on prod, dataSource fixture)
- [x] T006 [P] Implement `environment-status.ts` (kind/protection labels + tones)
- [x] T007 [P] Implement `settings-helpers.ts` (normalizeTab, default select options)
- [x] T008 [P] Helper unit tests: `environment-status.test.ts`, `settings-helpers.test.ts`

## Phase 3: US1 Environments table (P1)

- [x] T009 [US1] Rewrite Environments tab in `index.tsx`: DataTable PrimaryCell, kind StatusChip, protection StatusChip, resources, nodes, default
- [x] T010 [US1] Composition contract tests in `environments-table.test.ts`

## Phase 4: US2 Environment DetailPanel (P1)

- [x] T011 [US2] Implement `environment-detail.tsx` sections + rules + promote/demote actions + loading/error/not-found
- [x] T012 [US2] Wire registerCollection + activate + render EnvironmentDetail when scope active
- [x] T013 [US2] Detail composition contract tests in `environment-detail.test.ts`

## Phase 5: US3 General form (P1)

- [x] T014 [US3] General tab: Field + Input/Textarea/Select, mock save toast, remove danger zone from General
- [x] T015 [US3] Contract asserts Field usage and no Access/Integrations tabs

## Phase 6: US4 Advanced danger zone (P2)

- [x] T016 [US4] Advanced: Field retention controls + Archive with A11yDialog confirm + honest mock toast
- [x] T017 [US4] Contract asserts Dialog / danger archive honesty

## Phase 7: Polish

- [x] T018 Run `pnpm --filter @polterware/feature-settings test`; confirm rubric green
- [x] T019 Confirm git diff does not reintroduce Access/Integrations tabs and does not touch forbidden packages

## Parallel opportunities

- T004–T008 after T003
- T011 can start after T004–T006; T009 after fixtures
- T014 and T016 independent of detail once helpers exist
