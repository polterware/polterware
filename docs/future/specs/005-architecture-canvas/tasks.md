# Tasks: Architecture Canvas Engine

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/

**Tests**: Risk-proportional per Constitution XII and spec Testing section — unit for engine pure modules (viewport, hit-test, history, clipboard, migrate), domain connection rules, autosave controller; integration for create/connect/undo/persist; performance harness for 500/1000 nodes; e2e when desktop suite available. Do not mark a task complete while required typecheck/lint/tests fail for the changed surface. Agents.md: do not run `pnpm dev` / full build unless the user asks.

**Organization**: Tasks grouped by user story for independent implementation and testing.

**Constitution**: Dependency direction IV (`feature-architecture` → `frontend-canvas`); contracts-first V (Layout RPC stable; body JSON v1); security VI (no secrets, untrusted import); no permanent inspector; SSoT XIV (remove static canvas before done); real code only XIII; no `@excalidraw/*` runtime.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: US1–US10 maps to [spec.md](./spec.md) user stories
- Include exact file paths in every task description

## Path Conventions (this feature)

- Engine: `packages/frontend/canvas/src/`
- Layout worker: `packages/frontend/canvas/worker/`
- Architecture feature: `packages/features/architecture/src/`
- Design system: `packages/frontend/design-system/src/`
- Data client: `packages/frontend/data/src/index.ts`
- Composition: `apps/desktop/src/renderer/App.tsx`
- Engine docs: `packages/frontend/canvas/README.md` or `docs/canvas-engine.md`
- Contracts reference: `specs/005-architecture-canvas/contracts/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Align package boundaries, exports, and test harness before engine code

- [x] T001 Inventory current canvas surface and layout wiring: `packages/features/architecture/src/architecture-canvas.tsx`, `packages/features/architecture/src/index.tsx`, `packages/frontend/canvas/src/index.ts`, `apps/desktop/src/renderer/App.tsx` (`getLayout`/`updateLayout`); record keep/replace notes in `specs/005-architecture-canvas/notes-inventory.md`
- [x] T002 [P] Create engine directory skeleton under `packages/frontend/canvas/src/` (`document/`, `viewport/`, `selection/`, `hit-test/`, `history/`, `clipboard/`, `commands/`, `pointer/`, `snap/`, `spatial/`, `render/`, `solid/`) with placeholder `index.ts` re-exports; keep package browser-safe (no Electron)
- [x] T003 [P] Ensure Vitest can run package tests for `@polterware/frontend-canvas` and `@polterware/feature-architecture` (update `packages/frontend/canvas/package.json` scripts and any workspace vitest config as needed; prefer matching sibling package patterns)
- [x] T004 [P] Add architecture feature subdirs `packages/features/architecture/src/{domain,document,canvas,nodes,edges,persistence,search}/` with empty index barrels ready for real exports (no scenic packages outside existing feature)

**Checkpoint**: Skeleton and inventory ready; no product behavior change required yet

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Document v1, migrate/serialize, domain registry stub, engine history/viewport primitives, host shell that loads a document — **MUST complete before story depth**

**⚠️ CRITICAL**: No US1–US10 interaction polish until this phase lands document + basic world host

### Document model + contracts

- [x] T005 Implement ArchitectureDocument v1 types and defaults in `packages/frontend/canvas/src/document/types.ts` per [data-model.md](./data-model.md)
- [x] T006 [P] Implement `createEmptyDocument`, `validateDocument`, and structured parse errors in `packages/frontend/canvas/src/document/document.ts`
- [x] T007 [P] Implement `serializeDocument` / `deserializeAndMigrate` (legacy format 0 `{nodes,edges}` → v1) in `packages/frontend/canvas/src/document/serialize.ts` and `packages/frontend/canvas/src/document/migrate.ts` per [contracts/architecture-document.md](./contracts/architecture-document.md)
- [x] T008 Unit tests for migrate/validate/serialize round-trip in `packages/frontend/canvas/src/document/document.test.ts` and `packages/frontend/canvas/src/document/migrate.test.ts`
- [x] T009 [P] Export document API from `packages/frontend/canvas/src/index.ts`; keep temporary re-exports of legacy `CanvasModel` helpers with deprecation path or adapters so architecture still compiles

### Domain registry (pure)

- [x] T010 [P] Implement node type + relationship type catalogs, default ports/sizes, and state labels in `packages/features/architecture/src/domain/registry.ts` per [data-model.md](./data-model.md)
- [x] T011 [P] Implement `canConnect` / connection validation helpers in `packages/features/architecture/src/domain/connection-rules.ts`
- [x] T012 [P] Unit tests for registry defaults and connection rules in `packages/features/architecture/src/domain/registry.test.ts` and `packages/features/architecture/src/domain/connection-rules.test.ts`
- [x] T013 Map domain adapter interface expected by engine in `packages/frontend/canvas/src/document/domain-adapter.ts` and wire architecture adapter factory in `packages/features/architecture/src/domain/adapter.ts`

### Engine core primitives

- [x] T014 Implement viewport math (`screenToDocument`, `documentToScreen`, `zoomAt`, `fitBounds`, `clampZoom`, `panBy`) in `packages/frontend/canvas/src/viewport/viewport.ts`
- [x] T015 [P] Unit tests for viewport focal zoom and fit in `packages/frontend/canvas/src/viewport/viewport.test.ts`
- [x] T016 [P] Implement history stack with capture modes `immediately` | `eventually` | `never`, patch apply/inverse, stack limits in `packages/frontend/canvas/src/history/history.ts` per [research.md](./research.md) R4
- [x] T017 [P] Unit tests for history coalesce (eventually→one undo) and redo invalidation in `packages/frontend/canvas/src/history/history.test.ts`
- [x] T018 [P] Implement command registry (`registerCommand`, `dispatch`, `matchShortcut`) in `packages/frontend/canvas/src/commands/registry.ts` per [contracts/commands-shortcuts.md](./contracts/commands-shortcuts.md)
- [x] T019 [P] Implement AABB spatial index stub in `packages/frontend/canvas/src/spatial/grid-index.ts`
- [x] T020 [P] Implement hit-test priority pipeline stub (port→handle→node→relationship→boundary→empty) in `packages/frontend/canvas/src/hit-test/hit-test.ts`

### Host shell + load path

- [x] T021 Implement architecture document load helpers (core body → v1 doc + warnings) in `packages/features/architecture/src/document/load.ts`
- [x] T022 Implement minimal `ArchitectureCanvasHost` world container (transform root, dotted background tokens, pointer target) in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx` per DESIGN.md
- [x] T023 Wire `ArchitectureView` to load v1 document (migrate layout body) and render host instead of/alongside static layout in `packages/features/architecture/src/index.tsx` without permanent inspector
- [x] T024 [P] Session viewport cache Map by systemId in `packages/features/architecture/src/canvas/session-viewport.ts`
- [x] T025 Ensure App still uses `getLayout`/`updateLayout` only for persistence in `apps/desktop/src/renderer/App.tsx` (no new Electron FS layout store)

