# Data Model: Inventory Resource Detail

**Feature**: `013-inventory-detail`  
**Date**: 2026-08-12  
**Scope**: UI/fixture DTOs for inventory resource entity detail (mock delivery). Not a database schema. Future core APIs SHOULD map into these shapes or explicit successors without a second UX.

---

## 1. Entities

### 1.1 Inventory resource

Feature type: `Resource` (`packages/features/resources/src/types.ts`)

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | Stable fixture/core id (`res-pg`) — not display-name identity |
| `name` | yes | Display name |
| `type` | yes | Resource type label (Service, Database, …) |
| `environment` | yes | Environment label |
| `provider` | yes | Provider product/platform label |
| `providerKind` | no | Finer kind (RDS, ECS, SQS, …) |
| `status` | yes | Lifecycle status (see §2) |
| `lastSyncAt` | no | Last inventory sync label; UI shows Never when absent |
| `lastSeenAt` | no | Last provider see / heartbeat freshness |
| `discoveredAt` | no | First discovery date label |
| `region` | no | Region label |
| `owner` | no | Operator ownership label |
| `systemId` | yes | System/project scope id for catalog mapping |
| `source` | no | Free-form source string (legacy/core) |
| `provenance` | no | `discovered` \| `imported` \| `manual` |
| `externalKey` | no | Stable external key |
| `providerResourceId` | no | Provider-side id/ARN |
| `externalUrl` | no | External console hand-off URL |
| `notes` | no | Operator annotation body |
| `tags` | no | Operator annotation tags |
| `lifecycleNote` | no | Operator-facing explanation for non-healthy lifecycle |
| `linkedNodeIds` | no | Architecture node id list (optional index) |
| `linkedArchitecture` | no | `{ id, name, type }[]` for DataTable |
| `activity` | no | `{ id, at, title, detail? }[]` for DataTable |
| `quickActions` | no | Interactive action entries |
| `capabilities` | no | Availability review entries |
| `metrics` | no | Present-only operational metrics (no invented history) |
| `dataSource` | yes | `core` \| `fixture` — honesty field |

**Validation / UI rules**:

- `dataSource === "fixture"` MUST render as fixture/mock in Source field.
- `status` in `stale` \| `missing` SHOULD carry `lifecycleNote` when operator clarity requires it.
- Secrets MUST NOT appear on this entity.
- Sparse rows (list-only fields) MUST still open detail with honest empties.

---

### 1.2 Quick action

Type: `QuickAction`

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | Stable action id |
| `label` | yes | Operator label |
| `description` | no | Helper when enabled |
| `kind` | yes | `external` \| `navigate` \| `operation` |
| `href` | no | Required for useful external open |
| `enabled` | yes | boolean |
| `disabledReason` | conditional | **Required when `enabled === false`** |

**Rules**: Disabled never offers success path. External uses hand-off only. Operation kind is mock-toast or disabled until core ops wire.

---

### 1.3 Capability

Type: `ResourceCapability`

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | |
| `label` | yes | Capability name |
| `available` | yes | boolean |
| `reason` | conditional | **Required when `available === false`** |

**Rules**: Table is review-oriented; does not replace header/quick actions for execution.

---

### 1.4 Linked architecture node (detail row)

Inline shape on Resource:

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | Architecture node id |
| `name` | yes | Display |
| `type` | yes | Node type for icon + column |

Empty array → honest empty (“may exist only in inventory”).

---

### 1.5 Activity event

Inline shape on Resource:

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | |
| `at` | yes | Relative/absolute display time |
| `title` | yes | Event title |
| `detail` | no | Optional detail line (not required in table columns) |

Empty array → “No recent activity.”

---

### 1.6 Metric

Type: `ResourceMetric`

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | |
| `label` | yes | |
| `value` | yes | Display value |
| `unit` | no | Appended when present |

Omit Metrics section when array empty/absent.

---

### 1.7 Resource draft (edit dialog — existing)

Type: `ResourceDraft` — name, type, environment, provider, notes. Used by create/edit dialog; not the full detail model. Annotations path for this delivery.

---

## 2. Lifecycle state machine (UI)

```text
                    ┌─────────────┐
         discover   │   healthy   │
              ─────►│             │◄──── recovery / fresh sync
             /      └──────┬──────┘
            /              │
           /     degrade   │  heartbeat delay
          /                ▼
┌─────────┴──┐      ┌─────────────┐      ┌─────────────┐
│  unknown   │      │  degraded   │─────►│    stale    │
│  (manual)  │      └──────┬──────┘      └──────┬──────┘
└────────────┘             │                    │
                           │ not returned       │ prolonged miss
                           ▼                    ▼
                    ┌─────────────┐      ┌─────────────┐
                    │    down     │      │   missing   │
                    └─────────────┘      └─────────────┘
                           ▲
                           │ failure
                    ┌───────┴───────┐
                    │    syncing    │  (refresh in progress)
                    └───────────────┘
```

**Invariants** (UI):

1. `missing` / `stale` MUST NOT be labeled as permanently deleted.
2. `syncing` MUST NOT use healthy tone.
3. Temporary absence prefers `stale` → `missing` narrative over delete.
4. StatusChip always filled; no hand-rolled pills.

---

## 3. Relationships

```text
Resource 1──* linkedArchitecture (Architecture node refs)
Resource 1──* activity
Resource 1──* quickActions
Resource 1──* capabilities
Resource 1──* metrics
Resource 1──* tags (annotation)
Resource *──1 systemId (project/system scope)
```

Inventory **MAY** exist without architecture links (product allows inventory-without-canvas).

---

## 4. Fixture coverage requirements

At least one **rich** fixture (`res-pg` or equivalent) MUST include:

- linkedArchitecture (≥1)
- activity (≥1)
- notes and/or tags
- provenance and externalKey (or providerResourceId)
- quickActions with ≥1 disabled + disabledReason
- capabilities with ≥1 unavailable + reason

Fixture set MUST also cover:

- sparse row (list fields only)
- ≥2 non-healthy among: degraded, down, unknown, syncing, stale, missing
- all rows `dataSource: "fixture"` for mock catalog

---

## 5. Mapping from core (future / existing thin path)

`mapCoreResource` produces:

- required identity fields
- `status: "unknown"`
- `dataSource: "core"`
- optional `source`, `externalKey`

Detail MUST tolerate missing optional sections (honest empties). No dual detail component for core vs fixture.
