# Tasks: Ark UI–Backed Interactive Primitives

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md, spec.md (user stories US1–US6)

**Tests**: Constitution XII — local API unit/export tests; static Kobalte/Ark/hand-roll greps. Do not complete while typecheck/tests fail for changed surfaces. **AGENTS**: no `pnpm dev` unless Erick asks.

**Organization**: By user story. **Headless backend = `@ark-ui/solid` only. Look = DESIGN + tokens + Tailwind. Local DS components = public API.**

**Constitution**: IV wrap-once; XIV single headless (Ark) + Kobalte purge; XIII no scenic packages.

## Format: `[ID] [P?] [Story] Description`

## Path Conventions

- DS: `packages/frontend/design-system/src/`
- Commander: `packages/features/commander/src/`
- Shell: `packages/features/shell/src/`
- Architecture: `packages/features/architecture/src/canvas/`
- Docs: `AGENTS.md`, `DESIGN.md`
- Notes: `specs/009-ark-ui-primitives/`

---

## Phase 1: Setup

**Purpose**: Baseline inventory + add Ark dependency (Kobalte still present until cutover)

- [x] T001 Capture baseline greps into `specs/009-ark-ui-primitives/notes-baseline-grep.md` (`@kobalte/core`, feature Escape/dialog shells, current DS headless imports) per [quickstart.md](./quickstart.md) §1
- [x] T002 [P] Add `@ark-ui/solid` dependency to `packages/frontend/design-system/package.json` and update workspace lockfile (`pnpm install` at repo root as needed); do **not** remove Kobalte until T-series purge phase
- [x] T003 [P] Create disposition tracker `specs/009-ark-ui-primitives/notes-disposition-status.md` from [contracts/primitive-disposition.md](./contracts/primitive-disposition.md)

**Checkpoint**: Ark available; baseline known

---

## Phase 2: Foundational

**Purpose**: Document layering rules; prepare export surface — blocks story depth

- [x] T004 Record layering rule (Ark = behavior; Polter = look; local wrappers = public API) in `specs/009-ark-ui-primitives/notes-disposition-status.md` and confirm no feature should import Ark/Kobalte when wrappers exist
- [x] T005 [P] Audit `packages/frontend/design-system/src/index.ts` export list for interactive vs presentational split (notes only if no code change yet)

**Checkpoint**: Foundation ready

---

## Phase 3: User Story 1 — Overlays behave the same (P1) 🎯 MVP

**Goal**: Dialog, Drawer, Popover (and commander/terminal) share Escape/focus contracts via **Ark-backed local** components with **unchanged Polter visuals**

**Independent Test**: Dialog / palette / drawer / switcher Popover dismiss + focus return; visual still Polter

### Tests

- [x] T006 [P] [US1] Extend `packages/frontend/design-system/src/dialog.test.tsx` for stable public `A11yDialog` API (open/onClose/title)
- [x] T007 [P] [US1] Extend `packages/frontend/design-system/src/drawer.test.tsx` for public Drawer props contract

### Implementation

- [x] T008 [US1] Re-implement `packages/frontend/design-system/src/dialog.tsx` on **Ark UI Dialog** (behavior only); keep `dialog.css` product frame, `chrome`/`dividers`, surface-panel-float; preserve public `A11yDialog`/`Dialog` props; remove Kobalte from this file
- [x] T009 [US1] Re-implement `packages/frontend/design-system/src/popover.tsx` on **Ark UI Popover**; keep `popover.css` + anchor/placement public API as practical; Polter surface only; remove Kobalte from this file
- [x] T010 [US1] Rebuild `packages/frontend/design-system/src/drawer.tsx` on Ark Dialog/Drawer sheet (CSS edge layout); remove document Escape; preserve public props
- [x] T011 [US1] Migrate `packages/features/commander/src/index.tsx` command palette to design-system `A11yDialog` (flush + product frame); remove local Escape/custom dialog shell
- [x] T012 [P] [US1] Ensure `packages/features/commander/package.json` depends on design-system only (no Ark/Kobalte)
- [x] T013 [US1] Migrate `packages/features/shell/src/terminal-side-panel.tsx` to design-system `Drawer`; remove local Escape/shell
- [x] T014 [US1] Verify `packages/features/shell/src/context-switchers.tsx` still works with Ark-backed `Popover` (fix only if broken)

**Checkpoint**: US1 — overlays Ark-backed locals; commander/terminal cleaned

---

## Phase 4: User Story 2 — Menus and section tabs (P1)

**Goal**: Tabs + honest menus / DropdownMenu via Ark; pill styles preserved

**Independent Test**: Tab arrows; no false menu roles; DropdownMenu keyboard if shipped

### Tests

- [x] T015 [P] [US2] Extend `packages/frontend/design-system/src/tabs.test.tsx` for `items`/`value`/`onChange` API
- [x] T016 [P] [US2] Add `packages/frontend/design-system/src/menu.test.tsx` and/or `dropdown-menu.test.tsx` for export contracts

### Implementation

