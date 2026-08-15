# Feature Specification: Architecture Components Tab

**Feature Branch**: `025-architecture-components-tab`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Restore Architecture **Components** tab as a real collection list over the same canonical architecture model as Canvas (docs/spec.md §8.2). Tabs MUST be Canvas · Components · Views. Stop silently redirecting `components` → `canvas`. Professional DataTable; row activate reuses Architecture node detail (arch-node); no second inspector; no scenic package; do not reimplement View detail or rewrite canvas engine."

**Related product docs**: `docs/spec.md` (§8.2 Architecture tabs; §20 canvas/components same aggregate), `AGENTS.md` (DataTable, StatusChip, DetailPanel, FilterPopover laws), `DESIGN.md`, existing Architecture Canvas + node detail under `packages/features/architecture`.

**Extends**: Architecture feature already delivers Canvas and Views. This feature restores the missing **Components** tab as the list surface of the same architecture document — not a second graph, inventory clone, or independent package.

**Parallelism**: Wave P0 peers own Process (021), Approval (022), Architecture View detail (023), Invitation (024). This feature MUST NOT implement view detail or rewrite place/configure canvas flows.

---

## Summary

Operators open **Architecture → Components** and see every architecture component from the **same canonical model** used by Canvas in a professional collection table. Selecting a row opens the **existing** architecture node detail in the global DetailPanel (`entityKind: arch-node`). Deep links and in-page tab selection keep `tab=components` — they never silently land on Canvas.

Today Components is listed in shell `SECTION_TABS` but the Architecture feature drops the tab and maps `components` → `canvas`, violating §8.2. This feature restores the required three-tab surface and a real list without inventing a second node store.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Open the Components tab (Priority: P1)

As an operator, when I am on Architecture, I can select the **Components** tab in the Architecture toolbar and stay on Components. The toolbar shows **Canvas · Components · Views**. Navigating to Architecture with tab `components` (deep link or shell tab registry) does **not** rewrite me to Canvas.

**Why this priority**: §8.2 mandatory tabs; silent redirect is a product-contract violation, not a polish issue.

**Independent Test**: Open Architecture; click Components; confirm URL/route tab remains `components` and the Components list surface renders. Deep-link `architecture` + `components` stays on Components.

**Acceptance Scenarios**:

1. **Given** Architecture is open on Canvas, **When** I select Components in the Architecture tabs, **Then** Components is selected and the components collection is shown (not the canvas host).
2. **Given** route/tab is `architecture` / `components`, **When** the feature normalizes the tab, **Then** the resolved tab remains `components` (not `canvas`).
3. **Given** Architecture toolbar is visible, **When** I inspect tabs, **Then** I see Canvas, Components, and Views as selectable items.
4. **Given** retired tabs `dependencies` or `changes`, **When** they are provided, **Then** they may still resolve away from Components (legacy behavior) without treating Components as retired.

---

### User Story 2 - Browse the canonical component list (Priority: P1)

As an operator, on Components I see a professional collection of architecture components derived from the **same document/fixture** as Canvas: name with domain icon by type, type, lifecycle or operational status via StatusChip, environment, and owner or link status. There is no Actions column. Empty and loading states are honest.

**Why this priority**: Components is the list twin of Canvas over one model (§8.2, §20). A scenic second list or inventing health breaks trust.

**Independent Test**: With fixture document loaded, Components rows match canvas node identities (ids/names); empty document shows honest empty copy; loading shows loading chrome.

**Acceptance Scenarios**:

1. **Given** the architecture document has N nodes, **When** Components loads, **Then** the table lists those components from that document (same ids as canvas nodes), not a second hard-coded graph.
2. **Given** rows render, **When** I inspect columns, **Then** I see a PrimaryCell (domain icon by type + single-line name) and at least one StatusChip column (lifecycle and/or link status), plus essential comparison fields (type, environment, owner and/or link).
3. **Given** the collection, **When** I look for bulk Actions, **Then** no Actions column exists; entity actions remain in DetailPanel when a row is active.
4. **Given** loading is true, **When** Components is shown, **Then** loading chrome is honest (not a blank panel pretending success).
5. **Given** zero matching components (empty document or filters yield none), **When** the table renders, **Then** empty copy is honest — no fabricated rows.

---

### User Story 3 - Inspect a component via global node detail (Priority: P1)

As an operator, activating a Components row opens the **same** Architecture node detail experience used from Canvas (`ArchitectureNodeDetail` / `entityKind: arch-node`) in the global DetailPanel. No new detail page, drawer, or second inspector is created.

**Why this priority**: Single entity detail host is LAW; Canvas and Components share one node identity.

**Independent Test**: Activate a Components row; DetailPanel shows that node; switch to Canvas and open the same node — same detail kind and identity, not a parallel inspector.

**Acceptance Scenarios**:

1. **Given** Components has rows, **When** I activate a row, **Then** the global DetailPanel targets that node with `entityKind: arch-node` and shows Architecture node detail content.
2. **Given** detail is open from Components, **When** I activate another row, **Then** detail switches to the new node without a second panel.
3. **Given** a collapsed panel, **When** I only change active row (same scope), **Then** activation does not invent a second entity-detail route/page.
4. **Optional**: **Given** node detail is open, **When** “Open on canvas” is available, **Then** it switches to Canvas focusing that node without creating a second inspector.

