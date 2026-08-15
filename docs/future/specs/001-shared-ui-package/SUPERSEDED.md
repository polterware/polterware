# Superseded

This feature extracted React design primitives into `packages/ui` (`@polterware/ui`).

That path was **removed** by `specs/002-architecture-big-bang`. Product UI is now:

- Solid design system: `packages/frontend/design-system` (`@polterware/frontend-design-system`)
- Solid features: `packages/features/*`
- Desktop composition: `apps/desktop/src/renderer/`

Keep this folder as Speckit history only. Do not reintroduce `packages/ui` or a React product renderer.