**Checkpoint**: Foundation ready — document v1 round-trips; host mounts; history/viewport/commands exist; stories can build interactions

---

## Phase 3: User Story 1 — Explore a large architecture (Priority: P1) 🎯 MVP

**Goal**: Fluid pan/zoom, fit document/selection, minimap, zoom indicator, restore last viewport

**Independent Test**: Open a multi-node document; pan (space-drag, middle button, trackpad); zoom about cursor; fit all; fit selection; see zoom %; reopen app and get same viewport; minimap navigates

### Tests for User Story 1

- [x] T026 [P] [US1] Unit tests for pan/zoom/fit helpers and clamp in `packages/frontend/canvas/src/viewport/viewport.test.ts` (extend)
- [x] T027 [P] [US1] Unit tests for session viewport cache get/set in `packages/features/architecture/src/canvas/session-viewport.test.ts`

### Implementation for User Story 1

- [x] T028 [US1] Implement pointer Hand tool + temporary Space-hand + middle-button pan in `packages/frontend/canvas/src/pointer/pointer-controller.ts` and bind from `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [x] T029 [US1] Implement wheel/pinch zoom about cursor (mod+wheel) in `packages/frontend/canvas/src/pointer/wheel-zoom.ts` and host wiring
- [x] T030 [US1] Implement `view.fit_document` and `view.fit_selection` commands in `packages/features/architecture/src/canvas/commands-view.ts` registered via engine registry
- [x] T031 [US1] Add compact zoom indicator + zoom +/- / reset controls in `packages/features/architecture/src/canvas/zoom-controls.tsx` (DESIGN tokens, not Excalidraw chrome)
- [x] T032 [US1] Implement minimap (extent, viewport rect, node dots, click/drag navigate) in `packages/features/architecture/src/canvas/minimap.tsx`
- [x] T033 [US1] Persist viewport into document on change (debounced later in US5) and restore on load from body + session cache in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`
- [x] T034 [US1] Render existing nodes from document as HTML cards at document coordinates (read-only placement OK) in `packages/features/architecture/src/nodes/arch-node-card.tsx` with viewport culling hook in `packages/features/architecture/src/canvas/use-visible-nodes.ts`
- [x] T035 [US1] Stable dotted/grid background under pan/zoom in `packages/features/architecture/src/canvas/canvas-background.tsx` using design tokens
- [x] T036 [US1] Disable traditional scrollbar-primary navigation on canvas host CSS in `packages/features/architecture/src/canvas/architecture-canvas-host.tsx`

