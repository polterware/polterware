# Tasks: Organization → Project Navigation Hierarchy

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/

**Tests**: Risk-proportional per Constitution XII — unit tests for route/hierarchy helpers and layout mode; manual validation in [quickstart.md](./quickstart.md). Do not mark a task complete while required typecheck/lint/tests fail for the changed surface. Agents.md: do not run `pnpm dev` / full build unless the user asks.

**Organization**: Tasks grouped by user story for independent implementation and testing.

**Constitution**: Dependency direction IV; contracts-first V; SSoT XIV (remove dual “All orgs → Settings”); real code only XIII (pages in shell, no scenic package); hierarchy above project Settings for discovery.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: US1–US5 maps to [spec.md](./spec.md) user stories
- Include exact file paths in every task description

## Path Conventions (this feature)

- Shell/route/chrome: `packages/features/shell/src/`
- Composition: `apps/desktop/src/renderer/App.tsx`
- Settings boundary: `packages/features/settings/src/index.tsx`
- Design system (reuse only): `packages/frontend/design-system/src/`
- Contracts reference: `specs/008-org-project-nav-hierarchy/contracts/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm current broken paths and package boundaries before edits

- [x] T001 Inventory current org breadcrumb (merged name+chevron), `onManageOrganizations` → Settings, workspace select-org auto-first-project, and sidebar always-on in `packages/features/shell/src/context-switchers.tsx`, `packages/features/shell/src/index.tsx`, and `apps/desktop/src/renderer/App.tsx`; note targets in a short bullet list at top of this file or `specs/008-org-project-nav-hierarchy/notes-nav-inventory.md`
- [x] T002 [P] Confirm no new package is allowed; hierarchy pages land in `packages/features/shell/src/` per [plan.md](./plan.md) and [research.md](./research.md) R7

**Checkpoint**: Baseline breakage and ownership clear

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Route model, hierarchy helpers, layout mode, fixture readiness — **MUST complete before story UI depth**

**⚠️ CRITICAL**: No US1–US5 product surface depth until this phase completes

### Route + pure helpers

- [x] T003 Extend `ChromeSectionId` with `organizations` | `organization` and include them in chrome/ALL section sets, `DEFAULT_TABS`, and `resolveSection` in `packages/features/shell/src/route.ts` per [contracts/hierarchy-shell.md](./contracts/hierarchy-shell.md)
- [x] T004 Add optional `organizationId` and `projectId` on `ClientRoute`; include both in `historySnapshot` and `routesEqual` in `packages/features/shell/src/route.ts` per [data-model.md](./data-model.md)
- [x] T005 Implement pure helpers `goOrganizationsRoot`, `goOrganizationHome`, `enterProject`, and `shellLayoutMode` in `packages/features/shell/src/route.ts` (or `packages/features/shell/src/hierarchy.ts` re-exported from route/index) per hierarchy-shell contract
- [x] T006 [P] Unit tests for chrome section membership, helper outputs, invalid org-home recovery, and history snapshot fields in `packages/features/shell/src/route.test.ts` (create file if missing; extend if present)
- [x] T007 Export new section types/helpers from `packages/features/shell/src/index.tsx` public surface so Desktop App can import them

### Fixtures + shell mode skeleton

- [x] T008 [P] Ensure multi-org multi-project fixtures remain valid and add helper to derive `projectCount` per org (function or precomputed) in `packages/features/shell/src/fixtures.ts` / `packages/features/shell/src/types.ts`
- [x] T009 Add `shellLayoutMode`-driven sidebar visibility in `packages/features/shell/src/index.tsx`: when mode is `hierarchy` (`organizations` | `organization`), do not render project `SidebarNav` column; keep title bar

**Checkpoint**: Foundation ready — route v2 hierarchy, helpers tested, sidebar can hide; stories can start

---

## Phase 3: User Story 1 — Breadcrumb splits org label from org switcher (Priority: P1) 🎯 MVP

**Goal**: Org breadcrumb is two controls: name navigates to org home; chevron opens switcher only (no navigation)

**Independent Test**: From Architecture, click org name → org home route; click only chevron → menu opens, stay on Architecture until a menu action

### Tests for User Story 1

- [x] T010 [P] [US1] Unit tests for org segment interaction contract (name handler ≠ chevron handler) via extractable pure/callback flags or component test hooks in `packages/features/shell/src/context-switchers.test.ts` (or document manual-only if Solid test harness missing — prefer pure handler split tested)

### Implementation for User Story 1

- [x] T011 [US1] Split organization segment into separate name button and chevron button in `packages/features/shell/src/context-switchers.tsx` per [contracts/breadcrumb-controls.md](./contracts/breadcrumb-controls.md); chevron toggles org menu only
- [x] T012 [US1] Add `onOpenOrganizationHome` (name click) prop through `packages/features/shell/src/context-switchers.tsx`, `packages/features/shell/src/title-bar.tsx`, and `packages/features/shell/src/index.tsx`
- [x] T013 [US1] Wire App: org name → `goOrganizationHome` with current org id; keep project surface until navigation in `apps/desktop/src/renderer/App.tsx`
- [x] T014 [US1] Add minimal `Match` for `section === "organization"` rendering a stub org-home host (title + placeholder list OK) in `apps/desktop/src/renderer/App.tsx` so name navigation is demoable before US2 polish
- [x] T015 [US1] Ensure dismissing org popover does not change route in `packages/features/shell/src/context-switchers.tsx`

**Checkpoint**: US1 independently testable — name vs chevron differ; name reaches org home section

---

## Phase 4: User Story 2 — Organization home lists projects (Priority: P1)

**Goal**: Full organization home page: search, project list for that org only, create project, enter project workspace

**Independent Test**: Open org home for Polterware; see only that org’s projects; open Polter → project sidebar returns

### Tests for User Story 2

- [x] T016 [P] [US2] Unit tests for filtering projects by `organizationId` and search query (pure function) in `packages/features/shell/src/organization-home.test.ts` or co-located filter helper test

### Implementation for User Story 2

- [x] T017 [US2] Implement `OrganizationHomePage` in `packages/features/shell/src/organization-home.tsx` (title, search, project rows/cards, New project, FixtureBanner when fixture) per [contracts/hierarchy-shell.md](./contracts/hierarchy-shell.md)
- [x] T018 [P] [US2] Export `OrganizationHomePage` from `packages/features/shell/src/index.tsx`
- [x] T019 [US2] Replace App stub Match with full `OrganizationHomePage` wired to fixtures + `route.organizationId` in `apps/desktop/src/renderer/App.tsx`
- [x] T020 [US2] Wire project row click → `enterProject` (default section `overview` or `architecture` — pick one, document in code comment) restoring project layout in `apps/desktop/src/renderer/App.tsx`
- [x] T021 [US2] Wire New project on org home to existing create/onboarding entry with org context when available in `apps/desktop/src/renderer/App.tsx`
- [x] T022 [US2] Change organization selection from switcher menu to `goOrganizationHome` (stop auto-enter first project) in `packages/features/shell/src/index.tsx` and/or `apps/desktop/src/renderer/App.tsx` per research R4

**Checkpoint**: US2 independently testable — org home lists projects and enters project shell

---

## Phase 5: User Story 3 — Organizations root above any organization (Priority: P1)

**Goal**: All organizations full-page index; chrome entry points never land in Settings for discovery

**Independent Test**: Chevron → All organizations → root list; open org → org home; no Settings chrome

### Tests for User Story 3

- [x] T023 [P] [US3] Unit test: `goOrganizationsRoot` sets section `organizations` and does not set settings tab in `packages/features/shell/src/route.test.ts`

### Implementation for User Story 3

- [x] T024 [US3] Implement `OrganizationsIndexPage` in `packages/features/shell/src/organizations-index.tsx` (Your Organizations, search, org cards with plan + project count, New organization, FixtureBanner)
- [x] T025 [P] [US3] Export `OrganizationsIndexPage` from `packages/features/shell/src/index.tsx`
- [x] T026 [US3] Wire App Match for `section === "organizations"` and org row → `goOrganizationHome` in `apps/desktop/src/renderer/App.tsx`
- [x] T027 [US3] Retarget `onManageOrganizations` / All organizations menu item to `goOrganizationsRoot` in `apps/desktop/src/renderer/App.tsx` (break Settings routing)
- [x] T028 [US3] Retarget any remaining shell “Manage organizations” paths (e.g. `packages/features/shell/src/project-switcher.tsx` if still used) to organizations root callbacks
- [x] T029 [US3] Wire New organization on root to onboarding/create entry in `apps/desktop/src/renderer/App.tsx`
- [x] T030 [US3] Add optional “All organizations” control on org home chrome linking to root in `packages/features/shell/src/organization-home.tsx`

**Checkpoint**: US3 independently testable — root index + chrome retarget complete

---

## Phase 6: User Story 4 — Project workspace remains project-scoped (Priority: P2)

**Goal**: Project sidebar/settings stay project-scoped; Settings Organization is admin only; project breadcrumb name vs chevron split

**Independent Test**: Settings → Organization shows admin form not multi-org catalog; project name opens overview; project chevron lists only current org projects

### Implementation for User Story 4

- [x] T031 [US4] Split project segment into name button + chevron button in `packages/features/shell/src/context-switchers.tsx` per breadcrumb-controls contract
- [x] T032 [US4] Project name activates `onOpenOverview`; project chevron only toggles project menu in `packages/features/shell/src/context-switchers.tsx`
- [x] T033 [US4] Confirm project switcher list is filtered to current `organizationId` only in `packages/features/shell/src/context-switchers.tsx`
- [x] T034 [US4] Audit Settings Organization tab: keep admin/config fields; remove any multi-org catalog UX; optional link “All organizations” → hierarchy root callback prop in `packages/features/settings/src/index.tsx`
- [x] T035 [US4] Ensure App `onOpenAccount` does not misuse Settings as org catalog if that was conflated; account may stay settings account tab but org catalog only via hierarchy in `apps/desktop/src/renderer/App.tsx`
- [x] T036 [US4] Verify project workspace sidebar returns after `enterProject` and stays for all `NavSectionId` navigations in `packages/features/shell/src/index.tsx` / App

**Checkpoint**: US4 independently testable — Settings ≠ catalog; project breadcrumb split; sidebar project-only

---

## Phase 7: User Story 5 — Hierarchy-aware chrome and empty states (Priority: P3)

**Goal**: Breadcrumb/chrome reflect hierarchy level; empty org and empty account guided create states

**Independent Test**: Zero-project org fixture shows empty + CTA; hierarchy mode does not show fake selected project name

### Implementation for User Story 5

- [x] T037 [US5] Hierarchy-mode breadcrumb: hide or disable project/environment segments on `organizations` / `organization` surfaces in `packages/features/shell/src/context-switchers.tsx` / `title-bar.tsx` per breadcrumb-controls
- [x] T038 [US5] Empty state for organization home with zero projects (copy + New project) in `packages/features/shell/src/organization-home.tsx`
- [x] T039 [P] [US5] Empty state for organizations root with zero orgs (copy + New organization) in `packages/features/shell/src/organizations-index.tsx`
- [x] T040 [US5] Recover invalid `organization` routes missing `organizationId` via `goOrganizationsRoot` in `apps/desktop/src/renderer/App.tsx` or route helper guard
- [x] T041 [US5] Align section labels in title bar for hierarchy sections (“Organizations” / org name) in `packages/features/shell/src/index.tsx` `sectionLabel` helper

**Checkpoint**: US5 independently testable — empty states + hierarchy chrome honesty

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: SSoT cleanup, exports, quickstart, convergence

- [x] T042 Grep and eliminate dual discovery paths: no chrome “All/Manage organizations” may call `setTab(goSection(..., "settings"), "organization")` in `apps/desktop/src/renderer/App.tsx` and `packages/features/shell/src/**`
- [x] T043 [P] Export hierarchy pages and helpers cleanly from `packages/features/shell/src/index.tsx`; remove unused stub-only code
- [x] T044 [P] Run shell unit tests for route/hierarchy (`pnpm exec vitest run` scoped to `packages/features/shell` as package scripts allow)
- [x] T045 Walk [quickstart.md](./quickstart.md) scenarios S1–S6 (and S8 history if interactive); record pass/fail in `specs/008-org-project-nav-hierarchy/notes-static-checks.md`
- [x] T046 Convergence check: code ↔ [spec.md](./spec.md) ↔ [plan.md](./plan.md) ↔ this tasks file (FR-001–FR-024, SC-001–SC-007); note residual gaps only if deferred with reason

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 Setup**: immediate
- **Phase 2 Foundational**: after Setup — **blocks all stories**
- **Phase 3 US1**: after Foundational (MVP)
- **Phase 4 US2**: after US1 stub Match exists (T014); completes org home
- **Phase 5 US3**: after Foundational; can parallelize with US2 after helpers exist; best after T014 for end-to-end demo
- **Phase 6 US4**: after US1 breadcrumb split patterns exist (shares `context-switchers.tsx`)
- **Phase 7 US5**: after US2 + US3 pages exist
- **Phase 8 Polish**: after desired stories complete

### User Story Dependencies

| Story | Depends on | Independently testable when |
| --- | --- | --- |
| **US1** (P1) | Foundational | Name vs chevron + land on org section |
| **US2** (P1) | Foundational + US1 stub route | Full project list + enter project |
| **US3** (P1) | Foundational | Root page + All orgs retarget |
| **US4** (P2) | US1 control split pattern | Settings admin-only + project split |
| **US5** (P3) | US2 + US3 pages | Empty states + hierarchy chrome |

### Parallel Opportunities

- T002 ∥ T001 (after planning)
- T006 ∥ T008 after T003–T005 API shape known
- T017 page build ∥ T024 page build after Foundational (different files)
- T018 ∥ T025 exports
- T038 ∥ T039 empty states
- T043 ∥ T044 polish

---

## Parallel Example: After Foundational

```bash
# Parallel page implementations (different files):
Task: "T017 OrganizationHomePage in packages/features/shell/src/organization-home.tsx"
Task: "T024 OrganizationsIndexPage in packages/features/shell/src/organizations-index.tsx"

# Then serial App wiring if same file:
Task: "T019–T022 App Match + enterProject + stop auto-first-project"
Task: "T026–T029 App organizations Match + retarget All organizations"
```

---

## Parallel Example: User Story 1

```bash
# After Foundational:
Task: "T011 Split org name/chevron in context-switchers.tsx"
# Then:
Task: "T012–T015 props + App wire + stub Match"
```

---

## Implementation Strategy

### MVP First (US1 + minimal org home)

1. Phase 1 Setup  
2. Phase 2 Foundational (route + sidebar hide + helpers)  
3. Phase 3 US1 (split controls + navigate to org section)  
4. **STOP**: validate name vs chevron (quickstart S1–S2)  
5. Phase 4 US2 completes the org home value  
6. Phase 5 US3 completes multi-org root  

### Incremental Delivery

1. Foundational → hierarchy routes exist  
2. US1 → breadcrumb fix demoable  
3. US2 → org → project path complete  
4. US3 → root complete; Settings dual-path killed  
5. US4 → project breadcrumb + Settings boundary  
6. US5 → empty/hierarchy polish  
7. Phase 8 → grep + quickstart + convergence  

### Suggested MVP scope

**US1 + T014 stub + T009 sidebar hide** is the smallest clickable correction.  
**Recommended ship slice**: Foundational + US1 + US2 + US3 (P1 stories) so hierarchy is real end-to-end.

---

## Notes

- Default land after `enterProject`: prefer `overview` if present, else `architecture` — pick one in T020 and keep SSoT  
- Do not create `packages/features/organizations`  
- Do not run full Desktop build/`pnpm dev` unless Erick asks  
- Commit after each logical group if committing is requested  
- Temporary dual path Settings vs root must be gone by T042  
