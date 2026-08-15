# Feature Specification: Architecture Node Detail Polish

**Feature Branch**: `026-architecture-node-detail`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Elevar Architecture Node detail ao padrão profissional de resource-detail / Resources node-detail / member-detail — StatusChip em lifecycle, link e evidence; overview denso; helpers + composition tests; preservar Open on canvas / Link resource e wiring Components/Views."

**Related product docs**: `docs/spec.md` §4.1 (canvas as operational model), §20 (architecture domain; component type ≠ resource type; architecture↔resource provenance), `AGENTS.md` (global DetailPanel; StatusChip; DataTable; no second inspector), `DESIGN.md` (sections layout), Components tab / View detail already delivered (reuse `ArchitectureNodeDetail`).

**Extends**: Existing thin `ArchitectureNodeDetail` (~163 lines) with loading/error/DataTable. Does **not** rewrite canvas engine, place flow, view-detail, or components-table — only consumes the same detail component those surfaces already open.

**Parallelism boundary**: Other P1 agents own workflow depth (027), organization team (028), integrations secondary (029), project settings (030), terminal sessions (031). This feature owns **only** Architecture Node detail polish under `specs/026-architecture-node-detail/`.

---

## Summary

Operators selecting an architecture component (canvas node or Components row) inspect a production-quality node record in the single global bottom DetailPanel: domain icon + title, honest lifecycle and resource-link posture via status chips, denser overview (environment, role, runtime, owner, layer), connected architecture with Proven/Unproven evidence chips, linked docs, and activity — without inventing a second inspector and without treating architecture lifecycle as live service health.

Detail remains mock/fixture-honest where data is mock; existing Open on canvas and Link resource actions are preserved. Components tab and Views detail wiring continue to work unchanged.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read lifecycle and link posture at a glance (Priority: P1)

As an operator, when I open an architecture node detail, I immediately see lifecycle (Planned, Linked, Observed, Stale, Drifted, Unreachable, …) and resource-link status (Linked / Unlinked / Partial) as filled status chips — not plain text — so I do not confuse placement posture with inventory health.

**Why this priority**: §4.1 requires the canvas card to resolve operational meaning; §20 forbids treating component type as resource type and requires honest provenance on architecture↔resource links. Today lifecycle and link are raw strings.

**Independent Test**: Open a linked observed node and an unlinked planned node; both show StatusChip for lifecycle and for link; linked may show resource name next to link chip; tones are not “healthy” for mere Planned/Linked placement alone.

**Acceptance Scenarios**:

1. **Given** an architecture node is selected from canvas or Components, **When** detail opens, **Then** the global DetailPanel shows the node label, a domain icon for the node type, and overview metrics including lifecycle and link as StatusChips (≥2 chip axes).
2. **Given** the node is linked to a resource, **When** Overview renders, **Then** link chip reads Linked (or Partial) and the resource name is visible when known — without claiming the external provider’s live health.
3. **Given** the node is unlinked and may accept a link, **When** header actions render, **Then** Link resource remains available; Open on canvas remains available when the host provides the handler.

---

### User Story 2 - Inspect connected architecture with honest evidence (Priority: P1)

As an operator, the Connected architecture table shows related nodes, relation type, and evidence as Proven vs Unproven status chips (never raw “Proven”/“Unproven” strings alone), with an honest empty state when no connections exist.

**Why this priority**: Connections and evidence are first-class architecture semantics; raw text fails product status grammar and under-densifies vs Inventory/Nodes details.

**Independent Test**: Open Payment Service fixture with mixed evidence rows — each evidence cell is a StatusChip; open a sparse node with no connections — empty copy is honest.

**Acceptance Scenarios**:

1. **Given** connected rows exist, **When** the table renders, **Then** Evidence column uses StatusChip labels Proven / Unproven with distinct tones; Node and Relation columns remain sortable; no Actions column.
2. **Given** no connections, **When** the section renders, **Then** empty copy is honest (no fabricated edges).
3. **Given** a connection row is activated, **When** the host provides `onOpenRelated`, **Then** selection jumps to the related node without a second inspector.

---

### User Story 3 - Dense overview, docs, and activity parity (Priority: P2)

As an operator, Overview surfaces environment, role, runtime, owner, and layer in a dense metric/metadata layout comparable to Inventory/Nodes detail; Linked docs and Activity tables remain usable with honest empty states.

**Why this priority**: Density and secondary context match operator expectations set by other polished entity details; docs/activity already exist and must not regress.

**Independent Test**: Rich fixture shows environment/role/runtime/owner/layer; empty docs/activity show honest empty copy; loading and not-found paths still work.

**Acceptance Scenarios**:

1. **Given** a rich node fixture, **When** Overview renders, **Then** environment, role, runtime, owner, and layer are visible without leaving the first fold of detail content.
2. **Given** docs or activity lists are empty, **When** sections render, **Then** empty copy is honest.
3. **Given** loading, error, or not-found, **When** detail is shown, **Then** existing loading / not-found / error panels remain honest.

