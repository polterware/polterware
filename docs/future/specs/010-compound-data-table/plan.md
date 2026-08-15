# Implementation Plan: Compound Data Table

**Branch**: `refactor/new` | **Date**: 2026-08-05 | **Spec**: [spec.md](spec.md)

## Summary

Replace the unused presentational `DataTable` with a typed Solid compound component, centralize table visuals and interaction in the frontend design system, and migrate every maintained operational collection to that API. Consumers retain domain filtering, data ownership, routing, and actions.

## Technical Context

**Language/Version**: TypeScript, Solid 1.9  
**Primary Dependencies**: `@polterware/frontend-design-system`, Hugeicons, existing TanStack Virtual
**Storage**: None  
**Testing**: Vitest + jsdom, static design lint, focused feature tests  
**Target Platform**: Electron renderer; browser-safe shared frontend  
**Performance Goal**: Stable sorting and responsive scrolling for 1,000 rows; opt-in virtualization for long/live collections  
**Constraints**: No new external dependency, no raw feature tables, no dividers, token-only opaque product states, no build/full suite  
**Scale/Scope**: 29 raw table blocks across 19 feature files plus hierarchy cards and Activity/Inbox collection lists

## Constitution Check

- **IV / XIII**: Extend the existing browser-safe design-system package; no new package or feature-local mini-system.
- **VI**: Renderer-only presentation; no new privileged bridge or secret exposure.
- **IX**: Long collections use bounded rendering and stable sorting without blocking Main.
- **XII**: Add unit interaction tests, representative migrations, large-collection coverage, and static gates.
- **XIV / XV**: Remove the old table API, raw feature tables, hierarchy grid/card dual view, and temporary compatibility paths before completion.
- **XVI**: Synchronize `DESIGN.md` and `AGENTS.md` with the implemented collection law.

## Public Component Contract

- `DataTable.Root<T>` owns or controls sort and optional selection state over consumer-provided rows and stable row IDs.
- `DataTable.Toolbar` composes existing feature controls without interpreting filter state.
- `DataTable.Table`, `Header`, `ColumnHeader`, `Body`, `Row`, and `Cell` preserve native table semantics.
- `ColumnHeader` registers a typed accessor/comparator and cycles `null → ascending → descending → null`.
- Unsorted column headers render text only; the active column alone shows its ascending or descending direction icon.
- `PrimaryCell` exposes `icon`/`leading`, a single-line `title`, and optional compact `trailing` state only; secondary `description` and `metadata` props are removed.
- `SelectionCell` renders select-all or row selection only when Root enables selection.
- `Root.selectionMode` distinguishes `none`, `single`, and `multiple`; the legacy `selectable` flag remains a temporary alias for multiple selection.
- `SelectionToolbar` exposes selected rows, IDs, count, and clear through a typed Solid render prop, then portals the compact action surface into the `MainSurface` host.
- `Empty` and `Footer` are optional layout slots.
- `virtualized` is opt-in and reuses the installed TanStack Virtual runtime.

## Implementation Phases

1. Build and test the compound component, CSS, sorting, selection, nested-action isolation, states, and virtualization.
2. Migrate raw feature tables in domain batches while preserving existing data/actions.
3. Convert Organizations, Projects, Activity, Inbox, Logs, and eligible collection lists; remove hierarchy grid/card toggles.
4. Synchronize design law and run focused tests, typecheck, design lint, residual inventory, visual smoke, and diff checks.
5. Add explicit first-cell domain icons to all migrated collections, using feature-local type resolvers only where row data supports a meaningful variation.
6. Remove the nine Actions columns, add single/multiple selectors, host the floating selection toolbar in `MainSurface`, and compose individual/common batch actions in Access, Knowledge, Inbox, Projects, and Workflows.

## Migration Rules

- Every data column declares a sort accessor; utility columns explicitly disable sorting.
- Consumers pass already-filtered rows and keep their existing search/filter signals.
- Row activation is optional; nested controls stop propagation through the shared component.
- Missing values render existing labels and sort after present values.
- Every operational row starts with an explicit Hugeicons domain icon; row-type mappings remain feature-owned and no global fallback or new domain capability is introduced.
- Each collection keeps only contextually essential comparison/action columns; omitted fields remain in source data for filtering and detail views.
- Row and header selectors share one 16px, `--color-primary`, vertically centered presentation in a 40px utility column with an opaque dark `--color-primary-foreground` mark.
- Table status chips use the semantic tone as an opaque fill, neutral text, and the shared edge-light rim/top highlight without a leading dot; non-table status dots remain unchanged.
- Action-bearing tables never render an Actions column. One selected row may expose contextual actions; multiple rows expose only common batch-safe actions, with high-impact batches confirmed through `A11yDialog`.
- Instruction lists, menus, picker options, forms, canvas nodes, charts, and isolated metric cards remain unchanged.

## Validation

- Unit: compound exports, stable sort, sort cycle, controlled state, selection/select-all, keyboard activation, nested actions, primary icon slot, empty/footer, and 1,000-row behavior.
- Integration: representative Resources, hierarchy, Activity/Inbox, and Logs behavior.
- Static: zero raw `<table>` in `packages/features`, zero hierarchy Grid/Table controls, `pnpm design:lint`, targeted typecheck, and `git diff --check`.
- Selection follow-up: zero Actions columns, exactly nine hosted selection toolbars, single/multiple-mode and common-action tests written, and no browser/build/test-suite execution for this delivery.
- Density follow-up: exactly 60 shared tables, zero Integrations collection grids, zero `PrimaryCell` descriptions/metadata, route coverage for `operations/schedules`, and visual checks for selection alignment.
- Visual: localhost smoke at 1280×720 and 1512×982 for rounded row hover, active sort header, no dividers, and horizontal overflow.
