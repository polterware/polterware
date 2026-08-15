# Tasks: Semantic Architecture Canvas

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/

**Tests**: Risk-proportional per Constitution XII — unit for registry search/synonyms, suggestions, canConnect, provisional cancel rules, align/distribute geometry; keep existing canvas/history/clipboard tests green. Do not mark a task complete while required typecheck/tests fail for the changed surface. Agents.md: do not run `pnpm dev` / full build unless the user asks.

**Organization**: Tasks grouped by user story (US1–US10) for independent implementation and testing.

**Constitution / guardrails**:

- **Preserve engine** — no rewrite of `packages/frontend/canvas` core; only extend types, hit-test edges, optional geometry helpers ([plan.md](./plan.md) §0)
- Dependency direction IV: `feature-architecture` → `frontend-canvas`
- SSoT XIV: single create catalog; retire dual CreateNodeDialog path before done
- Security VI: no secrets in document; soft-fail save must keep canvas mounted
- No permanent inspector; no `@excalidraw/*` runtime; no React product UI

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: US1–US10 maps to [spec.md](./spec.md) user stories
- Include exact file paths in every task description

## Path Conventions (this feature)

- Engine: `packages/frontend/canvas/src/`
- Architecture feature: `packages/features/architecture/src/`
- Domain: `packages/features/architecture/src/domain/`
- Host: `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- View shell: `packages/features/architecture/src/index.tsx`
- Design system: `packages/frontend/design-system/`
- Composition: `apps/desktop/src/renderer/App.tsx`
- Contracts: `specs/006-semantic-architecture-canvas/contracts/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Align types/exports and confirm inventory without product behavior change

- [x] T001 Re-read preserve/modify/replace matrix in `specs/006-semantic-architecture-canvas/plan.md` §0 and confirm touch list against current tree (`packages/features/architecture/src/domain/registry.ts`, `connection-rules.ts`, `canvas/architecture-canvas-host.tsx`, `index.tsx`, `create-node-dialog.tsx`, `packages/frontend/canvas/src/document/types.ts`); note no new packages
- [x] T002 [P] Ensure Vitest scripts remain runnable for `@polterware/feature-architecture` and `@polterware/frontend-canvas` in their `package.json` (no scenic test packages)
- [x] T003 [P] Add domain module stubs ready for real exports: `packages/features/architecture/src/domain/type-prefs.ts`, `packages/features/architecture/src/domain/suggestions.ts` (thin files, no empty scenic packages outside feature)

**Checkpoint**: Inventory confirmed; engine rewrite not planned

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Full type registries, extended document type IDs, prefs port, host mode types — **MUST complete before story UI flows**

**⚠️ CRITICAL**: No US1–US10 chrome until registry + type unions + suggestion API exist

### Document type extensions

- [x] T004 Extend `NodeType` and `RelationshipType` unions for catalog gaps (`mobile_application`, `desktop_application`, `scheduled_job`, `serverless_function`, `search_engine`, `data_warehouse`, `load_balancer`, `api_gateway`, `cdn`, `network`, `dns`, `package`, `ci_pipeline`, `build_system`, `artifact_registry`, `saas_provider`, `third_party_api`, `auth_provider`, `payment_provider`, `hosted_on`, `replicates_to`, `synchronizes_with`, `exposes`, etc.) in `packages/frontend/canvas/src/document/types.ts` per [data-model.md](./data-model.md)
- [x] T005 [P] Update migrate/unknown-type fallback so unknown ids remain loadable; unknown/custom type ids round-trip with safe card fallback label (not dropped) in `packages/frontend/canvas/src/document/migrate.ts` and `packages/frontend/canvas/src/document/serialize.ts`
- [x] T006 [P] Re-export any new helpers from `packages/frontend/canvas/src/index.ts` without breaking existing document API

### Full component + relationship registries

