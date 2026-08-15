# Research: Architecture Canvas Engine

**Feature**: `005-architecture-canvas`  
**Date**: 2026-08-03  
**Status**: Complete — planning unknowns from spec handoff resolved  
**Inputs**: [spec.md](./spec.md), local Excalidraw clone (`/Users/erickpatrickbarcelos/codes/excalidraw`), Polterware monorepo audit

---

## R1. Engine ownership vs embedding Excalidraw

**Decision**: Build a **Polter-owned canvas engine** in `@polterware/frontend-canvas` (expanded). Do **not** add `@excalidraw/excalidraw` (or React) as a product runtime dependency. Treat Excalidraw source as interaction/state **reference only**.

**Rationale**: Spec Reference Audit; Constitution (no React product UI); domain nodes are semantic architecture objects, not freehand geometry; DESIGN.md forbids whiteboard aesthetic; HTML-rich nodes + evidence-aware edges need first-class control.

**Alternatives considered**:

| Option | Why rejected |
| --- | --- |
| Embed `@excalidraw/excalidraw` | React stack, whiteboard model, visual clone risk, hard domain binding |
| Fork Excalidraw packages into monorepo | Massive React surface, wrong element model, maintenance burden |
| xyflow / React Flow style libs | Typically React-bound; still need full history/clipboard/viewport depth |
| Canvas-only pure draw (no HTML nodes) | Loses design-system node chrome and a11y of controls |

---

## R2. Package split and module map

**Decision**: **Expand** existing `@polterware/frontend-canvas` into a layered public API; keep domain in `@polterware/feature-architecture`. No new package unless the engine public surface and architecture feature both need a third shared pure-domain package later (Y3 — do not create now).

```text
@polterware/frontend-canvas          # generic engine (browser-safe, Solid-optional host)
  src/
    document/     # Document, Element base, serialize, migrate (generic)
    viewport/     # matrix, zoom-about-point, fit, restore
    selection/    # selection set, marquee modes
    hit-test/     # priority, spatial index
    history/      # transactions, capture modes, stacks
    clipboard/    # graph-aware copy/paste helpers
    commands/     # Command registry + keyboard binder (framework-agnostic)
    pointer/      # tool machine, pointer capture lifecycle
    snap/         # grid + guides
    spatial/      # R-tree or simple grid index
    render/       # host-agnostic scene descriptors + edge path helpers
    worker/       # layout worker (existing stub expanded)
    solid/        # thin Solid host: CanvasViewport, OverlayLayer (optional entry)

@polterware/feature-architecture     # domain + product UI
  src/
    domain/       # node types, relationship types, port rules, validation
    document/     # ArchitectureDocument mapping, legacy migrate
    canvas/       # ArchitectureCanvasHost, tools chrome, minimap, toolbar
    nodes/        # HTML node components by type/LOD
    edges/        # relationship visual + connect preview
    persistence/  # autosave controller (uses injected save port)
    search/       # document search index helpers
    …existing tabs, detail, dialogs
```

**Rationale**: Package already exists with consumers; Constitution XIII forbids scenic empty packages; engine must stay free of Electron and Architecture-only hardcodes via a **domain registry** interface.

**Alternatives considered**: Split `frontend-canvas-engine` + `frontend-canvas-solid` immediately (extra package without second consumer); put engine inside feature-architecture (blocks reuse and couples domain).

---

## R3. Document vs interaction state

**Decision**: Three layers, Excalidraw-inspired:

| Layer | Persist? | Contents |
| --- | --- | --- |
| **ArchitectureDocument** | Yes (`body_json`) | formatVersion, nodes, relationships, boundaries, order, canvas prefs, viewport, meta |
| **Session UI state** | Partially (viewport yes; selection optional in-session only) | selection, environment filter, open tool — selection **not** required in durable body; may keep in route/`selectedNodeId` |
| **Ephemeral interaction** | Never | hover, marquee, drag preview, connect rubber-band, guides, text composition, context menu open |

**Rationale**: Spec entities; Excalidraw `AppState` storage conf pattern; prevents autosave thrash on hover.

**Alternatives considered**: Single mega-model with everything persisted (rejected — noise + conflict risk); Redux-like global app store for canvas (rejected — wrong ownership).

---

## R4. History model (soft-delete vs hard-delete + capture modes)

**Decision**:

1. **Capture modes** (from Excalidraw `CaptureUpdateAction`):
   - `immediately` — commit undo entry now
   - `eventually` — merge into next immediate commit (in-progress drag/resize/text)
   - `never` — remote/load/ephemeral (no history)
2. **Transaction = structural patch** (element id → before/after fields + selection snapshot optional), not full-document clone for every edit.
3. **Delete strategy**: **hard remove from document maps** with inverse patch restoring full records (simpler for architecture graph than soft-delete tombstones). Soft-delete is optional later if collab lands.
4. Continuous drag: many `eventually` mutations → one `immediately` on pointerup.
5. Bound stacks (e.g. 100 transactions or ~memory budget); clear redo on new immediate commit.

