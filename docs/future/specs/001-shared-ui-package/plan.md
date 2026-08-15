# Implementation Plan: Shared UI Package

**Branch**: `001-shared-ui-package` | **Date**: 2026-07-17 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-shared-ui-package/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Extract Polterware’s browser-safe design-system primitives from `apps/desktop/src/renderer/components/ui` into a first-class workspace package `@polterware/ui`, consumable by Desktop now and Web later. Move the shared `cn` helper, design tokens/surface material CSS needed by those primitives, and pure presentational utilities with them. Keep Desktop shell/chrome, dock mechanics, feature modules, and authority adapters app-owned. Preserve visual/interaction parity on retained workbench surfaces; no privileged APIs enter the package.

Technical approach: monorepo source package (same pattern as `@polterware/core`), explicit public exports, Desktop depends via `workspace:*`, optional thin re-export shims only during import migration, then remove duplicate local primitive trees.

## Technical Context

**Language/Version**: TypeScript 5.x / 6.x (workspace), React 19, JSX react-jsx

**Primary Dependencies**: React + React DOM (peer); existing Desktop UI stack moved with primitives — Radix/Base UI (`radix-ui`, `@radix-ui/*`, `@base-ui/react`), `class-variance-authority`, `clsx`, `tailwind-merge`, `@hugeicons/react`, plus primitive-specific libs already used (`cmdk`, `vaul`, `embla-carousel-react`, `input-otp`, `react-day-picker`, `react-resizable-panels`, `recharts`, `sonner`, `next-themes` only if still required by extracted primitives)

**Storage**: N/A (no persistence domain). Sidebar cookie preference remains browser cookie only if sidebar ships in package (browser-safe).

**Testing**: Package `tsc --noEmit`; Desktop existing Vitest + Testing Library suites for workbench/features; static boundary check (no privileged imports in package); optional thin package unit tests only where helpers are non-trivial. No new e2e required for MVP.

**Target Platform**: Browser-safe React renderers — Electron renderer (Desktop) and future Web client. Not Node/main/preload.

**Project Type**: Shared UI library package inside pnpm/turbo monorepo + Desktop consumer wiring

**Performance Goals**: No operator-visible regression on dense boards/lists/overlays; package must not force extra runtime layers beyond current primitive cost

**Constraints**:
- Browser-safe only: no Electron, Node FS/exec/PTY, secrets, window lifecycle
- DESIGN.md hard constraint (tokens, radius 8px, glass/frosted materials, overlay patterns)
- Tailwind + CSS variables remain styling system
- Create only `packages/ui` now (no Web app, no feature-* packages, no crates/services)
- Retain workbench features; extraction is move/reuse, not redesign or deletion
- Agents: no `pnpm dev/build/dist` unless Erick asks; prefer typecheck/static review

**Scale/Scope**:
- ~59 UI primitive modules under `apps/desktop/src/renderer/components/ui`
- Desktop features already importing those primitives (commander, processes, repositories, system, settings, shell, shared)
- MVP: design-system package + Desktop consumption + docs/ownership boundary
- Out of scope: domain feature extraction, Web product runtime, presentation app

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. System Control Plane First**: PASS — packaging enables operator UI consistency; does not reframe product as launcher/dashboard clone.
- **II. Authority And Surface Boundaries**: PASS — package is shared browser-safe; Desktop remains live consumer; Web readiness is consumability, not claiming live Web runtime.
- **III. Evidence Before Trust**: PASS — no model/evidence lifecycle changes.
- **IV. Safe Path From Intent To Action**: PASS — no mutation path; structural UI ownership only.
- **V. Thin Electron, Privileged Rust Core**: PASS — UI stays thin/unprivileged; no privilege moves into package; Tailwind + DESIGN tokens retained; workbench screens retained.
- **VI. Honest Scope And Progressive Delivery**: PASS — Phase 0 runway package justified by second-surface readiness and constitution VII; only `packages/ui` created.
- **VII. Professional Operator Interface**: PASS — directly implements “share design system + domain feature views; specialize shells/adapters” for the design-system half.
- **VIII. Security, Performance, And Quality Bar**: PASS — security boundary explicit (no privileged imports); performance = parity; quality = DESIGN.md + workbench parity.

**Post-design re-check**: PASS — contracts and data model keep privilege out, limit structure to one package, and document shell vs shared ownership. No Complexity Tracking violations.

## Project Structure

### Documentation (this feature)

```text
specs/001-shared-ui-package/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── package-public-api.md
│   ├── ownership-boundary.md
│   └── consumer-integration.md
└── tasks.md             # /speckit-tasks — not created here
```

### Source Code (repository root)

```text
packages/
├── core/                          # unchanged (non-visual)
└── ui/                            # NEW — @polterware/ui
    ├── package.json
    ├── tsconfig.json
    ├── components.json            # optional shadcn config for future adds
    └── src/
        ├── index.ts               # public barrel (stable primitives)
        ├── lib/
        │   └── utils.ts           # cn()
        ├── hooks/                 # only hooks required by primitives (e.g. use-mobile)
        ├── components/            # moved design primitives
        └── styles/
            ├── index.css          # package CSS entry (tokens + shared surfaces)
            └── texture-button.css # co-located material CSS with primitive

apps/desktop/
├── package.json                   # depend on @polterware/ui workspace:*
├── components.json                # retarget aliases/paths toward package over time
├── electron.vite.config.ts        # ensure workspace UI resolves/bundles like core if needed
├── src/renderer/
│   ├── styles.css                 # thin: tailwind + import @polterware/ui styles + desktop-only leftovers
│   ├── shell.css                  # DESKTOP-OWNED chrome/dock/app-region
│   ├── lib/utils.ts               # remove or re-export from package after migration
│   ├── components/ui/             # remove after move, or temporary re-export shims only
│   └── features/**                # stay desktop; update imports to @polterware/ui
└── …main/preload unchanged

apps/presentation/                 # out of scope
```

**Structure Decision**:
- Create **only** `packages/ui` (`@polterware/ui`).
- Modify Desktop renderer consumer paths and CSS ownership.
- Do **not** create Web app, `design-tokens` split package, `polterware-ui` domain package, crates, or services in this feature.
- Authority: renderer-safe package; shells/adapters remain Desktop-owned.
- Why package now: constitution UI reuse rule + Phase 0 runway + concrete second-surface (Web) readiness without waiting for Web scaffold.

## Complexity Tracking

> No constitution violations requiring justification.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
