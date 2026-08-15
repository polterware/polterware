# Feature Specification: Architecture View Detail

**Feature Branch**: `023-architecture-view-detail`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Seguindo o padrão de docs/spec.md e o DetailPanel de production-ready (Architecture node / Teams), desenvolver a tela de details de Architecture Views — entrega completa, mockada, production-ready. Extrair composição dedicada, enriquecer fixtures de view (description, node membership, environments, owner, updated, optional filters) e alinhar com Canvas · Views no Architecture."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect a saved architecture view (Priority: P1)

As an operator on Architecture → Views, I select a saved view and open its detail panel to understand what the view represents, how many nodes it includes, when it was updated, who owns it, and which environments or filters scope it — before opening it on the canvas.

**Why this priority**: Views are the operator’s named lenses on the system map. A one-line summary is not enough to decide which view to open or edit.

**Independent Test**: With Architecture → Views open and fixture views loaded, activating any view row shows a complete DetailPanel with identity, summary, membership metrics, and open/edit actions.

**Acceptance Scenarios**:

1. **Given** Architecture → Views lists saved views, **When** the operator activates a view row, **Then** the global DetailPanel shows the view name and a domain eye/view icon, with body content beyond a single summary paragraph.
2. **Given** a view detail is open, **When** the operator reviews Overview, **Then** they see description, node count, updated time, and (when fixture-enriched) owner, environment scope, and purpose tags — without a second page.
3. **Given** the view is found in fixtures, **When** the panel opens, **Then** loading is not required in the mock stage and the UI never invents live layout persistence claims beyond fixture autosave product rules already documented elsewhere.

---

### User Story 2 - Open or edit the view from detail (Priority: P1)

As an operator, I use header actions to open the view on the canvas or enter the existing view editor flow. Actions must not invent a second entity inspector.

**Why this priority**: Detail without open/edit is dead-end documentation.

**Independent Test**: Open any view detail; Open switches to canvas with that view active; Edit enters existing view-edit deep flow (or mock toast if flow unavailable).

**Acceptance Scenarios**:

1. **Given** view detail is open, **When** the operator triggers Open, **Then** Architecture tab becomes Canvas (or remains canvas) with that view applied as the active lens.
2. **Given** view detail is open, **When** the operator triggers Edit, **Then** the existing view editor deep flow opens (or honest mock feedback if only name edit is available in mock stage).
3. **Given** optional Duplicate / Delete actions exist, **When** invoked in mock stage, **Then** feedback is honest and does not claim core layout API persistence beyond fixture session rules.

---

### User Story 3 - Understand which nodes/components belong to the view (Priority: P2)

As an operator comparing views, I open detail and see a structured membership list (or clear node count + sample members) rather than only a scalar count.

**Why this priority**: Node count alone does not explain coverage; membership is the operational meaning of a view.

**Independent Test**: Primary Payment flow / System view fixtures show membership collection or explicit member names when enriched; empty membership shows honest empty state.

**Acceptance Scenarios**:

1. **Given** a view fixture with member node ids/names, **When** detail opens, **Then** Members (or Included components) appears as a structured collection with empty state when none.
2. **Given** membership rows reference known architecture nodes, **When** the operator activates a member (if navigation is wired), **Then** architecture node detail may open without leaving Architecture section.
3. **Given** only nodeCount exists without member list, **When** detail opens, **Then** the UI may show the metric honestly and an explicit “Membership list not available in mock” empty — never invents fake member names.

---

### User Story 4 - Honest empty, missing, and partial states (Priority: P2)

As an operator, when a view id is unknown or description is empty, the detail remains honest.

**Independent Test**: Force not-found; panel shows not-found. Sparse description shows empty copy.

**Acceptance Scenarios**:

1. **Given** unknown view id, **When** panel resolves, **Then** not-found is shown.
2. **Given** empty description, **When** Summary renders, **Then** honest empty message appears.
3. **Given** mock stage, **When** edit/open fires, **Then** no claim of cloud-synced view library authority.

