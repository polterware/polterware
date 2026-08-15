# Feature Specification: Semantic Architecture Canvas

**Feature Branch**: `006-semantic-architecture-canvas`

**Created**: 2026-08-03

**Status**: Draft

**Input**: User description: "Evoluir o canvas existente da página Architecture do Polter de um editor genérico de nós para um editor semântico completo de arquitetura de software."

**Repository note**: Product repository audited is **polterware** at `/Users/erickpatrickbarcelos/codes/polterware` (path `/Users/erickpatrickbarcelos/codes/polter` does not exist on this machine). Excalidraw interaction reference available at `/Users/erickpatrickbarcelos/codes/excalidraw`. Typed bridge is `window.polterware`. This feature **extends** `005-architecture-canvas` incrementally; it does **not** authorize engine replacement.

---

## Current Implementation Audit

Mandatory pre-spec audit of the Architecture canvas as of 2026-08-03. Findings ground all requirements below. Engine replacement is **not** proposed: no hard technical blocker was found that the existing document/history/viewport/hit-test stack cannot satisfy through domain and host evolution.

### 1. Where the Architecture page is implemented

| Layer | Location |
| --- | --- |
| Feature surface | `packages/features/architecture/src/index.tsx` — `ArchitectureView` (tabs: Canvas, Components, Dependencies, Views, Changes) |
| Canvas host | `packages/features/architecture/src/canvas/architecture-canvas-host.tsx` — `ArchitectureCanvasHost` |
| Thin re-export | `packages/features/architecture/src/architecture-canvas.tsx` |
| Node detail page | `packages/features/architecture/src/node-detail.tsx` |
| Create dialog (legacy path) | `packages/features/architecture/src/create-node-dialog.tsx` — form "Create architecture node" |
| Composition root | `apps/desktop/src/renderer/App.tsx` — wires layout load/save, routes, soft-fail save |
| Routing | Shell `ClientRoute` architecture section, tab, detail page, `entityId` / `selectedNodeId`, dialog `arch-node-create` |

### 2. Where canvas state is maintained

| Concern | Owner |
| --- | --- |
| **Document of record** | Parent `ArchitectureView` signal `doc: ArchitectureDocument` |
| **Revision / save status** | Parent signals + `createAutosaveController` |
| **Viewport (interactive)** | Host local signal `vp`; written back via `onViewportChange` into document |
| **Selection** | Host local `sel: SelectionState`; parent also holds `selectedNodeId` from route |
| **History** | Host local `history: HistoryState` (`createHistory` / `commit` / undo-redo) |
| **Tools / transient UI** | Host: tool, marquee, connect drag, editingId, contextMenu, clipboard fragment |
| **Session restore (detail round-trip)** | `session-viewport.ts` in-memory map keyed by systemId (viewport, selection, environment) |
| **Environment filter** | Parent signal `envId` (`all` \| named env) |

Document mutations flow: host → `onDocumentChange(next, CaptureMode)` → parent `setDoc` + autosave schedule.

### 3. How nodes and connections are represented

**Package**: `@polterware/frontend-canvas` document v1 (`packages/frontend/canvas/src/document/types.ts`).

- **ArchNode**: `id`, `name`, `type` (union of ~16 node types), `state` (planned/linked/observed/stale/drifted/unreachable/unknown), geometry (`x/y/width/height/zIndex`), `ports`, `origin`, `provider`, `environmentIds`, `resourceId` / `externalKey`, `boundaryId`, locks, metadata, etc.
- **Relationship**: `id`, `sourceNodeId` / `targetNodeId`, optional ports, `type` (union of ~14 relationship types), optional `label` / `protocol` / `direction` / `state` / `evidence` / style.
- **Boundary**: named rect with `kind` and member association via node `boundaryId`.
- **ArchitectureDocument**: `formatVersion`, `systemId`, nodes, relationships, boundaries, viewport, preferences, timestamps.

Serialization: `serializeDocument` / `deserializeAndMigrate` with max body size; layout stored as opaque `body_json` via Connect `GetLayout` / `UpdateLayout`.

### 4. What from the previous canvas feature already exists

Working or substantially present (engine/host level):

- Infinite surface, pan (hand / middle / wheel), zoom (wheel+mod, buttons), fit document
- Tools: select, hand, connect, add_node, add_boundary
- HTML nodes + SVG orthogonal edges + arrow marker + connect preview path
- Marquee multi-select, node drag, boundary drag with members
- Undo/redo stack with `immediately` / `eventually` / `never` capture modes
- Copy/cut/paste fragment helpers + internal clipboard
- Context menu skeleton (empty / node / relationship / multi)
- Inline rename input on nodes
- Autosave controller (debounce, status: clean/dirty/saving/saved/error)
- Session snapshot for returning from detail
- Minimap with node dots + viewport rect + click-to-navigate
- Domain registry lists (node types, relationship types, states) + light `inferRelationshipType` / `canConnect`
- Import/export JSON paths and SVG/PNG export stubs under feature `document/`
- Search helpers under feature `search/`
- Document search filter + environment dimming in host
- Auto layout button (`layoutSync` horizontal/vertical)
- Accessibility node list module, palette command IDs

