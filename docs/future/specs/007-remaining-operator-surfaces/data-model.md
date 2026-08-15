# Data Model: Remaining Operator Surfaces

**Feature**: `007-remaining-operator-surfaces`  
**Authority**: Presentation/view models in Desktop renderer + feature packages. Domain authority remains polterware-core (see `specs/002-architecture-big-bang/data-model.md` and local protos).  
**This document** defines UI-facing entities, route state, and fixture shapes—not a second persistence schema.

**Extends**: `specs/004-operator-product-surfaces/data-model.md` (ClientRoute, seven sections, process drawer, entity kinds).

---

## Principles

1. **System is root**; UI may say Project (Overview, switcher).  
2. **Machine/Node = Target**.  
3. **Honest sources**: `dataSource` knowable to UI.  
4. **Ephemeral vs durable**: Process → drawer; Notifications → drawer; durable entities → pages.  
5. **Secrets**: write-once / rotate; install tokens short-lived and shown only during registration window.  
6. **Single nav SSoT**: eight primary destinations; chrome sections never enter `DEFAULT_NAV`.  
7. **Hand-off over clone**: external observability/provider consoles are links, not embedded products.

---

## ClientRoute (navigation state)

| Field | Type | Rules |
| --- | --- | --- |
| section | `SectionId` | Nav or chrome section |
| tab | string | Per-section default; ignored or defaulted for chrome sections without tabs |
| page | `index` \| `detail` | Detail only with entityId (+ entityKind) |
| entityId | string? | Required for detail |
| entityKind | `EntityKind`? | Required when section has multiple entity types |
| environmentId | string? | Default `production` |
| search | string | Default `""`; for `search` section, holds query |
| filter | string | Section-specific; default `all` |
| dialog | `DialogState` \| null | Short focused overlays |
| drawer | `DrawerState` \| null | `process` or `notifications` |
| selectedNodeId | string? \| null | Architecture canvas selection |
| wizardStep | number? | Optional multi-step page wizard progress (0-based) |

### SectionId

**Nav (DEFAULT_NAV only):**

`architecture` | `resources` | `ops` | `observability` | `knowledge` | `access` | `integrations` | `settings`

**Chrome (valid route, never sidebar):**

`overview` | `search` | `sync` | `onboarding`

### EntityKind (extensions)

004 kinds plus:

`trace` | `monitor` | `provision-plan` | `provision-execution` | `team` | `invitation` | `agent-session` | `webhook` | `audit-event` | `sync-conflict` | `schedule` | `view` | `drift-item`

### DrawerState

```ts
type DrawerState =
  | { type: "process"; processId: string }
  | { type: "notifications" }
  | null;
```

### Tab catalog (defaults first)

| Section | Tabs |
| --- | --- |
| overview | _(none — single page)_ |
| search | _(category chips as filter, not tabs)_ |
| sync | `status`, `offline-queue`, `conflicts`, `history` |
| onboarding | _(stepper via wizardStep)_ |
| architecture | `canvas`, `components`, `dependencies`, `views`, `changes` |
| resources | `inventory`, `nodes`, `processes`, **`provisioning`** |
| ops | `runs`, `workflows`, `approvals`, `terminal` (+ schedules as secondary strip on workflows) |
| observability | `overview`, `logs`, `metrics`, `traces`, `monitors` |
| knowledge | `docs`, `decisions`, `runbooks`, `ownership` |
| access | `members`, **`teams`**, `agents`, `policies`, `audit`, **`invitations`** |
| integrations | `connected`, `catalog`, `credentials`, `sync`, **`webhooks`** |
| settings | `project`, **`organization`**, `environments`, `notifications`, `core`, **`developer`**, **`billing`**, **`security`**, **`data`** |

### Provisioning sub-tabs (resources + tab provisioning)

`catalog` | `executions` | `drift` — stored in `filter` or nested route field `subTab` (prefer `filter` with values `catalog|executions|drift` to avoid route field explosion).

### Transitions

| From | Event | To |
| --- | --- | --- |
| any | click logo / project name | section `overview`, page index |
| overview/shell | open project switcher | popover (not route) |
| shell | open notifications | drawer `{ type: "notifications" }` |
| palette | show all results | section `search`, search=query |
| footer core status | open sync | section `sync`, tab `status` |
| sync conflicts | open conflict | detail + entityKind `sync-conflict` |
| any nav | select destination | nav section + default tab |
| observability traces | open row | detail + `trace` |
| observability monitors | open row | detail + `monitor` |
| resources provisioning | start template | page wizard configure / plan |
| resources provisioning | open execution | detail + `provision-execution` |
| access teams | open team | detail + `team` |
| access agents | open session | detail + `agent-session` |
| access audit | open event | detail + `audit-event` |
| integrations | add provider | wizard page or multi-step detail |
| no project | app start | section `onboarding` |
| onboarding complete | project created | `overview` or `architecture` |
| process row | open | drawer process (unchanged) |

