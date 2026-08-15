# Data Model: Architecture Canvas Engine

**Feature**: `005-architecture-canvas`  
**Format version**: `1`  
**Persistence**: `ArchitectureService` Layout.`body_json` (opaque string) + RPC `revision`  
**Related**: [research.md](./research.md) · [contracts/architecture-document.md](./contracts/architecture-document.md)

---

## 1. Authority and layers

| Layer | Storage | Lifetime |
| --- | --- | --- |
| **ArchitectureDocument** | SQLite via core `architecture_layouts.body_json` | Durable |
| **Layout.revision** | Core column (optimistic concurrency) | Durable |
| **Session canvas cache** | Renderer memory (Map by systemId) | Until app quit |
| **History stacks** | Renderer memory | Until document close/reload |
| **InteractionState** | Renderer memory | Ephemeral |

Domain root remains **System**. UI may label Project as alias.

---

## 2. ArchitectureDocument (formatVersion = 1)

```text
ArchitectureDocument
├── formatVersion: 1
├── id: string                    # document id (may match layout id)
├── systemId: string
├── workspaceId?: string
├── activeEnvironmentId?: string | "all"
├── nodes: Node[]
├── relationships: Relationship[]
├── boundaries: Boundary[]
├── preferences: CanvasPreferences
├── viewport: Viewport
├── createdAt?: string (ISO)
├── updatedAt?: string (ISO)
└── meta?: { migratedFrom?: number; generator?: string }
```

**Invariants**

- All node/relationship/boundary ids unique within the document.
- Relationship endpoints reference existing node ids (or are dropped on migrate with log).
- `boundaryId` on a node references an existing boundary or is null.
- No secrets, tokens, or credential material in any field.
- `formatVersion` required after migration; legacy bodies without it migrate as 0→1.

---

## 3. Node

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| id | string | yes | Stable opaque id |
| name | string | yes | Display name; sanitized text |
| description | string | no | Short |
| type | NodeType | yes | Registry key |
| subtype | string | no | Extensible |
| origin | `manual` \| `discovered` | yes | Default manual |
| provider | string | no | aws, vercel, … |
| environmentIds | string[] | no | Empty = unspecified / all |
| state | NodeState | yes | Default `unknown` |
| x, y | number | yes | Document coordinates (top-left) |
| width, height | number | yes | Defaults by type |
| zIndex | number \| string | yes | Order key (integer v1; fractional later if needed) |
| boundaryId | string \| null | no | Explicit membership |
| collapsed | boolean | no | Node compact UI |
| linkStatus | `unlinked` \| `linked` \| `partial` | no | |
| resourceId | string \| null | no | Link to Resource |
| externalKey | string \| null | no | Discovery key |
| owner | string | no | |
| tags | string[] | no | |
| links | { label, href }[] | no | Non-secret URLs |
| locked | boolean | no | Skip drag |
| layoutPinned | boolean | no | Preserve in auto-layout |
| ports | Port[] | no | If omitted, type default ports apply |
| metadata | Record<string, string \| number \| boolean> | no | Non-secret only |

### NodeType (initial registry)

`system` · `web_application` · `api` · `service` · `worker` · `database` · `cache` · `queue` · `event_stream` · `storage` · `repository` · `runtime_host` · `external_service` · `integration` · `custom_resource`

Unknown types on migrate → `custom_resource` + preserve original in `metadata.legacyType`.

### NodeState (initial)

`planned` · `linked` · `observed` · `stale` · `drifted` · `unreachable` · `unknown`

### Port

| Field | Type | Notes |
| --- | --- | --- |
| id | string | Stable per node |
| direction | `in` \| `out` \| `both` | |
| side | `left` \| `right` \| `top` \| `bottom` \| `auto` | |
| label | string | optional |
| maxConnections | number \| null | optional rule |

---

## 4. Relationship

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| id | string | yes | |
| sourceNodeId | string | yes | |
| targetNodeId | string | yes | |
| sourcePortId | string | no | |
| targetPortId | string | no | |
| type | RelationshipType | yes | |
| label | string | no | |
| direction | `forward` \| `bidirectional` | yes | default forward |
| protocol | string | no | http, grpc, … |
| state | string | no | free or enum later |
| evidence | boolean | no | true = observed/evidence-backed |
| style | `default` \| `emphasis` \| `muted` | no | semantic hint only |
| zIndex | number | no | |
| path | optional waypoints | no | v1 may auto-route without stored waypoints |
| metadata | object | no | non-secret |

### RelationshipType (initial)

`depends_on` · `calls` · `reads_from` · `writes_to` · `publishes_to` · `consumes_from` · `authenticates_with` · `deploys_to` · `stores_in` · `monitors` · `routes_to` · `contains` · `related` · `custom`

