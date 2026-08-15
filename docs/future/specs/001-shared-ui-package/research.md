# Research: Shared UI Package

**Feature**: `001-shared-ui-package`  
**Date**: 2026-07-17

## R1 — Package identity and layout

**Decision**: Create `packages/ui` published in-workspace as `@polterware/ui`.

**Rationale**:
- Matches existing `@polterware/core` / `@polterware/desktop` naming.
- Aligns with `docs/ARCHITECTURE.md` target growth (`packages/ui`).
- Constitution VII explicitly maximizes shared design-system packages for Electron + Web.
- `pnpm-workspace.yaml` already includes `packages/*`.

**Alternatives considered**:
- `@polterware/design-system` — clearer long name, but architecture docs and conventions already say `ui`.
- Split `ui` + `design-tokens` now — premature; tokens can live as CSS entry inside `ui` until a second non-React consumer needs tokens alone.
- Put UI inside `@polterware/core` — rejected; core is non-visual/Node-heavy and must not grow renderer React deps.

## R2 — Source distribution model

**Decision**: Source-first workspace package (export TypeScript/CSS sources), same pragmatic model as `@polterware/core` for monorepo consumption; Desktop bundler resolves and bundles it.

**Rationale**:
- Desktop already resolves workspace packages with `preserveSymlinks: false` and can externalize/exclude as needed.
- Avoids premature dual build pipeline for a private monorepo package.
- Future Web (Vite/Next-like) can consume sources or a later build step without changing ownership.

**Alternatives considered**:
- Prebuilt `dist/` with `tsc` emit only — fine later; not required for MVP if typecheck works on sources.
- Publish to npm registry — out of scope; internal workspace dependency is enough.

## R3 — What moves in MVP vs stays Desktop

**Decision — moves into `@polterware/ui`**:
- All pure design primitives under `apps/desktop/src/renderer/components/ui/**` that are browser-safe (buttons, inputs, dialogs/sheets, menus, tabs, cards, feedback, form primitives, texture button/card, file-tree if purely presentational, etc.).
- `cn` helper (`lib/utils.ts`).
- Hooks required only by those primitives (e.g. `use-mobile` for sidebar).
- Shared design tokens and surface utility CSS needed by primitives (`polterware-surface`, `polterware-glass-surface`, `polterware-frosted-surface`, radius/theme variables currently in `styles.css` that define the design system).
- Co-located primitive CSS such as `texture-button.css`.

**Decision — stays Desktop-owned**:
- `shell.css` and Electron chrome (`app-region`, dock mechanics, window frame).
- `features/**` including process boards, commander composition, repositories, system map views.
- `features/shared/*` domain/helpers (SurfaceCard/EmptyState may later re-export package primitives but stay Desktop composition for MVP unless trivially pure and already multi-use).
- Main/preload/IPC/`window.polterware` adapters.
- Workbench layout ownership.

**Rationale**: Spec FR-007/FR-012 and constitution “specialize shells and authority adapters”; extract when shareable and justified, not force-share feature screens.

**Alternatives considered**:
- Move domain feature views now — rejected until second consumer exists.
- Move only a minimal subset (Button/Input/Dialog) — weaker; current tree is already one design kit and Desktop imports broadly.

## R4 — CSS / token ownership

**Decision**: Package owns design-system CSS entry (`@polterware/ui/styles` or equivalent export). Desktop `styles.css` becomes a thin consumer: Tailwind entry + package styles import + any truly desktop-only leftovers. `shell.css` remains Desktop.

**Rationale**:
- DESIGN.md requires single visual source of truth.
- Primitives already depend on CSS variables and `polterware-*` surface classes.
- Web readiness requires tokens not locked inside Desktop-only files forever.

**Alternatives considered**:
- Keep all tokens only in Desktop and document “Web must copy” — violates single source of truth and SC token consistency.
- Separate `packages/design-tokens` immediately — extra package without a non-React consumer yet.

## R5 — Import migration strategy