Gaps relative to **semantic authorship** (this feature’s focus):

- `+` / add_node / double-click create **generic** cards (`"New component"`, hard-coded type `api` or `service`, default `planned`)
- Relationships **auto-inferred and persisted without confirmation**
- No relationship type popover / quick Connect-type mode
- No searchable type registry UI (icons, synonyms, recents, favorites)
- Cards show type label + state text only — weak semantic chrome (no icons, provider, link indicators)
- Manual **Save layout** still primary in page header
- Bottom **Open** lacks clear “open details” context
- Environment select labeled poorly as bare **All**
- Contextual floating toolbars incomplete vs desired node/edge actions
- Type registry is flat labels only — not a full extensible type definition (ports, search terms, appearance, relationship rules)
- Connect hit-test hit area on edges is weak (SVG `pointer-events: none` on edge layer)
- `typeMenu` signal exists but no real type picker UI
- CreateNodeDialog is a separate form path, not the architecture type catalog

### 5. How the `+` button creates elements today

Three paths, all generic:

1. **Canvas tool Add node (N / + toolbar)**: sets `tool = "add_node"`. On click, `docAddNode` with `name: "New component"`, `type: NODE_TYPES[2]?.id ?? "api"` (typically **api**), places at pointer, starts inline edit, returns to select.
2. **Double-click empty canvas**: same pattern with hard-coded `type: "service"`.
3. **Components tab header `+`**: opens route dialog → `CreateNodeDialog` form → `submitCreate` maps free-text type via `mapDraftType` and stacks position; not canvas placement mode.

None open a searchable type catalog; none require explicit type choice as a first-class step before place.

### 6. How connections are created today

- **Connect tool** or port/node drag: `connectFrom` + rubber-band path; on up, target node/port resolved via hit-test.
- Type chosen by `inferRelationshipType(source, target)` (heuristic: data stores → `reads_from`, queues → `publishes_to`, system → `contains`, else `calls`).
- Immediately `addRelationship` + history commit if `canConnect` ok (notes blocked; self largely blocked).
- **No user confirmation**, no popover, no invalid-target highlighting beyond drop failure, no quick-mode preset type.

### 7. How undo and redo work

- Host history via `@polterware/frontend-canvas` `commit` with full document snapshots.
- Drag uses `eventually` then final `immediately` to coalesce.
- Toolbar undo/redo + keyboard; palette IDs `history.undo` / `history.redo`.
- **Gap**: transient incomplete creates (place-without-confirm-name, pending relationship) are not yet modeled as non-persistent drafts outside history; Escape cancel policies incomplete.

### 8. How layout is saved

- Autosave: on document change with mode ≠ `never`, parent schedules `serializeDocument` → `onSaveLayout` → App `UpdateLayout` with expected revision.
- Manual **Save layout** button still exposed in Architecture page header when canvas tab + save handler present.
- Soft-fail: save errors must not blank the canvas (recent App + index behavior); local edits retained; status can show error.
- Known operational risk: Connect fetch failures can still surface as toasts/errors if save fails — must remain non-destructive.

### 9. How viewport is persisted

- Document embeds `viewport` in serialized body (when written).
- Interactive pan/zoom updates host `vp` and `onViewportChange` into parent document.
- Session snapshot (`setSessionSnapshot`) captures viewport + selection + env when opening node details; restored on index load effect.
- **Gap**: ensure consistent session restore after soft-fail saves and revision bumps; viewport should not depend on manual save.

### 10. How the minimap is rendered

- Host-local `Minimap` component: 140×96 overlay, absolute-positioned node rects scaled to document bounds, viewport rectangle, click navigates pan.
- Functional enough for navigation; may feel sparse without boundaries/edges and without drag-scrub of viewport. Not a pure placeholder, but incomplete polish vs full document fidelity.

### 11. How detail routes work

- Select node → **Open** (bottom bar) or double-click node or context **Open details** → `onOpenNode` → App route detail with `entityId`.
- `ArchitectureNodeDetail` shows richer fields; back returns to Architecture index.
- Session snapshot intended to restore zoom/viewport/env/selection.

### 12. Existing dialog / popover / command / menu / combobox components

| Primitive | Status in product |
| --- | --- |
| **A11yDialog** | `@polterware/frontend-design-system` — used by CreateNodeDialog |
| **Toast** | design-system `pushToast` |
| **Command palette** | `packages/features/commander` — `CommandPalette` product-level ⌘K |
| **Tabs / Button / Badge / Status / PageHeader / DataTable / Drawer** | design-system |
| **Native select / input** | Used heavily in architecture chrome |
| **Context menu** | Custom absolute div + `MenuItem` in canvas host (not Kobalte) |
| **Popover / Combobox / DropdownMenu / ContextMenu (accessible headless)** | **Not** present as shared packages yet |

### 13. What can be imported from Solid UI, Kobalte, or Corvu

Per AGENTS.md / constitution operator UI rules:

