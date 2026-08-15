---
description: "Task list for Shared UI Package implementation"
---

# Tasks: Shared UI Package

**Input**: Design documents from `/specs/001-shared-ui-package/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Not requested in the feature specification — no TDD/test-authoring phase. Validation uses typecheck, boundary scans, existing Desktop tests (when Erick asks), and quickstart.md checks.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Shared UI package**: `packages/ui/`
- **Desktop consumer**: `apps/desktop/`
- **Current primitives source**: `apps/desktop/src/renderer/components/ui/`
- **Design tokens/surfaces source**: `apps/desktop/src/renderer/styles.css`
- **Desktop shell (do not move)**: `apps/desktop/src/renderer/shell.css`, `apps/desktop/src/renderer/features/shell/`
- **Domain features (do not extract now)**: `apps/desktop/src/renderer/features/*`
- **Contracts reference**: `specs/001-shared-ui-package/contracts/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the workspace package skeleton and dependency wiring without moving all primitives yet

- [x] T001 Create package root layout at `packages/ui/package.json`, `packages/ui/tsconfig.json`, and `packages/ui/src/` directories (`components/`, `hooks/`, `lib/`, `styles/`)
- [x] T002 Define `@polterware/ui` package metadata, `type: module`, workspace-friendly `exports` (`.` + styles entry), `peerDependencies` for `react`/`react-dom`, and `scripts.typecheck` in `packages/ui/package.json`
- [x] T003 [P] Configure TypeScript for browser-safe React JSX sources in `packages/ui/tsconfig.json` (strict, no Node privilege libs, include `src/**/*`)
- [x] T004 [P] Add empty public barrel skeleton in `packages/ui/src/index.ts` and styles placeholder entry in `packages/ui/src/styles/index.css`
- [x] T005 Add `"@polterware/ui": "workspace:*"` dependency in `apps/desktop/package.json` and run workspace link resolution (`pnpm install` only if Erick asks / if required for local verification)
- [x] T006 [P] Document package purpose and forbidden imports in `packages/ui/README.md` (browser-safe design system; no Electron/Node privilege)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared helpers, token/CSS ownership split, import conventions, and Desktop consumption path — MUST complete before story work finishes moving primitives

**⚠️ CRITICAL**: No user story completion until this foundation exists

- [x] T007 Move `cn` helper from `apps/desktop/src/renderer/lib/utils.ts` into `packages/ui/src/lib/utils.ts` and export it from `packages/ui/src/index.ts`
- [x] T008 Extract design tokens + shared surface utilities from `apps/desktop/src/renderer/styles.css` into `packages/ui/src/styles/index.css` (theme variables, `:root`/`.dark` design tokens, `polterware-surface*`, `polterware-glass-surface`, `polterware-frosted-surface`, related material utilities required by primitives)
- [x] T009 Slim `apps/desktop/src/renderer/styles.css` to app toolchain imports (Tailwind/tw-animate/shadcn/font) + `@import` of `@polterware/ui` styles entry + any truly desktop-only leftovers; keep `apps/desktop/src/renderer/shell.css` Desktop-owned and unmoved
- [x] T010 Establish package-internal import convention (relative imports only; no `@/` Desktop aliases) and note it in `packages/ui/README.md`
- [x] T011 [P] Inventory all primitive modules under `apps/desktop/src/renderer/components/ui/` into a move checklist note in `specs/001-shared-ui-package/contracts/package-public-api.md` (or adjacent implementation note) listing public export names
- [x] T012 [P] Identify mixed/privileged candidates (if any) under `apps/desktop/src/renderer/components/ui/` and `apps/desktop/src/renderer/features/shared/` using `specs/001-shared-ui-package/contracts/ownership-boundary.md`; mark stay-local vs split vs move
- [x] T013 Ensure Desktop bundler can resolve workspace UI sources in `apps/desktop/electron.vite.config.ts` / `apps/desktop/tsconfig.json` if needed (same class of workspace resolution used for `@polterware/core`)
- [x] T014 Create temporary re-export strategy files only if needed: `apps/desktop/src/renderer/lib/utils.ts` re-exports `cn` from `@polterware/ui`; plan `apps/desktop/src/renderer/components/ui/*` shims for migration (no duplicate implementations after move)

**Checkpoint**: Package skeleton, tokens entry, and Desktop dependency path exist — primitive move/consumption can proceed

---

## Phase 3: User Story 1 - Desktop keeps working on a shared design system (Priority: P1) 🎯 MVP

**Goal**: Desktop consumes `@polterware/ui` for extracted design primitives with visual/interaction parity on retained workbench surfaces

**Independent Test**: Desktop workbench flows (commander, settings, process boards, repository panels, system views) use package primitives; no parallel local primitive implementations remain; typecheck passes

### Implementation for User Story 1

- [x] T015 [US1] Move pure primitive modules from `apps/desktop/src/renderer/components/ui/` into `packages/ui/src/components/` (full current kit: button, input, dialog/sheet/drawer, menus, tabs, card, form primitives, texture-button/card + `texture-button.css`, etc.)
- [x] T016 [US1] Move primitive-only hook `apps/desktop/src/renderer/hooks/use-mobile.ts` into `packages/ui/src/hooks/use-mobile.ts` if still required by `sidebar`/responsive primitives
- [x] T017 [US1] Rewrite package-internal imports in `packages/ui/src/components/**` and `packages/ui/src/hooks/**` from `@/lib/utils` / `@/components/ui/*` / `@/hooks/*` to relative package paths
- [x] T018 [US1] Export all moved public primitives and helpers from `packages/ui/src/index.ts` per `specs/001-shared-ui-package/contracts/package-public-api.md` (stable names, no API rename unless required by privilege split)
- [x] T019 [US1] Wire Desktop feature imports to `@polterware/ui` in consumer files: `apps/desktop/src/renderer/App.tsx`, `apps/desktop/src/renderer/features/commander/commander-dialog.tsx`, `apps/desktop/src/renderer/features/processes/*.tsx`, `apps/desktop/src/renderer/features/repositories/repository-sidebar.tsx`, `apps/desktop/src/renderer/features/settings/*.tsx`, `apps/desktop/src/renderer/features/system/system-views.tsx`, `apps/desktop/src/renderer/features/shared/components.tsx`, `apps/desktop/src/renderer/features/shell/desktop-shell.tsx`, `apps/desktop/src/renderer/features/shell/workspace-panel-layout.tsx`, and remaining `@/components/ui` call sites
- [x] T020 [US1] Replace or delete duplicate implementations under `apps/desktop/src/renderer/components/ui/` so end state is either removed directory or thin re-export shims only (no second implementation tree)
- [x] T021 [US1] Update Desktop tests that import local UI paths (`apps/desktop/src/renderer/features/processes/process-command-dialog.test.tsx`, `apps/desktop/src/renderer/features/processes/process-views.test.tsx`, and any other affected tests) to resolve through `@polterware/ui` or temporary shims
- [x] T022 [US1] Retarget `apps/desktop/components.json` aliases/paths so future shadcn additions land in `packages/ui` (not a new Desktop-local primitive tree)
- [x] T023 [US1] Run static validation for US1: `pnpm --filter @polterware/ui typecheck` and `pnpm --filter @polterware/desktop typecheck` (or repo `pnpm typecheck` if preferred); fix breakage from move/import rewires
- [x] T024 [US1] Confirm DESIGN.md implementation pointers still accurate or update `DESIGN.md` CSS/config paths to package-owned styles where tokens moved (`packages/ui/src/styles/index.css`)

**Checkpoint**: US1 MVP — Desktop is on the shared design system with no parallel primitive source of truth

---

## Phase 4: User Story 2 - Web can consume the same browser-safe UI package (Priority: P2)

**Goal**: Package is browser-safe and documented as consumable by a future Web surface without Electron/Node privilege

**Independent Test**: Boundary scan clean; public exports + CSS entry usable without Desktop adapters; documented Web readiness smoke in quickstart/contracts

### Implementation for User Story 2

- [x] T025 [US2] Audit `packages/ui/src/**` for forbidden imports (Electron, `node:*`, FS/exec/PTY, `window.polterware`, `@polterware/core` privileged paths) and remove/split any violations per `specs/001-shared-ui-package/contracts/ownership-boundary.md`
- [x] T026 [P] [US2] Finalize package `exports` map in `packages/ui/package.json` for JS entry (`.`) and CSS entry (`./styles.css` or equivalent) usable by browser bundlers
- [x] T027 [P] [US2] Ensure peer/runtime dependency declarations in `packages/ui/package.json` are Web-safe (React peers; no Electron deps; primitive libs only)
- [x] T028 [US2] Add Web readiness notes to `packages/ui/README.md` and align with `specs/001-shared-ui-package/contracts/consumer-integration.md` (how Web imports primitives + loads CSS; non-goals: full Web app)
- [x] T029 [US2] Prove consumability without Desktop: document or add a minimal browser-safe import smoke referenced from `specs/001-shared-ui-package/quickstart.md` step 6 (type-level import of `Button`/`cn` from `@polterware/ui` without Electron types)
- [x] T030 [US2] Re-run boundary scan command from `specs/001-shared-ui-package/quickstart.md` against `packages/ui` and keep it clean

**Checkpoint**: US2 — package is honestly Web-ready at the design-system boundary without shipping Web product runtime

---

## Phase 5: User Story 3 - Clear share vs shell boundary for future reuse (Priority: P3)

**Goal**: Contributors can decide package vs shell vs feature ownership quickly; Desktop shell/adapters remain specialized

**Independent Test**: Ownership docs + inventory show only browser-safe design system in package; shell/features/adapters stay app-owned

### Implementation for User Story 3

- [x] T031 [P] [US3] Update `CONVENTIONS.md` UI reuse section to point at `@polterware/ui` / `packages/ui` as the design-system home and keep shells/adapters app-local
- [x] T032 [P] [US3] Update `docs/ARCHITECTURE.md` active checkout / package growth notes to include `packages/ui` as created and list share-vs-do-not-share rules consistently
- [x] T033 [US3] Embed or link the decision table from `specs/001-shared-ui-package/contracts/ownership-boundary.md` into `packages/ui/README.md` (primitive vs shell vs feature vs adapter)
- [x] T034 [US3] Verify Desktop shell remains local: `apps/desktop/src/renderer/shell.css` and `apps/desktop/src/renderer/features/shell/**` not moved into package; dock/window chrome ownership unchanged
- [x] T035 [US3] Verify domain feature views remain Desktop-owned under `apps/desktop/src/renderer/features/**` (processes, commander composition, repositories, system, settings composition) and only import shared primitives
- [x] T036 [US3] Remove temporary Desktop re-export shims if still present under `apps/desktop/src/renderer/components/ui/` and `apps/desktop/src/renderer/lib/utils.ts` once all consumers import `@polterware/ui` directly (or document why a minimal shim remains)
- [x] T037 [US3] Confirm contributor can answer “shared package or app?” using `packages/ui/README.md` + `CONVENTIONS.md` + `docs/ARCHITECTURE.md` in under five minutes (SC-006)

**Checkpoint**: US3 — ownership boundary is explicit and enforceable for future work

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final validation and doc consistency across stories

- [x] T038 [P] Align `specs/001-shared-ui-package/contracts/package-public-api.md` export list with final `packages/ui/src/index.ts` reality
- [x] T039 [P] Align `specs/001-shared-ui-package/contracts/consumer-integration.md` with final Desktop dependency/styles wiring
- [x] T040 Run full quickstart validation checklist in `specs/001-shared-ui-package/quickstart.md` (package exists, boundary scan, Desktop dependency, typecheck; tests only if Erick asks)
- [x] T041 [P] Ensure `AGENTS.md` progressive disclosure still points correctly (no claim that Web runtime is live; package is runway)
- [x] T042 Final repo hygiene: no orphaned empty `apps/desktop/src/renderer/components/ui` implementation files, no duplicate `texture-button.css`, no stale `@/components/ui` imports outside intentional shims

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup — **BLOCKS** story completion
- **User Story 1 (Phase 3)**: Depends on Foundational; delivers MVP
- **User Story 2 (Phase 4)**: Depends on package content from US1 existing; can start boundary hardening once primitives live in `packages/ui`
- **User Story 3 (Phase 5)**: Depends on US1 ownership reality; docs can draft earlier but verify after moves
- **Polish (Phase 6)**: Depends on desired stories complete (MVP can polish after US1 only)

### User Story Dependencies

- **US1 (P1)**: After Foundational — no dependency on US2/US3
- **US2 (P2)**: Needs primitives in package (US1 move) to audit real code; independently testable via boundary/consumability
- **US3 (P3)**: Needs final ownership shape from US1; independently testable via docs + tree review

### Within Each User Story

- Move/export before consumer rewires (US1)
- Boundary audit after code is in package (US2)
- Docs/verify after structure stabilizes (US3)
- Story complete before treating next priority as done

### Parallel Opportunities

- Phase 1: T003/T004/T006 can run in parallel after T001
- Phase 2: T011/T012 in parallel; CSS extract (T008) sequential with Desktop slim (T009)
- US1: consumer file import updates in T019 can be parallelized by feature folder after T018
- US2: T026/T027 in parallel after audit starts
- US3: T031/T032 in parallel
- Polish: T038/T039/T041 in parallel

---

## Parallel Example: User Story 1

```bash
# After T015–T018 land package exports, parallelize consumer rewires by area:
Task: "Update commander imports in apps/desktop/src/renderer/features/commander/commander-dialog.tsx"
Task: "Update processes imports in apps/desktop/src/renderer/features/processes/*.tsx"
Task: "Update repositories/settings/system/shell/shared imports under apps/desktop/src/renderer/features/"
```

## Parallel Example: User Story 2

```bash
Task: "Finalize exports map in packages/ui/package.json"
Task: "Finalize peer/runtime deps in packages/ui/package.json"
Task: "Write Web readiness notes in packages/ui/README.md"
```

## Parallel Example: User Story 3

```bash
Task: "Update CONVENTIONS.md UI reuse pointers"
Task: "Update docs/ARCHITECTURE.md package growth notes"
Task: "Verify shell.css and features/shell remain Desktop-owned"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (`packages/ui` skeleton + Desktop workspace dep)
2. Complete Phase 2: Foundational (`cn`, tokens CSS entry, Desktop styles slim, conventions)
3. Complete Phase 3: US1 (move primitives, rewire Desktop, delete duplicates, typecheck)
4. **STOP and VALIDATE** using quickstart steps 1–5
5. Ship/demo Desktop-on-shared-UI before Web-readiness polish if needed

### Incremental Delivery

1. Setup + Foundational → package path exists
2. US1 → Desktop parity MVP
3. US2 → browser-safe Web readiness proof
4. US3 → ownership docs and shim cleanup
5. Polish → contracts/quickstart alignment

### Parallel Team Strategy

1. One owner finishes Setup + Foundational
2. Then:
   - Owner A: US1 move + Desktop rewires
   - Owner B: draft US3 docs (finalize after US1)
3. After US1: Owner B/C run US2 boundary audit + export map
4. Everyone shares typecheck/boundary gates before polish

---

## Notes

- [P] = different files, no incomplete-task dependency
- [USn] maps to spec user stories for traceability
- Do **not** create `apps/web`, services, crates, or domain `feature-*` packages in this feature
- Do **not** move `shell.css`, dock/window chrome, or privileged adapters into `packages/ui`
- Do **not** run `pnpm dev` / `build` / `dist` unless Erick asks; prefer typecheck + static scans
- Prefer small, reviewable moves: helpers/tokens first, then primitives, then consumer rewires
- Commit after each logical group if Erick requests commits; no auto-commit by default