**Decision**:
1. Move implementations into `@polterware/ui`.
2. Point Desktop features at `@polterware/ui` (barrel and/or subpath exports).
3. Allow **temporary** thin re-export shims under `@/components/ui/*` and `@/lib/utils` only if needed to reduce churn, then delete shims once imports are updated.
4. Success condition: no duplicated primitive implementation remains under Desktop.

**Rationale**: SC-001 forbids parallel copies; thin re-exports are adapters, not copies. Mass import update is acceptable in one feature.

**Alternatives considered**:
- Forever keep `@/components/ui` as the only public path — hides shared package from Web consumers and muddies ownership.
- Codemod-only big bang with zero shims — also fine if typecheck stays green; shims are optional safety valve.

## R6 — Public API shape

**Decision**:
- Primary export: `@polterware/ui` barrel for stable primitives + `cn`.
- CSS export: dedicated style entry (e.g. `@polterware/ui/styles.css`).
- Prefer named exports matching current component names (`Button`, `Dialog`, …) to minimize semantic change.
- Do not export privileged or shell modules.
- Internal relative imports inside package; no `@/` Desktop aliases inside package source.

**Rationale**: Clear consumer contract for Desktop/Web; avoids leaking app path aliases into shared code.

**Alternatives considered**:
- Deep imports only (`@polterware/ui/button`) without barrel — good tree-shaking story later; barrel is fine for app source consumption with bundlers.
- Namespace export (`UI.Button`) — unnecessary API churn.

## R7 — Dependency placement

**Decision**: React/ReactDOM as **peerDependencies** of `@polterware/ui`. Primitive runtime libraries used only by the package become package dependencies (or peerDependencies when they must be singleton, e.g. React). Desktop continues to provide React and Tailwind toolchain; package does not depend on Electron.

**Rationale**: Prevents duplicate React; keeps package browser-safe; matches library packaging norms.

**Alternatives considered**:
- Leave all UI deps only on Desktop and hope package resolves them — fragile for a second consumer.
- Bundle everything into package with no peers — risks duplicate React in multi-app graphs.

## R8 — Tailwind / shadcn workflow

**Decision**: Keep Tailwind compilation at the app (Desktop/Web) layer. Package ships components as Tailwind-class source + CSS variables/utilities. Update Desktop `components.json` over time so new shadcn additions land in `packages/ui`, not a new Desktop-local tree.

**Rationale**: Current stack is Tailwind v4 + shadcn radix-luma in Desktop; apps own the CSS pipeline, package owns component source and design tokens entry.

**Alternatives considered**:
- Package ships precompiled CSS modules only — fights existing Tailwind utility model and DESIGN.md workflow.
- Force CSS Modules as primary — constitution forbids requiring CSS Modules as primary system.

## R9 — Validation strategy

**Decision**:
- Static: package typecheck; repo-wide typecheck after Desktop rewires; ripgrep/boundary lint for forbidden imports in `packages/ui`.
- Behavioral: existing Desktop feature tests (commander/processes/etc.) remain green.
- Visual/contract: manual or checklist parity on commander overlays, settings, process boards, repository panels, system views against DESIGN.md rules.
- Consumability proof: minimal browser-safe import smoke (documented in quickstart) — package can be imported without Electron types.

**Rationale**: Spec success criteria emphasize parity + boundary honesty, not new heavy e2e.

**Alternatives considered**:
- Full visual regression suite now — valuable later, not required to plan/ship extraction.
- Skip Desktop test re-run guidance — rejected; parity is P1.

## R10 — Documentation / ownership

**Decision**: Feature contracts document share-vs-shell rules; update root progressive-disclosure docs only where needed for long-term ownership (`CONVENTIONS.md` / `docs/ARCHITECTURE.md` pointers) during implementation tasks, not invent new doc sprawl.

**Rationale**: AGENTS.md progressive disclosure; contributors must find boundary in under 5 minutes (SC-006).

## Resolved clarifications

No open `NEEDS CLARIFICATION` items remain from Technical Context. Defaults locked by constitution + current Desktop UI inventory.