**Checkpoint**: US1 independently testable — explore/navigate large map without authoring

---

## Phase 4: User Story 2 — Create conceptual components (Priority: P1)

**Goal**: Add typed nodes via button, palette, context menu, double-click empty; rename; cancel-safe; required type

**Independent Test**: Create node four ways; type required; name edit; Escape cancels in-progress create; node appears at predictable position

### Tests for User Story 2

- [x] T037 [P] [US2] Unit tests for create-node transaction (id, defaults, ports from registry) in `packages/features/architecture/src/domain/create-node.test.ts`

### Implementation for User Story 2

- [x] T038 [US2] Implement document mutation `addNode` transaction helper in `packages/frontend/canvas/src/document/mutations.ts` (or architecture wrapper `packages/features/architecture/src/document/mutations.ts`)
- [x] T039 [US2] Implement Add Node tool (click-to-place / center place) in `packages/frontend/canvas/src/pointer/tools/add-node-tool.ts` + host
- [x] T040 [US2] Wire create command `tool.add_node` / `node.create` from toolbar button in `packages/features/architecture/src/canvas/canvas-toolbar.tsx`
- [x] T041 [US2] Empty-canvas context menu “Add node” in `packages/features/architecture/src/canvas/context-menu.tsx` (Kobalte/design-system menu primitives)
- [x] T042 [US2] Double-click empty canvas creates node in pointer controller / host
- [x] T043 [US2] Integrate existing create dialog (`packages/features/architecture/src/create-node-dialog.tsx`) to commit into document via same mutation path
- [x] T044 [US2] Immediate inline name edit on create in `packages/features/architecture/src/nodes/inline-rename.tsx` with IME-safe Escape cancel
- [x] T045 [US2] Enforce required type selection UI (dialog or compact type picker) in `packages/features/architecture/src/nodes/type-picker.tsx`
- [x] T046 [US2] Register architecture create commands for command palette contribution in `packages/features/architecture/src/canvas/palette-commands.ts` and wire from `apps/desktop/src/renderer/App.tsx` / commander items
- [x] T047 [US2] History: create commits as single immediate transaction in mutation path

**Checkpoint**: US2 independently testable — author nodes without connect yet

---

## Phase 5: User Story 3 — Connect components (Priority: P1)

**Goal**: Directed semantic relationships via ports, preview, validation, labels, reconnect, undo/redo

