# Implementation Plan: Semantic Architecture Canvas

**Branch**: `006-semantic-architecture-canvas` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-03 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/006-semantic-architecture-canvas/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `005-architecture-canvas` engine + host (implemented); polterware-core `ArchitectureService` layout RPC; DESIGN.md; commander palette

**Engineering reference**: Local Excalidraw at `/Users/erickpatrickbarcelos/codes/excalidraw` — interaction patterns only (menus, transient state, shortcuts). **Not** a runtime dependency.

**Risk control**: This plan **forbids** full canvas engine rewrite. Evolution is domain registry + host UX + small engine extensions only.

---

## 0. Preserve / Modify / Replace (mandatory first)

This matrix is the primary guardrail for implementers and for `/speckit-tasks`.

### Preserve (do not rewrite)

| Area | Location | Why |
| --- | --- | --- |
| Document v1 core | `packages/frontend/canvas/src/document/*` | Stable body_json; migrate only |
| Mutations API | `mutations.ts` (`docAddNode`, `addRelationship`, `update*`, `deleteSelection`, boundaries) | Already history-compatible |
| History capture modes | `history/history.ts` (`immediately` / `eventually` / `never`) | Correct for drag coalesce |
| Viewport math | `viewport/*` | Pan/zoom/fit proven |
| Selection + marquee | `selection/*` | Multi-select works |
| Hit-test core | `hit-test/*` | Extend, do not replace |
| Clipboard fragment | `clipboard/*` | ID remapping base |
| Layout sync/worker API | `layout-algo.ts`, `worker/layout.ts` | Auto-layout path |
| Spatial index / snap | `spatial/*`, `snap/*` | Performance path |
| ArchitectureCanvasHost shell | `architecture-canvas-host.tsx` | Pointer loop, tools, world transform |
| HTML nodes + SVG edges hybrid | host render | Matches DESIGN + performance |
| Autosave controller | `persistence/autosave.ts` | Status machine exists |
| Layout RPC | App `getLayout` / `updateLayout` | Revision concurrency |
| Session snapshot | `session-viewport.ts` | Detail round-trip |
| Soft-fail save | App + ArchitectureView | Canvas stays usable on fetch fail |
| Page-based detail | routes + `node-detail.tsx` | No permanent inspector |
| Security boundary | preload `window.polterware` only | Constitution VI |

### Modify (incremental — primary work)

| Area | Change |
| --- | --- |
| Domain registry | Expand flat lists → full `ComponentTypeDefinition` / `RelationshipTypeDefinition` (icons, categories, synonyms, ports, appearance, suggestion rules) |
| `connection-rules.ts` | Richer suggestions + validity matrix; no silent persist without explicit type |
| Host add-node tool | `+` / `N` / dblclick / context → catalog dialog → placement preview → name confirm → commit |
| Host connect flow | Pending relationship + type popover; quick mode with preselected type |
| Node card chrome | Registry-driven icon/type/state/env/provider/link indicators |
| Edge chrome | Type label at zoom; wider invisible hit path; direction clarity |
| Contextual toolbars | Floating node / relationship / multi toolbars |
| Context menus | Full empty/node/rel/multi menus per spec |
| Header UX | Demote/remove primary Save layout; clarify Environment filter label; persistence status primary |
| Bottom Open | Label **Open details**; keep session snapshot |
| CreateNodeDialog | Replace with shared **Add to architecture** catalog (single source) |
| Type unions | Extend `NodeType` / `RelationshipType` for catalog gaps + migrate unknown → safe fallback |
| Hit-test edges | Enable selectable edges (remove pure `pointer-events: none` dead zone) |
| Escape stack | Layered dismiss: popover → placement → tool → selection |
| Preferences | Optional local recents/favorites (not in architecture document) |
| Align/distribute | Pure geometry helpers in engine or domain util; wired from multi toolbar |
| Search center | Wire existing search helpers to centerOn + select |
| Dependency highlight | Host transient emphasis mode |
| Minimap | Keep; optional boundary dots / drag scrub polish |
| Commander | Register catalog + connect + layout commands |
| Changes tab | Surface local history labels / save events where cheap |

### Replace (only these narrow surfaces)

