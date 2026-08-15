# Contract: Integrations UI

**Owner**: `packages/features/integrations`  
**Consumers**: `App.tsx`

## Tabs

`connected` | `catalog` | `credentials` | `sync`

## Connected

- Cards: provider, counts, status, last sync  
- Open → integration detail page  
- Add integration → catalog or connect dialog

## Catalog

- Grouped providers; search  
- Unavailable connectors: disabled or “coming soon”—not fake connected

## Integration detail

- Account facts, imported counts by type, permissions, Sync now, Open console (external hand-off)

## Credentials

- List: name, provider, scope, status, rotated  
- **Never** render secret material after create  
- Create dialog: write-only secret field cleared after submit

## Sync

- Rows: provider, last sync, duration, imported, changes, result  
- Sync all / configure interval when permitted  
- Partial/failed results must not display as full health