**Independent Test**: Connect API→Database with `reads_from`; preview + invalid target feedback; undo removes one relationship; redo restores

### Tests for User Story 3

- [x] T048 [P] [US3] Unit tests for relationship create/reconnect/delete patches in `packages/features/architecture/src/domain/relationships.test.ts`
- [x] T049 [P] [US3] Unit tests for edge path geometry helpers in `packages/frontend/canvas/src/render/edge-path.test.ts`

### Implementation for User Story 3

- [x] T050 [US3] Port hit targets on node cards (hover/selection/connect tool visibility) in `packages/features/architecture/src/nodes/ports.tsx`
- [x] T051 [US3] Connect tool + rubber-band preview in `packages/frontend/canvas/src/pointer/tools/connect-tool.ts`
- [x] T052 [US3] Valid/invalid target highlighting using `canConnect` in host connect flow
- [x] T053 [US3] Commit relationship with type pick/infer UI in `packages/features/architecture/src/edges/relationship-type-picker.tsx`
- [x] T054 [US3] SVG relationships layer (evidence-aware stroke, arrowheads, labels) in `packages/features/architecture/src/edges/relationships-layer.tsx` using `packages/frontend/canvas/src/render/edge-path.ts`
- [x] T055 [US3] Select relationship, edit label inline, reconnect endpoints in `packages/features/architecture/src/edges/relationship-editor.ts`
- [x] T056 [US3] Delete relationship via command + context menu; history immediate entry
- [x] T057 [US3] Update edges fluidly while endpoint nodes move (subscribe to moved node ids only) in relationships layer
- [x] T058 [US3] Hit-test expanded stroke for relationships in `packages/frontend/canvas/src/hit-test/hit-test.ts`
- [x] T059 [US3] Register `tool.connect` shortcut C when canvas focused in commands binder `packages/features/architecture/src/canvas/shortcut-binder.ts`

**Checkpoint**: US3 independently testable — semantic connect loop with undo

---

## Phase 6: User Story 4 — Edit a multi-selection (Priority: P1)

**Goal**: Click/shift/marquee/select-all; collective drag/duplicate/delete; copy-paste graph-safe; one history transaction per gesture

**Independent Test**: Marquee three nodes; drag keeps relative positions; edges follow; one undo; clipboard scenario from spec

### Tests for User Story 4

- [x] T060 [P] [US4] Unit tests for marquee contain vs intersect in `packages/frontend/canvas/src/selection/marquee.test.ts`
- [x] T061 [P] [US4] Unit tests for clipboard fragment rewrite (new ids, internal edges only) in `packages/frontend/canvas/src/clipboard/clipboard.test.ts`
- [x] T062 [P] [US4] Unit tests for multi-drag single history transaction in `packages/frontend/canvas/src/history/history.test.ts` (extend)

### Implementation for User Story 4

- [x] T063 [US4] Selection model + click/shift-toggle in `packages/frontend/canvas/src/selection/selection.ts`
- [x] T064 [US4] Marquee selection UI + left-to-right contain / right-to-left intersect in pointer select tool `packages/frontend/canvas/src/pointer/tools/select-tool.ts`
- [x] T065 [US4] Select-all command and empty-click clear (modifier exceptions) in selection commands
- [x] T066 [US4] Multi-node drag with pointer capture, eventually capture during move, immediately on up in select tool
- [x] T067 [US4] Selection chrome + compact floating contextual toolbar in `packages/features/architecture/src/canvas/selection-chrome.tsx` and `packages/features/architecture/src/canvas/selection-toolbar.tsx` (no permanent inspector)
- [x] T068 [US4] Implement copy/cut/paste/duplicate in `packages/frontend/canvas/src/clipboard/clipboard.ts` + architecture wrappers; successive paste offset
- [x] T069 [US4] Wire system clipboard fallback safely in `packages/features/architecture/src/canvas/clipboard-host.ts`
- [x] T070 [US4] Delete selection (nodes + dependent relationships) as one transaction in `packages/features/architecture/src/document/delete-selection.ts`
- [x] T071 [US4] Snap optional grid + alignment guides during drag in `packages/frontend/canvas/src/snap/snap.ts` and guides overlay `packages/features/architecture/src/canvas/guides-overlay.tsx`
- [x] T072 [US4] Z-order commands bring forward/back in `packages/features/architecture/src/document/z-order.ts`
- [x] T073 [US4] Keyboard shortcuts for select/clipboard/delete when canvas focused in `packages/features/architecture/src/canvas/shortcut-binder.ts`

