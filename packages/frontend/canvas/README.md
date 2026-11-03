# @polterware/frontend-canvas

Browser-safe **canvas engine** for Polter Architecture (and future map surfaces).

## Layers

| Layer | Responsibility |
| --- | --- |
| `document/` | ArchitectureDocument v1, migrate, serialize, mutations |
| `viewport/` | Pan/zoom math, fit, culling rects |
| `selection/` | Click, marquee contain/intersect |
| `history/` | immediately / eventually / never capture |
| `clipboard/` | Graph-safe fragment copy/paste |
| `commands/` | Registry + shortcut matching |
| `hit-test/` | Priority hit testing |
| `spatial/` | Grid index |
| `render/` | Edge path helpers |
| `snap/` | Grid + object guides |
| `worker/` | Auto-layout off main thread (sync API + worker-ready) |

## Domain boundary

Engine does **not** import Architecture catalogs. Inject `DomainAdapter` from `@polterware/feature-architecture` for ports and connection rules.

## Persistence

Serialize with `serializeDocument` into core `Layout.body_json`. Concurrency is RPC `revision`, not body-embedded.

## Legacy

`CanvasModel` / `serialize` / `deserialize` remain for transitional tests; new code should use document v1 APIs.

## Non-goals

No Electron, no React, no Excalidraw runtime dependency.