- [x] T017 [US2] Rebuild `packages/frontend/design-system/src/tabs.tsx` on Ark Tabs; keep `.pw-tabs` CSS and public API; remove hand-rolled keydown-as-sole-behavior
- [x] T018 [US2] Typecheck/fix Tabs consumers under `packages/features/*/src` that import `Tabs` if props break
- [x] T019 [US2] Fix `packages/frontend/design-system/src/menu.tsx`: presentational rows for Popover lists; remove false `role="menu"`/`menuitem` without Menu root
- [x] T020 [US2] Add Ark-backed `packages/frontend/design-system/src/dropdown-menu.tsx` (Menu) with `.menu-panel` Polter styles
- [x] T021 [US2] Export DropdownMenu from `packages/frontend/design-system/src/index.ts`
- [x] T022 [US2] Optional first compact-menu consumer or note “wrapper ready” in `notes-disposition-status.md`

**Checkpoint**: US2 complete

---

## Phase 5: User Story 3 — Feature hand-rolls gone (P1)

**Goal**: Known hand-rolls use local DS only; residual Escape/dialog shells cleaned

**Independent Test**: Residual grep clean or deferred written; no feature Ark/Kobalte

- [x] T023 [P] [US3] Update residual scan notes `specs/009-ark-ui-primitives/notes-residual-grep.md`
- [x] T024 [US3] Migrate `packages/features/architecture/src/canvas/relationship-type-popover.tsx` to design-system `Popover` + rows
- [x] T025 [P] [US3] Verify `packages/features/shell/src/target-picker.tsx` and `user-menu.tsx` composition (Popover + presentational rows only)
- [x] T026 [US3] Resolve remaining feature `document` Escape / custom `aria-modal` under `packages/features/**` or document deferrals in disposition notes
- [x] T027 [US3] Enforce zero `@ark-ui/solid` and `@kobalte/core` under `packages/features`

**Checkpoint**: US3 complete

---

## Phase 6: User Story 4 — Visual SSOT / presentational leave (P1)

**Goal**: No visual library leakage; presentational atoms stay simple; DESIGN.md not rewritten for Ark

- [x] T028 [P] [US4] Audit presentational exports remain free of forced Ark roots; mark in `notes-disposition-status.md`
- [x] T029 [P] [US4] Visual spot-check notes: dialog/popover/tabs/drawer still Polter (no Ark docs theme) in `specs/009-ark-ui-primitives/notes-visual-parity.md`
- [x] T030 [US4] Optional toast harden in `packages/frontend/design-system/src/toast.tsx` or defer with rationale
- [x] T031 [US4] Confirm `ResizableSplit` out-of-scope; no DESIGN.md rewrite for Ark

**Checkpoint**: US4 complete

---

## Phase 7: User Story 5 — Kobalte removed; Ark only (P1)

**Goal**: SC-008/009 — full Kobalte purge; Ark sole headless backend

- [x] T032 [US5] Remove all remaining `@kobalte/core` imports under `packages/frontend/design-system/src/**`
- [x] T033 [US5] Remove `@kobalte/core` from `packages/frontend/design-system/package.json` and refresh lockfile
- [x] T034 [US5] Workspace `rg '@kobalte/core'` over `packages` + `apps/desktop` is empty (fix presentation if accidental; product paths must be clean)
- [x] T035 [US5] Retarget any tests still asserting Kobalte markup/API under `packages/frontend/design-system/src/*.test.tsx`

**Checkpoint**: Kobalte gone from product UI stack

---

## Phase 8: User Story 6 — Inventory SSOT (P2)

**Goal**: Disposition 100%; dual paths gone; docs LAW

- [x] T036 [US6] Finalize `notes-disposition-status.md` all rows done/deferred
- [x] T037 [US6] Remove temporary dual local implementations if any remain under design-system
- [x] T038 [P] [US6] Rewrite AGENTS.md A11y LAW: Ark UI behavior-only via DS wrappers; Kobalte forbidden; look = DESIGN/tokens/Tailwind
- [x] T039 [P] [US6] Update DESIGN.md stack pointer (Ark via DS; no visual kit defaults)
- [x] T040 [US6] Run quickstart static §1 + package tests; record `specs/009-ark-ui-primitives/notes-static-checks.md`

**Checkpoint**: Inventory + docs closed

---

## Phase 9: Polish

- [x] T041 [P] Typecheck design-system + shell + commander + architecture as touched
- [x] T042 Convergence: code ↔ spec ↔ plan ↔ tasks
- [x] T043 Confirm completion criteria in [quickstart.md](./quickstart.md) §5 (not compile-only)

---

## Dependencies & Execution Order

```text
Setup → Foundational
  → US1 (Dialog/Popover/Drawer + commander/terminal)  🎯 MVP
  → US2 (Tabs/Menu)           [can parallel US1 after T008 if careful on index.ts]
  → US3 (hand-rolls)          after Popover/Drawer ready
  → US4 (visual/presentational)
  → US5 (Kobalte purge)       after all Kobalte call sites migrated
  → US6 (inventory/docs)
  → Polish
```

### MVP

T001–T005 → T006–T014 (US1) → validate overlays → then continue.

### Single-agent order

T001→T043 sequential as listed; parallel only [P] pairs that do not touch the same file.

---

## Notes

- **Ark = how it works; DESIGN/tokens/Tailwind = how it looks**  
- Local styled components over Ark are **required**, not a violation  
- Do not hand-roll focus/Escape/portal/position when Ark provides them  
- Do not apply Ark docs themes  
- Searchable switchers = Popover + presentational rows  
- View filters ≠ section Tabs  
- Compile ≠ done  
)
