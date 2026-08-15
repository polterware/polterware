# Baseline grep (pre/post implement)

**Date**: 2026-08-03

## After implement (done)

| Check | Result |
| --- | --- |
| `@kobalte/core` in packages + apps/desktop product | **empty** |
| `@ark-ui/solid` under packages/features | **empty** |
| `@ark-ui/solid` under design-system | dialog, popover, tabs, drawer, dropdown-menu |
| document keydown Escape hand-rolls | **empty** in features + DS |
| feature `aria-modal` / bare `role="dialog"` shells | **empty** |

## Manifests

- `@polterware/frontend-design-system`: `@ark-ui/solid` only (Kobalte removed)
- `apps/desktop`: Kobalte dep removed; vite config uses `@ark-ui/solid`
