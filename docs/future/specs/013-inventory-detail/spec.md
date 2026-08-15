# Feature Specification: Inventory Resource Detail

**Feature Branch**: `013-inventory-detail`

**Created**: 2026-08-11

**Status**: Implemented

**Input**: User description: "Seguindo o padrão de docs/spec.md, desenvolver a tela de details de inventory no padrão production-ready, mockado. Detail no global DetailPanel; entrega completa de UX/a11y/design-system; dados fixture."

**Related product docs**: `docs/spec.md` (§8.3 Resources tabs, §21 resources domain shape, §111 Resources invariants, inventory as operational resource surface), `AGENTS.md` (global DetailPanel only; StatusChip; DataTable; no second inspector), `DESIGN.md` (sections layout), `specs/004-operator-product-surfaces/contracts/resources-ui.md` (Inventory row → resource detail).

**Extends**: Resources Inventory list already delivered under `packages/features/resources` — this feature elevates **inventory entity detail** from a thin overview to production operator quality. Does **not** implement live provider discovery, real sync, or a second resource detail page/route.

---

## Summary

Operators select a resource on **Resources → Inventory** and inspect a complete, honest inventory record in the single global bottom DetailPanel: identity, health/lifecycle, provider provenance, environment context, linked architecture, activity, capability-gated actions, and operator annotations — without leaving the Inventory collection and without inventing a second inspector or page.

Detail is **mock-backed** for this delivery: structure, copy, a11y, and design-system grammar are production-ready; values come from workspace/resource fixtures and local mock action feedback (toasts / disabled reasons). No claim of live provider state or live polter-core inventory APIs.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect inventory identity and health (Priority: P1)

As an operator, when I select an inventory row, the detail panel opens with identity (name + type icon matching the table), status lifecycle (healthy / degraded / down / unknown / syncing / stale / missing), provider, environment, region, last sync / last seen, and source honesty so I know whether the record is trustworthy.

**Why this priority**: §8.3 and §21 make inventory the primary resource discovery surface; §111 requires provenance, distinguishable lifecycle, and stable identity beyond display name.

**Independent Test**: Open Inventory → select prod-postgres (healthy) and a stale or degraded fixture. Overview/metrics show different StatusChip tones and lifecycle copy; neither claims a live provider call.

**Acceptance Scenarios**:

1. **Given** Inventory has at least one resource, **When** I activate a row, **Then** the global DetailPanel shows that resource’s name and domain icon matching the table row, with no second page, drawer, or modal for entity detail.
2. **Given** detail is open, **When** I read the summary metrics, **Then** I see status (StatusChip), provider, environment, and last sync (or Never / —) without scrolling past the first fold of detail body content.
3. **Given** a resource with stale or missing lifecycle, **When** I open detail, **Then** status is explicit and a lifecycle note explains that temporary disappearance does not mean immediate delete — product does not pretend full health.
4. **Given** detail is open, **When** I activate another inventory row, **Then** content switches to the new resource without unmounting the panel host or inventing a second inspector.

---

### User Story 2 - Understand provenance and provider context (Priority: P1)

As an operator, I can see how the resource entered the inventory (discovered / imported / manual), provider kind, external/provider resource id when known, external console hand-off when available, and that provider-owned fields are distinct from my annotations.

**Why this priority**: §111: every discovered resource has provenance; provider-owned fields update from provider; user annotations must not be overwritten by refresh; deep links preferred over cloned consoles.

**Independent Test**: Open a discovered AWS RDS fixture vs a sparse manual fixture; provenance and external id appear when present; Open provider is external hand-off only.

**Acceptance Scenarios**:

1. **Given** a discovered resource with external key / provider resource id, **When** Overview opens, **Then** provenance, provider, provider kind, and external id are visible (mono for ids).
2. **Given** an external console URL or quick action of kind external, **When** I use Open provider, **Then** hand-off is external (or mock-honest when URL absent) — not an embedded provider admin clone.
3. **Given** a manually annotated resource, **When** Notes/tags render, **Then** they are presented as operator annotations, separate from provider identity fields.
4. **Given** dataSource is fixture, **When** Source field renders, **Then** it is labeled as fixture/mock honesty, never as live core observation.