| Area | Replace with | Not allowed |
| --- | --- | --- |
| Silent generic create (`New component` + hard-coded type) | Typed catalog + placement + name | New whiteboard engine |
| Silent `inferRelationshipType` → immediate persist | Suggest + confirm (or quick-mode preselect) | Modal-center heavy wizard as only path |
| Primary manual **Save layout** CTA | Autosave + status + retry | Second storage backend |
| Parallel CreateNodeDialog form as long-term path | Single catalog UX | Dual permanent create UIs |

### Explicitly out of bounds (reject in review)

- New React canvas / Excalidraw embed
- Rewriting history to OT/CRDT
- New Electron IPC for document edit
- Permanent dual static+engine canvas
- Permanent side inspector
- Freehand / shapes / stickers

**No technical limitation found that requires engine replacement.** Gaps are domain + host interaction state machines.

---

## Summary

Evolve the **existing** Architecture canvas from a generic node editor into a **semantic architecture authoring surface**: typed components, typed relationships, contextual editing, autosave-first, honest lifecycle states—while preserving the `005` engine (document, viewport, history, hit-test, hybrid render).

**Technical approach** (see research):

1. **Registry-first domain** in `feature-architecture/domain` (and thin type extensions in `frontend-canvas` only where document unions need new IDs).
2. **Host interaction modes**: `placement`, `naming-provisional`, `pending-relationship`, `connect-quick` — ephemeral, not persisted until confirm.
3. **UI primitives**: reuse `A11yDialog` + TanStack Virtual for catalog; selective **Kobalte** only if popover/menu a11y needs it; no full solidui-cli init.
4. **Persistence**: keep layout body_json + autosave; demote manual save; soft-fail + retry.
5. **Delivery slices**: P1 (typed create/connect/edit/autosave/history) → P2 (organize/search/highlight) → P3 (states polish / env compare readiness).

Tasks produced by `/speckit-tasks` (not this command).

---

## Technical Context

**Language/Version**: TypeScript (strict) SolidJS renderer; Go polterware-core unchanged for layout concurrency

**Primary Dependencies**:

- Existing: SolidJS, Tailwind v4, `@polterware/frontend-canvas`, `@polterware/frontend-design-system`, `@polterware/frontend-data` (Connect), @hugeicons/core-free-icons, Vitest, `@tanstack/solid-virtual`
- Allowed: `@kobalte/core` selectively for accessible popover/menu/combobox if custom menus fail a11y
- **Forbidden**: React, `@excalidraw/*` runtime, unrestricted Node in shared packages

**Storage**: SQLite `architecture_layouts` via Connect `GetLayout` / `UpdateLayout` (opaque `body_json` + `revision`). Local prefs (recents/favorites) via `localStorage` or in-memory session only.

**Testing**: Vitest — registry search/synonyms, suggestions, provisional cancel, history after confirm, clipboard rewire; host unit where pure; typecheck on touched packages. Full test suites only when requested for verification.

**Target Platform**: Polterware Desktop (Electron + Solid renderer)

**Project Type**: Incremental feature on monorepo desktop product

**Performance Goals**:

- Preserve ≈60 FPS pan/zoom/drag class on common hardware
- Catalog opens < 100 ms perceived; list virtualized if > ~50 types
- Autosave non-blocking; no save on every pointermove (keep eventually coalescing)
- Viewport culling retained for large maps

**Constraints**:

- Constitution v2; no permanent dual create paths after feature done
- DESIGN.md quiet dark operator UI
- No permanent inspector; detail is page-based
- Soft-fail network: never blank canvas on save/fetch error
- Domain rules pure/testable without Electron

**Scale/Scope**: Full semantic authoring on Architecture canvas; provider import stubs ok; multiplayer out of scope

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-checked after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture Product** | PASS | Semantic architecture map, not whiteboard clone |
| **II. Domain Before Framework** | PASS | Registry + connection rules pure; host is adapter UI |
| **III. Strict Runtime Separation** | PASS | Renderer interaction; core layout store; no new Main logic |
| **IV. Modular Monorepo** | PASS | apps → feature-architecture → frontend-canvas → design-system |
| **V. Contracts-First** | PASS | Layout RPC stable; document schema extends with migrate |
| **VI. Security By Default** | PASS | No secrets in document; unprivileged renderer; soft-fail only |
| **VII. Safe And Auditable Execution** | PASS | Layout authoring only; no infra apply from canvas |
| **VIII. Local-First** | PASS | Offline canvas + save error state |
| **IX. Performance** | PASS | Culling, virtual catalog, coalesced history |
| **X. Resilience** | PASS | Autosave retry/conflict; provisional discard safety |
| **XI. Observability** | PASS | Save errors user-visible; existing core logs |
| **XII. Testability** | PASS | Pure domain tests + focused host tests |
| **XIII. Real Code** | PASS | Extend existing packages only |
| **XIV. Single Source Of Truth** | PASS | One catalog create path; one document model |
| **XV. Decisive Refactors** | PASS | Incremental on 005; no second engine |
| **XVI. Documentation** | PASS | plan/research/data-model/contracts/quickstart |

