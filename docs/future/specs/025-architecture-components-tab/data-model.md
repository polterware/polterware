# Data Model: Architecture Components Tab

## Entities

### ArchitectureComponentRow (list projection)

| Field | Source | Notes |
| --- | --- | --- |
| id | ArchNode.id | Stable identity shared with canvas |
| name | ArchNode.name | PrimaryCell title |
| type | ArchNode.type | Display via registry label when known |
| subtype | ArchNode.subtype | Icon resolution |
| provider | ArchNode.provider | Icon resolution |
| lifecycle | ArchNode.state → label | StatusChip |
| linkStatus | ArchNode.linkStatus or derived from resourceId | linked / unlinked / partial |
| environment | first environmentIds → label | Display label |
| owner | ArchNode.owner | Optional — |

Not stored separately — always derived from `ArchitectureDocument.nodes`.

### ArchitectureDocument (existing SSOT)

Owned by canvas document types (`@polterware/frontend-canvas`). Components tab is a read projection; placing nodes on canvas mutates the same document and appears on Components.

### Detail target

```text
scopeId: "architecture:components"
entityId: <node id>
entityKind: "arch-node"
```

## Filters (session UI only)

- lifecycle: set of NodeState values
- type: set of NodeType strings present in document
- environment: set of environment ids

Not persisted across restarts unless shell filters already handle that (out of scope).
