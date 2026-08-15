# Data Model: Operator Product Surfaces

**Feature**: `004-operator-product-surfaces`  
**Authority**: Presentation/view models in Desktop renderer + feature packages. Domain authority remains polterware-core (see `specs/002-architecture-big-bang/data-model.md` and local protos).  
**This document** defines UI-facing entities, route state, and fixture shapes—not a second persistence schema.

**Extends**: `specs/003-operator-design-system/data-model.md` (Resource, ArchitectureNode, ClientRoute base, DialogState, dataSource honesty).

---

## Principles

1. **Card ≠ Resource**; canvas cards are visual only.  
2. **System is root**; UI may say Project as alias.  
3. **Machine/Node = Target**, never System owner.  
4. **Honest sources**: `dataSource` knowable to UI.  
5. **Ephemeral vs durable**: Process → drawer; durable entities → pages.  
6. **Secrets**: credential secret material never retained in view models after create response.

---

## ClientRoute (navigation state)

| Field | Type | Rules |
| --- | --- | --- |
| section | `SectionId` | Exactly one of the seven destinations |
| tab | string | Per-section default; see Tab catalog |
| page | `index` \| `detail` | Detail only with entityId (+ entityKind) |
| entityId | string? | Required for detail |
| entityKind | `EntityKind`? | Required for detail when section has multiple entity types |
| environmentId | string? | Default `production` |
| search | string | Default `""` |
| filter | string | Section-specific; default `all` |
| dialog | `DialogState` \| null | Focused overlays |
| drawer | `DrawerState` \| null | Process only |
| selectedNodeId | string? \| null | Architecture canvas selection (index only) |

### SectionId

`architecture` | `resources` | `ops` | `knowledge` | `access` | `integrations` | `settings`

(Keep `ops` id for Operations to minimize churn; label is “Operations”.)

### EntityKind

`resource` | `node` | `arch-node` | `run` | `workflow` | `approval` | `document` | `decision` | `runbook` | `member` | `agent` | `policy` | `integration`

### Tab catalog (defaults first)

| Section | Tabs |
| --- | --- |
| architecture | `canvas`, `components`, `dependencies`, `views`, `changes` |
| resources | `inventory`, `nodes`, `processes` |
| ops | `runs`, `workflows`, `approvals`, `terminal` |
| knowledge | `docs`, `decisions`, `runbooks`, `ownership` |
| access | `members`, `agents`, `policies`, `audit` |
| integrations | `connected`, `catalog`, `credentials`, `sync` |
| settings | `project`, `environments`, `notifications`, `core`, `account` |

### Transitions

| From | Event | To |
| --- | --- | --- |
| any index | change tab | same section, new tab, page index, clear entity |
| any | open durable entity | page detail + entityId + entityKind |
| detail | back | page index, clear entity |
| architecture canvas | select node | selectedNodeId set; page stays index |
| architecture canvas | open node | detail + entityKind `arch-node` |
| resources processes | open process | drawer `{ type: "process", processId }` (not page) |
| any | open dialog | dialog set; page unchanged |
| dialog cancel/success | close | dialog null |
| drawer close | — | drawer null |
| legacy section id | navigate | map to new section+tab (research R1) |

### History

Include: section, tab, page, entityId, entityKind, environmentId, search, filter.  
Exclude by default: dialog, drawer, selectedNodeId (selection is ephemeral).

---

## DialogState

| type | Purpose | Key fields |
| --- | --- | --- |
| `resource-create` / `resource-edit` | Resource form (003) | entityId?, draft |
| `arch-node-create` | Create architecture node | draft: name, type, environment, owner, description, linkedResourceId? |
| `run-command` | Request command execution | targetId, command, cwd?, environmentId |
| `workflow-create` | Optional create workflow | draft |
| `invite-member` | Access invite | email, role |
| `register-agent` | Access agent | name, type, scope |
| `policy-create` / `policy-edit` | Policy editor | draft |
| `credential-create` | Integration credential | provider, scope, secret (write-only) |
| `integration-connect` | Connect from catalog | providerId |
| `confirm` | Destructive confirm | message, actionId |
| `create-view` | Architecture saved view | name, description |

---

## DrawerState

| type | Fields | Rules |
| --- | --- | --- |
| `process` | `processId: string` | Only allowed drawer type in this feature |

---

## ShellStatus

| Field | Type | Rules |
| --- | --- | --- |
| coreOnline | boolean | From health |
| nodesConnected | number? | Connected targets/nodes count |
| actorName | string? | Display |
| actorRole | string? | Display |
| dataMode | `live` \| `fixture` \| `offline`? | Optional shell-level honesty |

---

## Shared presentation types

### StatusSignal

`healthy` | `degraded` | `down` | `unknown` | `syncing` | `online` | `offline` | `running` | `pending` | `failed` | `success` | `active` | `disabled` | `expired`

Always pair color with text and/or icon.

### QuickAction

| Field | Type |
| --- | --- |
| id | string |
| label | string |
| kind | `navigate` \| `dialog` \| `external` \| `operation` |
| href or target | string? |
| permission | string? |
| enabled | boolean |

