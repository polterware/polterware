# Implementation Plan: Project Settings Environments

**Branch**: `030-project-settings-environments` | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/030-project-settings-environments/spec.md`

## Summary

Raise Project Settings from a shallow form + plain Environments table to a production-ready mock aligned with docs/spec.md §8.12 and Operator UI LAW: General forms on Field primitives with honest save toast; Environments as DataTable with StatusChip for kind and protection plus global DetailPanel for protection posture; Advanced danger zone with design-system confirm Dialog for mock archive only. Keep tab IA General · Environments · Advanced only.

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9  
**Primary Dependencies**: `@polterware/frontend-design-system`, `@polterware/frontend-state`  
**Storage**: N/A (session mock / fixtures only)  
**Testing**: Vitest (node env) — helper unit tests + source composition contracts  
**Target Platform**: Electron Desktop renderer (Solid)  
**Project Type**: Feature package in monorepo (`packages/features/settings`)  
**Performance Goals**: Instant mock UI; no network  
**Constraints**: DESIGN.md / AGENTS.md Operator UI LAW; no Access/Integrations in Settings; no real core wipe  
**Scale/Scope**: One feature package; ~3 environments fixtures; 3 tabs

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. Operational Architecture Product**: PASS — project settings + environment protection posture.  
- **II. Domain Before Framework**: PASS — helper mappers for kind/protection pure TS.  
- **III. Strict Runtime Separation**: PASS — renderer feature package only.  
- **IV. Modular Monorepo**: PASS — settings → design-system + state; no deep cycles.  
- **V. Contracts-First**: PASS — `contracts/project-settings-ui.md`.  
- **VI. Security By Default**: PASS — archive is mock confirm; no privilege escalation.  
- **VII. Safe And Auditable Execution**: PASS — mock toasts honest; no silent destructive claim.  
- **VIII. Local-First**: PASS — fixtures offline.  
- **IX–XI**: N/A / light (no heavy work, no observability product surface).  
- **XII. Testability**: PASS — helpers + contracts.  
- **XIII. Real Code**: PASS — real SettingsView consumers.  
- **XIV. SSoT**: PASS — single SettingsView + EnvironmentDetail.  
- **XV–XVI**: Docs in this feature folder.

**Hard stops**: none.

## Project Structure

### Documentation (this feature)

```text
specs/030-project-settings-environments/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── project-settings-ui.md
├── checklists/
│   └── requirements.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
packages/features/settings/
├── package.json          # + vitest, frontend-state
├── vitest.config.ts
├── tsconfig.json
└── src/
    ├── index.tsx                 # SettingsView tabs + wiring
    ├── types.ts                  # ProjectEnvironment, protection, etc.
    ├── fixtures.ts               # SETTINGS_ENVIRONMENT_MOCKS, general snapshot
    ├── settings-helpers.ts       # normalizeTab, default env options
    ├── environment-status.ts     # kind/protection label + tone
    ├── environment-detail.tsx    # DetailPanel composition
    ├── environment-detail-helpers.ts
    ├── settings-helpers.test.ts
    ├── environment-status.test.ts
    ├── environment-detail.test.ts
    └── environments-table.test.ts
```

**Structure Decision**: Keep all production Settings UI in `@polterware/feature-settings`. Package-local fixtures avoid coupling to shell breadcrumb `EnvironmentItem` shape (kind-only). Wire DetailPanel via `@polterware/frontend-state` like other collections.

## Complexity Tracking

None — standard feature-package pattern matching Access/Operations detail features.
