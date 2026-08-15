# Data Model: Resources Nodes Detail

**Feature**: `014-nodes-detail`  
**Date**: 2026-08-12  
**Scope**: UI/fixture DTOs for Polter Node entity detail (mock delivery). Not a database schema. Future core/node APIs SHOULD map into these shapes or explicit successors without a second UX.

---

## 1. Entities

### 1.1 Target node (Polter Node)

Feature type: `TargetNode`

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | Collection / row key (`node-nitro`) |
| `name` | yes | Display name (may match hostname) |
| `nodeId` | no | Stable registry identity when different presentation needed; UI uses `nodeId ?? id` |
| `location` | yes | Operator location label |
| `os` | yes | OS string; sparse may be `—` |
| `version` | yes | Agent version; sparse may be `—` |
| `status` | yes | See health status enum |
| `lastHeartbeatAt` | yes | Relative label (`8s ago`, `Never`) |
| `hostname` | no | Machine hostname |
| `ip` | no | Address when known; never invent |
| `arch` | no | e.g. `x86_64`, `aarch64` |
| `cpu` | no | Human capacity label |
| `memory` | no | Human capacity label |
| `storage` | no | Human capacity label |
| `healthNote` | no | Operator honesty copy (heartbeat ≠ process death, drain/upgrade notes) |
| `connection` | no | See connection summary |
| `capabilities` | no | `string[]` or `NodeCapability[]` |
| `assignedResources` | no | Inventory-like assignments on this node |
| `recentOperations` | no | Recent dispatch/command evidence |
| `upgradeAvailable` | no | Target version string or null |
| `dataSource` | yes | `fixture` \| `core` |

**Validation / UI rules**:

- `nodeId` (resolved) MUST be displayable distinct from `name` when both present.
- `dataSource: "fixture"` MUST render as fixture/mock honesty in Source field.
- Secrets, cert PEMs, private keys MUST NOT appear on this entity.
- Sparse nodes (list fields only) remain valid; optional sections use — / empty tables.

---

### 1.2 Health status

Type: `NodeHealthStatus`

```text
registering | online | degraded | offline | draining | upgrading | revoked | unknown
```

| Status | StatusChip tone (normative UI) | Dispatch / Terminal (mock UI) |
| --- | --- | --- |
| online | healthy | allowed |
| degraded | warn | allowed |
| offline | down | denied |
| registering | warn | denied |
| draining | warn | denied |
| upgrading | warn | denied |
| revoked | danger | denied |
| unknown | unknown | denied |

**Invariant**: Offline / missed heartbeat MUST NOT be presented as proof that remote processes exited.

---

### 1.3 Connection summary

Type: `NodeConnection`

| Field | Required | Notes |
| --- | --- | --- |
| `state` | no | `connected` \| `reconnecting` \| `disconnected` \| `unknown` |
| `lastConnectedAt` | no | Relative label |
| `modelNote` | no | Outbound control model copy |

---

### 1.4 Announced capability

Type: `NodeCapability` (or bare string in fixtures)

| Field | Required | Notes |
| --- | --- | --- |
| `id` | no | Stable key; generated if absent |
| `name` | yes | Capability label (Docker, Terminal, …) |
| `authorized` | no | When set, UI shows Authorized / Denied chip |

**Rules**: Node only executes announced + authorized capabilities conceptually; mock UI surfaces advertisement honesty.

---

### 1.5 Assigned resource

Type: `NodeAssignedResource`

| Field | Required | Notes |
| --- | --- | --- |
| `name` | yes | Display |
| `type` | yes | Runtime, Database, Cache, Service, … |
| `status` | yes | Free string mapped to StatusChip tone helpers |

Empty list → honest empty state (“No resources assigned.”).

---

### 1.6 Recent operation

Type: `NodeRecentOperation`

| Field | Required | Notes |
| --- | --- | --- |
| `at` | yes | Relative time |
| `command` | yes | Command / dispatch label |
| `actor` | yes | Human or agent label |
| `result` | yes | Success \| Failed \| Denied \| Pending \| … |

Result maps to StatusChip via `operationResultTone`. Empty → “No recent operations.”

---

### 1.7 Process (context only)

Type: `ProcessRow` — lives in same module for Processes tab; **not** embedded as required node detail entity. Future deep link may filter processes by `nodeId` without folding Processes into node detail sections.

---

## 2. Relationships

```text
TargetNode 1──* NodeCapability (announced)
TargetNode 1──* NodeAssignedResource
TargetNode 1──* NodeRecentOperation
TargetNode 1──? NodeConnection
TargetNode 1──* ProcessRow (via ProcessRow.nodeId; Processes tab)
```

---

## 3. State transitions (mock UI)

### 3.1 Health (fixture-static)

Fixtures are static snapshots for this delivery. No in-panel health machine editor.

Conceptual registry lifecycle (product domain, not editable UI):

```text
REGISTERING → ONLINE ⇄ DEGRADED
              ONLINE → DRAINING → OFFLINE / ONLINE
              ONLINE → UPGRADING → ONLINE / DEGRADED
              * → REVOKED (terminal for dispatch)
              * → OFFLINE (heartbeat loss; processes unknown)
```

### 3.2 Operator mock actions

| Action | When available | Effect |
| --- | --- | --- |
| Open terminal | online \| degraded | Invoke `onTerminal` (App mock / Terminal page) |
| Run command | online \| degraded | Invoke `onRunCommand` (App mock dialog) |
| (not in header) Drain / Revoke / Upgrade | — | Out of scope for this detail header; may appear later as operations |

---

## 4. Fixture sets (SSOT)

| Export | Purpose |
| --- | --- |
| `NODE_FIXTURES` | Full node catalog for Nodes list + detail |
| `PROCESS_FIXTURES` | Processes tab (related, not required for node detail completeness) |

### Fixture coverage expectations

| Shape | Example id | Purpose |
| --- | --- | --- |
| Rich online | `node-nitro` | capabilities, resources, ops, machine, connection, nodeId ≠ name |
| Online secondary | `node-worker-01` | second rich path |
| Degraded | `node-prod-vps` | warn health, reconnecting, upgrade available |
| Draining | `node-staging` | no dispatch affordance |
| Offline + health note | `node-legacy` | heartbeat honesty |
| Upgrading | `node-build` | upgrade pause copy |
| Revoked | `node-revoked` | no dispatch; danger chip |
| Sparse unknown | `node-sparse` | empty optional sections |

All fixtures: `dataSource: "fixture"`.

---

## 5. Detail section → data map

| Section / block | Primary fields |
| --- | --- |
| Metrics | status, lastHeartbeatAt, version, capabilities.length |
| Identity | nodeId, name, hostname, location, os, arch, ip, status, dataSource |
| Machine | cpu, memory, storage, arch |
| Connection & health | connection.*, healthNote, upgradeAvailable, version |
| Capabilities | capabilities[] |
| Assigned resources | assignedResources[] |
| Recent operations | recentOperations[] |
| Header actions | status gates → onTerminal / onRunCommand |

---

## 6. Non-goals (data)

- Persisted node certificate or bootstrap credential rows
- Live heartbeat streams / high-frequency history tables
- Process kill / remote desktop session records
- Dual schema for “node page” vs panel detail
- Provider-owned inventory lifecycle (owned by Inventory resource detail)
