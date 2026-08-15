# Feature Specification: Architecture Canvas Engine

**Feature Branch**: `005-architecture-canvas`

**Created**: 2026-08-03

**Status**: Draft

**Input**: User description: "Implement the complete Polter architecture canvas using the local Excalidraw source as the primary engineering reference for interactions, state management, history, viewport, selection, pointer events, shortcuts, clipboard, persistence, and performance—while remaining a domain-specialized architecture surface (not a whiteboard clone)."

**Repository note**: Product repository audited is `polterware` at `/Users/erickpatrickbarcelos/codes/polterware` (path `/codes/polter` does not exist). Excalidraw reference audited at `/Users/erickpatrickbarcelos/codes/excalidraw`. Typed bridge is `window.polterware` (not `window.polter`).

---

## Reference Audit

Read-only audit of Polterware and the local Excalidraw clone, performed before this specification. Excalidraw is treated as an **interaction and state-engineering reference**, not as code or assets to copy blindly.

### A. Polterware inventory

#### What already exists and can be kept

| Area | Finding |
| --- | --- |
| Product shell | Seven destinations, integrated sidebar, MainSurface, title-bar route history, dark-first tokens in `DESIGN.md` / Tailwind base. |
| Architecture feature | `packages/features/architecture`: index tabs (Canvas, Components, Dependencies, Views, Changes), create-node dialog, fixture tables/feeds, page-based node detail. |
| Routing | `ClientRoute` supports architecture section, tab, detail page, `entityId` / `entityKind: "arch-node"`, `selectedNodeId`, dialog `arch-node-create`. |
| Composition root | `apps/desktop/src/renderer/App.tsx` wires ArchitectureView to core clients, layout load/save, route transitions. |
| Shared canvas package | `@polterware/frontend-canvas`: `CanvasModel`, serialize/deserialize, operational map builder, minimal whole-model history helpers, layout worker stub. |
| Design system | Button, Badge, Tabs, PageHeader/PageContent, empty/loading/error, dialogs, toast, status chips, drawer (process only). |
| Command palette | `packages/features/commander` with navigation + actions; `⌘K` pattern already product-level. |
| Persistence path | Connect RPC `ArchitectureService.GetLayout` / `UpdateLayout` with optimistic `revision` and opaque `body_json`. SQLite `architecture_layouts`. Domain rule: discovery MUST NOT silently rewrite layout. |
| Security boundary | Preload exposes only `window.polterware` (session, app, commander). `contextIsolation`, no renderer FS/exec. |
| Domain language | System root, Resource, Environment, Relationship with evidence distinction, page-based depth (no permanent inspector). |
| Prototypes | `/prototype` PNG composition references for dense operator chrome (not pixel-literal sources). |

#### What must be refactored

| Area | Gap |
| --- | --- |
| `architecture-canvas.tsx` | Static band layout, CSS `scale` zoom only, overflow scroll as “pan”, no free pan, no drag, no marquee, non-interactive edges, no tools, no minimap, no floating context toolbar. |
| `CanvasModel` | Too thin for a real editor: no format version, viewport, boundaries, ports, relationship semantics, z-order policy, interaction/document split, migration story. |
| History | Snapshot-stack helpers exist but are unused; no coalescing, no delta model, no selection restore policy. |
| Layout save | Manual “Save layout”; no autosave, no dirty/saving/error indicator, no viewport persistence, weak validation. |
| Create flow | Dialog exists; canvas lacks double-click empty-space create, contextual create, immediate rename, Connect tool. |
| Layer bands | Hardcoded visual lanes, not first-class boundaries/sections with membership. |
| Search | Header filter only; no center-on-result, keyboard result walk, or multi-field search. |
| Detail return | Route back exists; viewport/selection/environment restoration not modeled as first-class session state. |
| Worker | `worker/layout.ts` is a trivial grid placeholder. |

#### What must be created

- Generic **canvas engine** (viewport, tools, pointer pipeline, selection, hit testing, history, clipboard, commands, snap/guides, layers, base serialization).
- Architecture **domain adapter** (node types/states, relationship types, ports/binding rules, environment filter, badges, detail navigation, semantic menus).
- Versioned **Architecture Document** format + migrations + import validation.
- Full interaction surface: tools, marquee, connect ports, boundaries (P2), minimap, zoom controls, shortcuts sheet, autosave UX, export (P2).
- Unit/integration/e2e/performance/visual regression coverage proportional to risk.
- Architectural documentation of engine vs domain vs app integration boundaries.

#### Public contracts that must not break

1. **`ArchitectureService` layout API**: `GetLayout` / `UpdateLayout` with system id, expected revision, body JSON string—body remains the versioned document payload (may evolve schema **inside** JSON with migrations; wire contract stays).
2. **Optimistic concurrency**: conflict on revision mismatch; never silent overwrite.
3. **Discovery rule**: discovery association must not silently rewrite user layout intent.
4. **`window.polterware` bridge**: stay minimal and typed; no raw `ipcRenderer` or Node in renderer.
5. **Shell route model**: architecture section, tabs, page-based detail, no permanent right inspector.
6. **Browser-safe packages**: shared frontend/canvas/UI must not import Electron/FS/PTY/secret stores.
7. **Constitution domain language**: System (Project alias allowed in UI), Resource, Environment, Evidence-aware relationships.
8. **License**: product Apache-2.0; any Excalidraw-derived ideas must respect MIT attribution if substantial code is adapted (prefer conceptual adaptation over code copy).

#### Package placement candidates

| Concern | Placement guidance (final layout decided in `/speckit-plan`) |
| --- | --- |
| Generic engine | Expand `@polterware/frontend-canvas` (or sibling engine package if public API splits cleanly). |
| Architecture domain UI + semantics | `packages/features/architecture` (and pure domain helpers if testable without Solid). |
| Design chrome | `@polterware/frontend-design-system` reuse; selective Kobalte/Solid UI for menus/dialogs. |
| Persistence adapter | Existing Connect client in `@polterware/frontend-data` + App composition; no second storage root. |
| Heavy layout | Worker under canvas package (existing stub). |

#### What needs IPC vs stays in renderer

