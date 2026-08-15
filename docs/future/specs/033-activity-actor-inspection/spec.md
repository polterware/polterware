# Feature Specification: Activity Actor Inspection

**Feature Branch**: `033-activity-actor-inspection`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Activity page + inspection depth aligned to docs/spec.md §4.11 / §8.8 (Activity distinguishes person vs agent). Professional DataTable, FilterPopover actorKind/type/environment, DetailPanel sections, rich person/agent/system fixtures, filter helpers + detail composition tests. Must not merge with Access Audit."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse operational activity with actor distinction (Priority: P1)

As a project operator, I open **Activity** (GENERAL nav) and see a professional operational feed of recent project changes. Each row shows the event title with a domain event icon, who acted, whether the actor is a **Person**, **Agent**, or **System**, the environment, and when it happened. I can filter by actor kind, event type, and environment, and search by title/description/actor/target. Activity is the **operational feed** — not the Access **Audit** compliance log.

**Why this priority**: Co-management of humans and agents (§4.11 / §8.8) is only observable if operators can distinguish who authored each change in the day-to-day feed.

**Independent Test**: With Activity open and fixtures loaded, the table shows person-, agent-, and system-authored rows; filtering Actor → People shows only person events; filtering Actor → Agents shows only agent events; empty filter combinations show an honest empty state.

**Acceptance Scenarios**:

1. **Given** Activity is open with fixture events, **When** the operator reviews the table, **Then** columns are Event (PrimaryCell + event icon), Actor, Actor kind (StatusChip), Environment, and When — with **no** Actions column.
2. **Given** fixtures include at least one person-authored, one agent-authored, and one system event, **When** the operator filters by Actor kind People, **Then** only person events remain; Agents filter keeps only agent events; System keeps only system events.
3. **Given** filters or search exclude all rows, **When** the table renders, **Then** an honest empty message appears (no blank crash, no invented rows).
4. **Given** Activity copy and chrome, **When** the operator reads labels, **Then** wording refers to operational activity / project feed — not audit trail, compliance log, or Access Audit.

---

### User Story 2 - Inspect an activity event in the global DetailPanel (Priority: P1)

As an operator investigating a recent change, I activate a row and open the global DetailPanel to inspect type, actor, actor kind (StatusChip), target entity, environment, when, and description. Optional non-mutating "Open target" appears when a target is present and a callback is supplied. The composition is dedicated (`ActivityEventDetail`), not a thin inline stub and not Audit detail.

**Why this priority**: Table browsing alone is shallow; inspection depth is required so operators understand what changed and who did it without leaving Activity.

**Independent Test**: Activating any fixture row mounts DetailPanel sections for that event; person vs agent vs system kinds show distinct StatusChip labels/tones; not-found id shows honest error state.

**Acceptance Scenarios**:

1. **Given** Activity lists events, **When** the operator activates a row, **Then** the global DetailPanel shows the event title with the event-type domain icon and sectioned body content.
2. **Given** an event is open, **When** the operator reviews overview, **Then** they see type, actor name, actor kind as StatusChip, environment, when, and description.
3. **Given** an event has a target (and optional target kind), **When** detail opens, **Then** Target section shows that entity; if `onOpenTarget` is provided, a header Open target action is available.
4. **Given** activation of an unknown id, **When** the panel resolves, **Then** a not-found / unavailable state appears without crash.
5. **Given** detail is open, **When** header actions are reviewed, **Then** no Approve/Reject/Delete/Edit mutation actions appear.

---

### User Story 3 - Filter helpers and composition tests (Priority: P2)

As a maintainer, pure filter helpers and detail composition contracts are covered by shell package tests so actor-kind filtering and DetailPanel structure cannot regress silently.

**Why this priority**: Parallel agents and frequent UI polish make untested filter/detail logic a regression risk.

**Independent Test**: `pnpm --filter @polterware/feature-shell test` runs activity filter helper tests and activity-event-detail composition tests green.

**Acceptance Scenarios**:

1. **Given** filter helper functions, **When** unit tests run with mixed fixtures, **Then** type / actorKind / environment / search filters compose correctly and empty results are empty arrays.
2. **Given** ActivityEventDetail source and page wiring, **When** composition tests run, **Then** DetailPanel sections layout, StatusChip actor kind, registerCollection/activateTarget wiring, and non-merge with Audit are asserted.

---

### Edge Cases

