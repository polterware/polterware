# Tasks: Compound Data Table

**Input**: [spec.md](spec.md) and [plan.md](plan.md)

## Phase 1: Shared component and tests

- [x] T001 Replace the legacy table shell with the typed compound API and centralized CSS in `packages/frontend/design-system/src/table.tsx` and `table.css`
- [x] T002 Export the compound API and public types from the design-system entrypoint
- [x] T003 Add focused tests for sorting, selection, keyboard activation, nested actions, primary cells, empty/footer slots, and large collections

## Phase 2: Raw table migration

- [x] T004 Migrate Resources and Architecture collections to `DataTable`
- [x] T005 Migrate Access, Knowledge, Integrations, and Settings collections to `DataTable`
- [x] T006 Migrate Operations and Observability collections, including live Logs, to `DataTable`
- [x] T007 Migrate Shell raw tables and Overview collections to `DataTable`

## Phase 3: Collection unification

- [x] T008 Replace Organizations and Projects card/grid views with the single table view
- [x] T009 Convert Activity and Inbox collection lists while preserving filters, counts, details, and actions
- [x] T010 Convert remaining eligible collection-like rows and document intentional exclusions

## Phase 4: Documentation and validation

- [x] T011 Update `DESIGN.md` and `AGENTS.md` with the shared collection law and remove conflicting view-toggle/card mandates
- [ ] T012 Run focused tests, typecheck, design lint, residual raw-table inventory, and `git diff --check`
- [x] T013 Validate representative tables visually at 1280×720 and 1512×982 without running a new build
- [ ] T014 Confirm all specification and task acceptance criteria and record any validation boundary

## Phase 5: Domain identity icons

- [x] T015 Add explicit Hugeicons domain icons to the first data cell of all 58 `DataTable` collections, including logs, audit, traces, and provisioning output
- [x] T016 Add feature-local type mappings, explicit Hugeicons package dependencies, and synchronize `DESIGN.md`, `AGENTS.md`, the spec, and the plan
- [x] T017 Run focused typechecks, static first-cell icon inventory, and `git diff --check` without browser, build, or test suites

## Phase 6: Selection action toolbar

- [x] T018 Write focused selection-mode, hosted-toolbar, common-action, and representative feature action tests without executing the suites
- [x] T019 Extend the compound table and `MainSurface` host with single/multiple selection and the floating selection toolbar
- [x] T020 Remove all nine Actions columns and migrate Access, Knowledge, Inbox, Projects, and Workflows to individual and common batch actions
- [x] T021 Add batch confirmation dialogs and synchronize `DESIGN.md`, `AGENTS.md`, the specification, and the implementation plan
- [x] T022 Run focused typechecks, static Actions-column/selection-toolbar inventory, `pnpm design:lint`, and `git diff --check` without browser, build, or test suites

## Phase 7: Essential density and integration collections

- [x] T023 Remove secondary `PrimaryCell` descriptions/metadata and reduce every maintained collection to contextually essential columns
- [x] T024 Convert Connected and Catalog integration grids to `DataTable` with provider icons and governed row actions
- [x] T025 Add the Operations Schedules tab and move the existing schedule table into the page-level tab surface
- [x] T026 Standardize row/header selectors at 16px with the shared primary accent, opaque dark mark, and centered utility-column alignment
- [ ] T027 Run focused tests, typechecks, design lint, static inventories, browser validation, and `git diff --check`
- [x] T028 Replace dotted neutral table status chips with dot-less opaque semantic fills, neutral text, shared edge-light chrome, and preserve non-table status dots

## Validation boundary

- Focused DataTable tests pass: 8/8.
- No manual feature `<table>` remains, and `git diff --check` passes.
- Package typechecks report no new errors in the changed collection surfaces, but the repository-wide gate remains red on existing CSS/Vitest declaration gaps plus existing Architecture and Integrations type errors.
- `pnpm design:lint` cannot complete through the managed pnpm wrapper because it requests a non-interactive modules purge. Its direct opacity check still reports seven pre-existing issues outside this table change; `table.css` introduces none.
- Visual validation passed for Activity, Projects, and Organizations at 1280×720 and 1512×982. The running app had empty live Resources/Components data, so those empty states were inspected while populated fixture tables covered row behavior.
- Static icon inventory passes for all 58 `DataTable` collections: each first data cell is a `PrimaryCell` with an explicit domain icon and no icon-only column was introduced.
- Focused feature typechecks report no new icon-surface errors; remaining failures are the previously recorded CSS/Vitest declaration gaps and existing Architecture, Integrations, and Resources type errors.
- Per the icon follow-up scope, no browser validation, full build, or test suite was run.
- Selection-action inventory passes with zero Actions headers, nine selection-enabled roots, nine selector headers, and nine hosted `SelectionToolbar` consumers.
- `pnpm design:lint` and `git diff --check` pass after the selection-toolbar follow-up.
- Focused typechecks report no new errors in changed source files; the existing CSS side-effect declaration and package-local Vitest type gaps remain, including the newly written but intentionally unexecuted test files.
- Per the approved validation scope, the selection-toolbar follow-up did not run a browser, full build, or any test suite.
