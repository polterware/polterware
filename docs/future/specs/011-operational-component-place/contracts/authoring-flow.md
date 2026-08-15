# Contract: Operational Authoring Flow

**Owner**: `ArchitectureCanvasHost` + `ArchitectureView` + configure/catalog dialogs  
**Related**: `specs/006-semantic-architecture-canvas/contracts/authoring-flows.md` (supersedes section A for identity-rich place)

---

## A. Triggers

| Trigger | Origin | Opens |
| --- | --- | --- |
| Toolbar `+` / Add tool | center or last pointer | Catalog |
| Shortcut `N` | center | Catalog |
| Double-click empty | click point | Catalog |
| Context “Add component here” | menu point | Catalog |
| Empty-state CTA | center | Catalog |
| Command palette | center | Catalog |
| Components tab `+` | center (switch to canvas on commit) | Catalog (same flow) |
| Catalog Connected “Link from Resources” | center | Resource-first path |

---

## B. State machine (model-first)

```text
idle
  → catalogOpen
       cancel/Escape → idle
       pick placeable type → identityOpen(type)
       pick non-placeable / stub import → toast unavailable → idle|catalog
       pick "Link from Resources" → resourceFirstOpen → …
  → identityOpen(type, origin?)
       fields: name*, environmentId*, purpose?, role?, owner?, resourceId?
       Cancel/Escape → idle (no node)
       Confirm valid:
         if origin known → commitPlace → selectNode → idle
         if origin null  → placementMode(draft) 
              click → commitPlace → selectNode → idle
              Escape → idle (no node)
```

### Resource-first (Connected)

```text
resourceFirstOpen
  → pick resource → identityOpen(prefilled type/name/env/resourceId)
  → cancel → idle
```

---

## C. Commit payload

```ts
// Conceptual — not a network API
docAddNode(doc, {
  name,
  type, subtype, provider, // from catalog
  environmentIds: [environmentId],
  description?, owner?,
  metadata: role ? { role } : undefined,
  origin: "manual",
  state: resourceId ? "linked" : "planned",
  linkStatus: resourceId ? "linked" : "unlinked",
  resourceId: resourceId ?? null,
  x, y, width, height, ports,
})
// history: immediately
// autosave: yes
```

---

## D. Guarantees

| Guarantee | Rule |
| --- | --- |
| No type-less create | Catalog or resource-first implies type |
| No secret fields | Place form never asks credentials |
| No provision | Confirm never starts provisioning |
| No target register | Confirm never creates Target/node agent |
| Cancel safety | Escape/cancel leaves zero new nodes |
| Name SSOT | Configure name; inline rename optional edit only |
| Honesty | Unlinked never presented as healthy/live |

---

## E. Post-commit UX

1. Select created node  
2. Open/keep global DetailPanel (half if collapsed via existing scope rules)  
3. If unlinked: show **Link resource** action in detail  
4. Card shows Planned or Linked label per honesty mapping  

---

## F. Forbidden

- Multipage wizard as primary create  
- Dual freeform CreateNodeDialog permanently  
- Autosaving provisional placement draft  
- Mapping `linked` → Healthy  
- Creating resources as side effect of Place  
