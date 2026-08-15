# Feature Specification: Spec Kit Document Modules

**Feature Branch**: `035-spec-kit-document-modules`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Extract and professionalize Spec Kit artifact details (Constitution · Specs · Plans · Tasks · Research · Checklists). Spec Kit is a SYSTEM destination, kind-first, with featureKey + logicalPath; it does not run Speckit CLI. Extract dedicated detail module(s) + helpers + tests; preserve document layout; index remains tabs + collection + registerCollection wiring only."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect a Spec Kit artifact as a document (Priority: P1)

As an operator on a project workspace, I open Spec Kit, choose a kind tab (Constitution, Specs, Plans, Tasks, Research, or Checklists), select an artifact in the table, and read and edit its Markdown body in the global detail panel — with fixed properties that identify kind, status, feature key, logical path, ownership, and freshness.

**Why this priority**: Spec Kit is the SYSTEM home for engineering standards shaped like Spec Kit on disk. Without a professional document detail, operators cannot trust path portability or kind identity, and the surface remains an unmaintainable monolith.

**Independent Test**: Open Spec Kit → Specs, activate any fixture row with a feature key, and confirm the detail panel shows document layout, Markdown body, StatusChip status, and both feature key and logical path in properties.

**Acceptance Scenarios**:

1. **Given** Spec Kit is open on any kind tab with at least one fixture artifact of that kind, **When** the operator activates a row, **Then** the global detail panel shows that artifact’s title and a document body (fixed properties + Markdown editor), not a second page or drawer.
2. **Given** an artifact detail is open, **When** the operator reviews properties, **Then** they see kind, status (as a status chip), owner, author, updated time, feature key when applicable, logical path in monospace, and source when present.
3. **Given** the detail panel is half-open, **When** the operator edits Markdown, **Then** the format toolbar is hidden; **When** the panel is maximized, **Then** the format toolbar appears for the writing surface.
4. **Given** any open artifact, **When** the operator triggers Copy markdown, **Then** the current body is copied with honest mock feedback and no claim of Speckit CLI or live repo write.

---

### User Story 2 - Browse kinds without mixing Knowledge (Priority: P1)

As an operator, I use kind-first tabs to list only artifacts of one Spec Kit kind at a time, optionally filter by feature key when the project has multiple features, and see an honest empty state when a tab has no rows — without any merge into Knowledge Docs, Decisions, or Runbooks.

**Why this priority**: Kind-first navigation and multi-feature filtering are the product identity of Spec Kit; conflating with Knowledge is explicitly forbidden.

**Independent Test**: Switch Constitution → Specs → Plans with multi-feature fixtures; each tab lists only its kind; feature filter narrows rows; empty filtered/unfiltered copy is kind-specific and honest.

**Acceptance Scenarios**:

1. **Given** multi-feature Spec Kit fixtures, **When** the operator opens Specs, **Then** only `spec` kind rows appear and the Feature column is visible.
2. **Given** Specs with multiple feature keys, **When** the operator filters to one feature key, **Then** only matching rows remain.
3. **Given** Constitution tab, **When** the list is shown, **Then** the Feature column and feature filter are omitted.
4. **Given** a kind tab with no fixtures (or filters that match nothing), **When** the list renders, **Then** an honest empty title and description for that tab appear, with a create affordance.
5. **Given** Spec Kit destination, **When** reading any product copy, **Then** no string claims that Speckit CLI runs inside the product.

---

### User Story 3 - Maintainable modules and regression tests (Priority: P1)

As a maintainer, Spec Kit page code is split so the index only wires tabs, collection, filters, create dialog, and detail registration, while artifact detail composition and pure helpers live in dedicated modules covered by automated tests (kind mapping, property composition including logical path, document layout contract).

**Why this priority**: The current monolith blocks parallel work and professional quality; “already implemented” inline is not acceptable delivery.

**Independent Test**: Package tests pass; dedicated detail module file exists; index imports and renders it; helper tests cover tab→kind and logical path property display.

**Acceptance Scenarios**:

1. **Given** the Spec Kit feature package, **When** inspecting sources, **Then** a dedicated artifact detail module owns DetailPanel document composition (not only inline in the index).
2. **Given** helper and composition tests, **When** the package test suite runs, **Then** kind mapping, detail composition (document layout, StatusChip, logicalPath/featureKey properties), and index wiring assertions all pass.
3. **Given** shell helpers `logicalPathForSpec` and `specKindForTab`, **When** Spec Kit maps tabs or creates paths, **Then** it reuses those helpers and does not reimplement path/kind mapping.

