# Contract: Persistence & Autosave

**Authority**: polterware-core `ArchitectureService.UpdateLayout`  
**Client**: `@polterware/frontend-data` `updateLayout` + App composition  
**Controller**: feature-architecture autosave module (injectable port)

## Port interface

```text
type LayoutSavePort = {
  load(systemId: string): Promise<{ revision: number; bodyJson: string }>
  save(systemId: string, expectedRevision: number, bodyJson: string):
    Promise<{ revision: number; bodyJson: string }>
}
```

Desktop implements via Connect. Tests inject memory port.

## Autosave policy

| Event | Action |
| --- | --- |
| Document mutation (history immediate commit) | Mark dirty; schedule debounce **800ms** |
| Viewport-only change | Mark viewport dirty; debounce **2000ms** or piggyback |
| Debounce fire | status=saving; call save; on success status=saved + revision |
| Route leave / detail open | flush pending save (await) |
| Window close | flush best-effort |
| Conflict | status=error; do not clobber local; user chooses reload vs confirm overwrite |
| Offline / core down | status=error; retry optional; UI honest |

## Status enum

`clean` | `dirty` | `saving` | `saved` | `error`

## Non-goals

- New Electron IPC file persistence  
- Saving history stacks  
- Saving ephemeral interaction state  
- Discovery rewriting body  

## Security

Validate/serialize client-side before send. Renderer never writes SQLite directly.  