### History

Include: section, tab, page, entityId, entityKind, environmentId, search, filter, wizardStep.  
Exclude by default: dialog, drawer, selectedNodeId, project-switcher open state.

---

## Core view models

### ProjectOverviewSnapshot

| Field | Notes |
| --- | --- |
| projectId, projectName | Current System (UI Project) |
| environmentId, environmentLabel | e.g. Production |
| systemStatus[] | `{ tone, label }` e.g. services healthy, nodes connected |
| attention[] | `{ id, label, hrefKind, hrefId }` |
| activity[] | `{ id, title, target, actor, at }` |
| dataSource | honesty |

### ProjectSwitcherItem

| Field | Notes |
| --- | --- |
| projectId, name | |
| organizationId, organizationName | grouping |
| environmentSummary | Production / Development / Local only |
| isCurrent | boolean |

### NotificationItem

| Field | Notes |
| --- | --- |
| id | |
| title, body | |
| category | `approval` \| `health` \| `sync` \| `system` \| … |
| read | boolean |
| createdAt | |
| actionLabel | optional CTA |
| targetKind, targetId | deep link |
| dataSource | |

### SearchHit

| Field | Notes |
| --- | --- |
| id | |
| category | architecture \| resources \| operations \| knowledge \| … |
| title, subtitle | |
| statusLabel? | |
| entityKind, entityId | open target |
| section, tab | navigation target |

### SyncEndpointStatus

| Field | Notes |
| --- | --- |
| id | `local-core` \| `polter-cloud` \| provider id |
| name | |
| health | connected \| degraded \| offline \| unknown |
| lastSyncAt / lastHeartbeatAt | |
| detail? | last error summary |

### OfflineQueueItem

| Field | Notes |
| --- | --- |
| id | |
| summary | |
| createdAt | |
| entityKind?, entityId? | |
| status | pending \| failed \| syncing |

### SyncConflict

| Field | Notes |
| --- | --- |
| id | |
| resourceId, resourceName | |
| fieldLabel | e.g. Storage configuration |
| polterValue, polterUpdatedAt, polterSource | |
| providerValue, providerUpdatedAt, providerSource | |
| status | open \| resolved \| ignored |
| resolution? | keep-polter \| accept-provider \| ignore |

### ObservabilityServiceHealth

| Field | Notes |
| --- | --- |
| serviceId, name | |
| status | healthy \| degraded \| down |
| latencyP95Ms? | |
| errorRate? | |
| queueDepth? | workers |

### LogEventView

| Field | Notes |
| --- | --- |
| id | |
| timestamp, level, service, message | |
| traceId?, resourceId? | links |
| fields | structured map |

### TraceSummary / TraceDetail / SpanView

| Field | Notes |
| --- | --- |
| traceId | |
| entryService, durationMs, spanCount, status, startedAt | summary |
| spans[] | id, parentId?, service, operation, durationMs, status, resourceId? |

### MonitorView / AlertContext

| Field | Notes |
| --- | --- |
| id, name, targetLabel | |
| status | normal \| alerting \| muted |
| conditionLabel | human readable |
| currentValue, series[] | sparkline |
| lastCheckAt, alertCount | |
| relatedResourceId?, runbookId? | |
| recentErrorTraceCount?, matchingLogCount? | |

### ProvisioningTemplate

| Field | Notes |
| --- | --- |
| id, name, category | compute \| database \| messaging \| application |
| provider | |
| schemaSummary | fields for configure step |

### ProvisioningPlan

| Field | Notes |
| --- | --- |
| id | |
| templateId | |
| resources[] | planned creates |
| policyChecks[] | `{ status: pass\|warn\|fail, label }` |
| estimatedMonthlyCost? | number + currency label |
| status | draft \| pending-approval \| approved \| rejected |

### ProvisioningExecution

| Field | Notes |
| --- | --- |
| id, planId, name | |
| status | running \| succeeded \| failed \| cancelled |
| steps[] | `{ id, label, status, durationLabel?, output? }` |
| startedAt | |
| providerOperationUrl? | hand-off |

### DriftItem

| Field | Notes |
| --- | --- |
| id, resourceId, resourceName | |
| differenceLabel | |
| desired, actual | |
| severity | low \| medium \| high |
| detectedAt | |
| status | open \| accepted \| restored \| ignored |