| Capability | Path |
| --- | --- |
| Document load/save/autosave | **Not new Electron IPC**—Connect to polterware-core (`getLayout` / `updateLayout`). |
| Session endpoint/token | Existing bridge `session.*` only. |
| Interaction, history, clipboard (browser), render, search-in-document, minimap | **Renderer-only**. |
| Native file pickers for import/export | Prefer browser File/Blob APIs first; add typed Main channels **only if** product requires native OS dialogs/paths. |
| Secrets / ops / terminal | **Out of scope**; never via canvas document. |

#### Renderer-only responsibilities (must stay unprivileged)

Pointer/tool state machine, viewport math, hit testing, selection, ephemeral previews, undo/redo stacks in memory, command layer, HTML node chrome, SVG/Canvas edge rendering, client-side validation of document schema before send, sanitization of pasted/imported text.

### B. Excalidraw inventory (local clone)

**License**: MIT (Copyright 2020 Excalidraw)—compatible with conceptual reference; any copied code requires attribution and license notice compliance. Prefer reimplementation of concepts over copying.

#### Relevant modules

| Module / path | Role relevant to Polter |
| --- | --- |
| `packages/element/src/types.ts` | Element model, ordering, soft-delete patterns. |
| `packages/element/src/Scene.ts` | Scene authority for elements; selection queries; non-deleted views. |
| `packages/element/src/store.ts` | `Store`, `StoreSnapshot`, `CaptureUpdateAction` (`IMMEDIATELY` / `EVENTUALLY` / `NEVER`), durable vs ephemeral increments. |
| `packages/element/src/delta.ts` | Structural deltas for history and collaboration-friendly apply/inverse. |
| `packages/excalidraw/history.ts` | Undo/redo stacks of deltas; stack invalidation; selection-aware apply. |
| `packages/excalidraw/appState.ts` | Separation of persistent vs ephemeral UI state; storage conf flags. |
| `packages/excalidraw/viewport.ts` | Zoom-to-fit, center-on, scroll constraints, cursor-focal zoom helpers. |
| `packages/element/src/selection.ts` | Click/group/frame-aware selection; marquee contain vs intersect modes. |
| `packages/element/src/collision.ts` | Hit testing geometry. |
| `packages/element/src/binding.ts` | Arrow/element binding (very freeform-specific). |
| `packages/element/src/frame.ts` / `groups.ts` | Containers and groups (map carefully to boundaries). |
| `packages/element/src/dragElements.ts` / `resizeElements.ts` / `align.ts` / `distribute.ts` | Transform utilities. |
| `packages/element/src/fractionalIndex.ts` | Stable z-order without full renumber thrash. |
| `packages/excalidraw/clipboard.ts` | Internal JSON + system clipboard fallbacks. |
| `packages/excalidraw/data/restore.ts` / `json.ts` | Import restore, normalization, migration-ish repair. |
| `packages/excalidraw/actions/*` | Command/action objects with `keyTest` and perform. |
| `packages/excalidraw/renderer/*` | Static scene vs interactive overlays; throttle RAF. |
| `packages/excalidraw/components/canvases/*` | Multi-layer canvas composition. |
| `packages/math/*` | Pure geometry primitives (conceptually useful). |
| `packages/excalidraw/tests`, `packages/element/tests` | Interaction tests without pure visual dependency (~98+ files). |

#### Patterns to adapt

1. **Document vs app/interaction state**: durable document (nodes, edges, boundaries, order) separated from ephemeral UI (tool, hover, marquee, drag preview, open menus).
2. **History capture modes**: immediate vs eventual vs never—coalesce continuous drag into one undo step.
3. **Delta-based history** (or equivalent structural patches) instead of unlimited full-document clones for large graphs.
4. **Command/action layer**: keyboard and palette call the same operations.
5. **Viewport math library**: screen↔document coordinates, zoom-about-cursor, fit bounds, restore viewport.
6. **Hit-test priority**: handles/ports > nodes > edges > boundaries > empty.
7. **Marquee modes**: contain vs intersect (left-to-right / right-to-left) if predictability holds.
8. **Soft delete + restore via history** as an option for robust undo (if chosen in plan).
9. **Versioned restore/migrate** on load; never execute imported content.
10. **Static + interactive layers**: expensive scene vs lightweight selection/guides overlays.
11. **Worker offload** for layout/heavy compute.
12. **Test pure functions** (coords, selection, history, clipboard graph rewrite) heavily.

#### Patterns that do not fit Polter

| Excalidraw pattern | Why not |
| --- | --- |
| Freehand/pencil, arbitrary shapes, stickers, images as first-class art | Whiteboard product, not operational architecture. |
| Rough.js hand-drawn aesthetic | Conflicts with `DESIGN.md` quiet technical control. |
| React component tree + Jotai editor state as runtime dependency | Product forbids React product UI. |
| Multiplayer cursors, collab rooms, encryption share links | Explicit non-goals now. |
| Library of generic drawings | Wrong domain. |
| Embedding `@excalidraw/excalidraw` as the editor | Visual mismatch, React stack, freehand-first data model, hard domain binding, performance and styling conflicts. |
| Permanent properties sidebar as primary edit surface | Polter uses page-based detail + compact floating chrome. |
| Element = pure geometry with optional text | Polter elements are typed domain nodes with providers, environments, link state, evidence. |

#### React → SolidJS port risks

- Excalidraw’s performance relies on **imperative canvas draws** and careful React memoization; naïvely porting to Solid **signals wrapping the whole scene each pointer move** will thrash.
- Solid fine-grained reactivity helps **if** document mutations and viewport transforms are modeled so only affected nodes/overlays recompute; still need imperative transforms for pan/zoom of large scenes.
- Action/App mega-class patterns from Excalidraw must become **testable pure modules + thin Solid hosts**, not a single 10k-line component.
- Lifetime of stores differs (React refs vs Solid roots); pointer capture and window listeners must be cleaned deterministically.
- Do not transplant Excalidraw’s element type hierarchy wholesale—it encodes freehand assumptions.

#### Engine decision

**Decision: Build a Polter-owned canvas engine**, conceptually informed by Excalidraw (and general graph-editor practice), implemented for Solid + domain architecture.

**Not chosen now:**