**Rationale**: Spec FR-047–049; Excalidraw store/history research; architecture graphs are smaller than freehand scenes but still need coalescing.

**Alternatives considered**: Full snapshot stack only (easy but O(n) memory on 1k nodes); soft-delete everywhere (extra complexity without collab).

---

## R5. Rendering strategy (HTML nodes + edges)

**Decision**: **Hybrid**

| Layer | Technology |
| --- | --- |
| Viewport transform | Single world container with CSS `transform: translate() scale()` **or** equivalent matrix on a world layer; prefer one transform root for pan/zoom without reflowing each node style x/y every frame when possible |
| Nodes | **HTML/Solid** components (design tokens, badges, ports as DOM) positioned in document space |
| Relationships | **SVG** overlay in the same world layer (paths, hit strokes, arrows, labels as SVG text or HTML foreignObject sparingly) |
| Guides / marquee / connect preview | SVG or absolute DOM overlay above nodes |
| Minimap | Lightweight canvas 2D or simplified SVG; not full node DOM |

**Why not full Canvas2D for nodes**: DESIGN.md node chrome, status chips, ports, and text editing are far cheaper as HTML; Canvas text/edit is a product regression.

**Why not SVG-only for everything**: Fine for edges; poor for dense control chrome.

**Culling**: Only mount/show node components whose bounds intersect expanded viewport (buffer). Keep spatial index updated on move. LOD by zoom per FR-023.

**Benchmark spike (implement during tasks, before locking edge algorithm)**:

1. 500 HTML nodes + 1000 SVG edges, pan/zoom 10s — frame time
2. Drag 20-node selection — frame time
3. Compare edge update strategies: full path recalc vs only edges of moved nodes

**Rationale**: Spec FR-031; Solid fine-grained updates; Excalidraw static/interactive split adapted to HTML host.

**Alternatives considered**: Full Canvas2D (bad for node chrome); full SVG nodes (layout/CSS pain); WebGL (overkill for 1k boxes).

---

## R6. Viewport persistence location

**Decision**: Store **viewport inside the versioned ArchitectureDocument** (`body_json`):

```json
"viewport": { "x": 0, "y": 0, "zoom": 1 }
```

Also keep an **in-memory session cache** keyed by `systemId` so detail round-trip restores even if autosave has not flushed yet. On successful autosave, document viewport is SSOT for restart.

**Rationale**: Single authority with layout revision (no second preference store for critical restore); FR-008; detail return SC-004.

**Alternatives considered**: Separate localStorage only (splits authority, races with multi-system); sibling core field (proto churn without need).

---

## R7. Shortcut conflict matrix

**Decision**: Canvas shortcuts apply only when canvas surface is focused and **not** typing in an input/contenteditable. Shell keeps global:

| Shortcut | Owner | Canvas behavior |
| --- | --- | --- |
| `⌘/Ctrl+K` | App command palette | Palette wins globally; canvas does not reclaim |
| `⌘/Ctrl+Shift+K` | Electron global commander overlay | Unchanged |
| `⌘1..n` (palette labels) | Navigation (palette) | Not canvas tools |
| `V H C N B T` | Canvas tools | When canvas focused |
| `Space` | Temporary hand | Canvas focused; do not scroll page |
| `⌘/Ctrl+Z` / `⇧⌘Z` | Canvas history when canvas focused; else no-op or future app history | Prefer canvas when Architecture canvas tab active and focused |
| `⌘/Ctrl+F` | Canvas find when canvas focused | Else browser/app default avoided via preventDefault on canvas |
| `⌘/Ctrl+A/C/X/V/D` | Canvas selection/clipboard when canvas focused | |
| `Escape` | Layered cancel (spec FR-011) then clear | |
| `Delete/Backspace` | Delete selection | Disabled during text edit |

Document searchable shortcuts panel lists both shell and canvas groups.

**Rationale**: App.tsx already binds mod+K; Excalidraw-style tool keys; avoid fighting OS text fields.

---

## R8. Spatial index & hit testing

**Decision**: Maintain a simple **uniform grid or RBush-like AABB index** for nodes and boundary boxes; relationships tested by expanded stroke distance only for candidates near pointer (segment AABB first). Priority: port → handle → node → relationship → boundary → empty.

**Rationale**: FR-018–020; 1k nodes need sub-linear hit tests.

**Alternatives considered**: Linear scan only (OK to ~100, fails 500+); full geometry library dependency (evaluate only if grid insufficient — default in-house AABB first).

---

## R9. Autosave / revision / conflict UX

**Decision**:

| Parameter | Value |
| --- | --- |
| Debounce after last document mutation | **800 ms** |
| Viewport-only changes | Debounce **2000 ms** (or fold into next doc save) |
| Flush triggers | debounce fire, tab hide, route leave architecture, app `beforeunload`/Electron window close hook if available, explicit flush API |
| Payload | `serialize(document)` → `updateLayout(systemId, expectedRevision, bodyJson)` |
| Success | update local revision from response; status `saved` |
| Conflict (`revision mismatch`) | status `error`; keep local dirty doc; toast “Layout changed elsewhere — reload or overwrite”; **Overwrite** uses latest expected revision only after user confirms (reload merges positions carefully in tasks) |
| Network/core down | status `error`; retry with backoff; never claim saved |
| Indicator | discrete in page header/toolbar: Saved / Saving… / Unsaved / Error |

