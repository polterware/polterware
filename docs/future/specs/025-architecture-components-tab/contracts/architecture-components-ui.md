# Contract: Architecture Components UI

## Tabs

| Tab id | Label | Surface |
| --- | --- | --- |
| canvas | Canvas | Architecture canvas host |
| components | Components | Components DataTable (this feature) |
| views | Views | Views table (existing) |

Normalization:

- `components` → `components` (MUST NOT become `canvas`)
- Retired: `dependencies` may map to `canvas` (legacy)
- `changes` owned by shell routing (activity) when resolved at shell layer

## Collection

- `scopeId`: `architecture:components`
- `entityKind`: `arch-node`
- `entityId`: architecture node id from document

## List columns (minimum)

1. Name — PrimaryCell + domain icon by type
2. Type — text label
3. Lifecycle — StatusChip
4. Environment — text
5. Link or Owner — StatusChip for link and/or text owner

Forbidden: Actions column.

## Detail

Reuse `ArchitectureNodeDetail`. No second entity page.

## Data

Rows ⊆ `ArchitectureDocument.nodes` for active system load path (fixture or core).
