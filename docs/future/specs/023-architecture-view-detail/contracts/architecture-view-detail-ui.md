# Contract: Architecture View Detail UI

**Feature**: `023-architecture-view-detail`  
**Surface**: Architecture → Views → global DetailPanel  
**Stage**: Mock / fixture (production-ready composition)

## Host

- Collection: Architecture Views table
- Detail host: MainSurface global DetailPanel only
- Scope id pattern: `architecture:views`
- Entity kind: `arch-view`

## Composition

- Dedicated `ArchitectureViewDetail` module: `packages/features/architecture/src/view-detail.tsx`
- Helpers: `view-detail-helpers.ts`
- Tests: `view-detail.test.ts`
- Header: Eye/view domain icon + view name + actions
- Body: `layout="sections"` + Grid / Section / Metric / Metadata
- Membership: DataTable when fixture `members` present; empty or unavailable copy otherwise

## Header actions

| Action | Tone | Behavior |
| --- | --- | --- |
| Open | primary | `setActiveViewId(viewId)` + Architecture tab → Canvas (existing activation) |
| Edit | default | `setEditViewName` + `deepMode("view-edit")` → existing `ViewEditor` |
| Duplicate / Delete | optional mock | honest toasts only (not required for MVP) |

## Required sections

1. Metrics — Nodes, Updated, Owner (or em dash)
2. Overview — description (or empty copy), purpose, owner, node count, data source, view id
3. Environments — list or “No environment scope.” + optional tags
4. Membership — DataTable (`aria-label="View members"`) or:
   - “No members in this view.” when `members: []`
   - “Membership list not available in mock.” when `members` undefined
5. Optional Notes / Filters when fixture fields present

## States

| State | UI |
| --- | --- |
| loading | PageLoading “Loading view…” |
| not-found / missing | PageError “View not found” |
| error | PageError with message |
| empty description | “No description provided for this view.” |

## Honesty

- no inventing member names when only nodeCount exists
- not-found / empty description required
- no dual canvas host; reuse Architecture canvas activation
- dataSource labeled fixture when present
