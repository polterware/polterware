# Implementation Plan: Architecture Components Tab

**Branch**: `025-architecture-components-tab` | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/025-architecture-components-tab/spec.md`

## Summary

Restore Architecture **Components** as a first-class tab: remove the feature-level redirect `components → canvas`, add Components to `ARCH_TABS`, render a DataTable over the **same** `ArchitectureDocument` nodes as Canvas, and activate existing `ArchitectureNodeDetail` (`entityKind: arch-node`). Keep work inside `@polterware/feature-architecture`. Shell already lists Components in `SECTION_TABS` and `ARCH_TABS` set.

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9

**Primary Dependencies**: `@polterware/feature-architecture`, `@polterware/frontend-canvas`, `@polterware/frontend-design-system`, `@polterware/frontend-state`

**Storage**: N/A (uses existing architecture document load / fixture path)

**Testing**: Vitest (node env) in `packages/features/architecture` + shell route tests already covering `setTab(..., "components")`

**Target Platform**: Desktop renderer (Electron Solid)

**Project Type**: Feature package UI within monorepo

**Performance Goals**: Table over fixture-scale node sets; reuse existing search filtering patterns

**Constraints**: No canvas engine rewrite; no scenic package; DetailPanel only; SSOT document; DESIGN opacity/radius laws

**Scale/Scope**: One feature tab + list module + route normalization fix + focused tests

## Constitution Check

- **I. Operational Architecture Product**: Strengthens architecture understanding via list twin of canvas — pass.
- **II. Domain Before Framework**: Projection helpers pure (filter/map from ArchNode); UI thin — pass.
- **III. Runtime Separation**: Renderer feature only — pass.
- **IV. Modular Monorepo**: Changes stay in feature-architecture (+ optional shell only if needed) — pass.
- **V. Contracts-First**: UI contract doc under `contracts/`; no new wire protocol — pass.
- **VI–VIII**: No secrets, no mutations requiring approval in this slice — pass.
- **IX**: List is lightweight; no Main work — pass.
- **XI–XIV**: Honest fixture source; real implementation only — pass.
- **UI laws**: DataTable, PrimaryCell, StatusChip, FilterPopover, DetailPanel — required.

Gate: **PASS** for implementation.

## Project Structure

### Documentation (this feature)

```text
specs/025-architecture-components-tab/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── architecture-components-ui.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (repository)

```text
packages/features/architecture/src/
├── index.tsx                    # ARCH_TABS + tab normalize + wire Components panel
├── tab.ts                       # pure normalizeArchitectureTab (testable)
├── components-table.tsx         # DataTable collection UI
├── components-list.ts           # project + filter pure helpers
├── domain/node-lifecycle.ts     # extend with StatusChip tone helpers
├── components-tab.test.ts       # tab normalize + list/filter + activation contract
└── (reuse) node-detail.tsx, fixtures, document load, canvas document signal
```

**Structure Decision**: Single feature package modules; pure helpers for list projection/filter; no new package.

## Implementation Approach

1. **Tab restoration**
   - `ARCH_TABS = canvas | components | views`
   - `normalizeArchitectureTab`: keep `components`; map only retired `dependencies`/`changes` → `canvas` (or existing shell behavior for changes → activity is shell-owned)

2. **Components collection**
   - Project `doc().nodes` → list rows (id, name, type label, lifecycle, linkStatus, environment, owner)
   - Icons via existing `resolveNodeIconName` + `NodeIcon` / type icons
   - DataTable columns per FR-003–006
   - Empty/loading via DataTable.Empty / PageLoading

3. **Detail wiring**
   - Collection scope `architecture:components` with targets `entityKind: arch-node`
   - Reuse `ArchitectureNodeDetail`; resolve `detailNode` for scopes `architecture:canvas` **or** `architecture:components`
   - Optional: detail action “Open on canvas” → `onTabChange("canvas")` + activate canvas scope same id

4. **Filters**
   - Local FilterPopover groups: lifecycle, type, environment (from document values)
   - Optional shell search already on props.search

5. **Tests**
   - `normalizeArchitectureTab("components") === "components"`
   - List projection ids match document nodes
   - Filter contracts
   - Activation target shape `{ entityKind: "arch-node", scopeId: "architecture:components" }`

## Complexity Tracking

None — standard feature tab restore.

## Phases

| Phase | Outcome |
| --- | --- |
| 0 Research | Confirm shell already keeps components; defect is architecture/index |
| 1 Design | data-model + UI contract |
| 2 Tasks | tasks.md |
| 3 Implement | code + tests to rubric |