- **MAY** selectively install/copy Solid UI patterns (dialog, dropdown, command, select, tabs, tooltip, popover, checkbox, toast) adapted to Tailwind v4 + Polter tokens; strip unused variants.
- **Do not** run full `solidui-cli init` or generators that overwrite tokens/global CSS/layout.
- **Kobalte** for headless accessible primitives when a11y/interaction complexity requires it.
- **Corvu** only for true resizable panels (not required for this canvas feature’s core).
- **TanStack Virtual** already available for long lists (type catalog).
- **One** icon set: Hugeicons Free Stroke Rounded (individual imports) — already used in host toolbar.
- **No React** product UI.

This feature will need searchable dialog + popover + menus; plan phase chooses native/A11yDialog vs selective Kobalte/Solid UI import without engine rewrite.

### 14. Preserve vs modify vs replace

#### Preserve (do not rewrite)

- `@polterware/frontend-canvas` document model, mutations, serialize/migrate, viewport math, history capture modes, selection/marquee, hit-test core, snap, spatial index, layout worker/sync API, clipboard fragment pipeline
- Canvas host interaction loop (pointer tools, pan/zoom, world transform, HTML nodes + SVG edges)
- Autosave controller contract and Connect layout revision API
- Session viewport snapshot concept
- Soft-fail save / keep canvas usable offline or on fetch error
- Architecture tabs structure and page-based detail (no permanent side inspector)
- Security: renderer only via `window.polterware`; no new privileged IPC for document edit

#### Modify (incremental)

- Host **add node** tool and double-click empty → type selector → placement preview → name confirm → history/autosave
- Host **connect** end → relationship picker popover (or quick-mode skip) instead of silent infer-persist
- Domain **registry** → full type definitions (icons, categories, search terms, default size/ports, relationship suggestion rules)
- Node card chrome (icon, semantic fields, state indicators) driven by registry
- Relationship rendering (type label at zoom, thicker hit path, direction clarity)
- Toolbar + floating contextual toolbars for node/relationship/multi
- Context menus expanded to specified actions
- Header UX: demote/remove primary manual Save; clarify environment filter; persistence status as primary
- Bottom Open → explicit “Open details”
- Connection rules + suggestions tables expanded
- Minimap fidelity (optional boundaries, clearer viewport drag)
- CreateNodeDialog either replaced by shared “Add to architecture” catalog or wired to same registry
- NodeType / RelationshipType unions extended for missing catalog entries (with migration)
- Escape stack for placement / pending relationship / menus

#### Replace only if proven necessary (not justified today)

- Full rewrite of pointer engine or document history model
- Switching off HTML/SVG hybrid for a new whiteboard renderer
- New persistence store parallel to layout body_json
- Permanent dual canvas implementations

#### No concrete technical limitation found requiring engine replacement

Semantic typing, confirmation flows, contextual chrome, and registry-driven cards are **domain + host UI** concerns on top of the existing document/history/viewport stack. Engine work is limited to gaps (e.g. edge hit area, placement preview cursor mode, pending draft entities) that fit extension patterns already used by tools and history modes.

---

## Feature Summary

Transform the existing Architecture canvas from a **generic graph editor** into a **semantic architecture authoring surface**: every component and relationship has an explicit type; add/connect flows require intentional type choice; frequent edits live next to the selection; autosave and undo cover all confirmed operations; the current engine, performance characteristics, and security boundaries stay intact.

## Problem

Engineers can already place cards and lines, but the map does not force architectural meaning. Generic “New component” / inferred edges produce diagrams that look like whiteboards, not systems. Manual save, weak chrome, and incomplete editing affordances reinforce that gap.

## Goals

1. Typed component creation with searchable catalog and placement-then-name flow.
2. Typed relationship creation with suggestion + confirmation (and a fast multi-connect mode).
3. Contextual editing (rename, duplicate, delete, connect, open, reverse, labels) without a permanent inspector.
4. Autosave and reliable undo/redo for all confirmed document mutations.
5. Semantic card/edge appearance and lifecycle states (planned → linked/observed → stale/drifted/unreachable).
6. Organization aids: multi-select, boundaries, align/distribute, auto layout, search-to-center, fit selection, dependency highlight, environment filter.
7. Persistence visibility, functional minimap, detail navigation with session restore, import/export, change history surface readiness, future impact-view readiness.
8. Extensible type registries; domain not coupled to renderer internals.

## Non-Goals

- Freehand drawing, arbitrary shapes, stickers, image editor
- Real-time multiplayer / remote cursors / chat
- Terminal, infrastructure apply, automatic provider mutations
- Permanent lateral inspector
- Visual clone of Excalidraw
- Replacing the canvas engine without a proven technical blocker
- Full provider import integrations (model/UI must not block them; live connectors may stub)
- Storing logs/metrics payloads inside the architecture document

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Add a typed component (Priority: P1)

As an engineer, I choose a component type before placing it so the map represents real architecture, not anonymous cards.

**Why this priority**: Core authoring; without typed create, the canvas remains generic.

**Independent Test**: From empty or existing map, open Add to architecture, pick “API”, place, name “Payments API”, confirm; node persists with type API and appears after reload/autosave.

