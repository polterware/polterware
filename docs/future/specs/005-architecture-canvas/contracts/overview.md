# Contracts Overview: Architecture Canvas Engine

**Feature**: `005-architecture-canvas`  
**Kind**: Document format + engine/domain APIs + UI composition + persistence adapter

Wire-level Connect protos for Layout remain unchanged (`GetLayout` / `UpdateLayout`). This feature contracts the **body_json schema** and renderer APIs.

| Contract | File | Audience |
| --- | --- | --- |
| Architecture document JSON | [architecture-document.md](./architecture-document.md) | serialize/migrate/import/export |
| Canvas engine API | [canvas-engine-api.md](./canvas-engine-api.md) | `@polterware/frontend-canvas` |
| Architecture domain + UI | [architecture-domain-ui.md](./architecture-domain-ui.md) | feature-architecture |
| Persistence / autosave | [persistence-autosave.md](./persistence-autosave.md) | App + feature |
| Commands & shortcuts | [commands-shortcuts.md](./commands-shortcuts.md) | engine + shell |

### Non-goals

- No new cloud multiplayer protocol  
- No MCP layout mutation surface in this feature  
- No protobuf field changes required for P1 (body remains string)  
- No Electron IPC expansion for persistence  
