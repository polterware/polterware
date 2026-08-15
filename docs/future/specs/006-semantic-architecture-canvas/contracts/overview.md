# Contracts Overview: Semantic Architecture Canvas

**Feature**: `006-semantic-architecture-canvas`  
**Kind**: Domain registry + authoring UX contracts on top of 005 engine

Wire-level Connect protos for Layout remain unchanged (`GetLayout` / `UpdateLayout`). This feature contracts **semantic authoring behavior**, **type registries**, **UI chrome**, and **session/persistence UX**.

| Contract | File | Audience |
| --- | --- | --- |
| Type registries & suggestions | [type-registry.md](./type-registry.md) | domain pure modules |
| Authoring flows (create/connect) | [authoring-flows.md](./authoring-flows.md) | ArchitectureCanvasHost |
| UI chrome (toolbars, menus, cards) | [ui-chrome.md](./ui-chrome.md) | feature-architecture UI |
| Persistence & session | [persistence-session.md](./persistence-session.md) | ArchitectureView + App |

### Inherited from 005 (still binding)

- Architecture document JSON body schema (`formatVersion: 1`)
- Canvas engine APIs (viewport, history, mutations, hit-test)
- Soft security: no secrets in document; unprivileged renderer

### Non-goals

- No new protobuf fields required for P1  
- No multiplayer protocol  
- No MCP layout mutation surface  
- No engine public API break without migrate helpers  
