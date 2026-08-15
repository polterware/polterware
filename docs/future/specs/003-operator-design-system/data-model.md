# Data Model: Operator Design System & Primary Surfaces

**Feature**: `003-operator-design-system`  
**Authority**: Presentation/view models in Desktop renderer + feature packages. Domain authority remains polterware-core (see `specs/002-architecture-big-bang/data-model.md`).  
**This document** defines UI-facing entities, route state, and fixture shapes—not a second persistence schema.

---

## Principles

1. **Card ≠ Resource**: Canvas cards are visual only; Resource is the operational entity.
2. **System is root**: Desktop may label System as Project only as transitional alias.
3. **Target ≠ System owner**: Machines are Targets when represented.
4. **Honest sources**: `dataSource: "core" | "fixture" | "empty"` must be knowable to UI for labeling.
5. **No silent dual types**: Prefer fields aligned with existing Connect resources/layout payloads; extend only in view models.

---

## ClientRoute (navigation state)

| Field | Type | Rules |
| --- | --- | --- |
| section | enum | Primary nav: at least `architecture`, `resources`; may include existing panels |
| page | `index` \| `detail` | Detail only when entityId set for sections that support detail |
| entityId | string? | Resource id or Architecture node id |
| environmentId | string? | Optional scope filter (e.g. `production`) |
| search | string | Default `""` |
| dialog | DialogState \| null | Focused create/edit/confirm |
| selectedNodeId | string? | Canvas selection without leaving index (ephemeral) |

### Transitions

| From | Event | To |
| --- | --- | --- |
| `*/index` | open entity | `*/detail` + entityId |
| `*/detail` | back / breadcrumb root | `*/index`, clear entityId |
| any | open create | dialog `{ type: "resource-create" }` |
| any | open edit | dialog `{ type: "resource-edit", entityId }` |
| dialog open | cancel / success | dialog null; on success may stay or go detail |
| architecture index | select node | selectedNodeId set; page remains index |
| architecture index | open node | page detail + entityId |

---

## DialogState

| Field | Type | Rules |
| --- | --- | --- |
| type | `resource-create` \| `resource-edit` \| `confirm` | Extend only when needed |
| entityId | string? | Required for edit |
| draft | ResourceDraft | Form fields |
| error | string? | Form-level error |
| submitting | boolean | Disable double-submit |

---

## Resource (view model)

Maps from core `listResources` + optional enrichment.

| Field | Type | Rules |
| --- | --- | --- |
| id | string | Required |
| name | string | Required, non-empty for create |
| type | string | e.g. Service, Database, Queue, Cache, Web App, Node, CI/CD |
| environment | string | Display scope; default from filter or `unknown` |
| provider | string | e.g. AWS, Vercel, GitHub, On-prem, Unknown |
| providerKind | string? | Optional finer grain (RDS, etc.) |
| status | ResourceStatus | See enum |
| lastSyncAt | string? | ISO or relative display derived |
| region | string? | Optional |
| owner | string? | Display name |
| systemId | string | System association |
| source | string? | Discovery/source from core |
| externalKey | string? | Provider key |
| notes | string? | Local/knowledge snippet |
| tags | string[] | Optional |
| linkedNodeIds | string[] | Architecture nodes |
| quickActions | QuickAction[] | Deep-links / ops entry points |
| dataSource | `core` \| `fixture` | Required for honesty |

### ResourceStatus

| Value | Presentation |
| --- | --- |
| `healthy` | Dot + “Healthy” text |
| `degraded` | Warning color + text |
| `down` | Danger + text |
| `unknown` | Muted + text |
| `syncing` | Info/muted + text |

Status **must not** be color-only (icon or text required).

### ResourceDraft (create/edit)

| Field | Validation |
| --- | --- |
| name | required, trimmed, max 128 |
| type | required enum/string |
| environment | required |
| provider | optional |
| notes | optional, max 2k |