| Option | Rejection rationale |
| --- | --- |
| Embed `@excalidraw/excalidraw` | React dependency forbidden for product UI; whiteboard model; hard to enforce domain semantics; visual clone risk; license/branding confusion. |
| Full code fork of Excalidraw packages | Massive React surface, wrong element model, high maintenance, license attribution burden without product fit. |
| Generic React Flow / Rete-style graph as-is | Often React-bound; node graph UX differs; still need custom viewport/history/clipboard depth; domain still custom. |
| Pure third-party “whiteboard” | Same mismatch as Excalidraw embed. |

**May evaluate in plan (not pre-selected):** small **headless** geometry/spatial-index libraries, pure math helpers, or layout algorithms—only if browser-safe, framework-agnostic, and clearly cheaper than in-house for that slice. Any dependency needs concrete leverage and DESIGN/security review.

**Justification**: Polter needs **semantic architecture objects**, HTML-rich node chrome under DESIGN tokens, page-based detail, evidence-aware edges, environment dimensions, Connect-backed versioned persistence, and Electron-safe renderer isolation. Interaction quality should match mature editors via **adapted patterns** (history modes, viewport math, actions, hit testing), not via embedding a whiteboard.

### C. Polter audit summary (keep / refactor / create)

1. **Keep**: shell, design tokens, Architecture tabs + detail page pattern, Connect layout API, discovery non-rewrite rule, command palette host, feature package boundaries.
2. **Refactor**: canvas model/document format, architecture canvas view, layout save UX → autosave, history, search integration, environment as filter dimension.
3. **Create**: engine capabilities listed above; domain adapter; migrations; tests; benchmarks; docs.
4. **Do not break**: contracts in §A public contracts.
5. **Independent package**: engine in frontend-canvas (generic); domain in feature-architecture.
6. **IPC**: only if native file dialogs proven necessary; persistence via existing core RPC.
7. **Renderer-local**: all interaction and presentation logic.

---

## Problem Summary

The Architecture page is still a **mostly static map**: auto-banded cards, simple scale zoom, non-interactive edges, manual save, and thin document model. Operators cannot fluidly explore, author, connect, multi-select, undo, or recover viewport/session state with confidence. The product needs a **complete architecture editing surface** with mature interaction quality, domain meaning, versioned persistence, and performance for hundreds of nodes—without becoming a generic whiteboard or Excalidraw clone.

---

## Goals

### Primary goal

Deliver a reliable, reusable **canvas engine** integrated with Polter Architecture, with interaction completeness comparable to mature diagram editors, specialized for operational architecture.

### Secondary goals

- Clear split: generic engine vs architecture domain vs app integration.
- Extensible element/relationship types without engine rewrites.
- Versioned persistence and migrations.
- Future-ready for auto-layout and collaboration **hooks** without implementing multiplayer now.
- Testable interactions without exclusive reliance on visual tests.
- Electron renderer remains browser-safe and unprivileged.
- Visual language strictly follows `DESIGN.md` and product shell composition.

---

## Non-Goals

This feature MUST NOT implement:

- Full Excalidraw clone or freehand/pencil tools
- Arbitrary presentation shapes, stickers, image editor
- Real-time multiplayer, remote cursors, chat, video
- Infrastructure mutation from the canvas, embedded terminal, full logs/metrics
- Secret storage in the document
- Autonomous agents rewriting resources from the canvas
- Permanent right-hand inspector replacing detail pages
- Traditional dashboard of disconnected cards as the primary Architecture UX
- Comments / live collaboration surfaces (deferred)
- Direct import of `.excalidraw` unless a clear semantic conversion is later justified (not required)

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Explore a large architecture (Priority: P1)

As an engineer, I navigate a large architecture quickly without losing orientation: pan, cursor-centered zoom, fit document/selection, search-and-center, minimap, zoom indicator, and restored last viewport.

**Why this priority**: Exploration is the daily entry path; without fluid navigation the canvas fails even if create/edit exist.

**Independent Test**: Open a document with dozens of nodes; pan/zoom with trackpad and mouse; fit all; fit selection; search a node and land centered at readable zoom; reopen and confirm viewport restoration.

**Acceptance Scenarios**:

1. **Given** a document larger than the viewport, **When** the operator pans (space-drag, middle button, or trackpad), **Then** navigation feels continuous and does not use traditional scrollbars as the primary mechanism.
2. **Given** the pointer is over the canvas, **When** the operator zooms (pinch or mod+wheel), **Then** zoom centers on the cursor and stays within min/max limits.
3. **Given** a selection or full document, **When** the operator invokes fit selection / fit document, **Then** content is framed legibly with padding.
4. **Given** a prior session viewport, **When** the app reopens the same architecture document, **Then** offset and zoom are restored.
5. **Given** a search hit, **When** the operator chooses a result, **Then** the node is selected and centered at a readable zoom.

---

### User Story 2 - Create conceptual components (Priority: P1)

As an engineer, I add conceptual components for parts of the system not yet imported or linked, with predictable placement, required type, immediate rename, and safe cancel.

**Why this priority**: Authoring empty or partial architectures is the start of modeling before discovery links exist.

**Independent Test**: Create nodes via Add button, command palette, context menu, and double-click empty canvas; cancel mid-create; confirm type required and name edit.

**Acceptance Scenarios**:

1. **Given** an empty or existing canvas, **When** the operator creates a node via any supported entry point, **Then** a typed node appears at a predictable position and enters name edit when appropriate.
2. **Given** create-in-progress, **When** the operator presses Escape or cancels, **Then** no orphan half-created durable node remains (or it is fully rolled back).
3. **Given** a new node, **When** type is missing, **Then** creation cannot complete without a type.

---

### User Story 3 - Connect components with semantic relationships (Priority: P1)

As an engineer, I create directed relationships between components using ports, previews, valid/invalid target feedback, labels, reconnect, delete, and undo/redo.

**Why this priority**: Relationships are the core of architecture understanding; decorative lines without semantics fail the product thesis.

**Independent Test**: Connect API → Database with “Reads from”; edit label; reconnect; undo once removes the relationship; redo restores it.

**Acceptance Scenarios**:

1. **Given** two nodes, **When** the operator drags from an output port to a valid input, **Then** a directed relationship is created with type chosen or inferred.
2. **Given** connect-in-progress, **When** the pointer is over an invalid target, **Then** the UI shows rejection feedback and does not create an edge.
3. **Given** an existing relationship, **When** the operator undoes, **Then** a single undo removes it; redo restores it.