Legacy edge kinds map: `contains` → `contains`, `related` → `related`.

**Validation**

- No edge without endpoints.
- Optional rule table may reject type pairs (e.g. invalid self-loop if disabled).
- Evidence false ⇒ visual must not claim live runtime wiring (DESIGN / constitution).

---

## 5. Boundary

| Field | Type | Notes |
| --- | --- | --- |
| id | string | |
| name | string | title |
| kind | `system` \| `domain` \| `context` \| `team` \| `deployment` \| `region` \| `custom` | |
| x, y, width, height | number | geometry |
| zIndex | number | behind nodes |
| collapsed | boolean | |
| locked | boolean | |
| memberNodeIds | string[] | optional denormalized; **source of truth is node.boundaryId** (keep in sync on write) |
| colorToken | string | optional neutral token key only |

Environment is **not** only a boundary; filter dimension is separate.

---

## 6. Viewport

| Field | Type | Notes |
| --- | --- | --- |
| x | number | document offset (scrollX equivalent) |
| y | number | |
| zoom | number | clamped e.g. 0.1–4 |
| focusedElementId | string \| null | optional |

Restored on load and detail return (session cache + document).

---

## 7. CanvasPreferences

| Field | Type | Default |
| --- | --- | --- |
| gridEnabled | boolean | false |
| snapToGrid | boolean | true when grid on |
| snapToObjects | boolean | true |
| gridSize | number | 16 |
| showMinimap | boolean | true |
| edgeRouting | `orthogonal` \| `bezier` \| `straight` | `orthogonal` |

---

## 8. InteractionState (non-persisted)

```text
activeTool: select | hand | connect | add_node | add_boundary | add_note
selection: { nodeIds, relationshipIds, boundaryIds }
hoverId / hoverKind
marquee: bounds | null
drag: { mode, origin, snapshot } | null
connectPreview: { from, pointer } | null
editingText: { targetId, kind } | null
contextMenu: { x, y, target } | null
guides: GuideLine[]
clipboardPasteCount: number
```

---

## 9. History

```text
HistoryState
├── undoStack: Transaction[]
├── redoStack: Transaction[]
└── limit: number

Transaction
├── id
├── label?: string
├── timestamp
├── patches: Patch[]          # per-entity field before/after
└── selectionBefore / selectionAfter?: SelectionSnapshot
```

Capture: `immediately` | `eventually` | `never` (research R4).

---

## 10. Legacy model (format 0)

Current `@polterware/frontend-canvas`:

```ts
{ nodes: CanvasNode[], edges: CanvasEdge[], revision?: number }
// CanvasNode: id, x, y, label, kind?, source?, externalKey?
// CanvasEdge: id, from, to, kind: contains|related, evidence: boolean
```

### Migration 0 → 1

| Legacy | v1 |
| --- | --- |
| node.label | name |
| node.kind system/resource/environment | type mapping (system→system, resource→custom_resource, environment→ metadata) |
| node.x/y | x/y; default width/height |
| edge | relationship with type from kind; sourceNodeId/targetNodeId from from/to |
| revision in body | ignored for concurrency (RPC wins); strip or keep in meta |
| missing viewport | `{ x:0, y:0, zoom:1 }` then optional fit-on-first-open flag in session |

---

## 11. External entities (existing product)

Not owned by canvas format but linked:

| Entity | Link field |
| --- | --- |
| System | document.systemId |
| Resource | node.resourceId |
| Environment | node.environmentIds / activeEnvironmentId |
| Evidence | relationship.evidence + future evidence ids in metadata |

Discovery must **not** rewrite document positions/relationships silently (existing domain rule).

---

## 12. State transitions (document lifecycle)

```text
empty → (create node) → dirty → saving → saved
saved → (edit) → dirty → …
saving → error → dirty (retry)
load → migrating? → ready
import validate → preview → replace (backup) → ready
```

UI statuses: `empty` | `loading` | `ready` | `dirty` | `saving` | `saved` | `error` | `invalid`.

---

## 13. Validation rules (load/import)

1. JSON parse succeeds.  
2. `formatVersion` known or migratable.  
3. Arrays present; ids unique.  
4. Endpoints resolve or edges dropped with warning list.  
5. Strings sanitized (no HTML script).  
6. Numbers finite.  
7. Reject executable fields (none allowed).  
8. Max size guard (plan: e.g. reject > 10MB body with error).  

Invalid import: **do not** replace current document.

---

## 14. Serialization

- Canonical JSON, stable key order optional (not required for equality).  
- `serialize(doc) → string` for RPC.  
- `deserialize(json) → { doc, warnings }` with migrate.  
- Clipboard fragment: subset `{ formatVersion, nodes, relationships, boundaries }`.  

RPC `Layout.revision` is **outside** the document body for concurrency.
