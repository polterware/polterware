# Contracts Overview: Operator Product Surfaces

**Feature**: `004-operator-product-surfaces`  
**Kind**: UI / composition contracts (not new public cloud APIs)

Backend Connect contracts remain under `schemas/polterware/local/v1/*` and generated clients in `@polterware/frontend-data` / Go gen. This feature documents **renderer contracts**: navigation, page composition, and feature view boundaries.

| Contract | File | Audience |
| --- | --- | --- |
| Shell & IA | [shell-ia.md](./shell-ia.md) | Shell + App composition |
| Page tabs & states | [page-tabs-states.md](./page-tabs-states.md) | All destinations + design-system |
| Architecture UI | [architecture-ui.md](./architecture-ui.md) | Architecture feature |
| Resources UI | [resources-ui.md](./resources-ui.md) | Resources feature |
| Operations UI | [operations-ui.md](./operations-ui.md) | Operations feature |
| Knowledge UI | [knowledge-ui.md](./knowledge-ui.md) | Knowledge feature |
| Access UI | [access-ui.md](./access-ui.md) | Access feature |
| Integrations UI | [integrations-ui.md](./integrations-ui.md) | Integrations feature |
| Settings UI | [settings-ui.md](./settings-ui.md) | Settings feature |
| Overlays | [overlays.md](./overlays.md) | Dialogs, drawer, command palette |
| Core data adapter | [core-data-adapter.md](./core-data-adapter.md) | App ↔ frontend-data + fixtures |

### Non-goals

- No new MCP surface by default  
- No protobuf schema churn unless blocked (document gap in tasks; prefer fixtures)  
- No specialized-provider admin API clone