- [x] T007 Replace flat `NODE_TYPES` / `RELATIONSHIP_TYPES` with full `ComponentTypeDefinition` / `RelationshipTypeDefinition` catalogs (categories, descriptions, icons keys, default size/ports, searchTerms, synonyms, order, placeable) in `packages/features/architecture/src/domain/registry.ts` per [contracts/type-registry.md](./contracts/type-registry.md)
- [x] T008 [P] Implement `searchComponentTypes(query)` (label, category, terms, synonyms) in `packages/features/architecture/src/domain/registry.ts`
- [x] T009 [P] Implement ordered `suggestRelationshipTypes(source, target)` in `packages/features/architecture/src/domain/suggestions.ts` (or registry) using table from [data-model.md](./data-model.md); keep `inferRelationshipType` as thin alias to first suggestion only for migration
- [x] T010 Expand `canConnect` validity (notes blocked, self-edge policy, optional allowlists) in `packages/features/architecture/src/domain/connection-rules.ts` per [contracts/type-registry.md](./contracts/type-registry.md)
- [x] T011 [P] Define pure `TypePrefsPort` interface and in-memory fake in `packages/features/architecture/src/domain/type-prefs.ts` (getRecents/pushRecent/getFavorites/toggleFavorite — **no localStorage, no DOM**); implement `createLocalStorageTypePrefs()` adapter in `packages/features/architecture/src/canvas/type-prefs-local.ts` (or `persistence/type-prefs-local.ts`) using keys `polterware.arch.typeRecents` / `polterware.arch.typeFavorites`
- [x] T012 Unit tests: search synonyms (postgres/rabbitmq/next/cron), suggestions order, canConnect rejects notes in `packages/features/architecture/src/domain/registry.test.ts`, `packages/features/architecture/src/domain/connection-rules.test.ts`, and new `packages/features/architecture/src/domain/suggestions.test.ts` / `type-prefs.test.ts` as needed

### Host foundation (modes only, no full UX yet)