---

### User Story 4 - Edit a multi-selection (Priority: P1)

As an engineer, I select one or many components (click, shift-click, marquee, select-all, boundary contents) and move, duplicate, delete, copy/paste them as one transaction.

**Why this priority**: Real architectures are edited in groups; single-node-only tools do not scale.

**Independent Test**: Marquee three nodes; drag; relative positions preserved; edges follow; one undo reverses the move.

**Acceptance Scenarios**:

1. **Given** multiple nodes, **When** the operator marquee-selects and drags, **Then** relative positions stay stable and relationships track endpoints.
2. **Given** a selection of three nodes and two internal relationships, **When** copy/paste runs, **Then** new IDs are generated, internal edges rewired, external edges not duplicated.
3. **Given** a multi-delete, **When** undo runs, **Then** nodes and their relationships restore coherently.

---

### User Story 5 - Persist the document (Priority: P1)

As an engineer, I close and reopen Polter without losing the canvas document or working viewport.

**Why this priority**: Trust requires durable state; manual save-only is insufficient for dense editing.

**Independent Test**: Edit layout; wait for autosave success; kill and relaunch; document, relationships, boundaries, and viewport match.

**Acceptance Scenarios**:

1. **Given** dirty changes, **When** autosave completes, **Then** status shows saved and core stores a new revision.
2. **Given** a failed save, **When** the error is shown, **Then** the last valid snapshot remains recoverable and the document is not corrupted.
3. **Given** an older format version, **When** the document loads, **Then** migrations apply or a clear recoverable error is shown without data loss of the stored blob.

---

### User Story 6 - Open full node detail and return (Priority: P1)

As an engineer, I open a node’s full detail page and return to Architecture with document, viewport, environment, and selection restored—without a permanent side inspector.

**Why this priority**: Progressive disclosure and page-based depth are product constitution, not optional chrome.

**Independent Test**: Select node at a zoom; open detail (double-click/Enter/menu); back; same viewport and selection.

**Acceptance Scenarios**:

1. **Given** a selected node, **When** detail opens, **Then** navigation uses the existing architecture detail route pattern.
2. **Given** return from detail, **When** Architecture shows, **Then** viewport, environment filter, and selection are restored.

---

### User Story 7 - Organize with boundaries (Priority: P2)

As an engineer, I group components into visual/semantic boundaries (system, domain, ownership, deployment) with membership, collapse/expand, rename, move-with-content, and external relationship preservation.

**Why this priority**: Medium-scale clarity depends on grouping; after core edit loop.

**Independent Test**: Create boundary from selection; move boundary; members move; collapse; external edges remain valid.

**Acceptance Scenarios**:

1. **Given** a selection, **When** “create boundary” runs, **Then** members are associated deterministically (not accidental intersection alone).
2. **Given** a boundary move, **When** completed, **Then** members move unless explicit move-without-content is used.
3. **Given** collapse, **When** expanded again, **Then** membership and external relationships remain intact.

---

### User Story 8 - Auto-layout (Priority: P2)

As an engineer, I reorganize selection or whole document (horizontal/vertical) with undo as one operation, preserving pinned elements, without freezing the UI.

**Why this priority**: Cleanup accelerates comprehension after import/discovery growth.

**Independent Test**: Auto-layout selection; undo restores previous positions in one step; UI stays responsive during compute.

**Acceptance Scenarios**:

1. **Given** overlapping nodes, **When** auto-layout runs on selection, **Then** overlaps reduce and one undo restores prior layout.
2. **Given** pinned nodes, **When** layout runs, **Then** pinned positions are preserved per rules.
3. **Given** a large graph, **When** layout is heavy, **Then** computation does not block pointer interaction (off main path / progressive apply).

---

### User Story 9 - Export a view (Priority: P2)

As an engineer, I export canonical JSON and visual PNG (SVG when rendering strategy allows) for documentation—selection or full document—without temporary overlays.

**Why this priority**: Sharing architecture outside the app is secondary to editing fidelity.

**Independent Test**: Export JSON of selection; export PNG of viewport/document; invalid import rejected earlier path remains safe.

**Acceptance Scenarios**:

1. **Given** a valid document, **When** JSON export runs, **Then** output is versioned and schema-valid.
2. **Given** PNG export, **When** completed, **Then** selection chrome and guides are omitted; background option respected.

---

### User Story 10 - Discoverability of commands and recovery states (Priority: P3)

As an engineer, I find operations via command palette and shortcuts reference, and recover from empty, error, migration, and large-document states without dead ends.

**Why this priority**: Completes operability after core edit/persist loops.

**Independent Test**: Open shortcuts help; run “Add node” from palette; open empty state CTAs; import invalid file safely.

**Acceptance Scenarios**:

1. **Given** command palette, **When** an architecture command runs, **Then** it uses the same command layer as shortcuts (no duplicate logic paths).
2. **Given** empty document, **When** empty state shows, **Then** create / import / template / essentials shortcuts are offered as available.
3. **Given** invalid import, **When** attempted, **Then** current document is unchanged and error is recoverable.

---

### Edge Cases

- Pointer leaves window mid-drag; pointer capture and cancel/commit rules still produce a coherent document.
- Rapid undo/redo during autosave in flight; revision conflicts surface without silent data loss.
- Paste when clipboard has non-architecture content; safe ignore or text sanitize only.
- Connect self-loop if disallowed by rules; rejected with feedback.
- Node deleted while its detail page is open; detail shows not-found; canvas remains consistent.
- Environment filter hides endpoints of a relationship; edge visibility/policy is defined and stable.
- Document with 0 nodes vs 500+ nodes; empty and stress paths both work.
- Corrupted `body_json` in storage; last good revision or explicit repair path; no crash loop.
- Concurrent edit from another surface of the same system (if any) hits revision conflict.
- IME composition during rename; global shortcuts do not fire.
- Reduced motion preference; non-essential motion suppressed.
- Locked or pinned elements refuse accidental moves.
- Import of unknown future format version; refuse replace, keep current.

---

## Requirements *(mandatory)*

### Functional Requirements

#### Viewport & navigation