---

### Edge Cases

- Artifact activated but missing from fixtures → honest not-found / no detail content rather than inventing fields.
- Constitution artifact without feature key → Feature property omitted; Path still shows constitution logical path.
- Non-constitution without feature key in bad data → Path helper may use placeholder feature segment; UI still shows Path mono when present.
- Loading or page-level error → collection registration reflects loading/error; empty and loading states remain honest.
- Deprecated and draft statuses → distinct StatusChip tones; never hand-rolled pills.
- Maximize then restore → format toolbar appears only while maximized; properties remain visible in document layout.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Spec Kit MUST remain a separate SYSTEM destination with kind-first tabs: Constitution, Specs, Plans, Tasks, Research, Checklists.
- **FR-002**: Each tab MUST list only artifacts of the mapped Spec Kit kind; operators MUST be able to filter non-constitution lists by feature key when multiple feature keys exist.
- **FR-003**: Activating a collection row MUST open the global detail panel for that artifact; product MUST NOT add a second detail page, modal inspector, or Knowledge-style alternate entity surface for Spec Kit.
- **FR-004**: Artifact detail MUST use document layout: fixed properties row plus Markdown body editor suitable as a writing surface when maximized.
- **FR-005**: Document properties MUST surface: artifact kind, status as a status chip, feature key when present, logical path in monospace, owner, author, updated time, and source (project or repo) when present.
- **FR-006**: Markdown format toolbar MUST appear only when the detail panel is maximized.
- **FR-007**: Operators MUST be able to copy the current Markdown body from the detail header with honest mock feedback.
- **FR-008**: Create flow MUST stay kind-scoped (title; feature key required for non-constitution) and MUST compute logical path via shared path helper — mock only, no live filesystem or CLI execution claims.
- **FR-009**: Empty states MUST be honest per tab for both unfiltered and filtered empty results.
- **FR-010**: Spec Kit package structure MUST extract dedicated detail composition and pure helpers; index MUST primarily wire tabs, filters, collection, registration, and create dialog.
- **FR-011**: Product copy and toasts MUST NOT claim Speckit CLI execution or live repo binding.
- **FR-012**: Spec Kit MUST NOT absorb or merge Knowledge Docs, Decisions, or Runbooks.

### Key Entities

- **Spec Kit artifact**: A Markdown-oriented engineering standard record with kind, status, optional feature key, logical path, ownership, timestamps, source, tags, and body.
- **Spec Kit kind**: One of constitution, spec, plan, tasks, research, checklist (plus model-only other without a tab).
- **Feature key**: Portable feature-folder identity used for multi-feature projects and path composition.
- **Logical path**: Display/export path that mirrors Spec Kit on-disk layout for future import/export (not a live FS bind in this phase).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: From Spec Kit open to readable artifact detail for a fixture row takes under 3 seconds of operator wait in the mock stage (immediate fixture resolve).
- **SC-002**: 100% of kind tabs show either matching-kind rows or an honest kind-specific empty state (no silent blank table).
- **SC-003**: For every non-constitution fixture with a feature key, detail properties show both feature key and logical path without the operator leaving Spec Kit.
- **SC-004**: Package automated checks for kind mapping, document detail composition, and logical path property presence pass with zero failures for the Spec Kit package.
- **SC-005**: Operators reviewing Spec Kit never see UI copy that claims Speckit CLI ran or that Knowledge docs were opened.
- **SC-006**: Maintainers can change document detail presentation without editing collection/tab wiring beyond import boundaries (dedicated detail module exists).

## Assumptions

- Phase remains mock/fixture-only via shell Spec Kit mocks; no live `.specify/` or `specs/` bind.
- Shared shell helpers already provide tab→kind mapping and logical path composition; Spec Kit reuses them.
- Global detail panel host, Markdown editor, DataTable, StatusChip, and design-system chrome already exist.
- Parallel P2 work owns other destinations; this feature only changes Spec Kit package (and may read shell mocks).
- Constitution is treated as project-level (no feature column); other kinds are feature-scoped when keys exist.

## Out of Scope

- Knowledge package Docs / Decisions / Runbooks
- Running Speckit CLI
- Live filesystem import/export or backend document persistence
- Overview, Activity, Audit surfaces
- Contracts-folder browser as a seventh tab
- Feature-first hub as primary navigation