**Acceptance Scenarios**:

1. **Given** the canvas is open, **When** I click `+` (or press `N`), **Then** the searchable dialog **Add to architecture** opens and **no** node is created yet.
2. **Given** the dialog is open, **When** I search `postgres`, **Then** Database (and relevant types) appear via synonyms.
3. **Given** I select a type, **When** I click the canvas, **Then** a placement preview becomes a provisional node and name editing starts.
4. **Given** name editing is open, **When** I press Enter with a valid name, **Then** the node is committed (history + autosave) with the chosen type and initial state consistent with origin (manual → planned when unlinked).
5. **Given** name editing is open, **When** I press Escape, **Then** the provisional node is discarded and nothing is persisted for that create.
6. **Given** I open add from empty-space double-click or context “Add component here”, **When** I complete the flow, **Then** the node appears near the action origin.

---

### User Story 2 — Create a typed relationship (Priority: P1)

As an engineer, I define how two components relate so dependencies and data flow are explicit.

**Why this priority**: Edges without types undermine semantic architecture.

**Independent Test**: Drag from A’s port to B; choose Calls; confirm; edge shows type and undoes cleanly.

**Acceptance Scenarios**:

1. **Given** two typed nodes, **When** I drag a connection in select/connect mode and drop on a valid target, **Then** a pending relationship appears and a compact popover near the edge asks for relationship type (suggested first).
2. **Given** the popover is open, **When** I confirm a type, **Then** the relationship is persisted with that type and participates in undo/autosave.
3. **Given** the popover is open, **When** I press Escape, **Then** the pending relationship is removed and nothing is saved for it.
4. **Given** Connect quick mode with type “Calls”, **When** I complete multiple drops, **Then** each edge is created as Calls without opening the popover; Escape exits the mode.
5. **Given** an invalid target (e.g. note), **When** I hover/drop, **Then** the UI indicates invalid and does not create a relationship.

---

### User Story 3 — Edit a component quickly (Priority: P1)

As an engineer, I rename, duplicate, delete, connect, and open a component without hunting through panels.

**Why this priority**: Daily edit speed; no permanent inspector by design.

**Independent Test**: Select node → floating toolbar Rename / Duplicate / Delete / Open / Connect all work; undo restores.

**Acceptance Scenarios**:

1. **Given** a selected node, **When** the selection is single, **Then** a compact floating toolbar offers Open | Rename | Connect | Duplicate | More.
2. **Given** More menu, **When** I change type / copy / cut / move to boundary / z-order / highlight dependencies / delete, **Then** each applicable action works and invalid actions are hidden.
3. **Given** multi-selection, **When** the toolbar shows multi actions, **Then** Duplicate, Align, Distribute, Group, Layout, and Delete apply to the set where valid.

---

### User Story 4 — Automatic persistence (Priority: P1)

As an engineer, confirmed changes save without relying on a manual primary Save button.

**Why this priority**: Manual save contradicts continuous authoring and risks data loss perception.

**Independent Test**: Create typed node; status goes Unsaved/Saving/Saved; reload shows node; failed save keeps local work + retry.

**Acceptance Scenarios**:

1. **Given** a confirmed edit, **When** the debounce elapses, **Then** status shows Saving then Saved (or Clean) without clicking Save layout.
2. **Given** autosave fails, **When** the error is shown, **Then** local document is kept, status is Save failed, and Retry is available.
3. **Given** the page header, **When** canvas authoring is primary, **Then** manual Save is not the primary call-to-action (demoted, optional, or removed).

---

### User Story 5 — Reliable history (Priority: P1)

As an engineer, I undo/redo create, move, edit, and delete without corrupting the document.

**Why this priority**: Trust in the editor.

**Independent Test**: Sequence of create, connect, move, rename, delete; undo to empty gesture; redo re-applies; no orphan edges.

**Acceptance Scenarios**:

1. **Given** confirmed operations, **When** I undo, **Then** document and selection restore consistently.
2. **Given** a multi-step drag, **When** I undo once, **Then** the whole gesture reverts (coalesced), not each pointer sample.
3. **Given** cancelled provisional create/connect, **When** I inspect history, **Then** no empty or half-committed entries remain.

---

### User Story 6 — Organize the system (Priority: P2)

As an engineer, I select, align, distribute, group, and auto-layout so large maps stay readable.

**Independent Test**: Multi-select three nodes; align left; distribute horizontal; create boundary; auto layout selection.

---

### User Story 7 — Find components (Priority: P2)

As an engineer, I search and navigate directly to a component.

**Independent Test**: Search name; result centers viewport and selects node; keyboard walks results.

---

### User Story 8 — Navigate dependencies (Priority: P2)

As an engineer, I highlight related components to understand impact and flow.

**Independent Test**: Highlight dependencies on a node dims others and emphasizes neighbors/edges; clear restores.

---

### User Story 9 — Represent connected architecture states (Priority: P3)

As an engineer, I distinguish planned, linked, observed, stale, drifted, and unreachable components.

**Independent Test**: Linked resource shows Linked/Observed; simulated stale/drift indicators appear without inventing false health.

---