- [x] T013 Define host ephemeral mode types and signals skeleton (`placement`, `provisional`, `pendingRelationship`, `connectQuickType`, escape layering helpers) in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx` or `packages/features/architecture/src/canvas/authoring-modes.ts` per [contracts/authoring-flows.md](./contracts/authoring-flows.md) — without removing existing tools yet
- [x] T014 [P] Extend edge hit-test (thicker relationship pick) in `packages/frontend/canvas/src/hit-test/hit-test.ts` so relationships are selectable for later toolbars
- [x] T015 [P] Add pure align/distribute geometry helpers in `packages/frontend/canvas/src/document/mutations.ts` or `packages/frontend/canvas/src/document/align.ts` and export from `packages/frontend/canvas/src/index.ts` (used by US6; land early to avoid host bloat)

**Checkpoint**: Registry searchable; suggestions pure-tested; type unions compile; host can hold mode state; stories may implement UI

---

## Phase 3: User Story 1 — Add a typed component (Priority: P1) 🎯 MVP core

**Goal**: `+` / `N` / empty dblclick / context open **Add to architecture** → choose type → place preview → name → confirm; Escape cancels; no silent generic cards

**Independent Test**: Quickstart M1 + M2 in [quickstart.md](./quickstart.md)

### Tests for User Story 1

- [x] T016 [P] [US1] Unit tests for search + recent push on type select in `packages/features/architecture/src/domain/registry.test.ts` / `type-prefs.test.ts`
- [x] T017 [P] [US1] Unit test: provisional create does not mark document dirty until confirm (pure helper if extracted) in `packages/features/architecture/src/domain/create-node.test.ts` (extend)

### Implementation for User Story 1

- [x] T018 [US1] Implement searchable **Add to architecture** dialog (categories, recents, favorites, keyboard) in `packages/features/architecture/src/canvas/add-to-architecture-dialog.tsx` using `A11yDialog` from `packages/frontend/design-system` and registry APIs; Connected-resource rows are non-placeable and show an honest unavailable toast
- [x] T019 [US1] Wire catalog open triggers (`+`, tool add_node, `N`, empty dblclick, context “Add component here”, empty-state CTA) in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx` per [contracts/authoring-flows.md](./contracts/authoring-flows.md)
- [x] T020 [US1] Implement placement mode + cursor preview + click-to-place provisional node (ephemeral, not autosaved) in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [x] T021 [US1] Implement inline name confirm (Enter) / discard (Escape) then `docAddNode` with registry defaults (ports, size, type, state planned, origin manual) + history `immediately` in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [x] T021a [US1] Map Organization catalog entries in registry: Text Note → placeable `note` node; System/Domain/Deployment/Team Boundary → placement creates `addBoundary` with matching `Boundary.kind` (not a generic ArchNode) in `packages/features/architecture/src/domain/registry.ts` + placement commit path in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx` per [data-model.md](./data-model.md) §4
- [x] T022 [US1] Remove silent generic create paths (`New component` + hard-coded `api`/`service`) from add_node click and empty dblclick in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [x] T023 [US1] Replace Components-tab / route `CreateNodeDialog` long-term path with shared catalog: update `packages/features/architecture/src/index.tsx` and either retire or re-export from `packages/features/architecture/src/create-node-dialog.tsx`
- [x] T024 [P] [US1] Register command palette entry “Add to architecture” in `packages/features/architecture/src/canvas/palette-commands.ts` (and commander wiring if required)

**Checkpoint**: No primary path creates untyped/generic cards; Escape discard works

---

## Phase 4: User Story 2 — Create a typed relationship (Priority: P1)

**Goal**: Drag connect → pending edge → type popover (suggested focus) → confirm; Escape discards; quick mode Connect:{Type} multi-drop without popover

**Independent Test**: Quickstart M3 + M4

### Tests for User Story 2

- [x] T025 [P] [US2] Unit tests for suggestion table cases (web→api calls, service→db reads_from, worker→queue consumes_from) in `packages/features/architecture/src/domain/suggestions.test.ts`
- [x] T026 [P] [US2] Unit tests canConnect invalid targets in `packages/features/architecture/src/domain/connection-rules.test.ts`

### Implementation for User Story 2

- [x] T027 [US2] Stop silent `inferRelationshipType` → immediate `addRelationship` on connect drop in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`; introduce pending relationship ghost state
- [x] T028 [US2] Implement relationship type popover (Suggested + Other, keyboard, Enter/Escape) in `packages/features/architecture/src/canvas/relationship-type-popover.tsx` anchored near drop/edge
- [x] T029 [US2] On confirm, `addRelationship` with explicit type + ports + history immediately; on cancel remove pending; never autosave pending in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [x] T030 [US2] Valid/invalid target highlighting during connect drag using `canConnect` in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [x] T031 [US2] Connect quick mode: toolbar type dropdown, session last type, multi-drop without popover, Escape exits in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx` per [contracts/authoring-flows.md](./contracts/authoring-flows.md)
- [x] T032 [US2] Render relationship type label at sufficient zoom; keep direction arrow; enable edge selection hit path in host SVG layer of `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`

**Checkpoint**: No relationship persists without explicit type (popover or quick preselect)

---

## Phase 5: User Story 3 — Edit a component quickly (Priority: P1)

**Goal**: Floating contextual toolbars + complete context menus for node / relationship / multi without permanent inspector

**Independent Test**: Quickstart M5

### Implementation for User Story 3

- [x] T033 [US3] Implement single-node floating toolbar (Open details | Rename | Connect | Duplicate | More) in `packages/features/architecture/src/canvas/contextual-toolbar.tsx` (or host module) per [contracts/ui-chrome.md](./contracts/ui-chrome.md); More P1: Change type, Copy, Cut, Delete, Highlight dependencies; **defer** z-order + Export selection + Move to boundary advanced to T033b
- [ ] T033b [P] [US6] Wire z-order (bring forward/send backward), export selection, toggle grid, move-to-boundary from menus/toolbars in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [x] T034 [US3] Implement relationship floating toolbar (Type ▾ | Edit label | Reverse | Delete | More) wired to `updateRelationship` / reverse helper in host + `packages/frontend/canvas/src/document/mutations.ts` if reverse helper needed; More P1: highlight ends; **defer** reconnect + protocol to T034b
- [ ] T034b [P] [US3] (P2 or late P1) Reconnect source/target and edit protocol for selected relationship in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx` + mutations helpers as needed
- [x] T035 [US3] Implement multi-select toolbar shell (Duplicate | Align | Distribute | Group | Layout | More) — hide Align/Distribute until T049 (US6) if helpers not yet wired; do not show broken stubs as complete
- [x] T036 [US3] Expand context menus (empty/node/rel/multi) to match [contracts/ui-chrome.md](./contracts/ui-chrome.md) P1 actions in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [x] T037 [US3] Wire rename, duplicate, **copy/cut/paste** (fragment API), delete, open details, change type (catalog or type menu) actions to history `immediately` in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [x] T038 [US3] Relabel bottom control **Open details** and call `setSessionSnapshot` (viewport, selection, environmentId) before navigate in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`

**Checkpoint**: Frequent edits reachable from selection; no permanent side inspector added

---

## Phase 6: User Story 4 — Automatic persistence (Priority: P1)

**Goal**: Autosave is primary; status + retry; demote manual Save; soft-fail never blanks canvas

**Independent Test**: Quickstart M6; confirmed edit shows Saving→Saved with core up

### Implementation for User Story 4

- [x] T039 [US4] Ensure provisional/pending modes never call autosave `schedule` in `packages/features/architecture/src/index.tsx` `onDocumentChange` path and host applyDoc modes
- [x] T040 [US4] Demote/remove primary **Save layout** button in `packages/features/architecture/src/index.tsx`; optional secondary “Save now” only
- [x] T041 [US4] Surface persistence status + **Retry** on error in canvas toolbar area of `packages/features/architecture/src/canvas/architecture-canvas-host.tsx` and/or page header; wire flush/retry via `createAutosaveController` in `packages/features/architecture/src/persistence/autosave.ts` and `packages/features/architecture/src/index.tsx`
- [x] T042 [US4] Verify soft-fail save in `apps/desktop/src/renderer/App.tsx` does not set full-page arch error that unmounts canvas on updateLayout/fetch failure; keep local edits
- [x] T043 [P] [US4] Clarify Environment control accessible name/label (“Environment” / “All environments”) in `packages/features/architecture/src/index.tsx` (also supports US10)

**Checkpoint**: Happy path needs no manual Save click; save failure leaves canvas interactive

---

## Phase 7: User Story 5 — Reliable history (Priority: P1)

**Goal**: Undo/redo covers confirmed create, connect, move, rename, delete; provisional never pollutes stack

**Independent Test**: Sequence create→connect→move→rename→delete; undo/redo integrity; no orphan edges

### Tests for User Story 5

- [x] T044 [P] [US5] Unit tests for history capture modes remain green in `packages/frontend/canvas/src/history/history.test.ts`; add cases if reverse/align patches added

### Implementation for User Story 5

- [x] T045 [US5] Audit all host mutations use correct CaptureMode (`eventually` drag, `immediately` commit, `never` ephemeral) in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [x] T046 [US5] Ensure Escape discard of provisional/pending does not push undo entries in host
- [x] T047 [US5] Verify delete selection removes incident relationships (existing `deleteSelection`) and undo restores both nodes and edges; fix gaps in `packages/frontend/canvas/src/document/mutations.ts` if needed
- [x] T048 [US5] Implement layered Escape priority per [contracts/authoring-flows.md](./contracts/authoring-flows.md) §D in host key handler

**Checkpoint**: History trustworthy for all confirmed authoring ops from US1–US3

---

## Phase 8: User Story 6 — Organize the system (Priority: P2)

**Goal**: Multi-select align/distribute, boundaries, auto layout from toolbars/menus

**Independent Test**: Select 3 nodes; align; distribute; create boundary; auto layout selection

### Implementation for User Story 6

- [x] T049 [US6] Wire align/distribute actions from multi toolbar/context menu to geometry helpers (T015) with history immediately in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx` (enables Align/Distribute hidden in T035 until ready); complete T033b advanced menu items if not done
- [x] T050 [US6] Polish boundary create/name/move-with-members from empty/multi menus using existing `addBoundary` / `moveBoundaryWithMembers` in host
- [x] T051 [US6] Keep/improve auto layout button + selection-scoped layout via `layoutSync` in host (already partial)
- [x] T052 [P] [US6] Fit selection command (selection AABB + `fitBounds`) in host toolbar/shortcuts

