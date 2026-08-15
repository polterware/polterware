# Visual pass notes

**Date**: 2026-08-02  
**Method**: Static implementation vs DESIGN.md + prototype composition; `pnpm build` for Desktop renderer succeeded.

## Automated

- Desktop production build: **pass** (electron-vite main/preload/renderer)
- Unit tests (filter, map-view, canvas): **pass**
- Pre-existing desktop Main IPC/updater test failures: **unchanged** (out of feature scope)

## Manual (user-run)

Run `pnpm dev:desktop` when ready and verify:

1. Integrated sidebar + rounded MainSurface  
2. Resources index → detail page → create dialog  
3. Architecture canvas without permanent inspector → node detail page  
4. Neutral selection; brand green only on status dots  
5. Keyboard focus-visible on nav and tables  

Agent did not start long-lived `pnpm dev` per Agents.md policy.
