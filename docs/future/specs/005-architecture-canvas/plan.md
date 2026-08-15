# Implementation Plan: Architecture Canvas Engine

**Branch**: `005-architecture-canvas` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-03 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/005-architecture-canvas/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: Operator shell + Architecture surface foundation (`003`, `004`); polterware-core `ArchitectureService` layout RPC; `@polterware/frontend-canvas` package exists

**Engineering reference**: Local Excalidraw clone at `/Users/erickpatrickbarcelos/codes/excalidraw` (patterns only; **not** a runtime dependency)

---

## Summary

Replace the static Architecture map with a **complete, domain-specialized architecture canvas** whose interaction quality matches mature editors (pan/zoom, tools, selection, connect, history, clipboard, autosave, performance), without becoming an Excalidraw/whiteboard clone.

**Technical approach** (from research):

1. Expand **`@polterware/frontend-canvas`** into a layered, browser-safe **engine** (viewport, pointer tools, selection, hit-test, history with capture modes, clipboard helpers, commands, snap, spatial index, worker).
2. Keep **architecture semantics and DESIGN chrome** in **`@polterware/feature-architecture`** via a domain registry + Solid host (HTML nodes, SVG edges, toolbar, minimap, autosave controller).
3. Persist **versioned ArchitectureDocument v1** inside existing **`Layout.body_json`** with migrate-from-legacy; concurrency stays on RPC **revision**.
4. **Hybrid render**: HTML/Solid nodes + SVG relationships + CSS/world transform; culling + LOD.
5. **No** `@excalidraw/excalidraw`, no React, no second persistence authority, no permanent inspector.

Delivery is **vertical P1 slices** then P2 (boundaries, auto-layout, export). Tasks are produced by `/speckit-tasks` (not this command).

---

## Technical Context

**Language/Version**: TypeScript (strict) for engine + SolidJS features; Go unchanged for layout domain concurrency; CSS via Tailwind v4 + DESIGN tokens

**Primary Dependencies**:

- Existing: Electron (Desktop composition only), SolidJS, Tailwind v4, `@polterware/frontend-canvas`, `@polterware/frontend-data` (Connect), `@polterware/frontend-design-system`, @hugeicons/core-free-icons, Vitest
- Allowed if needed: `@kobalte/core` for accessible context menus/dropdowns; small pure spatial library only if in-house AABB fails benchmarks
- **Forbidden**: React, `@excalidraw/*` runtime packages, unrestricted Node in renderer

**Storage**: polterware-core SQLite `architecture_layouts` via Connect `GetLayout` / `UpdateLayout` (opaque `body_json` + `revision`)

**Testing**: Vitest unit (engine pure modules + domain rules); feature integration with mock save port; existing desktop e2e harness when applicable; performance harness for 500/1000 nodes; design:lint / typecheck on touched packages

**Target Platform**: Polterware Desktop (macOS primary; Windows/Linux compatible pointer/trackpad behavior)

**Project Type**: Desktop monorepo feature — shared frontend engine library + architecture feature + thin App wiring

**Performance Goals**:

- Fluid pan/zoom/drag (≈60 FPS class on common desktop hardware)
- Usable at **500 nodes / 1000 relationships**
- Stress **1000 / 2000** documented
- First interaction feedback ≤ **100 ms**
- Autosave and search non-blocking

**Constraints**:

- Constitution v2; renderer only `window.polterware`; browser-safe shared packages
- Page-based node detail; no permanent inspector
- Evidence-aware edges; no secrets in document
- DESIGN.md quiet dark operator visual language
- Single engine implementation (no permanent dual static/engine canvas)

