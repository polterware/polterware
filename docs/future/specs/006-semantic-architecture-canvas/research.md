# Research: Semantic Architecture Canvas

**Feature**: `006-semantic-architecture-canvas`  
**Date**: 2026-08-03  
**Status**: Complete (no unresolved NEEDS CLARIFICATION)

---

## R1 — Engine rewrite vs incremental host/domain evolution

**Decision**: **Incremental evolution only.** Preserve `@polterware/frontend-canvas` document/history/viewport/hit-test and `ArchitectureCanvasHost` pointer loop. Change create/connect semantics, registry, chrome, toolbars.

**Rationale**:

- Spec audit found no hard technical blocker in the engine for typed authoring.
- 005 already delivered hybrid HTML+SVG, history capture modes, autosave, minimap, session snapshot.
- Rewrite risk is high (regress pan/zoom/drag/history) for no proportional gain.

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| Embed Excalidraw | React; whiteboard product DNA; DESIGN conflict; constitution |
| New canvas package v2 dual | Violates XIV single source of truth |
| Rewrite host from scratch | Throws away working interaction stack |

---

## R2 — Where type registries live

**Decision**: **Authoritative semantic registry in `packages/features/architecture/src/domain/`**. Document unions in `frontend-canvas` remain serializable IDs (`NodeType` / `RelationshipType` string unions or string brands). Engine does not import Solid or catalog UI.

**Rationale**: Domain-before-framework; engine stays browser-safe and reusable; architecture feature owns product taxonomy.

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| All types only in engine | Couples generic canvas to architecture product catalog |
| Types only as free strings with no union | Weaker validation/migrate for known set; still allow `custom` |

**Pattern**:

```text
ComponentTypeDefinition { id, label, category, description, icon, defaultSize,
  defaultPorts, searchTerms, synonyms, order, appearance, relationshipHints }
RelationshipTypeDefinition { id, label, description, searchTerms, order }
suggestRelationship(sourceTypeId, targetTypeId) → RelationshipTypeId[]
canConnect(source, target, relType) → { ok, reason? }
```

---

## R3 — Provisional create / pending relationship vs history

**Decision**:

- **Provisional node** and **pending relationship** live in **host ephemeral state** only.
- They are **not** written to `ArchitectureDocument` until confirm (or are written with `mode: "never"` if needed for hit geometry then **discarded** without history push).
- Preferred: keep provisional **outside** document for name phase; on confirm call `docAddNode` once with `immediately`.
- Pending edge: draw ghost SVG from host state; on confirm `addRelationship` once.

**Rationale**: Spec requires no persist before confirm; keeps undo stack clean; avoids “undo empty name” junk.

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| Always add to document then undo on Escape | Pollutes dirty/autosave; race with debounce save |
| Special document flag `provisional: true` persisted | Risks saving incomplete nodes |

---

## R4 — Relationship confirmation UX

**Decision**: **Compact popover near the drop point / edge midpoint**, not a centered modal. Keyboard: list navigation, Enter confirm, Escape cancel. Suggested type focused first.

**Quick mode**: Toolbar shows `Connect: {Type}` dropdown; drops commit immediately with that type; Escape exits connect tool.

**Rationale**: Spec FR-015/018; preserves visual context of the edge; matches Excalidraw-style local menus without copying visuals.

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| Center modal each connect | Breaks flow; loses spatial context |
| Always silent infer | Violates typed relationship goal |

---

## R5 — UI primitives for catalog / popover / menus

**Decision**:

1. **Add to architecture**: `A11yDialog` (existing) + searchable list; **TanStack Virtual** if list length warrants; Hugeicons icons per type.
2. **Relationship picker**: Prefer lightweight positioned panel first; adopt **`@kobalte/core` Popover/Menu** only if focus trap/roving tabindex becomes hard.
3. **Context menus / floating toolbars**: Custom DESIGN-token panels acceptable if keyboard complete; Kobalte DropdownMenu if a11y gaps.
4. **Never** run full solidui-cli init; no React libraries.

**Rationale**: AGENTS.md selective import policy; avoid token/layout overwrite.

---

## R6 — Extending NodeType / RelationshipType

**Decision**: Expand TypeScript unions in `frontend-canvas` document types for **initial catalog IDs** from the spec (applications/data/infra/dev/external/org). Map close existing IDs (`storage` ↔ object storage, `runtime_host` ↔ runtime/server family) rather than duplicating synonyms as separate persisted IDs when one ID covers the concept.

New IDs only when semantics differ (e.g. `mobile_application`, `api_gateway`, `cdn`, `system_boundary` as node vs boundary entity).

**Unknown types on load**: render as generic Custom with raw id label; do not drop nodes.

**Migration**: additive; `formatVersion` stays `1` with optional `meta.generator`; no wire RPC change.

---

## R7 — Autosave vs manual Save layout

**Decision**: Autosave remains source of truth. **Demote** header **Save layout** (remove primary button or ghost secondary “Save now”). Surface `saveStatus` in canvas toolbar (already partial) + retry action on error.

Keep soft-fail: ConnectError must not replace Architecture page with full PageError for save failures.

**Rationale**: Spec FR-033/034; operational pain already observed (`Failed to fetch` blanking risk).

---

## R8 — Favorites / recents storage

**Decision**: **Session + `localStorage`** keys scoped by workspace/user-agnostic machine local (`polterware.arch.typeRecents`, `polterware.arch.typeFavorites`). **Not** inside architecture document body (favorites must not dirty layout revision).

**Implementation note**: Domain exposes `TypePrefsPort` only. Browser `localStorage` lives in a canvas/feature adapter (`type-prefs-local.ts`) so domain stays free of runtime APIs (Constitution II).

---

## R9 — Align / distribute / dependency highlight

**Decision**:

- Align/distribute: pure functions operating on node geometry maps; commit once `immediately`.
- Dependency highlight: host-only set of emphasized node/edge IDs; does not mutate document unless user explicitly edits.
- Fit selection: reuse `fitBounds` on selection AABB.

---

## R10 — Excalidraw reference usage

**Decision**: Study only:

- Layered Escape handling
- Transient tool states
- Context menu vs selection toolbar split
- History capture batching patterns

Do **not** copy CSS, component structure, or freehand model.

---

## R11 — Delivery completeness

**Decision**: Tasks must not mark feature complete after dialog-only work. Acceptance list from spec (30 items) is the done bar; P1 slice can ship as MVP milestone but feature status complete requires P2 organization + persistence/minimap/detail restore per global criteria.

---

## Resolved clarifications

| Topic | Resolution |
| --- | --- |
| Engine rewrite? | No |
| Registry location? | feature-architecture/domain |
| Modal for connect? | No — popover |
| Save button? | Demote; autosave primary |
| New packages? | None |
| Provider import live? | Stub OK |
| Env comparison full? | Filter + readiness; full compare can be later if tasks time-box |
