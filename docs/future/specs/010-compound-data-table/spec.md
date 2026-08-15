# Feature Specification: Compound Data Table

**Feature Branch**: `refactor/new`

**Created**: 2026-08-05

**Status**: Approved

**Input**: User description: "Create one configurable compound table pattern for Polterware, use it for operational data collections, provide sortable headers, optional filters and selection, remove row and column dividers, and use spacing plus rounded row hover states."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Scan and sort operational collections (Priority: P1)

As an operator, I see the most important attributes of each entity in a calm, dense table and can sort any data column without leaving the page.

**Why this priority**: Operational indexes are repeated throughout the product and currently behave and look differently.

**Independent Test**: Open an index with multiple rows, sort different columns through every direction, and open a row using pointer and keyboard.

**Acceptance Scenarios**:

1. **Given** an unsorted collection, **When** the operator activates a sortable header, **Then** rows are ordered ascending and the active direction is announced and shown.
2. **Given** an ascending column, **When** the operator activates it again, **Then** rows are ordered descending; a third activation restores source order.
3. **Given** a row that opens detail, **When** the operator activates its primary target by pointer or keyboard, **Then** the existing detail route or action opens.
4. **Given** a narrow window, **When** all essential columns do not fit, **Then** the table scrolls horizontally without silently hiding data.

---

### User Story 2 - Compose filters and optional selection (Priority: P1)

As an operator, I can use each page's relevant search and filter controls above the same table pattern, select actionable rows, and use their individual or common batch actions from the shell bottom toolbar.

**Why this priority**: Reuse must not erase domain-specific filtering or introduce inactive controls.

**Independent Test**: Compose search, select, category filters, optional row selection, and a nested row action without changing their existing behavior.

**Acceptance Scenarios**:

1. **Given** a feature with search or filters, **When** those controls change, **Then** the table renders the consumer-provided result set and retains its shared visual language.
2. **Given** selection is enabled, **When** a row receives hover, keyboard focus, or selection, **Then** its checkbox is discoverable and does not activate row navigation.
3. **Given** select-all is used, **When** a filtered result set is visible, **Then** only visible rows are selected.
4. **Given** one actionable row is selected, **When** the selection toolbar appears, **Then** it shows the applicable individual and batch actions without an Actions column.
5. **Given** multiple rows are selected, **When** their eligibility differs, **Then** only batch actions valid for every selected row remain available.
6. **Given** a high-impact batch action, **When** it is activated, **Then** one accessible confirmation dialog names the action, count, and consequence before execution.

---

### User Story 3 - Recognize one collection language everywhere (Priority: P1)

As an operator moving between product areas, I encounter the same table rhythm, states, sorting model, and interaction feedback across operational collections.

**Why this priority**: A shared component only delivers value when maintained product surfaces actually adopt it.

**Independent Test**: Review representative collections in hierarchy, resources, activity, inbox, knowledge, access, operations, integrations, settings, and observability.

**Acceptance Scenarios**:

1. **Given** any maintained operational data collection, **When** it is inspected, **Then** it uses the shared table language rather than a private table or card-grid implementation.
2. **Given** Organizations or Projects, **When** the index opens, **Then** the table is the single collection view and no Grid/Table switch is shown.
3. **Given** Activity, Inbox, or Logs, **When** the collection renders, **Then** their existing filters, live state, detail access, and relevant actions remain available in table form.
4. **Given** any operational row, **When** its first data cell renders, **Then** an explicit 16px Hugeicons icon appears before the text, varying by real row type when available and otherwise representing the table domain.
5. **Given** an operational collection, **When** its rows render, **Then** the primary cell remains single-line and only attributes needed to identify, compare, or act remain as columns.

### Edge Cases

