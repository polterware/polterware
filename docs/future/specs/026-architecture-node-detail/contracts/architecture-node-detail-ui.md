# Contract: Architecture Node Detail UI

**Feature**: 026-architecture-node-detail  
**Surface**: Architecture canvas + Components → global DetailPanel (`entityKind: arch-node`)

## Host → Detail props

| Prop | Required | Behavior |
| --- | --- | --- |
| node | for ready state | `ArchitectureNodeView` projection |
| loading | optional | PageLoading in sections layout |
| error | optional | PageError |
| notFound | optional | Node not found |
| onOpenRelated | optional | Activate related architecture node |
| onOpenOnCanvas | optional | Header action Open on canvas |
| onLinkResource | optional | Header action Link resource when `canLinkResource` |

## Visual contract

1. Header: domain icon + title; actions Open on canvas / Link resource without regression.
2. Overview: StatusChip for lifecycle and link; dense environment/role/runtime/owner/layer.
3. Connected: DataTable; Evidence column StatusChip Proven/Unproven; no Actions column.
4. Docs + Activity: DataTable; honest empties.
5. No second inspector/page/drawer for entity detail.

## Non-goals

Canvas engine, place flow, view-detail, components-table rewrites; live Core mutations beyond existing host link mock.