**Checkpoint**: Large maps can be organized without leaving canvas

---

## Phase 9: User Story 7 — Find components (Priority: P2)

**Goal**: Search finds nodes and centers + selects; keyboard walk if feasible

**Independent Test**: Quickstart M8

### Implementation for User Story 7

- [ ] T053 [US7] Wire Architecture header search to document search helper in `packages/features/architecture/src/search/document-search.ts` and host `centerOn` + selection in `packages/features/architecture/src/index.tsx` / host
- [ ] T054 [US7] On match, set viewport via `centerOn` / `fitBounds` and select node in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [ ] T055 [P] [US7] Unit tests for multi-field search in `packages/features/architecture/src/search/search.test.ts` (extend)

**Checkpoint**: Named component reachable from search without manual pan hunt

---

## Phase 10: User Story 8 — Navigate dependencies (Priority: P2)

**Goal**: Highlight related nodes/edges from selection; clear restores normal chrome

**Independent Test**: Highlight dependencies dims others; Escape/clear restores

### Implementation for User Story 8

- [x] T056 [US8] Implement dependency neighbor computation (1-hop relationships) pure helper in `packages/features/architecture/src/domain/` or `packages/features/architecture/src/search/`
- [x] T057 [US8] Host highlight mode: emphasize related ids, dim others; wire from toolbar/context “Highlight dependencies” in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [x] T058 [US8] Clear highlight on Escape layer or explicit Clear; do not persist highlight in document