- Empty, loading, error, offline, and permission states remain distinguishable and do not render misleading rows.
- Equal sort values retain source order.
- Missing values sort consistently after present values and render the existing unavailable label.
- Selection uses stable row identity, survives sorting, and drops identities no longer present in the supplied collection.
- Interactive content inside a row never triggers row activation as a side effect.
- Live logs remain newest-first in source order; an explicit sort temporarily controls their order until sorting is cleared.
- Collections with at least 1,000 rows remain scrollable and sortable without blocking the surrounding shell.
- Instruction lists, menus, picker options, form options, canvas nodes, charts, and isolated metric cards are not converted into tables.
- A selection checkbox may precede the domain icon, but no data column or icon-only header is added for the icon.
- Single selection has no select-all control; multiple selection selects only the currently supplied visible rows.
- Filtering, changing tabs, or removing rows prunes identities outside the supplied set so hidden rows cannot receive actions.
- The hosted selection toolbar clears after successful actions, remains after a cancelled confirmation, and never covers the final table rows.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The product MUST provide one shared, composable data-table pattern for maintained operational collections.
- **FR-002**: Each data-bearing column MUST provide a `rounded-full` header button that cycles source order, ascending, descending, and back to source order; unsorted headers show text only and the direction icon appears only on the active header.
- **FR-003**: Sorting MUST be stable and limited to one active column at a time.
- **FR-004**: Selection MUST remain the only non-sortable utility column; Actions columns MUST NOT render.
- **FR-005**: The table MUST allow each consumer to compose existing search, select, filter, and action controls without taking ownership of domain filtering.
- **FR-006**: Row selection MUST be optional, keyed by stable identity, and support explicit `single` and `multiple` modes only when real row actions exist.
- **FR-007**: Select-all MUST operate on the currently supplied visible result set.
- **FR-008**: Every operational row MUST render an explicit decorative 16px Hugeicons domain icon before the first data-cell text. Features MUST vary the icon when a real row type exists, MUST otherwise select the table-domain icon explicitly, and MUST NOT use status or a global fallback to choose it.
- **FR-009**: Table presentation MUST use spacing rather than row or column dividers and MUST show rounded neutral hover, focus, and selected states.
- **FR-010**: The table MUST have no mandatory card frame or elevated host around the collection.
- **FR-011**: Essential columns MUST remain available through horizontal scrolling at narrow widths; automatic column hiding is forbidden.
- **FR-012**: Sort direction, row navigation, selection, and nested actions MUST be keyboard-operable and expose accurate accessible names and state.
- **FR-013**: Empty and optional footer content MUST be composable without creating fake data rows.
- **FR-014**: Long collections and live logs MUST support an opt-in bounded rendering mode.
- **FR-015**: All maintained raw product tables and eligible data-collection lists MUST migrate to the shared pattern in this delivery.
- **FR-016**: Organizations and Projects MUST remove card-grid presentation and Grid/Table view toggles in favor of the single shared table.
- **FR-017**: Activity and Inbox MUST preserve existing search, filters, counts, detail access, and actions after conversion.
- **FR-018**: Logs MUST preserve monospaced content, live/pause state, clear, download, detail access, and chronological source order when no explicit sort is active.
- **FR-019**: Existing domain values, navigation routes, permissions, status semantics, and actions MUST be preserved. Batch execution MAY apply an existing action to multiple eligible rows but MUST NOT invent a new domain capability or bypass the governed mutation path.
- **FR-020**: Product guidance MUST identify the shared table as the collection source of truth and remove conflicting hierarchy card/view-toggle mandates.
- **FR-021**: Historical specifications MUST remain unchanged; this specification supersedes only conflicting collection-presentation requirements.
- **FR-022**: Selecting rows with actions MUST render one compact opaque toolbar centered at the bottom of `MainSurface`, with selected count, applicable actions, empty-common-action feedback, and a clear control.
- **FR-023**: Multiple selection MUST expose only batch actions valid for every selected row; partial execution over an incompatible selection is forbidden.
- **FR-024**: Revoke, Delete, Reject, and Deprecate batches MUST require one `A11yDialog` confirmation before execution.
- **FR-025**: `PrimaryCell` MUST render a single-line title without secondary description or metadata; omitted context MUST remain available to search and detail surfaces when already present in the source data.
- **FR-026**: Row and header selection controls MUST render at 16px, centered in a 40px utility column, and use the same `--color-primary` accent as primary actions with an opaque dark `--color-primary-foreground` mark for the checked state.
- **FR-027**: Maintained collections MUST retain only contextually essential columns; moving a field out of a table MUST NOT delete it from domain or fixture data.
- **FR-028**: Semantic status badges inside tables MUST render without a leading dot, using the semantic tone as an opaque background, a readable dark shade of the same semantic tone for text, and the shared edge border/top highlight instead of a flat colored border.