### User Story 10 — Filter by environment (Priority: P3)

As an engineer, I filter by environment and prepare for future comparison of local/dev/staging/production.

**Independent Test**: Filter Production dims/hides non-matching; control labeled Environment; Compare environments control is disabled or honest empty (no fake diff).

---

### Edge Cases

- Cancel placement after type select but before click: exit placement mode, no node.
- Cancel name edit on provisional node: remove provisional, no autosave.
- Cancel relationship popover: remove pending edge.
- Drop connection on self: reject unless type explicitly allows (default reject).
- Notes: cannot be relationship endpoints.
- Locked nodes: cannot move; connect policy remains explicit.
- Paste with internal edges: remapped IDs; external dangling edges dropped or listed.
- Duplicate multi-selection: new IDs; internal relationships rewired.
- Autosave conflict (revision): keep local; surface conflict; no silent overwrite.
- Core offline / Failed to fetch: canvas remains usable; save status error + retry; no full-page “Something went wrong” replacement of canvas.
- Import invalid JSON: reject with validation messages; no partial corrupt document apply without user choice.
- Very large documents: type dialog virtualized; canvas continues viewport culling.
- Escape layering: popover → placement → tool mode → clear selection, in defined order.
- Empty architecture: empty state with Add component CTA opening the same catalog.

---

## Requirements *(mandatory)*

### Functional Requirements

#### Component catalog & create

- **FR-001**: System MUST NOT create a component from `+` without an explicit type selection.
- **FR-002**: System MUST open a searchable dialog titled **Add to architecture** from `+`, shortcut `N`, empty double-click, context “Add component here”, command palette entry, and empty-state CTA.
- **FR-003**: Catalog MUST group initial types into Applications, Data, Infrastructure, Development, External, Organization, and Connected resources (as specified in product brief), including placeholders for link/import actions even if providers are stubbed.
- **FR-004**: Each catalog option MUST show icon, name, short description, category, optional shortcut, and recent-use indicator when applicable.
- **FR-005**: Search MUST match name, category, synonyms, related providers, and common technical terms (e.g. postgres→Database, rabbitmq→Queue, next→Web Application, cron→Scheduled Job).
- **FR-006**: Catalog MUST support recent types, favorite types (favorites do not mutate the architecture document), and full list.
- **FR-007**: After type selection, system MUST enter placement mode with cursor-following preview; pan remains available; click places provisional node and opens inline name edit.
- **FR-008**: Name confirm (Enter) MUST commit node; Escape MUST discard provisional node; definitive persist only after confirm.
- **FR-009**: Creates originating from a canvas point MUST place near that origin.

#### Type registry

- **FR-010**: System MUST maintain a central component type registry with stable IDs, labels, categories, descriptions, icons, default dimensions, allowed ports, supported metadata, relationship rules, compact/normal appearance hints, required/optional fields, search terms, and selector order.
- **FR-011**: Adding a type MUST be predominantly a registry change, not engine forks or scattered `if type === …` across unrelated components.
- **FR-012**: Model MUST allow custom types even if visual authoring of custom types ships later.
- **FR-013**: System MUST maintain a central relationship type registry (Calls, Depends on, Reads from, Writes to, Publishes to, Consumes from, Stores in, Authenticates with, Routes to, Deploys to, Hosted on, Monitors, Replicates to, Synchronizes with, Exposes, Custom, plus existing contains/related as needed) with stable IDs and display labels.

#### Relationships

- **FR-014**: System MUST NOT persist a relationship without an explicit type (quick mode counts as explicit pre-selection).
- **FR-015**: Contextual connect MUST show pending edge + nearby popover with Suggested and Other types; most likely suggestion focused; user confirms, changes, or cancels.
- **FR-016**: System MUST suggest relationship types from source/target component types using a documented suggestion table (extensible in registry).
- **FR-017**: Invalid targets MUST be indicated; drop MUST NOT create edges on invalid targets.
- **FR-018**: Connect quick mode MUST allow preselecting a relationship type, create multiple edges without popover, show active type in toolbar, remember last type for the session, and exit on Escape.
- **FR-019**: Selecting a relationship MUST show a compact contextual toolbar with: type change, edit label, reverse, delete, and More. **P1 More**: highlight ends, open related nodes if navigable. **P2 More** (may ship later): edit protocol, reconnect source, reconnect target, duplicate edge.
- **FR-020**: Relationship labels MUST be inline-editable; type MUST appear on the edge when zoom allows; direction MUST be visually clear; hit target MUST be larger than the visible stroke.

#### Cards, states, chrome

- **FR-021**: Component cards MUST display type icon, name, type label, state, and when relevant environment, provider, link indicator, and stale/drift/error indicators—without strong per-type color coding; brand accent used sparingly per DESIGN.md.
- **FR-022**: States Planned, Linked, Observed, Stale, Drifted, Unreachable, Unknown MUST follow the product meanings; manual unlinked creates default to Planned; Linked/Observed require real association/observation evidence semantics (no fake healthy status).
- **FR-023**: System MUST NOT use a permanent lateral inspector; deep info stays on dedicated detail pages.