**Checkpoint**: Impact neighborhood visible without inspector

---

## Phase 11: User Story 9 — Connected architecture states (Priority: P3)

**Goal**: Cards show honest Planned/Linked/Observed/Stale/Drifted/Unreachable; icons + link cues

**Independent Test**: Manual unlinked node shows Planned; linked resource shows Linked/Observed cues without fake health

### Implementation for User Story 9

- [x] T059 [US9] Registry-driven card chrome (icon, type label, state chip, env/provider/link indicators, LOD) in node render section of `packages/features/architecture/src/canvas/architecture-canvas-host.tsx` per [contracts/ui-chrome.md](./contracts/ui-chrome.md) and DESIGN.md
- [x] T060 [US9] Map linkStatus/resourceId/origin to state display rules (no arbitrary planned for discovered) in domain helper + host
- [x] T061 [P] [US9] Status tone reuse from `statusTone` in `packages/features/architecture/src/domain/registry.ts` with design-system StatusChip if appropriate

**Checkpoint**: Cards read as architecture components, not generic sticky notes

---

## Phase 12: User Story 10 — Environments (Priority: P3)

**Goal**: Clear environment filter; dim/hide non-matching; readiness for future compare (no full multi-env diff required)

**Independent Test**: Quickstart M9

### Implementation for User Story 10

- [x] T062 [US10] Finalize environment filter labeling and filter/dim behavior in `packages/features/architecture/src/index.tsx` + host `envDimmed` logic
- [x] T063 [US10] Persist `activeEnvironmentId` into document and session snapshot; on Architecture index mount after detail, **restore** viewport + environment + **selection** (node/rel/boundary ids that still exist) via `getSessionSnapshot` in `packages/features/architecture/src/index.tsx` and host props/signals — drop missing ids quietly (`packages/features/architecture/src/canvas/session-viewport.ts`)
- [ ] T064 [P] [US10] Optional stub “Compare environments” disabled/honest empty on Changes tab in `packages/features/architecture/src/changes-feed.tsx` or views — no fake data

**Checkpoint**: Operators know filter is by environment; session restores env

---

## Phase 13: Polish & Cross-Cutting Concerns

**Purpose**: Completeness bar from spec acceptance list; remove dual paths; IO; minimap; commander; performance

- [x] T065 Retire dual create paths completely (`create-node-dialog.tsx` → catalog only) and remove dead generic defaults in host
- [ ] T066 [P] Import/export preserve new type ids in `packages/features/architecture/src/document/import-json.ts` / `export-json.ts` and tests
- [ ] T067 [P] Minimap polish (optional boundary dots; accurate viewport) in host `Minimap` in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [x] T068 [P] Expand commander/palette commands (add component, connect mode, fit, highlight) in `packages/features/architecture/src/canvas/palette-commands.ts`
- [ ] T069 Changes tab: surface recent local edit labels if cheap (history labels / save events) in `packages/features/architecture/src/changes-feed.tsx` without fake multiplayer timeline
- [ ] T070 Impact visualization readiness only: command/stub entry “Show impact” that reuses highlight mode (US8) — document in comment/spec note, no full simulator
- [ ] T071 Virtualize catalog list if needed with `@tanstack/solid-virtual` in `packages/features/architecture/src/canvas/add-to-architecture-dialog.tsx`
- [ ] T072 [P] DESIGN.md visual pass: tokens only, quiet chrome, no rainbow types — adjust host/dialog styles
- [ ] T073 Security smoke: no secrets in serialize path; soft-fail canvas remains (spot-check App + serialize)
- [ ] T074 Convergence: code ↔ [spec.md](./spec.md) acceptance list (1–30) ↔ plan preserve matrix; update checklist notes if needed in `specs/006-semantic-architecture-canvas/checklists/requirements.md`
- [ ] T075 Run [quickstart.md](./quickstart.md) manual scenarios M1–M10 when user allows `pnpm dev:stack`; do not claim complete without M1–M5 at minimum