- Actor kind person vs agent vs system — StatusChip labels and tones must match table and detail.
- Event type icons remain type-driven (`ActivityEventIcon`); actor kind is a separate chip/icon channel.
- Environment "All" vs named environments in filters.
- Over-constrained filters yield honest empty state.
- Activity must not reuse Access Audit entity kinds (`audit-event`) or Audit detail composition.
- Collection active-row state is independent of panel snap mode; row activate updates target without forcing maximize.
- Period "Recent" shortcut (if retained) is optional UX; actor/type/environment remain the required filters.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Activity MUST remain a first-class GENERAL nav page (not a tab inside Overview, Access, or Settings).
- **FR-002**: Activity collection MUST use design-system compound `DataTable` with columns: Event (PrimaryCell + `ActivityEventIcon`), Actor, Actor kind (StatusChip), Environment, When. MUST NOT include an Actions column.
- **FR-003**: Activity MUST provide filters for actor kind (`person` | `agent` | `system`), event type (architecture / operation / access / …), and environment, plus search over title, description, actor, and target.
- **FR-004**: Row activation MUST register and activate a DetailPanel target in scope `activity:index` with entity kind `activity-event` (or equivalent non-audit kind).
- **FR-005**: Activity event detail MUST use the single global `DetailPanel` with `Body layout="sections"` + `Grid`; MUST NOT invent a second page, drawer, modal, or fixed inspector.
- **FR-006**: Detail MUST present overview: type, actor, actor kind StatusChip, environment, when, description; and target entity when present.
- **FR-007**: Detail MAY offer optional non-mutating Open target via callback when target data exists; MUST NOT offer Approve/Reject/Delete/Edit.
- **FR-008**: Fixtures MUST include ≥1 person-authored, ≥1 agent-authored, and ≥1 system event with distinct titles suitable for filter demos.
- **FR-009**: Empty filter/search results MUST show an honest empty table state.
- **FR-010**: UI copy MUST differentiate Activity (operational feed) from Access Audit (compliance / governance trail). MUST NOT fuse the two pages or share Audit detail composition.
- **FR-011**: Actor kind (and type when shown as status) MUST use `StatusChip` — never hand-rolled pills.
- **FR-012**: Filter logic MUST live in testable pure helpers (or equivalent pure functions) covered by unit tests.
- **FR-013**: Dedicated `ActivityEventDetail` module MUST exist (not only inline thin block in the page).
- **FR-014**: Loading / error / not-found honest states MUST follow sibling operational detail patterns (ApprovalDetail, AuditDetail).
- **FR-015**: Parallel P2 surfaces (Overview rewrite, Knowledge, Spec Kit, Audit detail deepening) are out of scope; coordinate only on shared fixture types if needed without rewriting Audit.

### Key Entities

- **Activity Event**: Point-in-time operational feed record — title, description, type, actor name, actor kind (`person` | `agent` | `system`), environment, when, optional target + target kind.
- **Actor Kind**: First-class distinction person vs agent vs system for co-management observability (§4.11 / §8.8).
- **Event Type**: Operational category (architecture, operation, access, person, agent, sync, node, settings, …) used for icon and type filter — not the same as actor kind.
- **Activity vs Audit**: Activity = day-to-day operational feed; Audit = append-only compliance/governance trail under Access.

### Constitution Constraints *(mandatory)*

- **Operational architecture (I)**: strengthens collaboration observability and auditability of human+agent co-management via Activity feed (not a SIEM product).
- **Runtime owner (III)**: Renderer feature package `@polterware/feature-shell`; mock fixtures only; no Main/Node privilege.
- **Domain vs adapter (II, IV)**: UI composition over activity feed concepts; no new authorization engine.
- **Contracts (V)**: UI contract under this feature directory for Activity detail/filter surface.
- **Mutation safety (VII)**: Activity inspection is read-only; optional open-target is navigation only.
- **Security (VI)**: unprivileged renderer; no secrets in activity fixtures.
- **Performance (IX)**: DetailPanel body sole vertical scroller; modest fixture list.
- **Test plan (XII)**: unit tests for filter helpers + detail composition contract.
- **Structure (XIII)**: extend shell feature only; no scenic package.
- **Single source of truth (XIV)**: one ActivityEventDetail composition; one Activity page filter path.
- **Non-goals**: no live activity store API, no merge with Access Audit, no Overview/Knowledge/Spec-kit/Audit rewrites.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can filter Activity by person vs agent vs system and obtain only matching fixture events.
- **SC-002**: Activating any fixture activity event opens a non-empty DetailPanel with type, actor, actor kind StatusChip, environment, when, and description.
- **SC-003**: Fixtures expose ≥1 person, ≥1 agent, and ≥1 system event used by the Activity list.
- **SC-004**: Empty filter combinations show a visible honest empty state.
- **SC-005**: Shell activity tests (filter helpers + detail composition) pass.
- **SC-006**: Activity page remains separate from Access Audit; no shared Audit detail module or compliance-log primary copy.

## Assumptions

- Mock stage only: ACTIVITY_MOCKS stand in for a future activity store.
- Activity already exists as a nav page with a basic DataTable; this feature deepens filters, columns, fixtures, and detail inspection — not a greenfield page.
- Access Audit remains owned by feature-access; this feature must not deepen or fuse with audit-detail.
- Design-system DataTable, StatusChip, FilterCategoryPopover/FilterPopover, DetailPanel are the required chrome.
- Portuguese chat / English UI strings remain project law.
- `docs/spec.md` §4.11 and §8.8 are the product sources for actor distinction.

## Rejection Rubric (implementation gate)

- [x] Person/agent filter works with distinct fixtures
- [x] Dedicated DetailPanel / `activity-event-detail` module (not zero inspection)
- [x] StatusChip or consistent icon tone for actorKind (and type/result when shown)
- [x] Tests pass
- [x] MUST NOT merge Activity with Audit page
- [x] MUST NOT claim "already implemented" from the shallow table alone