### ActivityItem

| Field | Type |
| --- | --- |
| id | string |
| at | string (ISO) |
| summary | string |
| actor | string? |
| result | string? |

### PageState envelope

```ts
type LoadState = "loading" | "ready" | "empty" | "error" | "offline" | "forbidden";
```

---

## Architecture entities

### ArchitectureComponent (table + map node)

| Field | Type | Rules |
| --- | --- | --- |
| id | string | Required |
| name | string | Required |
| type | string | Service, Worker, Database, Cache, Queue, Web App, CI/CD, … |
| owner | string? | Team display |
| environment | string | |
| status | StatusSignal | |
| changedAt | string? | |
| layer | string? | CLOUD / APPLICATIONS / DATA … |
| linkedResourceId | string? | |
| version | string? | |
| dataSource | core \| fixture | |

### DependencyEdge

| Field | Type |
| --- | --- |
| id | string |
| sourceId | string |
| targetId | string |
| relationship | string | calls, reads/writes, caches, publishes, consumes, … |
| protocol | string? | HTTPS, gRPC, TCP, Async |
| status | StatusSignal |
| external | boolean | default false |

### ArchitectureView

| Field | Type |
| --- | --- |
| id | string |
| name | string |
| description | string? |
| nodeCount | number |
| updatedAt | string |
| nodeIds | string[] |

### ArchitectureChangeEvent

| Field | Type |
| --- | --- |
| id | string |
| at | string |
| title | string |
| detail | string? |
| actor | string |
| category | string? |

### ArchitectureNodeDetail (page model)

Extends ArchitectureComponent with: role, runtime, lastDeployAt, endpoint, connected[], relationshipMini, notes, tags, compliance, costCenter, activity[], quickActions[].

---

## Resources entities

### Resource

As in 003; retain columns name, type, environment, provider, status, lastSyncAt, linkedNodeIds, quickActions, dataSource.

### TargetNode (Resources → Nodes)

| Field | Type | Rules |
| --- | --- | --- |
| id | string | Target id |
| name | string | Hostname display |
| location | string? | |
| os | string? | |
| version | string? | Agent/node version |
| status | `online` \| `offline` \| unknown | |
| lastHeartbeatAt | string? | |
| capabilities | string[] | Docker, Terminal, Filesystem, … |
| assignedResourceIds | string[] | |
| recentOperations | ActivityItem[] | |
| dataSource | core \| fixture | |

### ProcessRow

| Field | Type | Rules |
| --- | --- | --- |
| id | string | |
| name | string | |
| pid | number \| string | |
| nodeId | string | |
| nodeName | string | |
| cpuPercent | number? | |
| memoryBytes | number? | |
| port | number \| string? | |
| status | running \| stopped \| unknown | |
| command | string? | |
| user | string? | |
| startedAt | string? | |
| dataSource | core \| fixture | |

Process detail drawer uses the same model; no full-page route.

---

## Operations entities

### OperationRun

| Field | Type |
| --- | --- |
| id | string |
| title | string |
| target | string |
| actor | string |
| status | pending \| running \| success \| failed \| cancelled |
| startedAt | string |
| finishedAt | string? |
| environment | string? |
| approvalId | string? |
| workflowId | string? |
| steps | RunStep[] |
| outputLines | string[] |
| dataSource | core \| fixture |

### RunStep

| Field | Type |
| --- | --- |
| id | string |
| name | string |
| status | waiting \| running \| success \| failed \| skipped |
| durationMs | number? |

### WorkflowDef

| Field | Type |
| --- | --- |
| id | string |
| name | string |
| description | string? |
| trigger | manual \| scheduled \| event |
| lastRunAt | string? |
| lastStatus | StatusSignal |
| owner | string? |
| steps | { id, name, notes? }[] |
| recentRunIds | string[] |
| dataSource | core \| fixture |

### ApprovalRequest

| Field | Type |
| --- | --- |
| id | string |
| title | string |
| requestedBy | string |
| target | string |
| risk | low \| medium \| high |
| status | pending \| approved \| rejected \| expired |
| waitingSince | string |
| expiresAt | string? |
| reason | string? |
| impact | string[] |
| policyChecks | { label: string; ok: boolean }[] |
| comments | { actor, at, body }[] |
| operationId | string? |
| dataSource | core \| fixture |

### TerminalSession

| Field | Type |
| --- | --- |
| id | string |
| nodeId | string |
| nodeName | string |
| shell | string |
| status | connected \| disconnected \| connecting |
| actor | string? |

---

## Knowledge entities

### KnowledgeDocument

| Field | Type |
| --- | --- |
| id | string |
| title | string |
| bodyMarkdown | string |
| linkedTo | string? |
| owner | string? |
| updatedAt | string |
| category | string? |
| linkedIds | string[] |
| dataSource | core \| fixture |

### DecisionRecord

