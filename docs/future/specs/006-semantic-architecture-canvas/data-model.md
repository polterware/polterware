# Data Model: Semantic Architecture Canvas

**Feature**: `006-semantic-architecture-canvas`  
**Base**: Architecture Document v1 from `005` (`packages/frontend/canvas/src/document/types.ts`)  
**Persistence**: Opaque `body_json` + `revision` via `ArchitectureService` (unchanged wire)

---

## 1. Architecture Document (persisted)

Unchanged container:

| Field | Type | Notes |
| --- | --- | --- |
| formatVersion | `1` | Keep; additive type IDs only |
| id | string | Document id |
| systemId | string | System (Project) root |
| nodes | ArchNode[] | Semantic components |
| relationships | Relationship[] | Typed edges |
| boundaries | Boundary[] | Organization regions |
| viewport | Viewport | x, y, zoom |
| preferences | CanvasPreferences | grid, snap, minimap, routing |
| activeEnvironmentId | string \| `"all"` | Optional filter default |
| createdAt / updatedAt | ISO string | Optional |
| meta | object | migrate/generator hints |

### Validation (persist)

- Every node has non-empty `name` (after confirm) and known-or-custom `type`
- Every relationship has `sourceNodeId`, `targetNodeId`, **required** `type`
- Referential integrity: edge endpoints exist; optional prune on load
- No secrets in `metadata` or labels
- Body size ≤ `MAX_BODY_BYTES` (10 MiB)

---

## 2. ArchNode (component)

| Field | Required | Notes |
| --- | --- | --- |
| id | yes | Stable |
| name | yes on commit | Sanitized ≤ 200 |
| type | yes | Registry id (`NodeType` union extended) |
| state | yes | See state machine |
| x, y, width, height, zIndex | yes | Geometry |
| origin | yes | `manual` \| `discovered` |
| ports | optional | Defaults from type registry |
| environmentIds | optional | Multi env |
| provider | optional | Display |
| resourceId / externalKey | optional | Link to Resource |
| linkStatus | optional | unlinked \| linked \| partial |
| boundaryId | optional | Membership |
| locked / layoutPinned | optional | Edit/layout constraints |
| subtype | optional | e.g. postgres under database |
| description, owner, tags, links | optional | Non-secret |
| metadata | optional | Non-secret key/values |

### Node state machine (honest)

```text
manual create, no resource  → planned
associate resource          → linked
integration confirms        → observed
stale threshold exceeded    → stale
observed config ≠ model     → drifted
integration cannot reach    → unreachable
cannot determine            → unknown
```

**Rules**:

- Do not default every node to `planned` if origin is discovered (prefer `observed`/`linked` when evidence exists).
- Manual unlinked → `planned`.
- Canvas must not invent health metrics.

---

## 3. Relationship

| Field | Required | Notes |
| --- | --- | --- |
| id | yes | Stable |
| sourceNodeId | yes | |
| targetNodeId | yes | |
| type | **yes** | Registry id — **never empty on persist** |
| sourcePortId / targetPortId | optional | |
| label | optional | Inline editable |
| protocol | optional | e.g. https, amqp |
| direction | yes | `forward` \| `bidirectional` (default forward) |
| state | optional | Free/light enum if needed |
| evidence | optional | boolean honesty cue |
| style | optional | default \| emphasis \| muted |
| metadata | optional | Non-secret |

**Invariants**:

- No persist without explicit type (popover confirm or quick-mode preselect).
- Notes cannot be endpoints (`canConnect` false).
- Self-edges rejected unless type explicitly allows (default: reject).

---

## 4. Boundary

Unchanged from 005; kinds: system, domain, context, team, deployment, region, custom.

Organization catalog items **System Boundary / Domain Boundary / …** may create:

- either a **Boundary** entity (preferred for regions), or  
- a **node** of type `note` / future `*_boundary` marker  

**Decision for plan**: Catalog **boundary kinds** create **Boundary** entities via existing `addBoundary`; **Text Note** creates a note node; **Team Boundary** etc. map to `Boundary.kind`.

---

## 5. Component type registry (not persisted in document)

Authoritative in feature domain (see contracts/type-registry.md).

| Field | Purpose |
| --- | --- |
| id | Stable NodeType id |
| label | UI name |
| category | applications \| data \| infrastructure \| development \| external \| organization \| connected |
| description | Short |
| icon | Hugeicons key or component id |
| defaultWidth / defaultHeight | Placement size |
| defaultPorts | Port[] |
| searchTerms / synonyms | Search index |
| order | Selector sort |
| appearance | compact/normal hints |
| allowedRelationshipTypes | optional allowlist |
| requiredFields / optionalFields | future forms |