### NodeInstallSession

| Field | Notes |
| --- | --- |
| id | |
| targetType | local \| remote \| docker \| kubernetes |
| tokenPreview | truncated display |
| tokenExpiresAt | |
| heartbeat | waiting \| received \| expired |
| installCommand | copyable |

### ArchitectureImportRow

| Field | Notes |
| --- | --- |
| discoveredId, discoveredName | |
| action | link-existing \| create-and-link \| ignore |
| componentId? | when linking |
| suggestedType? | |

### SnapshotCompareSummary

| Field | Notes |
| --- | --- |
| leftLabel, rightLabel | |
| added[], removed[], changed[] | component diffs |

### ArchitectureViewEdit

| Field | Notes |
| --- | --- |
| id, name, description | |
| includedComponentIds[] | |
| filters | environment, owner, … |
| layout | horizontal-flow \| … |

### WorkflowStepEdit / WorkflowSchedule

| Field | Notes |
| --- | --- |
| step id, type, name, config | approval fields when type=approval |
| schedule id, workflowId, cadenceLabel, nextRunAt, status, timezone | |
| retryPolicy?, notifyTeamId? | |

### Team / Invitation / RoleMatrixCell

| Field | Notes |
| --- | --- |
| team: id, name, memberCount, ownedCount, onCallName, updatedAt | |
| invitation: id, email, role, teamId?, status, expiresAt | |
| role matrix: permissionKey × roleKey → allow \| deny \| request-only | |

### AuditEventDetail

| Field | Notes |
| --- | --- |
| id, action, actorName, targetLabel | |
| at, source, requestId | |
| changes[] | field, before, after |
| integrity | verified?, previousHash?, currentHash? |

### AgentSessionDetail

| Field | Notes |
| --- | --- |
| id, agentId, agentName | |
| status | active \| completed \| terminated \| failed |
| objective | |
| steps[] | label, status |
| tools[] | name, decision allow\|deny |
| approvalId? | |
| startedAt | |

### WebhookView

| Field | Notes |
| --- | --- |
| id, name, destinationHost | |
| eventsSummary | |
| status | active \| failed \| paused |
| lastCallAt, lastResponseLabel | |
| signingSecretConfigured | boolean only |
| retryPolicyLabel | |

### OrganizationSettings / DeveloperStatus / BillingSnapshot / SecuritySession / RetentionPolicy

| Field | Notes |
| --- | --- |
| org: name, slug, logoUrl?, defaultRole, domains[], allowDomainJoin | |
| developer: cliInstalled, cliVersion?, mcpRunning, mcpEndpoint?, toolsExposed?, authRequired | |
| billing: planName?, priceLabel?, renewsAt?, meters[], estimatedInvoice?, available | |
| session: id, clientLabel, location, lastActiveAt, isCurrent | |
| retention: auditDays, operationOutputDays, logsLocalDays, logsCloudDays, snapshotsMode | |

### OnboardingState

| Field | Notes |
| --- | --- |
| step | org \| project \| connect \| architecture |
| orgMode | create \| join |
| projectName, description | |
| environments[] | |
| storage | local-first \| cloud |
| completed | boolean |

---

## Validation rules (UI-level)

- Install token expired → disable “I installed” success until regenerate; show waiting honestly.  
- Conflict resolve requires explicit radio selection before primary CTA.  
- Provision plan with failed policy checks cannot apply without override permission (default: block).  
- Webhook secret never rehydrated into form after create.  
- Billing meters show `current / limit`; unavailable edition hides fake plan.  
- Invitation expired cannot be accepted; resend creates new invite.  
- Agent terminate is confirm + audited.  
- Notification mark-all-read only affects items visible to actor.

---

## State machines (summary)

### ProvisioningExecution.status

`pending` → `running` → `succeeded` | `failed` | `cancelled`

### SyncConflict.status

`open` → `resolved` | `ignored` (ignored may reopen on next divergent sync)

### NodeInstallSession.heartbeat

`waiting` → `received` | `expired` → (regenerate) `waiting`

### AgentSession.status

`active` → `completed` | `terminated` | `failed`

### NotificationItem.read

`false` → `true` (no unread transition without new event)

### OnboardingState.step

`org` → `project` → optional `connect` / `architecture` → exit to overview

---

## Fixture honesty

Every list/detail view model SHOULD carry or inherit:

```ts
type DataSource = "core" | "fixture" | "empty" | "cached" | "unavailable";
```

Banners:

- fixture → “Demo data” / not live  
- cached → “Showing last known state”  
- unavailable → capability not in this edition / disconnected  
- empty → setup CTA where applicable