---

### Edge Cases

- Node selected but document missing the id → not-found, not a blank panel.
- Fixture-only detail when document node exists only as fixture fallback → still chips + density; dataSource honesty when shown.
- Lifecycle label already humanized (“Observed”) vs raw state key (“observed”) → same chip label/tone.
- Link partial without resource name → Partial chip without inventing a resource.
- Unlinked with `canLinkResource` false → no Link resource action; Open on canvas may still show.
- Zero connections with optional proven/unproven chart → chart omitted; empty table state only.
- Components tab and Views detail still open their targets; Views detail does not depend on this polish for correctness.

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Architecture node detail MUST render inside the single global DetailPanel (sections layout + grid); MUST NOT introduce a second page, drawer, or modal for entity detail.
- **FR-002**: Header MUST show a domain icon matching the architecture node type (or honest type fallback) and the node label as the single title.
- **FR-003**: Header MUST preserve Open on canvas and Link resource actions when the host supplies handlers and the model allows Link resource (`canLinkResource`).
- **FR-004**: Overview MUST surface lifecycle and resource-link posture as StatusChips (filled chips; no leading status dots as the labeled badge).
- **FR-005**: Overview MUST surface environment, role, runtime, owner, and layer when present (honest “—” or omission when unknown — no invented values).
- **FR-006**: Connected architecture table MUST show Evidence as StatusChip Proven / Unproven; MUST NOT leave Proven/Unproven as the sole presentation without a chip.
- **FR-007**: Connected, Linked docs, and Activity collections MUST use DataTable without Actions columns; empty states MUST be honest.
- **FR-008**: Loading, error, and not-found states MUST remain available and honest.
- **FR-009**: Pure helpers MUST centralize lifecycle/link/evidence labels and tones (reuse architecture lifecycle helpers where they already exist).
- **FR-010**: Dedicated composition (and helper) tests MUST exist and pass under the architecture package vitest config.
- **FR-011**: Components tab and Views detail wiring MUST continue to open architecture node / view targets without regression from this polish.
- **FR-012**: MUST NOT rewrite canvas engine, place flow, view-detail, or components-table implementations beyond consuming shared detail.

### Key Entities

- **Architecture node (detail view)**: Operator-facing projection of a placed architecture component — identity, kind/type, lifecycle posture, resource link posture, environment/role/runtime/owner/layer, connections with evidence, docs, activity.
- **Connection ref**: Related node id, label, relation, evidence boolean (proven vs unproven).
- **Resource link**: Optional inventory resource id/name; link status unlinked | linked | partial; provenance is architecture↔resource, not ownership transfer of the external resource.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In a single inspection of a rich linked node, operators see at least two StatusChip axes (lifecycle and either link or evidence) without reading raw Proven/Unproven-only cells.
- **SC-002**: 100% of evidence cells in the Connected table for fixture rows with connections render as chips, not chip-less text alone.
- **SC-003**: Header domain icon and both primary actions (Open on canvas, Link resource when allowed) remain discoverable in one glance on a rich unlinked-capable node and a rich linked node respectively.
- **SC-004**: Dedicated architecture node-detail composition test suite passes on package vitest.
- **SC-005**: Components tab node activation and Views detail activation still resolve to their existing detail hosts after this change (no broken Show wiring).
- **SC-006**: Operators do not need a second surface to learn lifecycle, link, and connection evidence for a selected architecture node.

---

## Assumptions

- Existing `ArchitectureNodeDetail` host wiring in Architecture feature (canvas + components scopes, `arch-node` entity kind) is correct and only needs denser content composition.
- Lifecycle/link tone SSOT already lives in architecture domain lifecycle helpers; detail helpers may wrap/normalize labels for chip display.
- Fixtures may be enriched (lifecycle, linkStatus, canLinkResource, denser fields) without claiming live Core reconciliation.
- Parallel P1 features are out of scope; process/approval and other closed P0 details are not touched.

## Out of Scope

- Canvas engine, place/commit flow, resource link picker redesign, view-detail rewrite, components-table rewrite.
- Live Core architecture document API beyond existing mock/fixture paths.
- Workflow builder, organization team, integrations secondary, project settings, terminal sessions UI.
- Treating architecture lifecycle as inventory or process health.

## Acceptance Rubric (delivery gate)

Any fail → FAILED:

- [x] StatusChip appears on ≥2 axes (e.g. lifecycle + evidence or link)
- [x] Evidence is not raw Proven/Unproven string alone without chip
- [x] Dedicated composition test exists and passes
- [x] Header has domain icon + actions without regressing Open on canvas / Link resource
- [x] Components tab and Views detail remain wired
- [x] Forbidden: claim “already implemented” solely because a thin `node-detail.tsx` exists
