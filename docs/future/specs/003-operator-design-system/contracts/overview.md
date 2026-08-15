# Contracts Overview: Operator Design System

**Feature**: `003-operator-design-system`  
**Kind**: UI / composition contracts (not new Connect services)

Backend Connect contracts remain those from `002-architecture-big-bang` / `packages/generated`. This feature adds **renderer contracts**: how shell, tokens, and feature views compose.

| Contract | File | Audience |
| --- | --- | --- |
| Design tokens | [design-tokens.md](./design-tokens.md) | CSS + DESIGN.md + components |
| Shell & navigation | [shell-navigation.md](./shell-navigation.md) | Shell feature + App composition |
| Resources UI | [resources-ui.md](./resources-ui.md) | Resources feature + App data wiring |
| Architecture UI | [architecture-ui.md](./architecture-ui.md) | Architecture feature + canvas |
| Core data adapter | [core-data-adapter.md](./core-data-adapter.md) | App ↔ `@polterware/frontend-data` |

### Non-goals

- No new public cloud API  
- No MCP surface changes  
- No protobuf schema churn unless blocked (escalate in tasks)
)