#### Editing, clipboard, selection

- **FR-024**: Single- and multi-selection contextual toolbars and context menus MUST cover authoring actions. **P1** must cover: Open details, Rename, Connect, Duplicate, Delete, Copy, Paste, Change type, Create boundary (multi), Highlight dependencies, Fit. **P2** may cover: Align, Distribute, Layout (if not already), z-order (bring forward/send backward), Export selection, Toggle grid, Move to boundary advanced, Reconnect edge ends, Protocol edit.
- **FR-025**: Rename, duplicate, delete, copy, cut, paste MUST work; paste/duplicate MUST allocate new IDs and rewire internal relationships.
- **FR-026**: Multi-select, marquee, and keyboard selection patterns already present MUST remain and extend to new actions.

#### Organization & navigation

- **FR-027**: Boundaries MUST remain first-class (create, name, move with members, membership).
- **FR-028**: Auto layout MUST apply to selection or whole document without destroying pinned/locked intent where supported.
- **FR-029**: Search MUST center and select matching components; fit selection and fit document MUST work.
- **FR-030**: Align and distribute MUST work on multi-selection.
- **FR-031**: Dependency highlight MUST emphasize related nodes/edges and allow clear.
- **FR-032**: Environment filter MUST clearly indicate it filters by environment (not an unlabeled “All” alone) and dim or hide non-matching nodes.

#### Persistence, minimap, detail, IO, history

- **FR-033**: Confirmed document mutations MUST autosave; UI MUST show Clean / Unsaved / Saving / Saved / Save failed with retry on failure; local edits MUST NOT be discarded on save failure.
- **FR-034**: Manual Save MUST NOT be the primary authoring CTA.
- **FR-035**: Minimap MUST reflect real node positions and current viewport and support navigation (enhance, do not fake).
- **FR-036**: Opening details and returning MUST restore viewport, environment filter, and selection (session snapshot).
- **FR-037**: Import/export of architecture document JSON MUST validate and round-trip semantic types; SVG/PNG remain honest exports of current geometry.
- **FR-038**: Architecture Changes surface MUST be able to list meaningful local change summaries over time (even if initially derived from history labels / save events); full multi-user timeline may be incremental.
- **FR-039**: Model and UI MUST leave room for future architectural impact visualization without implementing full impact simulation in this feature.

#### Cross-cutting

- **FR-040**: All confirmed operations MUST participate in undo/redo and autosave.
- **FR-041**: Domain type rules MUST not be hard-wired into low-level render math; host consumes registry/adapter.
- **FR-042**: Engine package remains the interaction/document foundation; this feature evolves host + domain + selective UI primitives.
- **FR-043**: Product visual language MUST follow DESIGN.md; Excalidraw is interaction reference only.
- **FR-044**: Electron security boundary MUST remain: no unrestricted Node/FS from renderer; layout continues over existing core session Connect path.

### Key Entities