---

### User Story 3 - Review linked architecture and activity (Priority: P1)

As an operator, Linked architecture lists architecture nodes tied to this resource, and Activity lists recent inventory-relevant events (sync, metrics refresh, status change) so I understand system context without opening Architecture or Operations first.

**Why this priority**: Product thesis: architecture is a living model linked to real resources; inventory detail is a primary jump-off for that link.

**Independent Test**: Select polter-api or prod-postgres with linked nodes + activity; select a sparse resource with empty lists — honest empty states.

**Acceptance Scenarios**:

1. **Given** linked architecture rows exist, **When** the section renders, **Then** a DataTable shows node name and type with domain icons; no Actions column.
2. **Given** activity events exist, **When** the section renders, **Then** a DataTable shows event title and when; status/detail optional without inventing history.
3. **Given** empty linked architecture or activity, **When** sections render, **Then** empty copy is honest — no placeholder fake nodes or events.
4. **Given** navigation hand-off is wired, **When** I activate a linked architecture row, **Then** product navigates toward that node context (or mock-toasts honestly if hand-off is not wired).

---

### User Story 4 - Act only through honest capabilities (Priority: P1)

As an operator, header and quick-actions surface capability-gated actions (open provider, refresh/sync, run health check, edit annotations). Unsupported actions stay unavailable with a reason; mock actions never simulate live success against a provider.

**Why this priority**: §111 actions depend on capabilities; unsupported never simulates success; destructive work is operation, not silent inventory update.

**Independent Test**: On prod-postgres, Open provider enabled; health check disabled with reason; Edit opens existing resource form path; Sync/refresh toast is mock-honest.

**Acceptance Scenarios**:

1. **Given** a resource with an enabled external quick action, **When** I open provider, **Then** external hand-off occurs without claiming Polter mutated the provider.
2. **Given** a disabled quick action with reason, **When** listed, **Then** control is disabled and reason is available (title/aria or visible helper) — never a fake success path.
3. **Given** Edit is available, **When** invoked, **Then** it opens the existing resource edit dialog path (annotations-oriented) without inventing a second detail editor.
4. **Given** Sync / refresh is offered as mock, **When** clicked, **Then** feedback is fixture-honest and does not claim live provider discovery completed.

---

### User Story 5 - Cover sparse and degraded inventory without scenic emptiness (Priority: P2)

As an operator, sparse fixtures (minimal metadata), degraded/down/unknown/syncing states, and not-found/loading still render usable detail chrome with honest empty sections.

**Why this priority**: Production readiness includes incomplete discovery and partial sync; operators hit these early.

**Independent Test**: Activate sparse fixtures and invalid selection states; panel shows header + sections with empty copy; loading/not-found match other operational details.

**Acceptance Scenarios**:

1. **Given** a resource with only core list fields, **When** detail opens, **Then** optional sections show — / empty states, not a broken layout.
2. **Given** loading props, **When** detail mounts, **Then** loading chrome appears without blank panel.
3. **Given** not-found or missing entity, **When** detail mounts, **Then** error/not-found copy is honest.
4. **Given** status syncing, **When** metrics render, **Then** StatusChip uses syncing tone, not healthy.

---

### Edge Cases