**Scale/Scope**: One Architecture canvas surface; engine reusable later; P1 editor complete + P2 organization/export sequenced

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture Product** | PASS | Semantic nodes/relationships, evidence cues, detail pages — not static diagram-only |
| **II. Domain Before Framework** | PASS | Domain registry + pure rules outside Electron; engine free of Connect |
| **III. Strict Runtime Separation** | PASS | Renderer interaction; core storage; Main/preload unchanged for persistence |
| **IV. Modular Monorepo** | PASS | apps → feature-architecture → frontend-canvas → contracts; no cycles |
| **V. Contracts-First** | PASS | Layout RPC stable; body JSON versioned contract documented |
| **VI. Security By Default** | PASS | Untrusted import; no secrets; no Node in packages; typed bridge only |
| **VII. Safe And Auditable Execution** | PASS | Layout authoring only; no infra apply from canvas |
| **VIII. Local-First** | PASS | Core local layout authority; honest offline/error |
| **IX. Performance** | PASS | Workers for heavy layout; culling; no save per pointermove; no Main heavy work |
| **X. Resilience** | PASS | Autosave conflict/error; last-good retention via non-clobber |
| **XI. Observability** | PASS | Save errors surfaced; optional correlation if core provides |
| **XII. Testability** | PASS | Pure engine tests + integration + benchmark plan |
| **XIII. Real Code** | PASS | Expand existing canvas package; no scenic empty packages |
| **XIV. Single Source Of Truth** | PASS | Temporary dual path only during swap; remove static canvas before done |
| **XV. Decisive Refactors** | PASS | Authorized replacement of static map on feature branch family |
| **XVI. Documentation** | PASS | plan/research/data-model/contracts; architecture note in delivery |

**Hard stops**: No security or dependency-direction violations. No permanent dual canvas.

**Complexity Tracking**: None required (no justified constitution violations).

### Post-design re-check

Design artifacts (research, data-model, contracts) preserve gates: hybrid render stays in renderer; persistence via existing port; engine/domain split documented. **PASS** to proceed to `/speckit-tasks`.

---

## Project Structure

### Documentation (this feature)

```text
specs/005-architecture-canvas/
├── plan.md              # This file
├── research.md          # Phase 0
├── data-model.md        # Phase 1
├── quickstart.md        # Phase 1
├── contracts/           # Phase 1
│   ├── overview.md
│   ├── architecture-document.md
│   ├── canvas-engine-api.md
│   ├── architecture-domain-ui.md
│   ├── persistence-autosave.md
│   └── commands-shortcuts.md
├── checklists/requirements.md
├── spec.md
└── tasks.md             # Phase 2 — /speckit-tasks (not created here)
```

### Source Code (paths this feature will modify)

```text
packages/frontend/canvas/
├── package.json
├── src/
│   ├── index.ts                 # public exports
│   ├── document/                # types, create, validate, migrate, serialize
│   ├── viewport/
│   ├── selection/
│   ├── hit-test/
│   ├── history/
│   ├── clipboard/
│   ├── commands/
│   ├── pointer/
│   ├── snap/
│   ├── spatial/
│   ├── render/                  # edge geometry helpers, scene queries
│   └── solid/                   # optional thin hosts
├── worker/
│   └── layout.ts                # expand for P2 auto-layout
└── src/**/*.test.ts

packages/features/architecture/
├── src/
│   ├── domain/                  # types registry, connection rules
│   ├── document/                # architecture defaults, legacy map
│   ├── canvas/                  # ArchitectureCanvasHost, toolbar, minimap
│   ├── nodes/                   # HTML node components + LOD
│   ├── edges/                   # relationship SVG layer
│   ├── persistence/             # autosave controller
│   ├── search/
│   ├── architecture-canvas.tsx  # replace or re-export host
│   ├── index.tsx                # wire host + tabs + detail
│   ├── node-detail.tsx
│   └── …tables/fixtures
└── src/**/*.test.ts

packages/frontend/data/src/index.ts   # reuse getLayout/updateLayout (no API break)

apps/desktop/src/renderer/App.tsx     # save/load port wiring, flush on route, palette commands

docs/ (or packages/frontend/canvas/README.md)
└── canvas-engine.md                  # architecture note engine vs domain

schemas/polterware/local/v1/architecture.proto  # unchanged for P1
backend/domains/architecture/                   # unchanged concurrency rules
```

**Structure Decision**:

- **Runtime owner**: Renderer for engine/UI; polterware-core for durable layout; Main/preload only if future native file dialogs (not P1).
- **Domain vs adapter**: Domain rules in feature-architecture pure modules; Connect adapter in frontend-data + App; engine is framework-light shared frontend.
- **Contracts touched**: JSON body schema v1 (documented); Connect Layout wire **unchanged**.
- **New package**: **None** for P1 — expand `@polterware/frontend-canvas`.