---

### User Story 4 - Filter the list without leaving Architecture (Priority: P2)

As an operator, I can narrow Components by lifecycle/status, type, and environment using the product FilterPopover / FilterChip patterns already used in Architecture or sibling features, without losing the tab or inventing a second filter system.

**Why this priority**: Large systems need comparison filters; secondary to tab + list + detail SSOT.

**Independent Test**: Apply type or lifecycle filter; row set shrinks; clear filters restores full document-derived set (subject to search if present).

**Acceptance Scenarios**:

1. **Given** multiple types/states/environments in the document, **When** I filter by one type, **Then** only matching components remain visible.
2. **Given** active filters, **When** I clear them, **Then** the full unfiltered document list is restored (respecting search if any).
3. **Given** filters yield zero rows, **When** the table renders, **Then** empty state is honest.

---

### Edge Cases

- Document empty: empty Components table; panel may collapse when collection has no targets (existing DetailPanel collection rules).
- Search (if shell search is passed): filters the same document-derived rows, does not invent nodes.
- Node present only on canvas document after place: appears on Components without a separate write path.
- Link status unlinked / linked / partial must not be shown as fake “healthy service health” when only lifecycle is known.
- Views tab and view detail remain owned by peer work; Components must coexist without breaking Views.
- Retired tabs `dependencies` / `changes` stay non-Components aliases as product already defines.
- Fixture vs core dataSource honesty: list does not claim live discovery when data is fixture.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Architecture toolbar MUST expose three tabs: **Canvas**, **Components**, **Views**.
- **FR-002**: Tab id `components` MUST remain `components` after feature and shell route normalization; it MUST NOT be rewritten to `canvas`.
- **FR-003**: Components list MUST be a design-system compound DataTable with sortable essential columns; Actions columns are forbidden.
- **FR-004**: First data cell MUST be PrimaryCell with 16px domain icon varied by component type and a single-line title (name); PrimaryCell descriptions/metadata are forbidden.
- **FR-005**: At least one column MUST render lifecycle and/or link status with StatusChip (filled); hand-rolled status pills are forbidden.
- **FR-006**: Essential columns MUST include name, type, lifecycle or status (StatusChip), environment, and owner and/or link status.
- **FR-007**: Row activation MUST open global DetailPanel for architecture node detail (`entityKind: arch-node`), reusing the Canvas node detail composition — not a new page, modal-as-detail, or second inspector.
- **FR-008**: List rows MUST derive from the same architecture document/model instance used by Canvas for the active system (SSOT). A second independent node graph or scenic fixture list that can drift from Canvas is forbidden.
- **FR-009**: Loading and empty states MUST be honest.
- **FR-010**: Filters for lifecycle/status, type, and environment MUST use product FilterPopover and/or FilterChip patterns when filters are offered.
- **FR-011**: Implementation MUST live in `@polterware/feature-architecture` (or existing architecture modules). A new scenic package is forbidden.
- **FR-012**: Tests MUST cover that tab/route normalization keeps `components` as `components`, and that the components list activation contract targets arch-node detail (not a page route).
- **FR-013**: This feature MUST NOT reimplement Architecture View detail, rewrite the canvas engine, or change Process / Approval / Invitation surfaces.
- **FR-014** (optional): Node detail MAY offer “Open on canvas” that switches to Canvas with the same node selected.

### Key Entities

- **Architecture component (node)**: Canonical architecture node identity (id, name, type, lifecycle/state, environment, owner, link status, optional resource link) shared by Canvas and Components.
- **Architecture document**: Single system layout aggregate (nodes + relationships + viewport metadata) that is the SSOT for both Canvas placement and Components listing.
- **Architecture tab**: One of `canvas` | `components` | `views` under section Architecture.

### Assumptions

- Shell `SECTION_TABS` and `resolveArchitectureTab` already treat `components` as a valid Architecture tab; the primary defect is feature-level tab list + redirect.
- Existing `ArchitectureNodeDetail` and canvas document load path are sufficient for detail; this feature wires list activation, not a new detail model.
- Fixture document used by Canvas is acceptable SSOT for mock delivery; no live discovery protocol is claimed.
- Peer agents own View detail and other P0 entities; Components only coexists with Views tab listing.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can open Architecture → Components in under 2 interactions from the Architecture section (tab click or equivalent) and see a real list surface 100% of the time when the feature is loaded (not Canvas).
- **SC-002**: 100% of deep links / tab selections with Architecture + Components retain Components (zero silent redirects to Canvas in normalization tests).
- **SC-003**: 100% of listed component ids on Components match nodes present in the active architecture document used by Canvas for the same load (SSOT check).
- **SC-004**: Activating any visible Components row opens the global node detail for that id without navigating to a separate entity page (verified by activation contract tests and manual UI check).
- **SC-005**: Operators can distinguish lifecycle/link posture for at least the fixture set via StatusChip labels without reading free-form prose alone.
- **SC-006**: Empty document yields a clear empty state rather than placeholder fake components.

## Out of Scope

- Architecture View entity detail (peer 023).
- Canvas place/configure engine rewrite, connection algorithm changes, autosave redesign.
- Live core layout protocol beyond existing fixture/core load paths.
- New scenic monorepo package for “components only.”
- Process, Approval, Invitation details.
- Inventory Resources list (different domain).