### Key Entities

- **Data collection**: The consumer-supplied ordered set shown in one table.
- **Row identity**: Stable identifier used for selection, activation, and updates.
- **Data column**: Labeled attribute with a sort value and visual cell content.
- **Utility column**: The selection column, which is not sortable.
- **Sort state**: No explicit sort, ascending, or descending for one column.
- **Selection state**: Optional set of selected visible row identities.
- **Collection controls**: Search, select, filter, and actions composed by the feature.

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: Improve discovery, scanning, and navigation across operational collections.
- **Runtime owner (III)**: Desktop Renderer and browser-safe shared frontend only.
- **Domain vs adapter (II, IV)**: Presentation owns layout and interaction; features retain domain filtering, actions, and routing.
- **Contracts (V)**: Public frontend component contract only; no backend or transport changes.
- **Product loop impact**: System indexes, evidence, operations, approvals, audit, and knowledge become consistently scannable.
- **Evidence expectations**: Component behavior tests, migration inventory, static design gates, and visual review.
- **Mutation safety (VII)**: Read-only presentation; existing mutations retain their current governed paths.
- **Operation audit fields**: Unchanged because no operation contract changes.
- **Security boundary (VI)**: Browser-safe renderer component; no secret rendering or privileged access.
- **Local vs cloud authority (VIII)**: Unchanged; consumers supply current local, fixture, or synchronized data honestly.
- **Performance (IX)**: Stable sort, bounded long-list rendering, and no shell blocking for large collections.
- **Resilience (X)**: Empty/error/offline/permission states remain explicit.
- **Observability (XI)**: Existing log and audit identifiers remain visible where already present.
- **Test plan (XII)**: Unit interaction tests, representative feature tests, large-collection test, static lint, and visual smoke.
- **Structure created now (XIII)**: Extend the existing frontend design-system package with concrete consumers; no new package.
- **Single source of truth (XIV)**: Remove raw maintained table implementations and conflicting collection patterns.
- **Big-bang / refactor context (XV)**: Part of the authorized renderer refactor branch; migration completes in one delivery.
- **Non-goals (I, XIII)**: No backend pagination, new domain actions, menus-as-tables, canvas conversion, or specialized-platform imitation.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of maintained eligible data collections use the shared table pattern, with zero raw product table implementations outside the design system.
- **SC-002**: 100% of sortable headers complete source → ascending → descending → source order and announce the active direction correctly.
- **SC-003**: Keyboard-only operators can sort, open a row, select rows where enabled, and activate nested actions in every representative acceptance surface.
- **SC-004**: All reviewed table rows use spacing with no row/column divider lines and show rounded hover/focus feedback consistent with the approved reference.
- **SC-005**: A 1,000-row collection remains scrollable and completes a sort without a multi-second shell freeze in the performance smoke.
- **SC-006**: Existing search, filter, status, detail, live-log, and action flows pass representative regression tests after migration.
- **SC-007**: Organizations and Projects expose one table view with zero Grid/Table view-switch controls.
- **SC-008**: Specification quality checklist passes with zero unresolved clarification markers.
- **SC-009**: 100% of maintained `DataTable` collections render a domain icon in the first data cell without adding an icon-only column or changing existing data-column order.
- **SC-010**: Zero maintained tables render an Actions column; all nine previously action-bearing tables use selection plus the shell-hosted toolbar.
- **SC-011**: Incompatible multi-row selections execute zero partial actions and expose `No shared actions` when no common action exists.
- **SC-012**: All 60 maintained collections use the shared table, zero `PrimaryCell` consumers render descriptions/metadata, and both Integrations index tabs use `DataTable` instead of card grids.

## Assumptions

- Consumers provide the current row set; server pagination is outside this delivery.
- Filtering remains feature-owned and is composed through the table toolbar.
- Selection is opt-in only for tables with real actions; single-only tables do not expose select-all or empty multi-selection workflows.
- Existing virtualization capability is reused; no new external dependency is introduced.
- User-facing strings remain English.
- Existing raw tables are the authoritative migration inventory, supplemented by Organizations, Projects, Activity, Inbox, and other collection-like rows discovered during implementation.