---

## Phase 0: Research

Completed in [research.md](./research.md). Key decisions:

| Topic | Decision |
| --- | --- |
| Engine | Own engine; no Excalidraw embed |
| Packages | Expand frontend-canvas + feature-architecture |
| Render | HTML nodes + SVG edges + world transform |
| History | Patches + immediately/eventually/never; hard delete + inverse |
| Viewport | Inside body_json + session cache |
| Autosave | 800ms doc / 2000ms viewport; RPC revision |
| Shortcuts | Canvas-focused; ⌘K stays App |
| Sequencing | P1a–e then P2a–c |

---

## Phase 1: Design & Contracts

| Artifact | Path |
| --- | --- |
| Data model | [data-model.md](./data-model.md) |
| Contracts | [contracts/](./contracts/) |
| Quickstart | [quickstart.md](./quickstart.md) |

---

## Implementation sequencing (input to `/speckit-tasks`)

### P1a — Document + viewport shell

- ArchitectureDocument v1 types, serialize, migrate legacy, validate
- Load via existing getLayout; empty/fixture honesty preserved
- Canvas world: pan, zoom-about-cursor, fit document, zoom indicator, dotted background
- Session viewport cache for detail round-trip skeleton

### P1b — Selection + drag + history

- Click, shift, marquee (contain/intersect), select all
- Multi-drag with coalesced history
- Hit testing priorities + spatial index baseline

### P1c — Create node + tools + commands

- Tools Select/Hand/Add; Space temporary hand; Escape layers
- Create via tool, palette, context menu, double-click empty; type required; rename
- Command registry + shortcut binder
- Toolbar chrome (DESIGN)

### P1d — Connect

- Ports, connect tool, preview, validation, labels, reconnect, undo
- SVG edges evidence-aware styling
- Relationship types registry

### P1e — Clipboard, search, autosave, detail

- Copy/cut/paste/duplicate graph-safe
- Search center + cycle
- Autosave status + flush on navigate
- Enter/double-click detail + restore viewport/selection/environment
- Delete selection; keyboard complete set

### P2a — Boundaries

- Create, membership, move with content, collapse, menus

### P2b — Auto-layout

- Worker horizontal/vertical; selection or all; pin respect; one undo

### P2c — Export

- JSON document/selection; PNG; SVG if quality OK

### P3 — Polish

- Shortcuts panel, empty CTAs, a11y structure list, stress bench record, docs

### Migration cleanup

- Remove static band-layout canvas path; single host only (XIV)

---

## Performance plan

1. Early harness generating N nodes / 2N edges in memory.  
2. Measure pan/zoom/drag interaction frames (manual + optional RAF counters in dev).  
3. Culling: skip offscreen node Solid trees when safe.  
4. Edge updates only for incidents of moved nodes during drag.  
5. No full serialize on pointermove; autosave debounced.  
6. Record results in feature notes or `docs` before completion claim.

---

## Security plan

- Import/export: validate + sanitize; no execute  
- No secrets in document fields  
- Renderer isolation unchanged  
- Payload size guard  
- Clipboard: internal format first; system text sanitized  

---

## Testing plan (proportional)

| Risk | Tests |
| --- | --- |
| Viewport math / zoom focal | unit |
| Marquee / selection / history coalesce | unit |
| Clipboard id rewrite | unit |
| Migrate 0→1 | unit |
| Connect validation | unit |
| Autosave debounce + conflict | unit with fake timers + mock port |
| Create-connect-undo flow | integration |
| Detail restore | integration |
| Large doc culling smoke | performance |
| Import invalid | unit + integration |
| E2E happy path | when desktop e2e available |

---

## Agent context

No `.specify` agent-context update script is present in this repo (`setup-plan.sh` only copies the plan template). Context for agents remains: root `AGENTS.md`, this feature dir, and `feature.json` → `specs/005-architecture-canvas`.

---

## Next command

```text
/speckit-tasks
```

Produces `tasks.md` from this plan (dependency-ordered implementation tasks). Do **not** start broad implementation until tasks exist unless the user explicitly orders a spike.
