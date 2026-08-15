# Data Model: Integration Detail Panel

**Feature**: `012-integration-detail`  
**Date**: 2026-08-11  
**Scope**: UI/fixture DTOs for integration entity detail (mock delivery). Not a database schema. Future core APIs SHOULD map into these shapes or explicit successors without a second UX.

---

## 1. Entities

### 1.1 Integration (connection)

Workspace type: `WorkspaceIntegration`

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | Stable fixture/core id (`int-github`) |
| `name` | yes | Display name |
| `provider` | yes | Provider product name |
| `status` | yes | `connected` \| `error` \| `disconnected` \| `disabled` |
| `account` | yes | Account/org label; may be `—` when disconnected |
| `environment` | yes | Primary environment label for table |
| `environments` | no | Affected environments list for Overview |
| `permissions` | yes | Human summary string of grants |
| `scopes` | no | Registered OAuth/API scopes at connect |
| `lastSync` | yes | Last sync **attempt** label |
| `lastSuccessfulSync` | no | Last **successful** sync; may differ on failure |
| `summary` | yes | Short connection summary |
| `resourceCount` | yes | Imported count (may exceed listed fixture sample) |
| `resourceIds` | yes | Related inventory ids (may be partial) |
| `error` | no | Operator-visible failure reason |
| `syncInterval` | no | Automatic interval label (`5m`) |
| `rateLimitState` | no | `ok` \| `backoff` \| `limited` |
| `rateLimitMessage` | no | Operator-facing backoff copy |
| `externalConsoleUrl` | no | External hand-off URL |
| `freshness` | no | `fresh` \| `stale` \| `unknown` |

**Validation / UI rules**:

- `status === "error"` MUST show `error` or generic auth-failure copy; MUST retain last-known summary/resourceCount.
- `lastSuccessfulSync` MAY differ from `lastSync` when latest attempt failed.
- Secrets MUST NOT appear on this entity.

**Local mock overlays** (session UI, not persisted fields):

| Overlay | Effect on effective status |
| --- | --- |
| disabled set | `connected` → `disabled` |
| disconnected set | → `disconnected` (wins over disabled) |

---

### 1.2 Integration resource

Workspace type: `WorkspaceIntegrationResource`

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | Row id |
| `integrationId` | yes | FK → Integration |
| `name` | yes | Display |
| `kind` | yes | RDS, Repository, Project, … |
| `externalId` | yes | Provider-side id |
| `environment` | yes | Environment label |
| `status` | yes | e.g. `synced`, `error` (string for fixture flexibility) |
| `inventoryResourceId` | no | Hand-off into Resources inventory |

**Rules**: Filter by `integrationId`. Empty list → honest empty state. Error status → non-healthy StatusChip.

---

### 1.3 Credential reference

Workspace type: `WorkspaceIntegrationCredential`

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | |
| `integrationId` | yes | |
| `name` | yes | Reference name (not secret) |
| `scope` | yes | Role/scope label |
| `status` | yes | `valid` \| `expiring` \| `expired` \| `revoked` |
| `rotated` | yes | Relative age label |

**Forbidden fields**: `secret`, `token`, `password`, raw key material.

**Create draft** (transient dialog only):

| Field | Notes |
| --- | --- |
| `name` | required non-empty |
| `provider` | select |
| `secret` | write-only; cleared on submit/cancel; never copied into fixture row |

---

### 1.4 Capability / permission grant

Workspace type: `WorkspaceIntegrationCapability`

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | |
| `integrationId` | yes | |
| `permission` | yes | Capability label |
| `allowed` | yes | boolean |

**Rules**: Coverage bar only when `count > 0`. Denied is first-class (least privilege visibility).

---

### 1.5 Sync run

Workspace type: `WorkspaceIntegrationSyncRun`

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | |
| `integrationId` | yes | |
| `startedAt` | yes | Display label |
| `finishedAt` | yes | Display label |
| `result` | yes | `success` \| `partial` \| `failed` \| `running` |
| `imported` | yes | number |
| `changed` | yes | number |
| `message` | yes | Operator message |

**Rules**:

| Result | StatusChip tone (normative UI) |
| --- | --- |
| success | healthy |
| partial | warn |
| failed | danger |
| running | syncing |

Partial/failed MUST NOT be labeled as full health.

---

### 1.6 Webhook registration

Workspace type: `WorkspaceIntegrationWebhook`

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | |
| `integrationId` | yes | |
| `name` | yes | |
| `destinationHost` | yes | Host/path; may truncate middle for display |
| `events` | yes | Event subscription summary |
| `status` | yes | `active` \| `failed` \| `paused` |
| `lastCall` | yes | |
| `signingSecretConfigured` | yes | boolean only |
| `retryPolicyLabel` | yes | |
| `lastResponse` | yes | e.g. `200 OK`, `401 Unauthorized` |

**Forbidden**: raw signing secret values.

---

### 1.7 Catalog provider (context only)

Workspace type: `WorkspaceCatalogProvider` — used for Catalog page + connect hand-off into setup or existing connection. Not an Integration detail entity; connecting a already-connected catalog card requests the Connected integration target.

---

## 2. Relationships

```text
Integration 1──* IntegrationResource
Integration 1──* CredentialReference
Integration 1──* Capability
Integration 1──* SyncRun
Integration 1──* WebhookRegistration
IntegrationResource *──? Inventory Resource (via inventoryResourceId)
```

---

## 3. State transitions (mock UI)

### 3.1 Connection effective status

```text
fixture.status
    │
    ├─ overlay disconnected ──────────► disconnected
    ├─ overlay disabled + was connected ► disabled
    └─ else ───────────────────────────► fixture.status
```

### 3.2 Operator mock actions

| Action | Effect |
| --- | --- |
| Sync now | Toast only (mock); no fixture mutation required |
| Disable | Add id to disabled set |
| Enable | Remove from disabled set |
| Disconnect | Add id to disconnected set; clear disabled for that id |
| Reconfigure | Open setup wizard (mock connect path) |
| Add credential | Toast + clear secret; optional append ref in future; never store secret |
| Open console | `window.open(externalConsoleUrl)` or hand-off callback |

---

## 4. Fixture sets (SSOT)

| Export | Purpose |
| --- | --- |
| `INTEGRATION_MOCKS` | Connections |
| `INTEGRATION_RESOURCE_MOCKS` | Per-integration resources |
| `INTEGRATION_CREDENTIAL_MOCKS` | Credential refs |
| `INTEGRATION_CAPABILITY_MOCKS` | Permissions |
| `INTEGRATION_SYNC_RUN_MOCKS` | Sync history |
| `INTEGRATION_WEBHOOK_MOCKS` | Webhooks |
| `CATALOG_PROVIDER_MOCKS` | Catalog page |

---

## 5. Detail section → data map

| Section | Primary entities |
| --- | --- |
| Overview | Integration (+ scopes, error, freshness, rate limit) |
| Resources | IntegrationResource[] |
| Credentials | CredentialReference[] + create dialog |
| Sync | Integration + SyncRun[] + rate limit |
| Permissions | Capability[] |
| Webhooks | WebhookRegistration[] |
| Advanced | Integration (console URL) + disconnect action |

---

## 6. Non-goals (data)

- Persisted secret vault rows
- Live OAuth token refresh records
- Provider-native resource deletion on disconnect
- Dual schema for “page credentials tab” vs detail credentials