**Checkpoint**: US4 independently testable — multi-edit + clipboard + history

---

## Phase 7: User Story 5 — Persist the document (Priority: P1)

**Goal**: Autosave with status, revision concurrency, migrate load, recover after restart, no corrupt last-good

**Independent Test**: Edit → Saved indicator → quit/reopen → full document + viewport restored; conflict surfaces recoverable error

### Tests for User Story 5

- [x] T074 [P] [US5] Unit tests for autosave debounce, dirty/saving/saved/error, and conflict handling with mock port in `packages/features/architecture/src/persistence/autosave.test.ts`
- [x] T075 [P] [US5] Unit tests for invalid body load warnings without crash in `packages/features/architecture/src/document/load.test.ts`

### Implementation for User Story 5

- [x] T076 [US5] Implement `LayoutSavePort` interface and autosave controller (800ms doc / 2000ms viewport) in `packages/features/architecture/src/persistence/autosave.ts` per [contracts/persistence-autosave.md](./contracts/persistence-autosave.md)
- [x] T077 [US5] Discrete save status indicator in Architecture header/toolbar `packages/features/architecture/src/canvas/save-status.tsx`
- [x] T078 [US5] Wire App `saveLayout`/`getLayout` as save port implementation in `apps/desktop/src/renderer/App.tsx`; remove reliance on manual-only Save as sole path (button may force flush)
- [x] T079 [US5] Flush autosave on route leave, detail open, and best-effort window lifecycle in App + host
- [x] T080 [US5] On revision conflict: toast + keep local dirty; offer reload vs confirm overwrite UX in `packages/features/architecture/src/persistence/conflict-ui.ts`
- [x] T081 [US5] Ensure serialize never includes secrets/interaction/history; payload size guard in `packages/frontend/canvas/src/document/serialize.ts`
- [x] T082 [US5] Replace legacy serialize path in architecture index with v1 document serialize for all saves in `packages/features/architecture/src/index.tsx`

**Checkpoint**: US5 independently testable — durable layout with honest status

---

## Phase 8: User Story 6 — Open full node detail and return (Priority: P1)

**Goal**: Detail page via double-click/Enter/menu/search; back restores document, viewport, zoom, selection, filters, environment

**Independent Test**: Select node at zoom; open detail; back → same viewport/selection/environment; no permanent inspector

### Tests for User Story 6

- [x] T083 [P] [US6] Unit tests for session restore snapshot (viewport + selection + environment) in `packages/features/architecture/src/canvas/session-viewport.test.ts` (extend)

### Implementation for User Story 6

- [x] T084 [US6] Snapshot session state before `onOpenNode` in host; restore on Architecture index remount in `packages/features/architecture/src/index.tsx`
- [x] T085 [US6] Double-click node and Enter → open detail (existing route `openDetail` / `arch-node`) from host + shortcut binder
- [x] T086 [US6] Context menu + selection toolbar “Open details” actions
- [x] T087 [US6] Enrich `packages/features/architecture/src/node-detail.tsx` from document relationships when available (still page-based)
- [x] T088 [US6] Deep link / route `entityId` selects and centers node when landing from external navigation in host
- [x] T089 [US6] Verify no `ResizableSplit` permanent inspector remains on Architecture canvas path in `packages/features/architecture/src/index.tsx` / canvas host

**Checkpoint**: US6 independently testable — page-based depth with restore

---

## Phase 9: User Story 7 — Organize with boundaries (Priority: P2)

