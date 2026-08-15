# Contract: Authoring Flows

**Owner**: `ArchitectureCanvasHost` + parent `ArchitectureView`  
**Engine**: document mutations + history only on **confirmed** steps

---

## A. Add component

### Triggers (all open same catalog)

| Trigger | Origin point for placement |
| --- | --- |
| Toolbar `+` / tool add | Viewport center or last pointer |
| Shortcut `N` | Same |
| Double-click empty | Click document point |
| Context “Add component here” | Menu document point |
| Command palette | Viewport center |
| Empty-state CTA | Viewport center |
| Components tab `+` | Viewport center (then switch to canvas tab if needed) |

### State machine

```text
idle
  → openCatalog
  → (select type | cancel → idle)
  → placementMode(typeId)
       pan allowed; Escape → idle
  → provisionalNode(typeId, x, y)
       inline name; Enter → commitNode → idle
       Escape → discard → idle
```

### Commit

```ts
docAddNode(doc, {
  name: trimmed,
  type: typeId,
  x, y,
  width, height, // from registry defaults
  ports: defaultPorts(typeId),
  origin: "manual",
  state: "planned",
})
// history: immediately
// autosave: yes
// pushRecent(typeId)
```

### Organization special cases

- Text Note → `docAddNode` type `note`
- * Boundary catalog rows → `addBoundary({ kind })` at click point (default size); name optional inline
- Connected-resource rows (`placeable: false`) → do not enter placement; show honest toast/empty (“Not available yet”)

### Forbidden

- Creating node on `+` click without type
- Autosaving provisional node
- Leaving orphan provisional in document after Escape

---

## B. Create relationship (contextual)

### State machine

```text
idle / select or connect tool
  → connectDrag(fromNode, fromPort)
  → drop on target
       invalid → cancel drag
       valid → pendingRelationship(source, target, ports)
            → openTypePopover(suggestions)
                 confirm(type) → addRelationship → idle
                 Escape / cancel → discard pending → idle
```

### Commit

```ts
addRelationship(doc, {
  sourceNodeId, targetNodeId,
  sourcePortId, targetPortId,
  type: chosenType, // required
  direction: "forward",
})
// history: immediately
// autosave: yes
```

### Visual requirements

- Rubber-band while dragging
- Valid targets subtly emphasized; invalid dimmed or forbidden cursor
- Popover anchored near edge midpoint / drop screen point
- Suggested type focused for keyboard Enter

---

## C. Create relationship (quick mode)

```text
setTool(connect) + setQuickRelType(type)
toolbar shows "Connect: {label}"
each successful drop → addRelationship with quick type (no popover)
Escape → clear quick type + leave connect or return select
change dropdown → update quick type for subsequent edges
remember last quick type in session memory
```

Still run `canConnect`; skip only the type chooser.

---

## D. Escape priority (host)

Highest first:

1. Open dialogs (catalog)
2. Relationship type popover / pending
3. Provisional name edit
4. Placement mode
5. Connect drag
6. Context menu
7. Connect quick mode / non-select tool → select
8. Clear selection

---

## E. History participation

| Action | History |
| --- | --- |
| Catalog open | never |
| Placement move | never |
| Provisional | never |
| Name confirm create | immediately |
| Pending edge | never |
| Type confirm edge | immediately |
| Node drag | eventually → immediately on up |
| Rename / delete / reverse / type change | immediately |
| Align / distribute / layout | immediately |
| Paste / duplicate | immediately |

---

## F. Removal of legacy paths

| Legacy | Replacement |
| --- | --- |
| `add_node` click → New component / api | Catalog + placement |
| dblclick → service New component | Catalog at point (or placement after last type if product chooses catalog always — **catalog always**) |
| `inferRelationshipType` auto-persist | suggest + confirm |
| CreateNodeDialog freeform long-term | Catalog (optional post-fields later) |