- **FR-001**: Canvas MUST present an effectively unbounded workspace with pan via space+drag, middle button, and trackpad-consistent scrolling.
- **FR-002**: Zoom MUST support pinch, mod+wheel, control buttons, cursor-focal zoom, min/max clamps, reset, fit document, and fit selection.
- **FR-003**: System MUST convert stably between screen and document coordinates.
- **FR-004**: Dotted (or design-token) background MUST remain visually stable under pan/zoom.
- **FR-005**: Traditional scrollbars MUST NOT be the primary navigation mechanism.
- **FR-006**: System MUST provide a compact minimap (extent, viewport rect, major nodes/boundaries) with click/drag navigation.
- **FR-007**: System MUST show a discrete zoom indicator.
- **FR-008**: Last significant viewport MUST be restorable with the document session (persist with layout payload or paired preference—plan decides storage fielding without second authority).

#### Tools & interaction machine

- **FR-009**: Tools MUST include at least: Select (default), Hand, Connect, Add node, Add boundary, Add text note.
- **FR-010**: Space MUST temporarily activate Hand without permanently changing the active tool.
- **FR-011**: Escape MUST cancel in layered order: text edit → in-progress connection → in-progress creation → open menu/popover → clear selection.
- **FR-012**: Pointer pipeline MUST support mouse, trackpad, and compatible pointer events; touch best-effort on desktop platforms.

#### Selection

- **FR-013**: Selection MUST support click, shift-toggle, marquee, select-all, select boundary contents, and select relationships.
- **FR-014**: Left-to-right marquee SHOULD select fully contained elements; right-to-left MAY select intersecting elements if behavior is documented and tested for predictability.
- **FR-015**: Empty-canvas click clears selection unless an applicable modifier is held.
- **FR-016**: Selection chrome MUST be clear; resize handles only when relevant; **no permanent side inspector**.
- **FR-017**: A compact floating contextual toolbar MAY appear for the selection; it MUST stay non-blocking and design-compliant.

#### Hit testing

- **FR-018**: Hit testing MUST distinguish node body, port, handle, label, relationship path, boundary, empty space, and stacking.
- **FR-019**: Relationships MUST have a larger invisible hit area than visual stroke width.
- **FR-020**: Hit priority MUST be explicit and tested (ports/handles above nodes above relationships above boundaries above empty).

#### Nodes

- **FR-021**: Nodes MUST be rich, typed, design-system-consistent objects (not freehand shapes).
- **FR-022**: Nodes MUST support select, drag, duplicate, delete, rename, type change when allowed, resize when allowed, ports, context menu, hover/focus, environment/provider/status badges, open detail, parent boundary, and zoom-level compaction.
- **FR-023**: Zoom LOD: distant = icon + short name + status; medium = name + type + key links cue; near = essential metadata only; full detail remains on the detail page.
- **FR-024**: Double-click or Enter on a selected node MUST open its detail page via existing routing style (`architecture` detail / node id).
- **FR-025**: Initial node types MUST include: System, Web Application, API, Service, Worker, Database, Cache, Queue, Event Stream, Storage, Repository, Runtime/Host, External Service, Integration, Custom Resource (extensible registry).
- **FR-026**: Initial states MUST include: Planned, Linked, Observed, Stale, Drifted, Unreachable, Unknown (extensible).

#### Relationships

- **FR-027**: Relationships MUST be directed semantic edges with type, optional label/protocol, ports, state, and visual order.
- **FR-028**: Creation MUST support port drag, live preview, snap to port, validation, reconnect endpoints, edit type/label, delete, hover, connected-node highlight, and path highlight on demand.
- **FR-029**: Routing SHOULD avoid node bodies when feasible and MUST update fluidly while endpoints move.
- **FR-030**: Initial relationship types MUST include: Depends on, Calls, Reads from, Writes to, Publishes to, Consumes from, Authenticates with, Deploys to, Stores in, Monitors, Routes to, Custom.
- **FR-031**: Rendering strategy (SVG vs Canvas2D vs hybrid) MUST be chosen in planning/benchmarks for HTML-rich nodes; not by library fashion.

#### Ports & binding

- **FR-032**: Ports MUST be stable-id, predictable, shown on hover/selection/connect tool, support in/out/both, survive move/save/reload, and enforce semantic rules when defined.
- **FR-033**: Model MUST allow future dynamic ports without engine rewrite.

#### Transform, snap, order

- **FR-034**: Drag of node, multi-selection, and boundary MUST use pointer capture, live preview, cancel, and single history commit on finish.
- **FR-035**: Continuous move MUST coalesce to one undo entry.
- **FR-036**: Optional grid snap, object snap, alignment guides, align/distribute commands, temporary snap modifier, and persisted snap preference MUST be supported.
- **FR-037**: Guides disappear when interaction ends.
- **FR-038**: Bring forward/to front/send backward/to back MUST keep serialization, hit testing, and display order consistent.
- **FR-039**: Layering policy MUST define boundaries behind nodes, relationships, labels, and temporary overlays.

#### Boundaries (P2 core behaviors still specified)

- **FR-040**: Boundaries represent system/domain/context/team/deployment/region/custom grouping; environment is primarily a filter dimension, not only a box.
- **FR-041**: Boundaries support create-by-drag, title, type, resize, select, move with/without content, membership add/remove, collapse/expand, reorder, context menu, detail navigation, external relationship preservation.
- **FR-042**: Membership MUST be deterministic (explicit membership model), not solely accidental geometric intersection.

#### Text editing

- **FR-043**: Inline edit for node name, relationship label, boundary title, text note.
- **FR-044**: Enter confirms when appropriate; Escape cancels; IME composition safe; global shortcuts suppressed while editing; paste sanitized; size follows content within limits; text undo integrates with document history predictably.

#### Clipboard

- **FR-045**: Copy, cut, paste, duplicate; paste to viewport center or near cursor when applicable; internal relationships preserved; new IDs; drop invalid external refs; successive paste offset; robust internal clipboard; safe system clipboard fallback.
- **FR-046**: Copy of mixed selection duplicates only relationships whose both ends are in the selection (unless a later explicit product rule changes this—default is internal-only).

#### History