- Resource exists in Inventory without linked architecture (allowed by product: inventory without canvas presence).
- Temporary missing/stale lifecycle before delete — never jump straight to “deleted” copy.
- Provider refresh must not be portrayed as wiping user notes/tags.
- External id empty — show —; do not invent provider ids.
- Long activity or architecture lists scroll only inside DetailPanel.Body.
- Simultaneous unknown status + partial metrics — metrics optional; omit decorative charts when meaningless.
- Quick action kind `operation` without live core — disabled or mock-toast only, never silent apply.
- Catalog-like provider labels (AWS, Vercel, On-prem) do not branch layout via `if provider ===`; layout is shape-driven.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Inventory resource detail MUST render exclusively in the global DetailPanel hosted by MainSurface; features MUST NOT create a fixed inspector page, modal-as-detail, drawer, or second panel for inventory entity detail.
- **FR-002**: Selecting a resource in Inventory MUST activate detail target `scopeId: resources:inventory`, `entityKind: resource`.
- **FR-003**: Detail MUST use operational `DetailPanel.Body layout="sections"` with Grid / Section / Metric / Metadata patterns (not knowledge `document` layout). Local detail section Tabs rail is optional and NOT required for this surface (unlike Integration’s seven local sections).
- **FR-004**: Detail header MUST show a 16px domain icon consistent with the Inventory PrimaryCell, a single-line title (resource name), and header actions via `DetailPanel.Action` only (Edit; Open provider when available; optional Refresh).
- **FR-005**: Status / health / lifecycle everywhere in inventory detail MUST use StatusChip (filled); no hand-rolled status pills.
- **FR-006**: Detail MUST surface summary metrics for status, provider, environment, and last sync (or honest — / Never).
- **FR-007**: Detail MUST show identity/context fields: type, region, owner, provenance when known, external/provider resource id when known, last seen when known, source/dataSource honesty.
- **FR-008**: Detail MUST list linked architecture in design-system DataTable (name, type); no Actions column; honest empty state.
- **FR-009**: Detail MUST list activity in design-system DataTable (event, when); no Actions column; honest empty state.
- **FR-010**: Detail MUST show operator annotations (notes, tags) separately from provider identity fields; empty notes allowed.
- **FR-011**: Quick actions / capabilities MUST show enabled vs disabled with reason; unsupported MUST NOT simulate success.
- **FR-012**: Destructive or privileged provider mutation MUST NOT be presented as a silent inventory field edit; operations remain operation-oriented; mock feedback only when offered.
- **FR-013**: Lifecycle states stale/missing (when present in model) MUST be distinguishable from healthy and from permanent delete messaging.
- **FR-014**: This delivery MAY use mock fixtures and local mock mutations; UI MUST NOT claim live provider connectivity, live discovery stream, or live polter-core inventory authority beyond fixture labels.
- **FR-015**: Loading / not-found / error detail states MUST remain honest when supplied.
- **FR-016**: Implementation MUST keep a dedicated `ResourceDetail` composition module (not only inline page markup) for reuse and testing.
- **FR-017**: Collection registration, open/collapsed/maximized snaps, and Escape/maximize behavior remain owned by existing DetailPanel / shell laws — this feature MUST NOT reimplement sizing chrome.
- **FR-018**: At least one fixture MUST be a rich inventory record (linked architecture, activity, notes/tags, provenance, external id, quick actions including one disabled); fixtures MUST also cover sparse rows and at least two non-healthy statuses among degraded/down/unknown/syncing/stale/missing.

### Key Entities

