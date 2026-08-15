# Tasks: Operator Design System & Primary Surfaces

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/

**Tests**: Risk-proportional per Constitution XII — unit tests for design-system primitives, pure view-model/filter helpers, and canvas selection/serialize. No full e2e suite required unless claiming SC keyboard/visual gates; manual checks live in [quickstart.md](./quickstart.md). Do not mark a task complete while required typecheck/lint/tests/build fail for the changed surface. Agents.md: do not run `pnpm dev` unless the user asks.

**Organization**: Tasks grouped by user story for independent implementation and testing.

**Constitution**: Dependency direction IV; contracts-first V; security VI; SSoT XIV; real code only XIII; no permanent inspector on Resources/Architecture; brand green restricted.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: US1–US6 maps to spec user stories
- Include exact file paths in every task description

## Path Conventions (this feature)

- Design SSOT: `DESIGN.md` (same inode as `design.md` on macOS)
- Tokens: `packages/configuration/tailwind/base.css`
- Design system: `packages/frontend/design-system/`
- Canvas: `packages/frontend/canvas/`
- Features: `packages/features/{shell,resources,architecture}/`
- Composition: `apps/desktop/src/renderer/App.tsx`, `styles.css`
- Prototypes (read-only guide): `prototype/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm layout, deps strategy, and inventory before writing product UI

- [x] T001 Inventory current UI surfaces and provisional chrome in `packages/features/shell/src/index.tsx`, `packages/features/resources/src/index.tsx`, `packages/features/architecture/src/index.tsx`, `apps/desktop/src/renderer/App.tsx`, `packages/frontend/design-system/src/`, and `packages/configuration/tailwind/base.css`; note what to replace vs reuse in a short comment block at top of `specs/003-operator-design-system/plan.md` Implementation Phases or a `specs/003-operator-design-system/notes-inventory.md` if preferred
- [x] T002 [P] Review all images under `prototype/` for composition grammar (sidebar, MainSurface, density, Resources table, Resource detail, Architecture canvas, node detail) — capture 5–10 bullet notes in `specs/003-operator-design-system/notes-prototype.md`
- [x] T003 [P] Decide dependency add sites: add `@hugeicons/core-free-icons` (and `@kobalte/core` only if needed this feature) to `packages/frontend/design-system/package.json` and/or `apps/desktop/package.json` peer/deps per research R6 — do not run solidui-cli init; do not add React deps
- [x] T004 [P] Confirm no new packages will be created; list only existing packages that will change in `specs/003-operator-design-system/notes-inventory.md`

**Checkpoint**: Inventory + prototype notes ready; dep plan clear

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Design SSOT, tokens, shared primitives, route/view-model types, fixtures — **MUST complete before user story UI**

**⚠️ CRITICAL**: No US1–US6 product UI work until this phase completes

### Design SSOT + tokens

- [x] T005 Rewrite root `DESIGN.md` as Polter quiet-dark operator design system covering sections 1–13 from the product brief (identity, principles, anatomy, tokens, typography, components, navigation, blueprints, responsiveness, a11y, performance, implementation map, acceptance checklist); supersede light alpha theme as Desktop operator default
- [x] T006 Implement design tokens and semantic aliases in `packages/configuration/tailwind/base.css` per [contracts/design-tokens.md](./contracts/design-tokens.md) (app/shell/surface ladder, borders, text, brand restricted, danger/warning/info, radius, ease, shell dimensions); map legacy `--color-bg` / `--color-surface` / `--color-fg` / `--color-muted` / `--color-border` for transition
- [x] T007 Ensure `apps/desktop/src/renderer/styles.css` imports shared base only and sets full-height root against app-background tokens (no conflicting ad-hoc theme)

### Shared types + fixtures

- [x] T008 [P] Add ClientRoute, DialogState, ShellStatus, EnvironmentOption types in `packages/features/shell/src/route.ts` (or `packages/frontend/state/src/client-route.ts` if kept shared) per [data-model.md](./data-model.md)
- [x] T009 [P] Add Resource, ResourceStatus, ResourceDraft, QuickAction view-model types in `packages/features/resources/src/types.ts` per data-model
- [x] T010 [P] Add ArchitectureNode, ConnectionRef, ActivityItem, DocLink view-model types in `packages/features/architecture/src/types.ts` per data-model
- [x] T011 [P] Create prototype-scale fixtures with `dataSource: "fixture"` in `packages/features/resources/src/fixtures.ts` and `packages/features/architecture/src/fixtures.ts` (~8–12 resources, layered nodes/edges mixed evidence)

### Design-system primitives (shared foundation)

- [x] T012 Restyle quiet `Button` in `packages/frontend/design-system/src/button.tsx` (+ `button.css` or token-only Tailwind): compact sizes, neutral/primary/ghost/danger variants, focus-visible, remove default scale-on-active gimmick and brand-green fills; keep API compatible where possible
- [x] T013 [P] Restyle `StatusDot` / status presentation in `packages/frontend/design-system/src/status.tsx` so status is never color-only (label or aria)
- [x] T014 [P] Restyle `Badge`, `Separator`, `Card` (or replace Card with Surface primitive) in `packages/frontend/design-system/src/{badge,separator,card}.tsx` to quiet tokens; avoid card-in-card defaults
- [x] T015 Evolve accessible dialog in `packages/frontend/design-system/a11y/dialog.tsx` (focus trap basics, Escape, overlay, MainSurface-friendly styling); use Kobalte only if native path insufficient
- [x] T016 [P] Add shared empty/loading/error presentational helpers in `packages/frontend/design-system/src/feedback.tsx` (or split files) for page-level states
- [x] T017 [P] Add toast region primitive in `packages/frontend/design-system/src/toast.tsx` (no React Sonner)
- [x] T018 Export public primitives from `packages/frontend/design-system/src/index.ts`
- [x] T019 [P] Unit tests for Button variants and StatusDot accessibility in `packages/frontend/design-system/src/button.test.tsx` (extend) and `packages/frontend/design-system/src/status.test.tsx`
- [x] T020 Add `@hugeicons/core-free-icons` individual-import icon helper usage pattern documented in DESIGN.md Implementation map; first icons wired from shell/resources tasks (no multi-icon libraries)

**Checkpoint**: Foundation ready — DESIGN.md + tokens + primitives + types/fixtures exist; user stories can proceed

---

## Phase 3: User Story 1 — Consistent operator shell and visual language (Priority: P1) 🎯 MVP

**Goal**: Integrated sidebar + single rounded MainSurface + neutral active nav + restricted brand green; calm dark operator chrome

**Independent Test**: Open Desktop shell (or render Shell in isolation); navigate primary sections; verify anatomy vs DESIGN.md and prototype composition; active nav not green; no permanent inspector chrome in shell layout

### Implementation for User Story 1

- [x] T021 [US1] Rewrite `packages/features/shell/src/index.tsx` to AppWindow anatomy: integrated AppSidebar (grouped nav CORE/INSIGHTS/CONFIGURATION as product needs), MainSurface host, page header slot, shell status footer (core online, actor), per [contracts/shell-navigation.md](./contracts/shell-navigation.md)
- [x] T022 [US1] Implement nav items with neutral `surface-selected` active state and compact icons (@hugeicons/core-free-icons) in `packages/features/shell/src/sidebar.tsx` (split file if cleaner)
- [x] T023 [US1] Implement MainSurface + PageHeader layout primitives in `packages/features/shell/src/main-surface.tsx` (large radius, subtle border, hosts children only)
- [x] T024 [US1] Wire ClientRoute section navigation in `apps/desktop/src/renderer/App.tsx` (replace flat provisional panel list chrome with Shell props: route, onNavigate, status, navItems)
- [x] T025 [US1] Add macOS traffic-light / drag-region awareness via CSS classes on shell chrome in `packages/features/shell/src/` and/or `apps/desktop/src/renderer/styles.css` (adjust `apps/desktop/src/main/window.ts` only if frame padding required)
- [x] T026 [US1] Restyle non-P1 section placeholders rendered inside MainSurface so they use tokens (honest stubs, no fake dashboards) when those sections are selected from App.tsx Switch
- [x] T027 [US1] Verify brand green appears only as restricted signals in shell (no green active nav, no green large fills) against DESIGN.md acceptance checklist

**Checkpoint**: US1 independently testable — shell anatomy matches design system

---

## Phase 4: User Story 2 — Browse and open Resources as pages (Priority: P1)

**Goal**: Resources index (search/filter/table) + Resource detail page + create/edit dialog; no permanent right inspector

**Independent Test**: From Resources section, search/filter, open detail page, back, open create/edit dialog, see empty/loading/error; no fixed inspector column

### Tests for User Story 2

- [x] T028 [P] [US2] Unit tests for resource filter/search pure helpers in `packages/features/resources/src/filter.test.ts`

### Implementation for User Story 2

- [x] T029 [US2] Implement mapCoreResources + fixture merge helpers with honest `dataSource` in `packages/features/resources/src/map.ts`
- [x] T030 [US2] Implement Resources index page UI (header, environment control, search, filter chips, dense table columns Name/Type/Environment/Provider/Status/Last sync) in `packages/features/resources/src/resources-index.tsx` per [contracts/resources-ui.md](./contracts/resources-ui.md)
- [x] T031 [US2] Implement Resource detail page (breadcrumb/back, identity, status chips, overview, linked architecture, activity, notes, quick actions) in `packages/features/resources/src/resource-detail.tsx`
- [x] T032 [US2] Implement Resource create/edit dialog using design-system dialog in `packages/features/resources/src/resource-form-dialog.tsx` with draft validation per data-model
- [x] T033 [US2] Export ResourcesView router-by-props (index vs detail vs dialog open) from `packages/features/resources/src/index.tsx`
- [x] T034 [US2] Wire App composition: listResources from `@polterware/frontend-data`, fixture fallback, navigate ClientRoute page/detail/dialog for resources in `apps/desktop/src/renderer/App.tsx`
- [x] T035 [US2] Ensure Resources index/detail have loading, empty, and error presentations using design-system feedback helpers
- [x] T036 [US2] Optional: virtualize long resource tables with `@tanstack/solid-virtual` in `packages/features/resources/src/resources-index.tsx` when row count warrants

**Checkpoint**: US2 independently testable — page-based Resources without permanent inspector

---

## Phase 5: User Story 3 — Explore Architecture canvas and node detail pages (Priority: P1)

**Goal**: Full-width Architecture canvas with neutral selection + node detail page; remove permanent inspector; keep shell interactive

**Independent Test**: Open Architecture, select node (neutral), open node detail page, back; pan/zoom; no permanent right inspector; empty/loading/error work

### Tests for User Story 3

- [x] T037 [P] [US3] Unit tests for node view-model mapping / layer assignment in `packages/features/architecture/src/map-view.test.ts` (extend or add beside `map-model.test.ts`)
- [x] T038 [P] [US3] Keep canvas serialize/select tests green in `packages/frontend/canvas/src/map.test.ts`; add selection helper tests if new APIs

### Implementation for User Story 3

- [x] T039 [US3] Remove permanent `ResizableSplit` inspector from Architecture canvas page in `packages/features/architecture/src/index.tsx` (and any split defaults)
- [x] T040 [US3] Implement full-width Architecture canvas view (environment, search/tools, SVG/DOM nodes, evidence-aware edges, neutral selection, empty state) in `packages/features/architecture/src/architecture-canvas.tsx`
- [x] T041 [US3] Add layered/grouped presentation helpers (Cloud / Applications / Data bands as view layout) in `packages/features/architecture/src/layers.ts` without claiming false live wiring
- [x] T042 [US3] Add pan/zoom controls and interaction that do not freeze shell chrome for prototype-scale graphs in `packages/features/architecture/src/architecture-canvas.tsx` (use `packages/frontend/canvas/worker/layout.ts` if layout cost grows)
- [x] T043 [US3] Implement Architecture node detail page in `packages/features/architecture/src/node-detail.tsx` (overview, connections, relationship mini-view, activity, notes, docs, quick actions)
- [x] T044 [US3] Wire open-node → ClientRoute detail and canvas selection without leaving index until open in `packages/features/architecture/src/index.tsx` + `apps/desktop/src/renderer/App.tsx`
- [x] T045 [US3] Preserve layout save via existing `getLayout`/`updateLayout` paths in App composition; fixtures when core empty with honest labeling
- [x] T046 [US3] Ensure Architecture loading/empty/error states; selection styling uses neutral tokens only (no brand glow)

**Checkpoint**: US3 independently testable — canvas + node page model

---

## Phase 6: User Story 4 — Navigate with page depth and focused overlays (Priority: P2)

**Goal**: Clear grammar: pages for deep context, dialogs for short tasks, toast for transient feedback, optional command palette overlay; no permanent dual inspectors

**Independent Test**: From shell, open detail pages, open create dialog, dismiss, trigger toast; Resources/Architecture lack permanent right inspectors; optional command palette does not consume layout

### Implementation for User Story 4

- [x] T047 [US4] Centralize navigation helpers (`goSection`, `openDetail`, `closeDetail`, `openDialog`, `closeDialog`) in `packages/features/shell/src/route.ts` and use from App + features
- [x] T048 [US4] Mount toast region in Shell/App (`packages/features/shell/src/index.tsx` + design-system toast) and fire sample feedback on dialog success/error paths in resources flow
- [x] T049 [US4] Ensure create/edit only uses dialog state (not split panes) across Resources; document navigation model adherence in DESIGN.md Navigation model if gaps remain
- [x] T050 [US4] Optional: wire lightweight command palette overlay reusing `packages/features/commander` patterns as non-layout-consuming overlay from Shell/App — skip with note if deferred, but keep nav model documented
- [x] T051 [US4] Audit App.tsx Switch/composition: no permanent inspector columns for resources/architecture routes

**Checkpoint**: US4 navigation grammar consistent across in-scope surfaces

---

## Phase 7: User Story 5 — Accessible, dense desktop operation (Priority: P2)

**Goal**: Keyboard paths, focus-visible, non-color-only status, reduced-motion, usable 1280–1728 widths

**Independent Test**: Keyboard-tab shell + Resources + Architecture; focus rings visible; reduce-motion OK; resize smoke at target widths

### Implementation for User Story 5

- [x] T052 [US5] Audit and fix focus-visible / tab order for sidebar, page headers, tables, canvas controls, and dialogs in shell/resources/architecture design-system components
- [x] T053 [US5] Ensure ResourceStatus and Architecture status chips always include text or aria-label in resources/architecture components
- [x] T054 [US5] Add `prefers-reduced-motion` guards for transitions in `packages/configuration/tailwind/base.css` and any feature motion classes
- [x] T055 [US5] Tune dense spacing/control sizes against DESIGN.md Desktop density for sidebar width and table row height in shell/resources CSS classes
- [x] T056 [US5] Document narrow-window behavior (compact sidebar or overflow) in DESIGN.md Responsiveness and implement minimal graceful degradation in `packages/features/shell/src/`

**Checkpoint**: US5 a11y/density smoke criteria met for in-scope screens

---

## Phase 8: User Story 6 — Honest loading, empty, error, and offline states (Priority: P3)

**Goal**: Operators always know loading vs empty vs failed vs offline; no fake live claims

**Independent Test**: Force loading/empty/error/offline paths on Resources and Architecture; shell core status honest

### Implementation for User Story 6

- [x] T057 [US6] Unify page-level loading/empty/error components usage on Resources index/detail and Architecture canvas/detail (ensure all four surfaces covered)
- [x] T058 [US6] Wire shell `coreOnline` from health resource in `apps/desktop/src/renderer/App.tsx` and display honest offline/degraded copy in shell footer
- [x] T059 [US6] Label fixture data in UI when `dataSource === "fixture"` (subtle banner or footnote) so it is not mistaken for live provider sync
- [x] T060 [US6] Ensure not-found detail route (unknown entityId) shows error/back path on Resource and Node detail pages

**Checkpoint**: US6 honesty states complete

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Convergence, cleanup, verification

- [x] T061 [P] Remove dead provisional shell/header patterns and unused texture-button CSS if fully replaced (`packages/frontend/design-system/src/button.css` cleanup)
- [x] T062 [P] Align DESIGN.md Implementation map + Acceptance checklist with final file list
- [x] T063 Remove temporary dual token usage on in-scope screens (prefer semantic tokens only in shell/resources/architecture)
- [x] T064 [P] Risk-proportional test fill: dialog open/close smoke in `packages/frontend/design-system/a11y/dialog.test.tsx` if missing
- [x] T065 Anti-pattern audit on in-scope screens: no permanent inspector, no green active nav, no card cascades, no heavy glow/glass, no pure-black-only UI — fix findings in feature files
- [x] T066 Run package typecheck/tests for design-system, shell, resources, architecture, canvas, desktop per [quickstart.md](./quickstart.md) §3–4; fix failures
- [x] T067 Run `pnpm build` for desktop (or turbo build filter) and fix regressions
- [x] T068 Convergence check: code ↔ spec SC-001–SC-010 ↔ plan ↔ this tasks.md; update checklist notes in `specs/003-operator-design-system/checklists/` if needed
- [x] T069 User-run visual pass against `prototype/` composition (document results in `specs/003-operator-design-system/notes-visual-pass.md`) — agent does not start `pnpm dev` unless user requests

---

## Dependencies & Execution Order

### Phase Dependencies

```text
Phase 1 Setup
    ↓