**Goal**: Boundaries with deterministic membership, move-with-content, collapse/expand, rename, external relationships preserved

**Independent Test**: Create boundary from selection; move boundary moves members; collapse/expand; external edges intact

### Tests for User Story 7

- [x] T090 [P] [US7] Unit tests for membership assign/remove and move-with-members in `packages/features/architecture/src/document/boundaries.test.ts`

### Implementation for User Story 7

- [x] T091 [US7] Boundary CRUD mutations + explicit `node.boundaryId` membership in `packages/features/architecture/src/document/boundaries.ts`
- [x] T092 [US7] Add Boundary tool (drag create) in `packages/frontend/canvas/src/pointer/tools/add-boundary-tool.ts`
- [x] T093 [US7] Boundary rendering behind nodes in `packages/features/architecture/src/canvas/boundaries-layer.tsx`
- [x] T094 [US7] Move with content vs modifier move-frame-only; resize; collapse/expand
- [x] T095 [US7] Context menus and create-boundary-from-selection command
- [x] T096 [US7] Select contents of boundary; z-order relative to nodes/edges

**Checkpoint**: US7 independently testable — organize systems visually

---

## Phase 10: User Story 8 — Auto-layout (Priority: P2)

**Goal**: Horizontal/vertical layout on selection or document; respect pins; one undo; non-blocking when heavy

**Independent Test**: Auto-layout selection reduces overlap; undo restores; UI stays interactive

### Tests for User Story 8

- [x] T097 [P] [US8] Unit tests for layout algorithm on small graphs in `packages/frontend/canvas/worker/layout.test.ts`

### Implementation for User Story 8

- [x] T098 [US8] Expand layout worker beyond trivial grid in `packages/frontend/canvas/worker/layout.ts` (horizontal/vertical; honor `layoutPinned`)
- [x] T099 [US8] Host applies worker result as single history transaction in `packages/features/architecture/src/canvas/auto-layout.ts`
- [x] T100 [US8] Toolbar/menu/palette “Auto layout” commands; selection vs whole document scope
- [x] T101 [US8] Loading/processing UI state without blocking pointer (async apply)

**Checkpoint**: US8 independently testable — cleanup layout

---

## Phase 11: User Story 9 — Export a view (Priority: P2)

**Goal**: Export versioned JSON (doc/selection) and PNG (SVG if correct); no selection overlays

**Independent Test**: Export JSON validates; PNG omits guides/selection chrome; invalid import path covered under US10

### Tests for User Story 9

- [x] T102 [P] [US9] Unit tests for export fragment JSON schema in `packages/features/architecture/src/document/export.test.ts`

### Implementation for User Story 9

- [x] T103 [US9] Export canonical JSON document/selection via download/blob in `packages/features/architecture/src/document/export-json.ts`
- [x] T104 [US9] PNG export of world (optional background) in `packages/features/architecture/src/document/export-png.ts` (DOM/SVG snapshot approach)
- [x] T105 [US9] SVG export when hybrid render allows faithful output in `packages/features/architecture/src/document/export-svg.ts` (skip or degrade honestly if not correct)
- [x] T106 [US9] Export UI entry (menu/palette) with filename convention and options (labels/boundaries/background)

**Checkpoint**: US9 independently testable — share/export

---

## Phase 12: User Story 10 — Discoverability & recovery states (Priority: P3)

**Goal**: Command palette parity, shortcuts reference, empty/error/import-invalid/large-doc states; search polish

**Independent Test**: Palette runs same commands; shortcuts help; empty CTAs; invalid import leaves doc unchanged

### Tests for User Story 10

- [x] T107 [P] [US10] Unit tests for import validation rejection (malformed/unknown version) in `packages/features/architecture/src/document/import.test.ts`
- [x] T108 [P] [US10] Unit tests for multi-field document search ranking in `packages/features/architecture/src/search/search.test.ts`

### Implementation for User Story 10