- **Inventory resource**: Normalized infrastructure/platform object in project inventory — identity, type, environment, provider, status/lifecycle, region, owner, provenance, external ids, last sync/seen, annotations, links, activity, quick actions, data source honesty.
- **Provenance**: How the resource entered inventory (discovered, imported, manual) — required conceptually for discovered resources per §111.
- **Lifecycle status**: Operational visibility state (healthy, degraded, down, unknown, syncing, stale, missing) distinct from permanent deletion.
- **Linked architecture node**: Architecture model node associated with this resource (id, name, type).
- **Activity event**: Recent inventory-relevant event (id, time, title, optional detail).
- **Quick action / capability**: Operator action entry (external hand-off, navigate, or operation) with enabled flag and optional disabled reason.
- **Operator annotation**: User-owned notes and tags that must survive provider refresh conceptually.

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: Discover-organize resources; understand architecture linkage; safe execution entry via capability-gated actions and external tools hand-off.
- **Runtime owner (III)**: Renderer feature package `packages/features/resources` + desktop composition; no Main/Node unrestricted access; no claimed live core stream.
- **Domain vs adapter (II, IV)**: UI consumes normalized Resource shapes from fixtures (stand-in for domain queries); no provider-specific layout branching for core structure.
- **Contracts (V)**: Desktop UI contract for Inventory resource detail; no protobuf change required for mock delivery.
- **Product loop impact**: Inventory list → DetailPanel inspect → Architecture link / provider hand-off / edit annotations / future operation request.
- **Evidence expectations**: Last sync, activity, lifecycle note, and linked architecture act as operator-visible evidence stand-ins; fixture source explicit.
- **Mutation safety (VII)**: Mock-only mutations with toasts; real plan→approve→apply not claimed; destructive path not silent inventory update.
- **Operation audit fields**: Mock actions identify resource target and action in toast/copy; no fake audit persistence required this increment.
- **Security boundary (VI)**: No secrets in inventory fixtures; renderer unprivileged; MCP not in scope.
- **Local vs cloud authority (VIII)**: Fixtures are local mock authority; providers remain authoritative for real provider state (not simulated as Polter-owned).
- **Performance (IX)**: Detail content scroll only in DetailPanel.Body; tables use existing DataTable patterns.
- **Resilience (X)**: Stale/missing/degraded/unknown/syncing are first-class UI states.
- **Observability (XI)**: External ids and source honesty visible for hand-off; no fake telemetry claims.
- **Test plan (XII)**: Unit/component tests for status mapping, section presence, capability disable invariants, fixture coverage for rich inventory models.
- **Structure created now (XIII)**: Extend `packages/features/resources` (+ desktop fixture coverage); no new scenic package.
- **Single source of truth (XIV)**: One ResourceDetail composition; no dual detail page + panel.
- **Big-bang / refactor context (XV)**: Continues operator surfaces on current architecture branch.
- **Non-goals (I, XIII)**: No full cloud console clone; no live discovery/sync APIs; no Inventory detail as page tabs; no Nodes/Processes redesign; no Provisioning wizard work; no Terraform/K8s admin surface.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: From Inventory list, an operator reaches full resource context (status, provider, links, activity, annotations) in under 15 seconds without leaving Resources.
- **SC-002**: 100% of fixture resources used in the Inventory list resolve to a detail view that never shows blank structural chrome (header + at least summary sections or honest empty states).
- **SC-003**: Operators can distinguish healthy vs non-healthy lifecycle (including stale/missing when present) from summary metrics without reading raw IDs.
- **SC-004**: 100% of disabled quick actions expose a reason and never present a success path that claims live provider mutation.
- **SC-005**: Detail uses only design-system DetailPanel + DataTable + StatusChip + Action grammar — no parallel hand-rolled detail chrome.
- **SC-006**: Fixture coverage test asserts expanded inventory shape (linked architecture, activity, provenance or external id, notes/tags or equivalent richness) for at least one primary resource fixture.
- **SC-007**: Visual and interaction patterns match other operational details (Run/Member/Node) for sections grid, StatusChip, and header actions.

## Assumptions

- Resources page tabs remain Inventory · Nodes · Processes (and Provisioning if already present from 007); this feature does not change tab IA.
- Existing resource create/edit dialog remains the edit path for annotations and basic fields.
- Architecture navigation hand-off may be a mock callback when wired by App; if absent, row activation may no-op or toast honestly.
- External console URLs may be fixture strings; opening may use `window.open` or mock toast when absent.
- Status vocabulary extends current product statuses with stale/missing for §111 lifecycle honesty while remaining StatusChip-compatible.
- Live polter-core inventory APIs are out of scope for this increment; swapping fixtures for real queries later must not force a second detail UX.
- Portuguese chat with Erick; English for code, UI strings, contracts, and this specification.