---

### Edge Cases

- Views with large node counts — membership table scrolls inside DetailPanel body only; collection may sample primary members if full graph is huge in future live mode (mock fixtures stay small).
- Default System view vs custom flow views — both use the same detail composition.
- Architecture Components tab is not required; Views is the collection host (Components routes to canvas in current IA).
- View detail must not conflict with canvas node detail mounting; only one entity kind active per scope.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Architecture → Views MUST open view detail in the single global DetailPanel; MUST NOT invent a second page/modal for views.
- **FR-002**: View detail MUST use `DetailPanel.Body layout="sections"` + `DetailPanel.Grid`.
- **FR-003**: View detail MUST show identity header: domain eye/view icon matching Views table PrimaryCell family, single-line view name, and `DetailPanel.Action` only.
- **FR-004**: View detail MUST expose Open (primary) and Edit actions; MAY expose Duplicate/Delete as mock actions.
- **FR-005**: View detail MUST present Overview: description, node count metric, updated timestamp, and when enriched: owner, environments, tags/purpose.
- **FR-006**: View detail MUST present Membership as a structured collection when fixture members exist; otherwise honest empty / unavailable copy — never invented names.
- **FR-007**: View detail SHOULD present optional Notes or filter summary when fixtures provide them.
- **FR-008**: View detail MUST support loading, error, and not-found honest states.
- **FR-009**: Fixture model for views MUST be enriched beyond name/description/nodeCount/updated for at least two primary views; desktop detail-fixture-coverage updated accordingly.
- **FR-010**: Implementation MUST extract a dedicated architecture view detail composition (not only a thin inline block in Architecture index).
- **FR-011**: Open action MUST reuse existing canvas view activation behavior; MUST NOT fork a second canvas host.
- **FR-012**: Collection activation remains independent of panel snap mode.
- **FR-013**: Scope is Architecture Views detail only. Full view DSL, live multi-user view sharing, and Components list IA revival stay out of scope unless already present.

### Key Entities

- **Architecture View**: Named saved lens over the system architecture document.
- **View Membership**: Set of architecture nodes/components included in the view.
- **View Scope**: Optional environment or filter metadata describing what the lens covers.
- **View Editor Intent**: Open on canvas vs edit metadata/membership in existing deep flow.

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: navigable system map lenses.
- **Runtime owner (III)**: `@polterware/feature-architecture` + view fixtures.
- **Contracts (V)**: `architecture-view-detail-ui.md` under this feature directory.
- **Single source of truth (XIV)**: one ArchitectureViewDetail composition; canvas remains SSOT for layout.
- **Mutation safety (VII)**: edit/open use existing mock/session paths; no dual persistence story.
- **Test plan (XII)**: composition tests + fixture coverage for enriched views.
- **Non-goals**: no new graph engine, no revival of Components as a separate product page unless already required by IA, no cloud view library.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can open any fixture view from Architecture → Views and inspect summary + membership posture without another page.
- **SC-002**: At least two primary views have enriched fields (owner/environments/members or explicit honest unavailable membership).
- **SC-003**: Open and Edit are one-click from detail and connect to existing canvas/editor flows.
- **SC-004**: Not-found and empty description/membership states are non-crashing.
- **SC-005**: Dedicated view detail module replaces the thin inline Architecture index block.
- **SC-006**: Detail matches production operational pattern (sections grid, StatusChip if status exists, DetailPanel.Action).

## Assumptions

- Architecture Views table and a thin inline view detail already exist.
- Canvas open-view and view-edit deep flows already exist and should be reused.
- View fixtures today are minimal (`ArchViewCard`); enrichment is in scope for this feature.
- Portuguese chat / English UI strings remain project law.
- Architecture node detail is a sibling surface and must not be rewritten here.