| Field | Type |
| --- | --- |
| id | string | e.g. ADR-014 |
| title | string |
| status | accepted \| proposed \| rejected \| superseded |
| owner | string? |
| date | string |

### Runbook

| Field | Type |
| --- | --- |
| id | string |
| title | string |
| service | string |
| lastTestedAt | string? |
| owner | string? |
| needsReview | boolean |

### OwnershipRow

| Field | Type |
| --- | --- |
| componentId | string |
| componentName | string |
| primaryOwner | string |
| secondaryOwner | string? |
| escalation | string? |

---

## Access entities

### Member

| Field | Type |
| --- | --- |
| id | string |
| name | string |
| email | string |
| role | admin \| operator \| viewer \| string |
| teams | string[] |
| status | online \| offline |
| lastSeenAt | string? |
| permissions | string[] |
| authMethods | string? |
| activity | ActivityItem[] |
| dataSource | core \| fixture |

### AgentActor

| Field | Type |
| --- | --- |
| id | string |
| name | string |
| type | ai \| application |
| scope | string |
| status | active \| disabled |
| lastActivityAt | string? |
| capabilities | { id, label, allowed: boolean }[] |
| owner | string? |
| dataSource | core \| fixture |

### PolicyDef

| Field | Type |
| --- | --- |
| id | string |
| name | string |
| description | string? |
| scope | string |
| enforcement | required \| optional |
| status | active \| disabled |
| updatedAt | string |
| appliesTo | string[] |
| conditions | string[] |
| requirements | string[] |
| exceptions | string[] |

### AuditEvent

| Field | Type |
| --- | --- |
| id | string |
| at | string |
| actor | string |
| action | string |
| target | string |
| result | success \| pending \| failed \| string |
| correlationId | string? |

---

## Integrations entities

### IntegrationConnection

| Field | Type |
| --- | --- |
| id | string |
| provider | string |
| status | connected \| error \| disabled |
| resourceCounts | Record<string, number> or summary string |
| lastSyncAt | string? |
| accountLabel | string? |
| accountId | string? |
| region | string? |
| permissions | { label: string; allowed: boolean }[] |
| dataSource | core \| fixture |

### CatalogEntry

| Field | Type |
| --- | --- |
| id | string |
| name | string |
| category | cloud \| platforms \| observability \| infrastructure \| messaging |
| available | boolean | false → coming soon honesty |

### CredentialRef

| Field | Type | Rules |
| --- | --- | --- |
| id | string | |
| name | string | |
| provider | string | |
| scope | string | |
| status | valid \| invalid \| expired | |
| rotatedAt | string? | |
| secret | never | **Must not exist on read models** |

### SyncRow

| Field | Type |
| --- | --- |
| provider | string |
| lastSyncAt | string |
| durationMs | number? |
| imported | number |
| changes | number |
| result | success \| failed \| partial |

---

## Settings entities

### SystemSettings (Project tab)

| Field | Type |
| --- | --- |
| systemId | string | display + copy |
| name | string |
| description | string? |
| defaultEnvironmentId | string |

### EnvironmentRow

| Field | Type |
| --- | --- |
| id | string |
| name | string |
| resourceCount | number |
| nodeCount | number \| string |
| protection | strict \| standard \| relaxed |
| isDefault | boolean |
| protectionRules | string[] |

### NotificationPreference

| Field | Type |
| --- | --- |
| eventId | string |
| eventLabel | string |
| desktop | boolean |
| email | boolean |
| slack | boolean |

Quiet hours: `{ start: string; end: string }`.

### CoreSettings

| Field | Type |
| --- | --- |
| status | online \| offline |
| version | string? |
| endpoint | string? |
| databaseLabel | string? |
| dataDirectory | string? |
| startWithApp | boolean |
| reconnectNodes | boolean |
| allowLan | boolean |
| autoUpdates | boolean |

### AccountSettings

| Field | Type |
| --- | --- |
| name | string |
| email | string |
| theme | dark \| … |
| density | compact \| comfortable |
| mfaEnabled | boolean |
| activeSessions | number |

---

## Validation rules (UI)

| Entity / draft | Rules |
| --- | --- |
| Names | required, trimmed, max 128 |
| Run command | target + command required; non-empty command |
| Credential create | provider + secret required; secret cleared from state after success |
| Invite member | email format + role required |
| Environment default | exactly one default when list non-empty |
| Approval resolve | only when status pending and actor permitted; self-approve blocked when policy says so |

---

## State machines (UI-visible)

### OperationRun.status

`pending` → `running` → `success` | `failed` | `cancelled`  
(`pending` may wait on approval)

### ApprovalRequest.status

`pending` → `approved` | `rejected` | `expired`

### TerminalSession.status

`connecting` → `connected` ↔ `disconnected`

### IntegrationConnection.status

`connected` | `error` | `disabled`

---

## Fixture catalog expectations

Each destination ships a **minimal demo fixture set** sufficient for visual acceptance and SC paths when core is offline. Fixtures must be labeled via `FixtureBanner` / `dataSource`. Do not invent fake success for destructive production applies without labeling demo mode.