**Checkpoint**: Feature complete only when global acceptance criteria in spec are met — **dialogs-only is not done**

---

## Dependencies & Execution Order

### Phase Dependencies

```text
Phase 1 Setup
    → Phase 2 Foundational (BLOCKS all stories)
        → Phase 3 US1 (typed create)  ──┐
        → Phase 4 US2 (typed connect) ──┼─ P1 Slice A (recommended MVP set)
        → Phase 5 US3 (contextual edit)─┤
        → Phase 6 US4 (autosave UX) ────┤
        → Phase 7 US5 (history) ────────┘
        → Phase 8 US6 (organize)     ── P2
        → Phase 9 US7 (search)
        → Phase 10 US8 (highlight)
        → Phase 11 US9 (card states) ── P3
        → Phase 12 US10 (environments)
        → Phase 13 Polish
```

### User Story Dependencies

| Story | Depends on | Notes |
| --- | --- | --- |
| US1 | Phase 2 | MVP entry |
| US2 | Phase 2; benefits from US1 nodes | Independently testable with fixture nodes |
| US3 | Phase 2; best after US1/US2 | Toolbars on existing selection |
| US4 | Phase 2; pairs with US1 commits | Soft-fail critical |
| US5 | US1–US3 mutations exist | Audit pass |
| US6 | T015 helpers; US3 multi toolbar | P2 |
| US7 | Phase 2 | Independent |
| US8 | Phase 2 | Independent |
| US9 | US1 cards exist | Chrome upgrade |
| US10 | T043 label | Filter polish |

### Parallel Opportunities

- T002–T003 after T001
- T004–T006 type unions / migrate / exports
- T007–T012 domain registry cluster (after T004 if new ids needed)
- T014–T015 hit-test + align helpers
- After Phase 2: US7/US8 can proceed in parallel with US1 if staffed
- US4 soft-fail App check (T042) parallel with dialog work
- Polish T066–T068 parallel

### Within each story

1. Domain tests where rules new (fail first when practical)
2. Domain/API
3. Host/UI wire
4. Remove legacy dual path for that story
5. Independent test checkpoint

---

## Parallel Example: User Story 1

```bash
# Domain tests/registry (already from Phase 2) then:
Task: "T018 Add to architecture dialog in .../add-to-architecture-dialog.tsx"
# then sequential host wiring:
Task: "T019–T022 placement + remove generic create in architecture-canvas-host.tsx"
Task: "T023 catalog replaces CreateNodeDialog in index.tsx"
Task: "T024 palette command"
```

## Parallel Example: After Foundational

```bash
Dev A: US1 create flow (T018–T024)
Dev B: US2 connect flow (T027–T032) using fixture nodes
Dev C: US4 soft-fail + header (T039–T043)
```

---

## Implementation Strategy

### Recommended MVP (Slice A)

Complete **Phase 1 + 2 + US1 + US2 + US4 + US5** (and US3 toolbars if time).  
Validate quickstart M1–M6. **Stop and demo** before P2 organization.

US1 alone is a thin MVP (typed create only); product value needs typed connect + autosave soft-fail.

### Incremental delivery

1. Setup + Foundational → registry live in tests  
2. US1 → typed create  
3. US2 → typed connect  
4. US3 → contextual edit  
5. US4–US5 → trust (save + undo)  
6. US6–US8 → organize/find/highlight  
7. US9–US10 → semantic polish  
8. Polish → acceptance list 1–30  

### Engine rewrite gate

If a task seems to require replacing `ArchitectureCanvasHost` or history/viewport wholesale, **stop** and document a concrete technical limitation in plan notes before proceeding. Default is extend-in-place.

---

## Notes

- [P] = different files, no incomplete deps
- Every task includes a path
- Do not run full `pnpm dev` / monorepo build unless user asks; prefer vitest on domain packages
- No permanent dual create UIs after T065
- Feature **not complete** after dialogs only (T075 + acceptance list)
- Commit after each logical task group when user requests commits
