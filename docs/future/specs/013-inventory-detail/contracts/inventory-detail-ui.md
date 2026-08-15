# Contract: Inventory Resource Detail UI

**Owner**: `packages/features/resources`  
**Consumers**: `ResourcesView` composition, `apps/desktop` detail fixture coverage  
**Extends**: `004` resources-ui Inventory row → detail; elevates existing `ResourceDetail`  
**Related**: [data-model.md](../data-model.md), [spec.md](../spec.md)

## Scope

Production-ready **Inventory** entity detail inside the global `DetailPanel`. Mock fixtures only for this delivery.

## Collection target

| Field | Value |
| --- | --- |
| `scopeId` | `resources:inventory` |
| `entityKind` | `resource` |
| Activation | Inventory DataTable row activate / active row change |

## Public composition API

```ts
ResourceDetail({
  resource: Resource | null | undefined;
  loading?: boolean;
  error?: string | null;
  notFound?: boolean;
  onEdit: () => void;
  onOpenArchitecture?: (nodeId: string) => void;
  onRefresh?: (resourceId: string) => void;
})
```

- MUST render `DetailPanel.Content` only (no page shell).
- Host owns panel mode/snaps/collection registration.
- `onEdit` opens existing resource form dialog path.
- `onOpenArchitecture` / `onRefresh` optional; default mock-honest toast when absent.

## Layout

- Host: global `DetailPanel` only (no page route, no second inspector).
- Body: `layout="sections"` + `DetailPanel.Grid`.
- Header: domain icon (same as Inventory PrimaryCell via `ResourceTableIcon`) + single-line name + `DetailPanel.Action`s.
- Local Tabs rail: **not required** (unlike Integration detail).

## Required content regions

| Region | Content |
| --- | --- |
| Metrics | Status (`StatusChip`), Provider, Environment, Last sync (or Never) |
| Lifecycle | When `lifecycleNote` present |
| Overview | Type, provider kind, region, owner, provenance, last seen, discovered, source honesty |
| Identity | Resource id, external key, provider resource id (mono) |
| Metrics detail | Optional present-only metric rows |
| Capabilities | DataTable: capability + availability chip (+ reason when unavailable) |
| Quick actions | Enabled buttons / disabled with reason |
| Linked architecture | DataTable name+type; activate → hand-off or toast; empty honest |
| Activity | DataTable event+when; empty honest |
| Annotations | Notes + tags; operator-owned copy |

## Header actions

| Action | When | Honesty |
| --- | --- | --- |
| Open provider | `externalUrl` or enabled external quick action href | External hand-off only |
| Refresh inventory | Always (mock default) | Toast unless `onRefresh` provided; never claim live discovery |
| Edit | Always | Opens resource form dialog |

## Status mapping

Helpers: `resourceStatusTone` / `resourceStatusLabel` (export from feature package).

| Status | Tone | Label |
| --- | --- | --- |
| healthy | healthy | Healthy |
| degraded | degraded | Degraded |
| stale | degraded | Stale |
| down | down | Down |
| missing | down | Missing |
| unknown | unknown | Unknown |
| syncing | syncing | Syncing |

MUST use `StatusChip` (filled). MUST NOT hand-roll status pills.

Inventory table (`ResourcesIndex`) MUST use the same tone/label helpers for row status.

## Honesty rules

- `dataSource: "fixture"` labeled Fixture (mock) in Source field.
- Disabled capability / quick action never fakes success; reason required when unavailable.
- Stale/missing ≠ deleted (lifecycle copy).
- Operator notes/tags separate from provider identity fields.
- Loading / not-found / error states keep panel structure with PageLoading / PageError.

## DataTable rules

- No Actions columns.
- Linked architecture and activity use design-system DataTable.
- Row activation only for navigation/hand-off patterns (architecture).

## Non-goals

- Live provider sync/discovery
- Nodes/Processes detail redesign
- Full-page resource detail
- Cloud console clone
- Protobuf / core API wiring
- Second detail composition dual path