Mutations that hit core MUST follow plan→review→approve→apply when governed; UI may start as draft-only/fixture write if core write path is not ready—must not claim applied remote provision.

---

## ArchitectureNode (view model)

| Field | Type | Rules |
| --- | --- | --- |
| id | string | Same id space as canvas node / resource when linked |
| label | string | Required |
| kind | `system` \| `resource` \| `environment` \| `service` \| … | Align with canvas + display |
| layer | `cloud` \| `applications` \| `data` \| `other` | View grouping for prototype bands |
| role | string? | e.g. payment processing |
| environment | string? | |
| runtime | string? | |
| owner | string? | |
| status | ResourceStatus | |
| resourceId | string? | Link to Resource when exists |
| connected | ConnectionRef[] | |
| activity | ActivityItem[] | |
| notes | string? | |
| docs | DocLink[] | |
| quickActions | QuickAction[] | |
| x, y | number | Canvas position |
| dataSource | `core` \| `fixture` | |

---

## ConnectionRef

| Field | Type | Rules |
| --- | --- | --- |
| nodeId | string | Required |
| label | string | Display name |
| relation | string | e.g. Upstream, Dependency, Queue, Database |
| evidence | boolean | If false, UI must not imply live wiring |

---

## ActivityItem

| Field | Type | Rules |
| --- | --- | --- |
| id | string | |
| at | string | Relative or absolute time |
| title | string | |
| detail | string? | |

---

## QuickAction

| Field | Type | Rules |
| --- | --- | --- |
| id | string | |
| label | string | |
| description | string? | |
| kind | `external` \| `navigate` \| `operation` | |
| href | string? | External hand-off |
| targetRoute | ClientRoute? | In-app navigate |
| operationType | string? | If operation; must go through safe path |
| enabled | boolean | False when provider missing |
| disabledReason | string? | |

---

## DocLink

| Field | Type |
| --- | --- |
| id | string |
| title | string |
| href or knowledgeId | string |

---

## CanvasModel (existing — reference)

Owned by `@polterware/frontend-canvas` (do not fork):

- `nodes: CanvasNode[]` — id, x, y, label, kind, source, externalKey  
- `edges: CanvasEdge[]` — id, from, to, kind, **evidence**  
- `revision: number`  
- `selection: string[]`

Layout persistence: core `getLayout` / `updateLayout` body JSON.

---

## EnvironmentOption

| Field | Type |
| --- | --- |
| id | string |
| label | string |
| statusDot | optional connectivity |

Used in page headers (Production selector in prototypes).

---

## ShellStatus

| Field | Type | Rules |
| --- | --- | --- |
| coreOnline | boolean | From health |
| nodesConnected | number? | If known |
| actorName | string? | Display |
| actorRole | string? | Display |

---

## DesignTokenSet (documentation entity)

Not runtime DB—documented in DESIGN.md and implemented in `base.css`:

- Color surfaces, text, border, brand, semantic  
- Radius, space, control sizes  
- Motion duration/easing  
- Z-index layers  
- Shell dimensions (sidebar width, main radius, header height)

---

## Fixture catalog

Named fixture set for offline visual acceptance (prototype-scale):

- ~8–12 Resources spanning types/providers  
- Architecture nodes + edges (mixed evidence)  
- One Resource detail and one Node detail fully populated  

Fixtures live in feature package or `packages/frontend` demo data module with explicit `dataSource: "fixture"`.

---

## Validation summary

| Entity | Key rules |
| --- | --- |
| ResourceDraft.name | non-empty |
| ClientRoute.detail | requires entityId |
| Edge display | evidence false → non-live styling |
| QuickAction.external | requires href or disable |
| Status | text or icon always |

---

## Out of model (this feature)

- Cloud Organization/Team tables  
- Full provider inventory schemas  
- Audit event persistence redesign  
- New protobuf messages unless a true API gap blocks list/detail (prefer view models)
)