**Hard stops**: None. **Complexity Tracking**: empty (no justified violations).

### Post-design re-check

Design artifacts keep engine/host split, registry purity, layout RPC, soft-fail persistence, and no permanent dual UIs. **PASS** → ready for `/speckit-tasks`.

---

## Project Structure

### Documentation (this feature)

```text
specs/006-semantic-architecture-canvas/
├── plan.md              # This file
├── research.md          # Phase 0
├── data-model.md        # Phase 1
├── quickstart.md        # Phase 1
├── contracts/           # Phase 1
│   ├── overview.md
│   ├── type-registry.md
│   ├── authoring-flows.md
│   ├── ui-chrome.md
│   └── persistence-session.md
├── spec.md
└── tasks.md             # /speckit-tasks (not this command)
```

### Source Code (touch list)

```text
packages/frontend/canvas/
  src/document/types.ts          # extend NodeType / RelationshipType unions if needed
  src/document/migrate.ts        # unknown type fallbacks
  src/document/mutations.ts      # align/distribute helpers; reverse relationship; optional
  src/hit-test/hit-test.ts       # thicker edge hit
  src/index.ts                   # re-exports

packages/features/architecture/
  src/domain/
    registry.ts                  # → full type definitions + search
    connection-rules.ts          # suggestions + canConnect matrix
    component-types.ts           # NEW: catalog data (or expand registry)
    relationship-types.ts        # NEW: catalog data
    suggestions.ts               # NEW: pure suggestion table
  src/canvas/
    architecture-canvas-host.tsx # modes, toolbars, chrome, menus
    add-to-architecture-dialog.tsx  # NEW: searchable catalog
    relationship-type-popover.tsx   # NEW
    contextual-toolbar.tsx          # NEW or inline
    placement-preview.tsx           # NEW or inline
    session-viewport.ts          # keep
    palette-commands.ts          # expand
  src/persistence/autosave.ts    # retry surface if needed
  src/index.tsx                  # header UX, env label, demote Save, wire catalog
  src/create-node-dialog.tsx     # retire or re-export catalog
  src/document/*                 # import/export aware of new types
  src/search/*                   # center-on-result wiring

apps/desktop/src/renderer/App.tsx  # only if save soft-fail / retry wiring needs it

packages/frontend/design-system/   # only if shared Popover/Menu extracted
packages/features/commander/       # register new commands if needed
```

**Structure Decision**:

- **Runtime owner**: Renderer (Solid) for all interaction; polterware-core for layout persistence only.
- **Domain vs adapter**: Pure registry/suggestions under `feature-architecture/domain`; host is UI adapter; canvas package stays geometry/document.
- **Contracts touched**: body_json schema extensions (additive types); no protobuf change required.
- **New packages**: **none**. No scenic scaffolds.

---

## Implementation Phases (for tasks command)

### Slice A — P1 foundation (must ship first)

1. Full type registries + search/synonyms tests  
2. Add to architecture dialog + placement + name confirm/cancel  
3. Relationship pending + popover confirm/cancel + quick connect mode  
4. Remove silent generic create paths  
5. Node/relationship contextual toolbars (core actions)  
6. Autosave-first header; demote Save; retry on error  
7. History covers all confirmed ops; provisional never dirty-save  

### Slice B — P1 polish + P2 organize

8. Semantic card chrome + edge labels/hit  
9. Context menus complete  
10. Align/distribute, fit selection, search center  
11. Boundaries polish, auto layout wiring, dependency highlight  
12. Environment filter labeling  

### Slice C — P3 + IO

13. State indicators honesty (linked/observed/stale/drifted)  
14. Import/export type fidelity  
15. Minimap polish  
16. Changes tab summaries  
17. Impact visualization readiness (command/hook only)  

**Done rule**: Feature not complete after “dialogs only.”

---

## Complexity Tracking

> No constitution violations requiring justification.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