- **System**: Domain root being modeled (UI may say Project).
- **Architecture Document**: Versioned canvas document (nodes, relationships, boundaries, viewport, preferences).
- **Architecture Component (node)**: Typed map element with geometry, state, optional resource link, environments, ports.
- **Component Type Definition**: Registry entry describing semantics and presentation for a component kind.
- **Relationship**: Typed directed (or bidirectional) association between components with optional label/protocol/ports.
- **Relationship Type Definition**: Registry entry for relationship kinds and suggestion rules.
- **Boundary**: Organizational region grouping components.
- **Environment**: Named deployment/context filter (local, development, staging, production, …).
- **Resource link**: Optional association from a component to a real Resource identity.
- **Persistence status**: Clean / dirty / saving / saved / error for operator feedback.
- **Provisional create / pending relationship**: Non-persisted intermediate UI states until confirm.
- **Session snapshot**: Viewport, selection, environment for detail round-trip.

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: Understand + discover-organize architecture as primary map entry; not specialized-platform replacement.
- **Runtime owner (III)**: Renderer (Solid) for interaction; polterware-core for layout persistence; Main/preload only existing session bridge.
- **Domain vs adapter (II, IV)**: Type registries and connection rules in architecture domain layer; canvas engine stays generic document/geometry; providers only via existing resource/integration adapters.
- **Contracts (V)**: Keep `ArchitectureService` GetLayout/UpdateLayout revision + body_json; evolve schema inside JSON with migrations; no silent discovery overwrite of layout intent.
- **Product loop impact**: System Map (Architecture) primary; detail pages for depth; Evidence-aware states on links; Operations/Approvals not executed from canvas chrome.
- **Evidence expectations**: Linked/Observed/Drifted require honest signals; Planned for manual unlinked; no fabricated health.
- **Mutation safety (VII)**: Document edits are local model mutations + layout save (not infra apply). No plan→apply of cloud resources from this feature.
- **Operation audit fields**: Layout update retains existing core audit/session identity; canvas does not invent privileged ops.
- **Security boundary (VI)**: Unprivileged renderer; secrets never in document; MCP cannot bypass; soft-fail network errors.
- **Local vs cloud authority (VIII)**: Local-first document + core SQLite layout; offline usable with save error state.
- **Performance (IX)**: Preserve viewport culling; virtualize long type lists; no Main-thread blocking layout for large graphs (worker path remains available).
- **Resilience (X)**: Autosave retry; revision conflict handling; provisional cancel safety.
- **Observability (XI)**: Correlate save failures with existing session/core logging where available.
- **Test plan (XII)**: Unit tests for registry, suggestions, create/connect cancel/confirm, history; contract for document migrate; focused host interaction tests; no full suite claim without run when requested.
- **Structure created now (XIII)**: Prefer extending `packages/features/architecture` + `@polterware/frontend-canvas` + design-system primitives; no scenic empty packages.
- **Single source of truth (XIV)**: One document model; one host; remove temporary dual create paths (generic click-create vs catalog) before done.
- **Big-bang / refactor context (XV)**: Incremental evolution on `refactor/new` canvas base from 005; not a second engine.
- **Non-goals (I, XIII)**: No Excalidraw clone, no permanent inspector, no freehand whiteboard, no dual permanent canvases.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of newly created components in the happy path have an explicit user-chosen type before commit (zero silent generic creates from primary add affordances).
- **SC-002**: 100% of newly created relationships in the happy path have an explicit type (popover confirm or quick-mode preselect) before commit.
- **SC-003**: Engineers can add a typed named component (open catalog → place → name → confirm) in under 20 seconds without leaving the canvas.
- **SC-004**: Engineers can create a typed relationship (drag → choose/confirm) in under 10 seconds.
- **SC-005**: After confirmed edits, persistence status reaches Saved (or equivalent) without requiring a primary manual Save click, under normal local core availability.
- **SC-006**: Undo restores the last confirmed gesture in one step for create, connect, move, rename, and delete in manual test scripts.
- **SC-007**: Save failure leaves the in-progress map fully interactive; user can retry without losing local nodes/edges.
- **SC-008**: Returning from component detail restores prior zoom/pan and selection in session for the same System.
- **SC-009**: Catalog search for synonym samples (postgres, rabbitmq, next, cron) returns the expected types in the first result page.
- **SC-010**: No permanent lateral inspector is introduced; deep fields remain on detail pages.
- **SC-011**: No regression to “full page error replaces canvas” on ordinary draw/save failure paths.
- **SC-012**: Operators perceive cards as architecturally typed (icon + type + state visible at normal zoom) in a design review against DESIGN.md.

---

## Data Model (specification level)

### Component

| Field | Notes |
| --- | --- |
| id | Stable unique |
| typeId | Registry key |
| name | Required on commit |
| state | planned \| linked \| observed \| stale \| drifted \| unreachable \| unknown |
| geometry | x, y, width, height, z |
| ports | From type defaults / overrides |
| environmentIds | Optional multi |
| provider / resourceId / externalKey | Linkage |
| origin | manual \| discovered |
| metadata | Non-secret |
| boundaryId | Optional |

### Relationship

| Field | Notes |
| --- | --- |
| id | Stable unique |
| sourceId / targetId | Components |
| typeId | Registry key — required on persist |
| label / protocol | Optional |
| sourcePortId / targetPortId | Optional |
| direction | forward \| bidirectional |
| state / evidence | Optional honesty fields |
| timestamps | Created/updated when available |

### Registries

ComponentTypeDefinition and RelationshipTypeDefinition as in FR-010/013; suggestion rules as (sourceType, targetType) → ordered typeIds.

### Migration

- Existing documents with current NodeType/RelationshipType unions remain valid.
- New type IDs added via migrate defaults; unknown types render as Custom/unknown safely.
- Generic historical nodes (`service`/`api` with “New component”) remain loadable; editing encourages explicit rename/type change but no forced bulk rewrite.

---

## Key Flows

### Create component

```text
+ / N / dblclick empty / context / palette / empty CTA
  → Add to architecture (searchable)
  → choose type
  → placement mode (preview follows cursor; pan ok)
  → click place
  → inline name (Enter commit / Escape discard)
  → history + autosave
```

### Create relationship (contextual)

```text
drag port/source → valid target
  → pending edge
  → popover: Suggested + Other
  → confirm | change | Escape cancel
  → history + autosave
```

### Create relationship (quick)

```text
Connect tool + type dropdown (e.g. Calls)
  → drag multiple pairs
  → each drop commits with preselected type
  → Escape leaves mode
```

### Edit

```text
select node/edge/multi → contextual toolbar + context menu
  → rename / type / reverse / align / …
  → history + autosave
```

### Persist

```text
confirmed mutation → dirty → debounced save → saving → saved | error+retry
viewport + session snapshot on detail navigation
```

---

## UI States

| State | Meaning |
| --- | --- |
| Idle select | Default tool |
| Placement mode | Type chosen; awaiting click |
| Naming provisional | Inline name; not yet durable |
| Connect drag | Rubber band |
| Pending relationship | Awaiting type confirm |
| Connect quick mode | Preselected type active |
| Saving / Saved / Save failed | Persistence chrome |
| Highlight dependencies | Emphasis mode |
| Environment filtered | Dim/hide non-matching |

