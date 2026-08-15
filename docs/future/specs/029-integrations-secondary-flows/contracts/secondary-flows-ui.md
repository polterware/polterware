# Contract: Integrations Secondary Flows UI (mock)

## Surfaces

| Surface | Entry | Primary actions | Detail hand-off |
| --- | --- | --- | --- |
| Setup wizard | Catalog connect / reconfigure | Next, Back, Cancel, Test connection, Finish | Connected + IntegrationDetail when fixture matches |
| Webhooks tab | Satellite / export | Create webhook (mock), Send test event | Row inline detail only (entity detail stays IntegrationDetail) |
| Sync center | Export / satellite | Sync now, open conflict | Conflict resolve overlay |
| Conflict resolve | Sync conflicts row | Resolve with choice, Cancel | Returns to conflict list |

## Status grammar

| Domain value | StatusChip tone (typical) |
| --- | --- |
| connected / active / success | healthy |
| degraded / partial / pending / running / stale | warn or syncing |
| failed / error / unauthorized | danger or down |
| paused / disconnected / ignored | muted |

## Conflict choices

```text
keep-local | keep-remote | merge
```

No resolve without explicit choice. Cancel leaves unresolved.

## Secrets

- Access keys / webhook signing secrets: write-only on create.
- List and detail show configured/missing only.