Phase 2 Foundational  ─────────── BLOCKS all stories
    ↓
Phase 3 US1 (Shell) 🎯 MVP
    ↓
Phase 4 US2 (Resources) ──┐
Phase 5 US3 (Architecture)┤── can proceed in parallel after US1 (both need Shell)
    ↓                     │
Phase 6 US4 (Navigation) ←┘  (needs US2/US3 routes for full audit)
    ↓
Phase 7 US5 (A11y/density)
    ↓
Phase 8 US6 (Honesty states) — can start earlier for partial, completes after US2/US3 pages exist
    ↓
Phase 9 Polish
```

### User Story Dependencies

| Story | Depends on | Independent test after |
| --- | --- | --- |
| US1 Shell | Phase 2 | Shell alone |
| US2 Resources | Phase 2 + US1 shell host | Resources section only |
| US3 Architecture | Phase 2 + US1 shell host | Architecture section only |
| US4 Navigation | US1 + preferably US2/US3 | Cross-route grammar |
| US5 A11y | US1–US3 surfaces | Keyboard/density smoke |
| US6 States | US2/US3 pages | Forced state paths |

### Within Each Story

1. Types/fixtures (foundational) already done  
2. Pure helpers + tests  
3. View components  
4. App composition wiring  
5. Checkpoint validation  

### Parallel Opportunities

- T002–T004 (setup notes/deps) parallel  
- T008–T011 (types/fixtures) parallel  
- T013–T014, T016–T017, T019 (primitives) parallel after T012 starts  
- After US1: US2 and US3 parallel by two implementers  
- T037–T038 parallel tests in US3  
- T061–T062, T064 parallel in polish  

---

## Parallel Example: User Story 2

```bash
# After foundational + shell:
Task: "T028 Unit tests for resource filter helpers in packages/features/resources/src/filter.test.ts"
Task: "T029 mapCoreResources helpers in packages/features/resources/src/map.ts"
# Then sequential UI:
Task: "T030 Resources index in packages/features/resources/src/resources-index.tsx"
Task: "T031 Resource detail in packages/features/resources/src/resource-detail.tsx"
Task: "T032 Resource form dialog in packages/features/resources/src/resource-form-dialog.tsx"
```

## Parallel Example: User Story 3 (alongside US2)

```bash
Task: "T037 map-view tests in packages/features/architecture/src/map-view.test.ts"
Task: "T039 Remove ResizableSplit inspector from packages/features/architecture/src/index.tsx"
Task: "T040 Architecture canvas in packages/features/architecture/src/architecture-canvas.tsx"
```

---

## Implementation Strategy

### MVP First (User Story 1 only)

1. Complete Phase 1 Setup  
2. Complete Phase 2 Foundational (DESIGN.md + tokens + primitives + types)  
3. Complete Phase 3 US1 Shell  
4. **STOP and VALIDATE** shell anatomy (quickstart §5.1)  
5. Demo quiet dark control-plane chrome  

### Incremental Delivery

1. Setup + Foundational → design system real  
2. US1 Shell → MVP chrome  
3. US2 Resources → inventory pages  
4. US3 Architecture → map pages  
5. US4–US6 + Polish → navigation grammar, a11y, honesty, gates  

### Suggested MVP Scope

**US1 only** (shell + DESIGN.md + tokens + core primitives).  
Recommended first shippable product slice: **US1 + US2 + US3** (matches spec P1 trio).

---

## Task Count Summary

| Phase | Story | Tasks | Count |
| --- | --- | --- | --- |
| 1 Setup | — | T001–T004 | 4 |
| 2 Foundational | — | T005–T020 | 16 |
| 3 | US1 Shell | T021–T027 | 7 |
| 4 | US2 Resources | T028–T036 | 9 |
| 5 | US3 Architecture | T037–T046 | 10 |
| 6 | US4 Navigation | T047–T051 | 5 |
| 7 | US5 A11y | T052–T056 | 5 |
| 8 | US6 States | T057–T060 | 4 |
| 9 Polish | — | T061–T069 | 9 |
| **Total** | | **T001–T069** | **69** |

| Story | Task count |
| --- | --- |
| US1 | 7 |
| US2 | 9 |
| US3 | 10 |
| US4 | 5 |
| US5 | 5 |
| US6 | 4 |
| Setup+Found+Polish | 29 |

---

## Format Validation

- All tasks use `- [x] Tnnn` checklist form  
- Story phases include `[USn]` labels  
- Setup/Foundational/Polish omit story labels  
- `[P]` only on parallelizable work  
- Every task includes at least one concrete file path  

---

## Notes

- Do not invent scenic packages or React paths  
- Fixtures must stay labeled  
- `/speckit-implement` should execute in phase order, stopping at checkpoints  
- Visual `pnpm dev` validation is user-driven unless explicitly requested  
)
