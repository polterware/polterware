---

description: "Task list template for feature implementation"
---

# Tasks: [FEATURE NAME]

**Input**: Design documents from `/specs/[###-feature-name]/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Include tests proportional to risk per Constitution XII (unit for
domain rules, contract for APIs/messages, integration at critical boundaries,
e2e for critical flows, security for sensitive ops, performance when streams/
logs/canvas/large collections change). Do not mark a task complete while
required tests, lint, typecheck, contract generation, or builds fail for the
changed surface. Mocks MUST NOT replace all critical-boundary integration tests.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

**Constitution**: Tasks MUST respect dependency direction (IV), contracts-first
(V), security (VI), single source of truth (XIV), and real-code-only (XIII).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Prefer paths from `plan.md` and Constitution IV dependency direction:

- **Desktop UI**: `apps/desktop/src/renderer/features/<feature>/` (retain terminal/dock/process assets)
- **Desktop coordination**: `apps/desktop/src/main/`, `preload/`, `shared/` (thin; no domain engine in Main)
- **Domains / ports / compositions**: packages or crates as defined by target architecture
- **Contracts**: shared protocol/types packages — no hand-duplicated runtime types
- **Providers / platform**: adapters implementing domain ports only
- **Shared browser-safe UI**: `packages/ui`, `feature-*` only with real consumers
- **Runtimes**: polter-core, polter-server, polter-node, CLI, MCP only when in scope
- **Tests**: unit (domain) | contract | integration | e2e | security | performance by risk

Create folders only when responsibility, public API, and concrete use exist
(Constitution XIII). No scenic architecture. Prefer extracting shared UI over
duplicating web/desktop. See `docs/ARCHITECTURE.md`.

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

- [ ] T004 Define/extend domain entities, invariants, and ports (no framework imports)
- [ ] T005 [P] Add or update shared typed contracts for the feature surface
- [ ] T006 [P] Place adapters in providers/platform; wire composition in the owning runtime
- [ ] T007 Wire authn/authz and policy checks for org/system/environment/resource/operation
- [ ] T008 Configure structured logging/audit with correlation IDs and secret redaction
- [ ] T009 Ensure DESIGN.md tokens/layout and performance constraints (no Main/UI blocking)
- [ ] T010 Create only packages this feature truly needs (API + consumer); no empty scaffolds
- [ ] T011 Define risk-proportional test surfaces (unit/contract/integration/e2e/security)

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - [Title] (Priority: P1) 🎯 MVP

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 1 (risk-proportional — Constitution XII) ⚠️

> **NOTE: Write domain/contract tests FIRST where rules are new; ensure they FAIL before implementation**

- [ ] T012 [P] [US1] Unit tests for domain invariants in [domain test path]
- [ ] T013 [P] [US1] Contract test for [API/message] in [contract test path]
- [ ] T014 [P] [US1] Integration test for critical boundary in [integration test path]

### Implementation for User Story 1

- [ ] T015 [P] [US1] Create domain entities/use cases in [domain path]
- [ ] T016 [P] [US1] Define/extend ports in [ports path]
- [ ] T017 [US1] Implement adapters/providers in [provider path] (depends on ports)
- [ ] T018 [US1] Compose runtime surface (UI/CLI/MCP/core) without duplicating domain rules
- [ ] T019 [US1] Add boundary validation, structured errors, and correlation IDs
- [ ] T020 [US1] Add audit/policy hooks if operation is executable

**Checkpoint**: User Story 1 functional, independently testable, constitution gates held

---

## Phase 4: User Story 2 - [Title] (Priority: P2)

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 2 (risk-proportional — Constitution XII) ⚠️

- [ ] T021 [P] [US2] Unit/contract tests for [rules or messages]
- [ ] T022 [P] [US2] Integration test for [critical boundary]

### Implementation for User Story 2

- [ ] T023 [P] [US2] Domain/ports changes in [path]
- [ ] T024 [US2] Adapter + runtime composition in [path]
- [ ] T025 [US2] Integrate with User Story 1 without dual implementations

**Checkpoint**: User Stories 1 AND 2 work independently

---

## Phase 5: User Story 3 - [Title] (Priority: P3)

**Goal**: [Brief description of what this story delivers]

**Independent Test**: [How to verify this story works on its own]

### Tests for User Story 3 (risk-proportional — Constitution XII) ⚠️

- [ ] T026 [P] [US3] Unit/contract tests for [rules or messages]
- [ ] T027 [P] [US3] Integration or e2e test for [critical flow]

### Implementation for User Story 3

- [ ] T028 [P] [US3] Domain/ports changes in [path]
- [ ] T029 [US3] Adapter + runtime composition in [path]

**Checkpoint**: All user stories independently functional

---

[Add more user story phases as needed, following the same pattern]

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] TXXX [P] Documentation updates (contracts, boundaries, decisions)
- [ ] TXXX Remove temporary dual paths / legacy dead code (XIV, XV)
- [ ] TXXX Performance: bounds, backpressure, virtualization as needed (IX)
- [ ] TXXX [P] Fill remaining risk-proportional tests (XII)
- [ ] TXXX Security review of sensitive operations (VI, VII)
- [ ] TXXX Convergence check: code ↔ spec ↔ plan ↔ tasks (Governance §6)
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
- Domain entities and ports before adapters
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
Task: "Define/extend ports in [ports path]"
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