Do **not** save every pointermove. History stack is memory-only.

**Rationale**: FR-058–059; existing optimistic concurrency domain; Excalidraw eventual capture analogy for continuous edits.

---

## R10. Clipboard graph rewrite

**Decision**: Internal clipboard MIME/type `application/x-polter-architecture-fragment+json` (plus plain text summary). On paste:

1. Generate new ids for all nodes/boundaries/relationships in fragment  
2. Keep only relationships whose both endpoints are in the fragment  
3. Offset by viewport center or +grid step for successive pastes  
4. Sanitize strings  
5. Single history transaction  

**Rationale**: Spec clipboard scenarios; Excalidraw serializeAsClipboardJSON concept without freehand baggage.

---

## R11. Boundaries membership

**Decision**: Explicit `boundaryId` on nodes (and nested boundaries later if needed). Geometric “contains” assists **propose membership** on create-from-selection or drop, but commit is explicit. Move boundary moves members unless modifier “move frame only”.

**Rationale**: FR-042 deterministic membership.

---

## R12. Environment model (v1)

**Decision**: Document may store `activeEnvironmentId` preference; nodes carry `environmentIds: string[]` or single `environment`. UI filter: **All** shows everything; specific env **de-emphasizes** (opacity) out-of-env nodes/edges rather than hard-hiding by default (hard hide optional toggle later). Compare mode out of scope.

**Rationale**: FR-056–057; progressive enhancement.

---

## R13. Lock / pin

**Decision**: Include optional `locked: boolean` on nodes/boundaries in format v1 (low cost). Locked elements skip drag; still selectable for open detail. Pin-for-layout (`layoutPinned`) separate flag for P2 auto-layout.

**Rationale**: Spec assumption — low cost include now.

---

## R14. Legacy migration

**Decision**: `formatVersion` field:

| Version | Shape |
| --- | --- |
| missing / 0 | Legacy `{ nodes, edges, revision? }` from current serialize |
| 1 | ArchitectureDocument v1 (this feature) |

Migrators pure functions in canvas package + architecture domain defaults (unknown types → Custom Resource / Custom relationship; edges `contains`/`related` → typed map; positions preserved).

Wire `revision` stays on Layout RPC, **not** inside body as concurrency token (body may store `docRevision` mirror optional — prefer RPC only).

**Rationale**: Spec migration strategy; avoid dual concurrency tokens.

---

## R15. Dependencies policy

**Decision**: Default **no new deps**. Allowed later with leverage proof:

- Small pure spatial index if in-house fails benchmarks  
- Kobalte menu/dropdown for context menus (already allowed by AGENTS)  
- Not allowed: React, Excalidraw packages, full graph editor frameworks  

**Rationale**: AGENTS.md / Constitution.

---

## R16. Testing strategy

**Decision**:

| Layer | What |
| --- | --- |
| Unit (Vitest) | viewport math, hit-test, marquee, history coalesce, clipboard rewrite, migrate, command key map, port validation |
| Integration | Architecture host: create/connect/select/drag/autosave mock port |
| E2E (Playwright desktop when suite exists) | mandatory scenarios from spec |
| Performance | `scripts` or package test generating 500/1000 nodes; assert interaction budgets loosely + manual record |
| Visual | optional snapshot of node chrome states only (not full infinite canvas) |

**Rationale**: Constitution XII; Excalidraw test organization inspiration (pure helpers first).

---

## R17. Delivery sequencing (for tasks)

**Decision**: Vertical slices, not horizontal “all models then all UI”:

1. **P1a** Document v1 + migrate + load/save + viewport pan/zoom/fit  
2. **P1b** Select + marquee + drag + history  
3. **P1c** Create node + rename + tools/commands  
4. **P1d** Connect ports + relationship model + undo  
5. **P1e** Clipboard + delete + search center + detail restore + autosave UX  
6. **P2a** Boundaries  
7. **P2b** Auto-layout worker  
8. **P2c** Export PNG/JSON  
9. **P3** Shortcuts panel polish, empty states, a11y list alternative, stress bench  

Each slice keeps DESIGN.md compliance and no permanent inspector.

---

## Resolved unknowns checklist

| Spec handoff item | Resolution |
| --- | --- |
| Package split | Expand `frontend-canvas` + domain in feature-architecture |
| SVG vs Canvas2D | Hybrid HTML nodes + SVG edges |
| Viewport storage | Inside `body_json` + session cache |
| Shortcut conflicts | Matrix R7; palette keeps ⌘K |
| Soft vs hard delete | Hard delete + inverse patches |
| Excalidraw embed | No |

All planning **NEEDS CLARIFICATION** items are resolved; remaining work is implementation detail for `/speckit-tasks`.