- **FR-047**: Transactional undo/redo for create/delete/move/resize/content/relationship/boundary membership/order/layout/paste/property changes.
- **FR-048**: MUST NOT push individual history entries for hover, pointer previews, continuous pan/zoom frames, or every drag frame.
- **FR-049**: Redo invalidates on new edit; memory bound; no accidental mutable shared refs in stacks; coherent selection restore after undo; extensive unit tests.

#### Commands & shortcuts

- **FR-050**: Central command layer; shortcuts and command palette invoke the same commands.
- **FR-051**: Minimum shortcuts: V select, H hand, C connect, N add node, B boundary, T note, Space temporary hand, Escape cancel, mod+Z undo, mod+shift+Z redo, mod+C/X/V/D, mod+A, Backspace/Delete, mod+K palette, mod+F search, Enter open/edit, shift+1 fit doc, shift+2 fit selection, +/- zoom, 0 reset zoom—adapted if they conflict with existing product shortcuts (document resolution in plan).
- **FR-052**: Searchable shortcuts reference UI.

#### Context menus

- **FR-053**: Empty, node, relationship, and multi-selection menus with the operations listed in the product brief (add, paste, fit, grid, layout, open details, rename, duplicate, connect, z-order, align, distribute, create boundary, etc.), using project menu primitives.

#### Search

- **FR-054**: Search by name, type, provider, environment, status, tag, owner, external id; non-blocking; keyboard navigate; center+select; readable zoom; cycle results; disambiguate duplicate names.

#### Toolbar

- **FR-055**: Primary toolbar: Select, Hand, Connect, Add, Boundary, Undo, Redo, Layout, Search—compact, tooltips with shortcuts, existing components, not Excalidraw-like colorful chrome.

#### Environments

- **FR-056**: Environment selector integrates All / Local / Development / Preview / Staging / Production / custom.
- **FR-057**: v1 MUST support filter and/or de-emphasize out-of-env elements; model MUST NOT block future compare/stale/multi-env presence indicators; viewport-per-environment MAY be included if low-cost.

#### Persistence

- **FR-058**: Persist document nodes, relationships, boundaries, order, labels, dimensions, positions, canvas preferences, viewport, format version, timestamps/migration metadata as applicable.
- **FR-059**: Autosave with debounce; flush on close/document switch; states dirty/saving/saved/error; atomic or equivalent safe write; keep last valid snapshot; validate before apply; no secrets in document; renderer without direct FS; use existing core layout API (revision) rather than inventing a second persistence authority.
- **FR-060**: Format versioning with explicit migrations between known versions.

#### Import / export

- **FR-061**: Canonical versioned JSON import/export with schema validation, migrate known versions, sanitize text, no code execution, recoverable errors, preview before replace, backup current before replace.
- **FR-062**: Export full or selection; PNG required; SVG if rendering strategy yields correct output; predictable filenames; optional transparent/canvas background; optional labels/boundaries; no temporary selection overlays.

#### Detail page

- **FR-063**: Detail opens from double-click, Enter, context menu, contextual toolbar, search, deep link; back restores document, viewport, zoom, selection, filters, environment.
- **FR-064**: Detail MAY show overview, connected architecture, relationships, activity, evidence, notes, docs, safe quick actions already in product—not a permanent inspector replacement.

#### UI states

- **FR-065**: Define empty, first-use, loading, saving, save error, invalid document, migration, import, no search results, incompatible selection, layout running, very large canvas, external resource unavailable, stale/unlinked element presentations.
- **FR-066**: Empty state offers create first node, import, template if available, essential shortcuts.

#### Accessibility

- **FR-067**: Keyboard-reachable toolbar; accessible menus/dialogs; visible focus; labels; tooltips not sole info; shortcuts off during text edit; reduced motion; adequate contrast; command palette alternatives for pointer-heavy ops; scalable a11y strategy (e.g., structured list alternative) rather than exposing every canvas object permanently to AT.

#### Architecture separation

- **FR-068**: Generic engine MUST NOT import Architecture domain types as hardwired-only elements; domain registers types/rules/renderers.
- **FR-069**: Architecture domain MUST NOT import Electron.
- **FR-070**: App integration owns routing, persistence adapters, workspace/system context, global palette, notifications.
- **FR-071**: Visual mutations go through document commands/transactions—not ad-hoc DOM state as source of truth.

### Non-Functional Requirements

- **NFR-001**: Pan, zoom, and drag remain visually fluid on common desktop hardware for the default benchmark document (target 60 FPS class smoothness).
- **NFR-002**: Document with **500 nodes / 1,000 relationships** remains usable for navigate, zoom, and move selection without severe degradation.
- **NFR-003**: Stress path **1,000 nodes / 2,000 relationships** exercised; issues documented with mitigations.
- **NFR-004**: Interaction feedback begins within **100 ms** of pointer/keyboard intent for primary tools.
- **NFR-005**: Search and autosave MUST NOT block pointer interaction.
- **NFR-006**: Heavy auto-layout MUST run off the critical render path when expensive.
- **NFR-007**: Load large documents progressively or without multi-second UI freeze (measurable budget set in plan, user-visible “usable canvas” quickly).
- **NFR-008**: No full-scene React/Solid tree rebuild on every pointer move; viewport culling / selective update strategies applied when safe.
- **NFR-009**: Security: preserve context isolation, no nodeIntegration in renderer, no remote module, typed IPC only if added, validate payloads, treat imports as untrusted data, no secrets in documents.
- **NFR-010**: Typecheck, lint, tests, and build for touched packages pass before feature completion claims; reproducible benchmark recorded.
- **NFR-011**: Works with mouse and trackpad on macOS; compatible with Windows and Linux desktop targets.
- **NFR-012**: Visual design matches `DESIGN.md` (dark-first, quiet, dense, brand green sparse, no clone of Excalidraw UI).

### Key Entities

