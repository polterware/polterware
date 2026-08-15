# Security smoke (T120)

**Date**: 2026-08-03

| Check | Result |
| --- | --- |
| `packages/frontend/canvas` Electron/FS imports | None (browser-safe engine) |
| `packages/features/architecture` Electron imports | None |
| Layout persistence | Connect `updateLayout` only |
| Import path | `validateImportJson` / `prepareImport` — no eval, sanitize text |
| Secrets in document | Forbidden by schema; no secret fields |
| Renderer bridge | Unchanged `window.polterware` |

Command used:

```bash
rg -n "electron|node:fs|ipcRenderer" packages/frontend/canvas/src packages/features/architecture/src || true
```