### Initial catalog mapping (illustrative)

Prefer reuse of existing IDs:

| Catalog label | Persisted type id |
| --- | --- |
| Web Application | `web_application` |
| API | `api` |
| Service | `service` |
| Worker | `worker` |
| Database | `database` |
| Cache | `cache` |
| Queue | `queue` |
| Event Stream | `event_stream` |
| Object Storage | `storage` |
| Repository | `repository` |
| Runtime / Server / VM / Container | `runtime_host` (+ subtype) |
| External Service | `external_service` |
| Integration | `integration` |
| Custom Resource | `custom_resource` |
| Text Note | `note` |
| System (root) | `system` |

**New IDs to add** (when product brief requires distinct semantics):

- Applications: `mobile_application`, `desktop_application`, `scheduled_job`, `serverless_function`
- Data: `search_engine`, `data_warehouse`
- Infra: `load_balancer`, `api_gateway`, `cdn`, `network`, `dns`, `region` (region may also be boundary)
- Dev: `package`, `ci_pipeline`, `build_system`, `artifact_registry`
- External: `saas_provider`, `third_party_api`, `auth_provider`, `payment_provider`

Connected-resources catalog rows are **actions** (not node types): link existing resource, import integration, import file.

---

## 6. Relationship type registry (not persisted as definitions)

| id | Label |
| --- | --- |
| calls | Calls |
| depends_on | Depends on |
| reads_from | Reads from |
| writes_to | Writes to |
| publishes_to | Publishes to |
| consumes_from | Consumes from |
| stores_in | Stores in |
| authenticates_with | Authenticates with |
| routes_to | Routes to |
| deploys_to | Deploys to |
| hosted_on | Hosted on (**new**) |
| monitors | Monitors |
| replicates_to | Replicates to (**new**) |
| synchronizes_with | Synchronizes with (**new**) |
| exposes | Exposes (**new**) |
| contains | Contains (existing; boundaries/system) |
| related | Related (existing weak) |
| custom | Custom |

---

## 7. Suggestion rules (pure domain)

`suggestRelationshipTypes(sourceType, targetType): RelationshipType[]` ordered.

Examples (from spec):

| Source | Target | Suggested first |
| --- | --- | --- |
| web_application / mobile_application | api | calls |
| api / service | database | reads_from |
| service | queue | publishes_to |
| worker | queue | consumes_from |
| *application* | auth_provider | authenticates_with |
| repository | runtime_host | deploys_to |
| api_gateway | service | routes_to |
| service | storage | stores_in |
| *monitor* | *application* | monitors |

Fallback: `[depends_on, calls, custom]`.

---

## 8. Ephemeral (host-only, not in body_json)

| Entity | Fields | Lifetime |
| --- | --- | --- |
| PlacementSession | typeId, preview point | until place or Escape |
| ProvisionalNode | typeId, x, y, draftName | until Enter/Escape |
| PendingRelationship | source, target, ports, ghost path | until type confirm/Escape |
| ConnectQuickMode | relationshipTypeId | until Escape / tool change |
| DependencyHighlight | nodeIds, relationshipIds | until clear |
| TypeRecents / Favorites | type ids | localStorage |

---

## 9. Session snapshot (in-memory)

| Field | Purpose |
| --- | --- |
| viewport | Restore pan/zoom |
| selection | node/rel/boundary ids |
| environmentId | Filter restore |

Key: `systemId`. Not a multi-device sync document.

---

## 10. Persistence status (UI)

`clean | dirty | saving | saved | error` — existing autosave controller.

Transitions:

```text
mutation confirm → dirty → (debounce) saving → saved → clean
                              ↘ error (local doc retained; retry → saving)
```

Provisional/pending must **not** trigger dirty.

---

## 11. Migration strategy

1. Load via existing `deserializeAndMigrate`.
2. Unknown node/rel type ids: keep string if union widened to include `string` fallback **or** map to `custom_resource` / `custom` with original id in `subtype`/`metadata.originalType`.
3. Prefer **widen unions** to full catalog for type safety.
4. Legacy generic `"New component"` names: leave as-is; no forced rename.
5. Existing inferred edges keep their types (already set).
6. No layout table migration beyond body content.

---

## 12. Clipboard fragment

Reuse 005 fragment: nodes + internal relationships + boundaries; on paste allocate new ids and rewire endpoints. External dangling edges dropped.
