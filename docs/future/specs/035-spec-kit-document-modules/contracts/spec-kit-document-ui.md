# Contract: Spec Kit Document UI (modules)

**Owner**: `packages/features/spec-kit`  
**Consumers**: Desktop `App.tsx` Spec Kit route  
**Supersedes for detail modularity**: inline monolith in prior Spec Kit index (004 contract still owns IA)

## Purpose

Professional document detail for Spec Kit artifacts with maintainable modules. Kind-first collection unchanged. Does **not** run Speckit CLI. Does **not** merge with Knowledge.

## Public composition

| Export / module | Role |
| --- | --- |
| `SpecKitView` (`index.tsx`) | Tabs, filters, DataTable, registerCollection, create dialog, activates detail |
| `SpecArtifactDetail` | Global DetailPanel document composition for one artifact |
| Helpers | Status tones, labels, empty/create copy, property builder |

## Detail contract

- `DetailPanel.Content` + `Header` (domain icon + title) + `Action` “Copy markdown”
- `DetailPanel.Body layout="document"`
- Properties (order):
  1. Kind (`specKind` label)
  2. Status — **StatusChip** only
  3. Feature — when `featureKey` present
  4. Path — `logicalPath`, monospace
  5. Owner
  6. Author
  7. Updated
  8. Source — when present (`project` | `repo`)
- Body: design-system `MarkdownEditor`
- `showToolbar` true **only** when detail panel mode is `maximized`
- Loading / not-found / error states honest when wiring supplies them

## Collection contract (index)

- Kind-first tabs unchanged
- Filter by status + featureKey (feature group omitted on Constitution)
- DataTable: Title (+ Feature when not constitution) · Status · Updated
- Empty states per tab (filtered vs unfiltered)
- `registerCollection` scope `speckit:{tab}` · `entityKind: "spec"`

## Path / kind SSOT

- `specKindForTab` and `logicalPathForSpec` from shell — **must not** reimplement
- Create dialog previews path via `logicalPathForSpec`

## Copy honesty

Forbidden phrases / claims:
- Running Speckit CLI
- Live bind to repo as done fact
- Knowledge docs/decisions/runbooks as Spec Kit content

Allowed:
- Mock toasts for create/copy
- “Logical path follows Spec Kit layout; content is session mock”

## Non-goals

- Knowledge package changes
- Live FS / CLI
- Per-kind detail component files (optional later if UX diverges)