---

## Shortcuts (initial)

| Action | Shortcut |
| --- | --- |
| Add component (catalog) | `N` |
| Select | `V` |
| Hand | `H` |
| Connect | `C` |
| Boundary | `B` |
| Undo / Redo | ⌘Z / ⇧⌘Z |
| Delete | Backspace/Delete |
| Escape | Dismiss top transient layer |
| Command palette | product ⌘K (register architecture commands) |

(Exact platform modifiers follow existing host keymap.)

---

## Migration Strategy (from current generic editor)

1. Keep document v1 and host interaction loop.
2. Expand registries and suggestions; migrate type unions carefully.
3. Replace silent add/connect with typed flows; remove dual generic create paths.
4. Upgrade card/edge chrome and contextual toolbars.
5. Demote manual Save; keep autosave + status + retry.
6. Wire catalog into commander palette.
7. Preserve layout API; soft-fail save behavior.
8. Tests first on registry, confirm/cancel create-connect, history.
9. Do not ship “dialogs only” as complete feature—cards, connect, history, autosave, minimap, and organization stories required for done.

---

## Risks

| Risk | Mitigation |
| --- | --- |
| Scope too large for one delivery | P1 slice first (typed create, typed connect, contextual edit, autosave, history); P2/P3 explicitly ordered in tasks |
| Accidental engine rewrite | Spec + plan preserve list; reject PRs that replace host wholesale without measured gap |
| Save/fetch failures blank UI | Keep soft-fail; status + retry; never PageError full replace on save |
| Type explosion / messy unions | Registry-driven; migrate unknown; avoid scatter conditionals |
| A11y of custom menus | Prefer Kobalte/Solid UI selective primitives for dialog/popover/menu |
| Dual create dialogs | One catalog; retire or adapt CreateNodeDialog |

---

## Dependencies

- Existing `005` canvas engine and Architecture host
- Connect layout API + polterware-core session
- DESIGN.md tokens; design-system + optional Kobalte/Solid UI
- Hugeicons Free Stroke Rounded icons
- Commander palette registration
- Excalidraw local clone as interaction reference only

---

## Acceptance Criteria (feature complete)

Feature is complete only when:

1. `+` does not create cards directly.
2. Component selector works (search, categories, keyboard).
3. Components are typed on commit.
4. Relationships are typed on commit.
5. No untyped relationship can be persisted.
6. Relationship suggestions work.
7. Connect quick mode works.
8. Rename, duplicate, delete work.
9. Copy/cut/paste preserve internal relations with new IDs.
10. Undo/redo cover relevant confirmed actions.
11. Contextual toolbars implemented for nodes and relationships.
12. Context menus implemented per brief.
13. Cards show type, state, and link cues.
14. Autosave is the main persistence path.
15. Persistence states are visible with retry on failure.
16. Minimap reflects real document + viewport navigation.
17. Search centers elements.
18. Fit selection works.
19. Boundaries work.
20. Auto layout works.
21. Align/distribute work.
22. Dependency highlight works.
23. Environment filter is clear and functional.
24. Import/export work for the document.
25. Detail page restores canvas context.
26. Visuals follow DESIGN.md.
27. No permanent lateral inspector.
28. No intentional performance regression on large maps (culling retained).
29. No Electron security regression.
30. Agreed tests/typecheck/lint/build for the changed surface pass when run for verification.

---

## Assumptions

- Repository of record is **polterware** (not a separate `polter` checkout).
- Existing document field names may map 1:1 to spec language (component ≡ ArchNode).
- Stub “Import from integration/file” catalog rows may open honest empty/disabled flows until integrations exist.
- P3 environment **comparison** may be UX scaffolding + filter only; full multi-env diff can be a follow-on if time-boxed in plan.
- Impact visualization is readiness (hooks/commands/model notes), not a full simulation engine.
- Soft-fail save and local-first editing remain product policy.
- Favorites/recents may live in local preferences (not architecture document body).

---

## Handoff for `/speckit-plan`

Plan **must** open with an explicit matrix:

| Area | Preserve | Modify | Replace |
| --- | --- | --- | --- |
| Document v1 + mutations + history | ✓ | extend types/migrate | ✗ |
| Viewport / hit-test / selection | ✓ | edge hit area, placement tool | ✗ |
| ArchitectureCanvasHost | ✓ shell | add/connect/toolbars/chrome | ✗ wholesale rewrite |
| Domain registry | | expand to full definitions | flat label lists only |
| CreateNodeDialog | | merge into catalog or retire | parallel generic form long-term |
| Autosave + layout API | ✓ | status UX, demote manual save | ✗ new store |
| Minimap | ✓ | fidelity | ✗ |

Then: data model deltas, UI primitive choices (A11yDialog vs Kobalte), P1→P3 task slices, test strategy, and migration of existing layouts—without proposing a new engine unless a measured technical limitation is documented.

**Next commands** (user-requested sequence): `/speckit-clarify` → `/speckit-plan` → `/speckit-tasks` → `/speckit-analyze` → `/speckit-implement`.

**Do not implement in specify phase.**