- [x] T109 [US10] Document search (name/type/provider/env/status/tag/owner/external id) + keyboard cycle + center selection in `packages/features/architecture/src/search/document-search.ts` and search UI `packages/features/architecture/src/canvas/search-panel.tsx`
- [x] T110 [US10] Environment filter selector (All + envs; de-emphasize out-of-env) in `packages/features/architecture/src/canvas/environment-filter.tsx`
- [x] T111 [US10] Import JSON with preview, backup current, sanitize; reject invalid without replace in `packages/features/architecture/src/document/import-json.ts`
- [x] T112 [US10] Empty/first-use/error/migration/import/no-results states in host using design-system PageEmpty/PageError patterns
- [x] T113 [US10] Searchable shortcuts help popover in `packages/features/architecture/src/canvas/shortcuts-help.tsx`
- [x] T114 [US10] Ensure commander palette lists architecture commands without duplicate logic in `packages/features/commander/src/index.tsx` + `palette-commands.ts`
- [x] T115 [US10] Structured a11y alternative list of nodes (not full canvas AT tree) in `packages/features/architecture/src/canvas/a11y-node-list.tsx`
- [x] T116 [US10] Text note tool (T) minimal implementation if not already covered in create path `packages/features/architecture/src/nodes/text-note.tsx`

**Checkpoint**: US10 independently testable — operability polish

---

## Phase 13: Polish & Cross-Cutting Concerns

**Purpose**: Performance, SSoT cleanup, docs, security smoke, e2e/benchmark

- [x] T117 Remove static band-layout canvas path and dead `layers.ts` band UI once host reaches parity; single canvas implementation only in `packages/features/architecture/src/architecture-canvas.tsx` (re-export host) and delete dual path (XIV)
- [x] T118 [P] Performance harness generating 500 nodes / 1000 edges and 1000 / 2000 stress in `packages/frontend/canvas/src/bench/generate-doc.ts` + notes in `specs/005-architecture-canvas/notes-benchmark.md`
- [x] T119 [P] Viewport culling + LOD (distant/medium/near) polish in `packages/features/architecture/src/nodes/arch-node-card.tsx` and `use-visible-nodes.ts`
- [x] T120 [P] Confirm no Electron/FS imports in engine/feature via static check; document in `specs/005-architecture-canvas/notes-security-smoke.md`
- [x] T121 Write architecture note engine vs domain vs app in `packages/frontend/canvas/README.md` (or `docs/canvas-engine.md`)
- [x] T122 [P] Update fixtures to ArchitectureDocument v1 in `packages/features/architecture/src/fixtures.ts` and `extra-fixtures.ts`
- [x] T123 Align Components/Dependencies tabs to read from live document model when available in `packages/features/architecture/src/components-table.tsx` and `dependencies-table.tsx`
- [x] T124 Desktop e2e happy path (create three nodes, connect, boundary if P2 done, undo/redo, reopen, detail restore) under existing e2e harness path used by repo (e.g. `apps/desktop` playwright tests) — add only if harness exists; else document manual [quickstart.md](./quickstart.md) runbook completion
- [x] T125 Run typecheck/lint/tests for `@polterware/frontend-canvas`, `@polterware/feature-architecture`, desktop filter when user permits verification; fix failures on changed surface
- [x] T126 [P] DESIGN.md compliance pass: toolbar quiet, no Excalidraw clone chrome, brand green sparse — checklist note in `specs/005-architecture-canvas/notes-visual-pass.md`
- [x] T127 Escape layered cancel fully (text → connect → create → menu → clear selection) verified in pointer/host integration test `packages/features/architecture/src/canvas/escape-layers.test.ts`

---

## Dependencies & Execution Order

### Phase dependencies