- **Architecture Document**: Persisted artifact—id, format version, workspace/system refs, active environment, nodes, relationships, boundaries, visual order, canvas preferences, saved viewport, created/updated timestamps, migration metadata.
- **Node**: Semantic architecture element—stable id, name, optional short description, type, extensible subtype, origin (manual/discovered), optional provider, environment, state, position, size, z-order, optional parent boundary, collapsed flag, link status, resource reference, optional owner, tags, links, non-secret metadata.
- **Relationship**: Directed semantic edge—stable id, source/target, source/target ports, type, optional label, direction, optional protocol, state, semantic style hints, metadata, z-order.
- **Boundary**: Semantic/visual grouping—system/domain/context/team/deployment/region/custom; title; membership; collapse; geometry; z-order behind nodes.
- **Viewport**: Separate from domain semantics—offset, zoom, visible size, focused element, last significant position; restorable.
- **Interaction State**: Ephemeral—active tool, selection, hover, creating element, connecting, marquee, drag/resize, context menu, guides, snap preview, clipboard op, text composition.
- **Port**: Stable connection endpoint on a node with direction and rules.
- **Command**: Named operation invokable by UI, shortcut, or palette.
- **History Transaction**: Coalesced undoable unit applying/inverting document (+ optional selection) changes.
- **Layout Revision**: Core concurrency token for `body_json` updates (existing).
- **System / Environment / Resource / Evidence**: Existing product entities the canvas links to; evidence still distinguishes claimed vs observed wiring.

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: Strengthen architecture understanding and resource organization; entry point to observe/create/link—not a static diagram toy.
- **Runtime owner (III)**: Renderer owns presentation/interaction; Main stays thin; Preload typed bridge only; polterware-core owns durable layout storage authority via existing service.
- **Domain vs adapter (II, IV)**: Engine generic; architecture semantics in domain/feature layer; adapters for Connect/persistence; no domain→Electron imports.
- **Contracts (V)**: `ArchitectureService` layout body remains versioned JSON; wire fields stable; migrations inside body; document any breaking body version carefully.
- **Product loop impact**: System Map (Architecture canvas) primary; Evidence-aware edges; detail pages; not Operations apply path from canvas.
- **Evidence expectations**: Visual relationships MUST NOT claim live wiring without link/evidence state cues already in product language.
- **Mutation safety (VII)**: Canvas document edits are **local model mutations** (layout/authoring), not infrastructure apply. No plan→apply ops from this feature.
- **Operation audit fields**: N/A for pure layout edits unless product later records layout audit events (optional enhancement, not required for v1 completeness).
- **Security boundary (VI)**: Unprivileged renderer; secrets never in document; imports untrusted; MCP cannot gain privilege via canvas.
- **Local vs cloud authority (VIII)**: Local core layout store is authority for this feature; offline/error states honest.
- **Performance (IX)**: No Main blocking; virtualization/culling/workers as needed; no UI freeze on autosave.
- **Resilience (X)**: Autosave retry/error; revision conflicts; recovery of last good snapshot; invalid import isolation.
- **Observability (XI)**: Correlation for save failures where core already supports; client logs sufficient for layout conflicts.
- **Test plan (XII)**: Unit (engine math, history, clipboard, hit test, migrate), integration (feature flows), e2e Electron happy path, performance benchmark, security review of import path.
- **Structure created now (XIII)**: Prefer expanding `@polterware/frontend-canvas` + architecture feature; new packages only with real public API + consumers.
- **Single source of truth (XIV)**: One engine implementation; temporary dual path only during migration from static canvas, removed before done.
- **Big-bang / refactor context (XV)**: Builds on modular monorepo / operator surfaces; may ship on current refactor branch family without resurrecting forbidden React paths.
- **Non-goals (I, XIII)**: No specialized-platform clone; no scenic empty packages; no Excalidraw embed as product editor.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can create two nodes, connect them with a typed relationship, undo the connection once, and redo it—without leaving Architecture—in under **2 minutes** on first attempt with in-app cues.
- **SC-002**: On a **500-node / 1,000-edge** sample document, pan/zoom/selection-drag remain responsive enough for continuous work (no multi-second freezes; primary interactions feel immediate, ≤100 ms to first feedback).
- **SC-003**: After autosave success, quit and relaunch restores **100%** of nodes, relationships, boundaries, positions, and last viewport for that system layout.
- **SC-004**: Returning from node detail restores prior viewport and selection in **one** back navigation.
- **SC-005**: Copy/paste of a subgraph preserves internal relationship structure with **new** ids in **100%** of automated clipboard tests.
- **SC-006**: Invalid import attempts leave the current document unchanged in **100%** of negative tests.
- **SC-007**: No permanent inspector column on Architecture canvas; detail is a dedicated page (design QA checklist).
- **SC-008**: Visual QA against `DESIGN.md`: dark quiet chrome, sparse brand accent, non-Excalidraw appearance (review sign-off).
- **SC-009**: Benchmark suite exists and is recorded for default and stress documents before calling the feature complete.
- **SC-010**: Typecheck/lint/tests/build for the changed surface pass in CI or local verify scripts used by the repo.

---

## Main Flows

1. **Explore**: open Architecture → load layout → pan/zoom/minimap/search → optional open detail → back with restore.
2. **Author node**: choose create entry → place → set type → name → autosave.
3. **Connect**: Connect tool or port drag → preview → validate → commit type/label → autosave.
4. **Multi-edit**: marquee → transform/duplicate/delete/clipboard → single history entries → autosave.
5. **Persist/recover**: debounce save → revision increment → conflict/error handling → reload.
6. **Organize (P2)**: boundary from selection → membership edits → collapse → layout → export.

---

## Acceptance Scenarios (mandatory set)

### Creation and connection

**Given** an empty document, **When** the operator creates an API and a Database and connects API→Database with `Reads from`, **Then** both nodes and the relationship appear, the change persists, one undo removes the relationship, and redo restores it.

### Multi-selection

**Given** several nodes, **When** the operator marquee-selects and drags, **Then** relative positions hold, relationships follow, and history records one transaction.

### Clipboard

**Given** a selection of three nodes and two internal relationships, **When** copy/paste runs, **Then** new ids are generated, copied relationships point to new nodes, and external relationships are not duplicated.

### Persistence

**Given** layout edits and completed autosave, **When** the app is closed and reopened, **Then** elements, positions, relationships, boundaries, and viewport restore.

### Detail page

**Given** a node selected at a zoom/position, **When** detail opens and the operator returns, **Then** document, viewport, environment, and selection restore.

### Large document

**Given** 500 nodes and 1,000 relationships, **When** navigating, zooming, and moving a selection, **Then** the UI stays responsive and does not needlessly re-render every component.

### Invalid import

