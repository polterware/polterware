# Inventory: Architecture Canvas (T001)

**Date**: 2026-08-03

## Current surface

| Path | Role | Disposition |
| --- | --- | --- |
| `packages/features/architecture/src/architecture-canvas.tsx` | Static band layout + CSS scale zoom | Replace with host (re-export) |
| `packages/features/architecture/src/index.tsx` | Tabs, model merge, detail, create dialog | Wire document v1 + host + autosave |
| `packages/frontend/canvas/src/index.ts` | Legacy CanvasModel serialize | Expand engine; keep legacy exports |
| `apps/desktop/src/renderer/App.tsx` | getLayout/updateLayout | Save port only; flush on navigate |

## Keep

- Connect Layout RPC (revision + body_json)
- Page-based node detail + create dialog entry
- Fixture honesty + FixtureBanner
- Design-system PageHeader/Tabs/Button

## Replace

- Band auto-layout as sole positioning
- Manual-only save as sole persistence path
- Non-interactive SVG edges

## No new packages

Expand `@polterware/frontend-canvas` only.