```text
Phase 1 Setup
    ↓
Phase 2 Foundational (BLOCKS all stories)
    ↓
Phase 3 US1 Explore          🎯 MVP navigation
    ↓
Phase 4 US2 Create nodes     (needs US1 host/viewport)
    ↓
Phase 5 US3 Connect          (needs nodes)
    ↓
Phase 6 US4 Multi-select     (can start after US1; ideally after US2 for real nodes)
    ↓
Phase 7 US5 Persist          (needs mutations from US2–4)
    ↓
Phase 8 US6 Detail restore   (needs US1 viewport + US5 optional flush)
    ↓
Phase 9 US7 Boundaries (P2)  (needs US4 selection)
    ↓
Phase 10 US8 Auto-layout (P2)
    ↓
Phase 11 US9 Export (P2)     (needs stable document)
    ↓
Phase 12 US10 Polish (P3)    (search/import/palette; search can partially parallel after US1)
    ↓
Phase 13 Cross-cutting
```

### User story dependency graph

| Story | Depends on | Can parallel with |
| --- | --- | --- |
| US1 Explore | Foundation | — |
| US2 Create | US1 host | US4 selection model partially |
| US3 Connect | US2 | — |
| US4 Multi-edit | US1 (+ US2 for value) | US3 after nodes exist |
| US5 Persist | US2+ (any dirty path) | US6 |
| US6 Detail | US1 + routing | US5 |
| US7 Boundaries | US4 | US8 later |
| US8 Layout | US4 | US7 |
| US9 Export | US5 document stability | US10 import |
| US10 Ops polish | Foundation + commands | late P1 |

### Within each story

1. Tests (marked [P] where pure) → 2. Mutations/domain → 3. Pointer/UI → 4. Wire host/App → 5. Checkpoint

### Parallel opportunities

- After T002 skeleton: T005–T007 document vs T010–T012 domain vs T014–T020 engine primitives in parallel
- Within US4: clipboard tests T061 parallel to marquee T060
- P2 US7/US8 after US4: sequential preferred; different files allow parallel if staffed
- Polish T118–T122 largely [P]

### Parallel example (Foundation)

```bash
# After T002–T004:
# Dev A: T005–T009 document
# Dev B: T010–T013 domain
# Dev C: T014–T020 engine primitives
# Then integrate T021–T025 host load
```

---

## Implementation Strategy

### MVP first (recommended)

1. Phase 1–2 foundation  
2. **US1 Explore** — proves canvas is navigable  
3. **US2 Create + US3 Connect + US4 Select** — core authoring  
4. **US5 Persist + US6 Detail** — trust + progressive disclosure  
5. Stop and validate [quickstart.md](./quickstart.md) P1 scenarios  
6. P2 US7–US9 → P3 US10 → Polish  

### Incremental delivery

- Each US phase checkpoint is demoable  
- Do not claim complete without T117 (single canvas SSoT) and T118 benchmark notes  
- Prefer vertical slices over “all models then all UI”

### Suggested MVP scope

**US1 + Foundation** is the smallest navigable MVP; product-valuable MVP for authoring is **US1–US6** (matches plan P1a–P1e).

---

## Task summary

| Phase | Story | Task IDs | Count |
| --- | --- | --- | --- |
| 1 Setup | — | T001–T004 | 4 |
| 2 Foundational | — | T005–T025 | 21 |
| 3 | US1 Explore | T026–T036 | 11 |
| 4 | US2 Create | T037–T047 | 11 |
| 5 | US3 Connect | T048–T059 | 12 |
| 6 | US4 Multi-edit | T060–T073 | 14 |
| 7 | US5 Persist | T074–T082 | 9 |
| 8 | US6 Detail | T083–T089 | 7 |
| 9 | US7 Boundaries | T090–T096 | 7 |
| 10 | US8 Layout | T097–T101 | 5 |
| 11 | US9 Export | T102–T106 | 5 |
| 12 | US10 Discoverability | T107–T116 | 10 |
| 13 Polish | — | T117–T127 | 11 |
| **Total** | | **T001–T127** | **127** |

### Format validation

- All tasks use `- [ ]`, sequential `Tnnn`, optional `[P]`, story `[USn]` on story phases only, and **file paths** in descriptions.

---

## Next command

```text
/speckit-implement
```

Or implement manually starting at T001. Optional: `/speckit-analyze` for cross-artifact consistency before coding.