**Given** a malformed or unknown-version file, **When** import is attempted, **Then** the current document is not replaced, a recoverable error is shown, and no content executes.

---

## Risks

| Risk | Mitigation |
| --- | --- |
| Underestimating interaction engine scope | Phase P1 engine+domain vertical slices; gate P2; avoid visual mock-only milestones. |
| Solid reactivity misuse causes jank | Imperative viewport layer + selective subscriptions; benchmarks early. |
| HTML nodes + edge rendering mismatch | Explicit hybrid strategy decision with prototype benchmark in plan. |
| Document format thrash | Version field + migrations from day one; fixtures for v1. |
| Shortcut conflicts with shell/palette | Inventory existing shortcuts; resolve in plan; single command registry. |
| Scope creep toward whiteboard | Non-goals enforced in review; DESIGN checklist. |
| Accidental React/Excalidraw dependency | Constitution + package dependency rules; review. |
| Save conflicts / data loss | Revision checks, last-good retention, clear error UX. |
| License issues copying Excalidraw | Conceptual reimplementation; no blind paste; MIT notice if substantial code used. |

---

## Dependencies

- Existing Architecture feature, shell routing, design system, commander palette host.
- polterware-core `ArchitectureService` layout persistence and revision semantics.
- `DESIGN.md` tokens and operator shell anatomy (`003` / `004` surfaces).
- Local Excalidraw clone as **read-only engineering reference** (not a runtime dependency).
- Optional pure libraries only after plan leverage analysis.

---

## Migration Strategy (current canvas → engine)

1. **Introduce versioned document schema** (`formatVersion: 1`) mapping today’s `{ nodes, edges, revision }` into nodes/relationships with defaults (types/states unknown/planned, edges → typed relationships with legacy kind mapping).
2. **Keep wire API**: continue storing JSON in `body_json`; bump revision on save as today.
3. **Compatibility loaders**: `deserialize` migrates legacy bodies; never drop unknown future versions silently—error recoverably.
4. **Replace static band renderer** behind Architecture Canvas tab with engine host; preserve tabs, detail page, create dialog entry points initially; fold create into canvas tools without removing dialog until parity.
5. **Feature-flag or dual-path only temporarily** on the implementation branch; remove static canvas path before completion (XIV).
6. **Fixtures**: update architecture fixtures to versioned documents; keep FixtureBanner honesty when not core-backed.
7. **Viewport**: add fields; default fit or center if absent on first migration.
8. **History**: empty on load; do not deserialize undo stacks.
9. **Workers**: replace trivial layout stub when P2 auto-layout lands.
10. **Documentation**: architecture note for engine/domain/app boundaries and format changelog.

---

## Global Acceptance Criteria

Feature is complete only when:

1. Operators can author architecture without mandatory external forms for core create/connect.
2. Pan, zoom, selection, and drag are fluid.
3. Nodes and relationships carry domain meaning.
4. Relevant destructive ops are undoable.
5. Copy/paste/duplicate preserve internal structure.
6. Document survives app restart via autosave path.
7. Viewport restores after detail navigation.
8. No permanent side inspector.
9. UI follows `DESIGN.md`.
10. Existing components reused when suitable.
11. Renderer stays isolated.
12. Minimum benchmark met and recorded.
13. Typecheck, lint, tests, build pass for the surface.
14. No regressions on other primary pages.
15. Architectural documentation exists.
16. Persisted format is versioned with migration.
17. Canvas does not look like an Excalidraw clone.
18. Engine is not coupled to Architecture domain.
19. Architecture domain is not coupled to Electron.
20. Mouse + trackpad work on macOS; Windows/Linux compatibility maintained.

---

## Out of Scope

- Multiplayer, comments, chat, video, remote cursors
- Freehand drawing and generic whiteboard assets
- Infra apply/mutations, terminal, full observability suites from canvas
- Secret management
- Agents auto-editing resources
- Permanent inspector
- Mandatory `.excalidraw` import
- Cloud multi-tenant collaboration product

---

## Assumptions

- Product repo is **polterware**; bridge is **`window.polterware`**.
- Layout durability continues via **Connect `UpdateLayout`**, not a new ad-hoc Electron file DB.
- Single-operator local-first is the primary mode for this feature.
- HTML-rich nodes are preferred for domain chrome; edges may use SVG/Canvas hybrid decided in plan.
- P1 delivers a usable editor (explore, create, connect, select/edit, persist, detail round-trip); P2 boundaries/layout/export may ship in the same initiative if sequenced, but P1 must not be blocked on perfect auto-layout/export.
- Environment v1 is filter/de-emphasize first.
- Templates in empty state may be minimal (even a single starter) without a large template marketplace.
- “Lock” on nodes is optional; include in model if low-cost, else defer with explicit note in plan.
- Collaboration-ready means immutable ids, ordered ops/history deltas, and pure document transforms—not live sync protocols.

---

## Blocking Questions

None that block writing this specification. Defaults above are sufficient for planning.

Non-blocking decisions deferred to `/speckit-plan` (not [NEEDS CLARIFICATION]):

- Exact package split and file tree
- SVG vs Canvas2D vs hybrid edge rendering after benchmark
- Whether viewport lives inside `body_json` or a sibling preference key (single authority still required)
- Shortcut conflict resolution table vs shell
- Soft-delete vs hard-delete+inverse for history implementation

---

## Handoff for `/speckit-plan`

Plan should:

1. Deepen technical design: document schema JSON, migration functions, engine module map, domain registry API, Solid host composition.
2. Choose rendering strategy with a short benchmark spike plan (HTML nodes + edges).
3. Define command registry and shortcut conflict matrix.
4. Specify autosave debounce, conflict UX, and revision handling sequences.
5. Sequence delivery: P1 vertical slices → P2 boundaries/layout/export → polish/a11y/benchmark.
6. Map tasks to packages (`frontend-canvas`, `feature-architecture`, `frontend-data`, desktop App wiring only as composition).
7. List tests and benchmark harness explicitly.
8. Confirm no `@excalidraw/excalidraw` runtime dependency unless a later plan **proves** superiority for domain, visual system, and performance (current decision: no).
9. Produce `plan.md` / research / data-model / contracts / quickstart per Spec Kit plan template.

**Do not implement during specify.** Implementation starts only after plan (and tasks) per project process.
