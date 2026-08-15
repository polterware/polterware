# Contract: Canvas Engine API

**Package**: `@polterware/frontend-canvas`  
**Runtime**: Browser-safe only (no Electron, FS, Node privileged APIs)  
**UI framework**: Pure TS core; optional Solid host helpers under `solid/`

## Responsibilities

Viewport, selection, hit testing, pointer tool machine, history, clipboard graph helpers, snap/guides, spatial index, base document types, serialize/migrate generic layer, layout worker messaging.

## Non-responsibilities

Architecture node type catalogs, evidence policy copy, Connect clients, routing, DESIGN-specific badges (domain supplies render props).

## Public surface (logical modules)

### Document

```text
createEmptyDocument(meta) → Document
applyTransaction(doc, tx) → Document
getElement(doc, id) → Element | undefined
```

Generic `Document` is the architecture document when used by Polter (same schema v1). Engine may type elements via discriminators: `node` | `relationship` | `boundary` | `note`.

### Viewport

```text
screenToDocument(vp, pt) → Point
documentToScreen(vp, pt) → Point
zoomAt(vp, focalScreen, nextZoom) → Viewport
fitBounds(vp, bounds, padding) → Viewport
panBy(vp, dx, dy) → Viewport
clampZoom(z) → number
```

### Selection

```text
selectClick(state, hit, mods) → Selection
selectMarquee(elements, rect, mode: contain|intersect) → Selection
selectAll(doc) → Selection
```

### History

```text
createHistory() → History
commit(history, doc, capture: immediately|eventually|never, patch) → { history, doc }
undo(history, doc) → { history, doc, selection? }
redo(history, doc) → { history, doc, selection? }
```

### Commands

```text
type Command = { id, label, run(ctx), isEnabled?(ctx), keyTest?(e) }
registerCommand(cmd)
dispatch(id, ctx)
matchShortcut(e, ctx) → Command | null
```

### Pointer / tools

```text
type Tool = select | hand | connect | add_node | add_boundary | add_note
createPointerController(opts) → { onPointerDown/Move/Up, onWheel, onKeyDown }
```

### Hit test

```text
hitTest(doc, vp, screenPt) → Hit: port|handle|node|relationship|boundary|empty
```

### Clipboard helpers

```text
buildFragment(doc, selection) → Fragment
pasteFragment(doc, fragment, at) → { doc, newSelection }  // new ids
```

### Domain registry (injection)

```text
type DomainAdapter = {
  defaultPorts(nodeType): Port[]
  canConnect(a, b, relType): boolean
  defaultNodeSize(type): Size
  // optional
}
```

Engine calls adapter for rules; does not hardcode architecture enums beyond generic ids.

## Solid host (optional)

- `CanvasWorld` — transform root + pointer target  
- Does not own Connect or routes  

## Testing contract

Pure functions unit-tested without Solid/Electron.  
