---

description: "Task list template for feature implementation"
---

# Tasks: [FEATURE NAME]

**Input**: Design documents from `/specs/[###-feature-name]/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Include tests proportional to risk per Constitution X (unit for
domain rules, contract for APIs/messages, integration at critical boundaries,
e2e for critical flows, security for sensitive ops, performance when streams/
logs/canvas/large collections change). Do not mark a task complete while
required tests, lint, typecheck, contract generation, or builds fail for the
changed surface. Mocks MUST NOT replace all critical-boundary integration tests.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

**Constitution**: Tasks MUST preserve current-scope honesty (I–II), runtime
separation (III), contracts/storage (VI), shared UI (VII), security (VIII),
maintainability (IX), and verification gates (X).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Prefer active paths from `plan.md` and Constitution III/VI/VII boundaries:

- **Desktop UI**: `packages/features/<feature>/` composed by `apps/desktop/src/renderer/`
- **Desktop coordination**: `apps/desktop/src/main/`, `preload/`, and `shared/`
- **Core domains/composition**: `backend/domains/` and `backend/compositions/`
- **Contracts**: `schemas/` and generated `packages/contracts/` messages
- **Providers/platform**: `backend/providers/` and `backend/platform/`
- **Shared frontend**: `packages/frontend/` with real consumers
- **Runtime**: `backend/runtimes/core/`; future runtimes only with an accepted spec
- **Tests**: unit (domain) | contract | integration | e2e | security | performance by risk

Create folders only when responsibility, public API, and concrete use exist
(Constitution IX). No speculative PPM, Web App, Cloud, or MCP scaffolding.
Prefer shared design-system primitives over feature-owned interaction chrome.

<!--
  ============================================================================
  IMPORTANT: The tasks below are SAMPLE TASKS for illustration purposes only.

  The /speckit-tasks command MUST replace these with actual tasks based on:
  - User stories from spec.md (with their priorities P1, P2, P3...)
  - Feature requirements from plan.md
  - Entities from data-model.md
  - Endpoints from contracts/

  Tasks MUST be organized by user story so each story can be:
  - Implemented independently
  - Tested independently
  - Delivered as an MVP increment

  DO NOT keep these sample tasks in the generated tasks.md file.
  ============================================================================
-->

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Create project structure per implementation plan
- [ ] T002 Initialize [language] project with [framework] dependencies
- [ ] T003 [P] Configure linting and formatting tools

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

Examples of foundational tasks (adjust based on feature plan and constitution):

- [ ] T004 Define or extend the current domain model in [exact path]
- [ ] T005 [P] Add or update shared typed contracts for the feature surface
- [ ] T006 [P] Add the required provider or persistence adapter in [exact path]
- [ ] T007 Wire the owning core service and typed Electron bridge in [exact path]
- [ ] T008 Preserve current secret/output limitations without claiming redaction that does not exist
- [ ] T009 Use DESIGN.md tokens and shared product primitives in [exact UI path]
- [ ] T010 Create only packages this feature truly needs, with a concrete consumer
- [ ] T011 Define risk-proportional test and static-validation commands

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - [Title] (Priority: P1) 🎯 MVP

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 1 (risk-proportional — Constitution X) ⚠️

> **NOTE: Write domain/contract tests FIRST where rules are new; ensure they FAIL before implementation**

- [ ] T012 [P] [US1] Unit tests for domain invariants in [domain test path]
- [ ] T013 [P] [US1] Contract test for [API/message] in [contract test path]
- [ ] T014 [P] [US1] Integration test for critical boundary in [integration test path]

### Implementation for User Story 1

- [ ] T015 [P] [US1] Create or extend domain behavior in [domain path]
- [ ] T016 [P] [US1] Define or extend contracts in [schema/contract path]
- [ ] T017 [US1] Implement adapters or providers in [provider path]
- [ ] T018 [US1] Compose the core, bridge, and UI surface without duplicate authority
- [ ] T019 [US1] Add boundary validation, structured errors, and correlation IDs
- [ ] T020 [US1] Document live, failed, partial, and unavailable behavior honestly

**Checkpoint**: User Story 1 functional, independently testable, constitution gates held

---

## Phase 4: User Story 2 - [Title] (Priority: P2)

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 2 (risk-proportional — Constitution X) ⚠️

- [ ] T021 [P] [US2] Unit/contract tests for [rules or messages]
- [ ] T022 [P] [US2] Integration test for [critical boundary]

### Implementation for User Story 2

- [ ] T023 [P] [US2] Domain/contract changes in [path]
- [ ] T024 [US2] Adapter + runtime composition in [path]
- [ ] T025 [US2] Integrate with User Story 1 without dual implementations

**Checkpoint**: User Stories 1 AND 2 work independently

---

## Phase 5: User Story 3 - [Title] (Priority: P3)

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 3 (risk-proportional — Constitution X) ⚠️

- [ ] T026 [P] [US3] Unit/contract tests for [rules or messages]
- [ ] T027 [P] [US3] Integration or e2e test for [critical flow]

### Implementation for User Story 3

- [ ] T028 [P] [US3] Domain/contract changes in [path]
- [ ] T029 [US3] Adapter + runtime composition in [path]

**Checkpoint**: All user stories independently functional

---

[Add more user story phases as needed, following the same pattern]

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] TXXX [P] Documentation updates (contracts, boundaries, decisions)
- [ ] TXXX Remove temporary dual paths and unrelated dead code (IX)
- [ ] TXXX Performance: bounds, backpressure, virtualization as specified
- [ ] TXXX [P] Fill remaining risk-proportional tests (X)
- [ ] TXXX Security review of typed boundaries and sensitive output (VIII)
- [ ] TXXX Convergence check: code ↔ current spec ↔ product vision ↔ plan ↔ tasks
- [ ] TXXX Run quickstart.md validation

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - May integrate with US1 but should be independently testable
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - May integrate with US1/US2 but should be independently testable

### Within Each User Story

- Risk-proportional tests MUST be written and FAIL before domain/contract implementation
- Domain behavior and contracts before adapters
- Adapters before runtime composition
- Core implementation before cross-story integration
- Story complete only when constitution gates and required checks hold

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All tests for a user story marked [P] can run in parallel
- Models within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch risk-proportional tests for User Story 1 together:
Task: "Unit tests for domain invariants in [domain test path]"
Task: "Contract test for [API/message] in [contract test path]"

# Launch domain work together:
Task: "Create domain entities/use cases in [domain path]"
Task: "Define/extend contracts in [schema/contract path]"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1
   - Developer B: User Story 2
   - Developer C: User Story 3
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing domain rules
- Do not complete while required lint/typecheck/tests/contract gen/builds fail
- No permanent dual implementations; no scenic empty packages
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- Implementation ends only after code ↔ spec ↔ plan ↔ tasks convergence
