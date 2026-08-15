# Data Model: Architecture Node Detail

**Feature**: 026-architecture-node-detail

## ArchitectureNodeView (detail projection)

| Field | Meaning | Chip? |
| --- | --- | --- |
| id, label | Identity | — |
| kind | Component type / kind label | drives domain icon |
| layer | Architecture layer | text |
| role, environment, runtime, owner | Overview facts | text |
| lifecycle | Lifecycle posture label or state key | **StatusChip** |
| linkStatus | unlinked \| linked \| partial | **StatusChip** |
| resourceId, resourceName | Linked inventory resource | companion text for link |
| canLinkResource | Gate for Link resource action | — |
| status | Optional coarse health (not primary lifecycle) | optional; do not replace lifecycle |
| connected[] | ConnectionRef list | evidence chip per row |
| docs[], activity[] | Secondary tables | — |
| dataSource | core \| fixture | honesty when shown |
| x, y | Layout residue (not primary detail content) | — |

## ConnectionRef

| Field | Meaning |
| --- | --- |
| nodeId, label | Related architecture node |
| relation | Relation type label |
| evidence | boolean → Proven / Unproven StatusChip |

## Label / tone mapping (helpers)

- Lifecycle: Planned / Linked / Observed / Stale / Drifted / Unreachable / Unknown → tones from `nodeLifecycleTone` (case-normalized).
- Link: Linked / Unlinked / Partial → `linkStatusTone`; display may append resource name when linked.
- Evidence: Proven (healthy/ok) vs Unproven (warn/muted).
