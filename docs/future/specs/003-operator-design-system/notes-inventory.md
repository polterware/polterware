# Inventory — Operator Design System

**Date**: 2026-08-02

## Current (replace / restyle)

| Path | Role | Disposition |
| --- | --- | --- |
| `packages/features/shell/src/index.tsx` | Provisional top bar + side nav | **Replace** with integrated sidebar + MainSurface |
| `packages/features/resources/src/index.tsx` | Stub list | **Replace** with index + detail + dialog |
| `packages/features/architecture/src/index.tsx` | Canvas + permanent ResizableSplit inspector | **Replace** inspector with page detail; keep canvas model |
| `apps/desktop/src/renderer/App.tsx` | Panel switch + core wiring | **Extend** with ClientRoute; keep data adapters |
| `packages/configuration/tailwind/base.css` | Sparse dark tokens + blue accent | **Rewrite** quiet dark ladder + brand green restricted |
| `packages/frontend/design-system/*` | Texture button, minimal primitives | **Restyle** quiet controls |
| `DESIGN.md` | Light alpha design system | **Rewrite** operator dark SSOT |

## Reuse

- `@polterware/frontend-canvas` model serialize/select/buildOperationalMap
- `@polterware/frontend-data` Connect client
- `@tanstack/solid-virtual` (desktop)
- `packages/frontend/canvas/worker/layout.ts`
- Feature stubs (ops, logs, …) — restyle host only

## Packages changed (no new packages)

`frontend-design-system`, `feature-shell`, `feature-resources`, `feature-architecture`, `frontend-canvas` (if needed), `configuration/tailwind`, `apps/desktop`, root `DESIGN.md`

## Dependencies

- `@hugeicons/core-free-icons` added to design-system, shell, resources, architecture
- `@kobalte/core` deferred (native dialog sufficient for v1)
