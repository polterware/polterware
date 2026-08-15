# Feature Specification: Overview Operational Command Center

**Feature Branch**: `032-overview-operational`

**Created**: 2026-08-12

**Status**: Draft

**Input**: User description: "Project Overview production-ready mock denser — command center per docs/spec.md §8.1: project/environment context, system health snapshot, recent changes/operations, pending approvals, nodes/integrations health, high-frequency shortcuts, StatusChip health grammar, empty/offline/fixture honesty, testable overview-model helpers, honest deep-links. No fake telemetry charts; no mandatory Summary/Activity tabs. Parallel P2 agents own Activity, Knowledge modules, Spec Kit modules, Audit depth — do not implement those."

**Related product docs**: `docs/spec.md` (§8.1 Overview), `AGENTS.md` (StatusChip grammar; no live runtime claims; Overview is project default landing), `DESIGN.md` (StatusChip filled semantic chrome).

**Extends**: Existing shell Overview under `packages/features/shell/src/` (`overview-page.tsx`, `overview-cards.tsx`, `overview-model.ts`, fixtures/workspace-mocks). Does **not** implement live telemetry, cloud sync, Activity page rewrite, or settings.

---

## Summary

Operators land on **Project Overview** as a dense **command center** for the current project/environment: health snapshot, attention (pending approvals), recent activity/operations samples, nodes and integrations health, and high-frequency shortcuts — without turning Overview into a configuration surface.

Existing cards are **not** “already done.” Gaps: cohesive context header, StatusChip on health surfaces, integrations health, fixture/offline honesty, shortcut strip, expanded model tests, and honest deep-links (Inbox pending, Operations, Resources, Architecture, Terminal).

Mandatory Summary/Activity page tabs are **out of scope** (§8.1).

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Read current project health at a glance (Priority: P1)

As an operator, I open Project Overview and immediately see the current project name, active environment, overall system health (StatusChip), nodes health, and integrations health without navigating into Settings or Architecture.

**Why this priority**: §8.1 starts with project/environment context and system/nodes/integrations health; without StatusChip grammar the page fails product law.

**Independent Test**: Load Overview with fixture summary → header shows project + environment; ≥2 health surfaces use StatusChip (e.g. system health + nodes and/or integrations); no hand-rolled status pill.

**Acceptance Scenarios**:

1. **Given** a fixture overview summary, **When** Overview renders, **Then** project name and environment label are visible in the overview context area (not buried only in Settings).
2. **Given** system health is degraded (or healthy/offline per fixture), **When** I read system health chrome, **Then** status is shown with StatusChip (filled semantic tone + label), not raw unstyled text alone or a prohibited badge pill.
3. **Given** node availability counts exist, **When** I read nodes health on Overview, **Then** status uses StatusChip or an established StatusChip-backed metric path (not raw “online/offline” text as the only status affordance).
4. **Given** integration fixture counts exist, **When** Overview renders, **Then** integrations health is summarized with StatusChip-backed status (connected/error/disconnected style), not omitted entirely.
5. **Given** zero components/nodes/integrations in an empty snapshot, **When** Overview renders, **Then** empty copy is honest (no invented live percentages or fake history charts).

---

### User Story 2 - Act on pending approvals from Overview (Priority: P1)

As an operator, I see pending approval count under attention and can open Inbox (pending) or a specific approval without hunting through navigation.

**Why this priority**: Approvals are a primary interruption path; §8.1 lists pending approvals on Overview.

**Independent Test**: Fixture with pending approvals → Needs attention shows count; “View inbox” (or equivalent) navigates to Inbox pending; row open hands off to approval detail callback.

**Acceptance Scenarios**:

1. **Given** N pending approvals in fixtures, **When** Overview renders, **Then** attention/pending surface shows the pending count (N ≥ 0).
2. **Given** pending approvals exist, **When** I activate “View inbox” / attention open-inbox control, **Then** the shell navigation callback for Inbox (pending) is invoked.
3. **Given** a pending approval row, **When** I activate the row, **Then** `onOpenApproval(id)` is called with that id.
4. **Given** zero pending approvals, **When** Overview renders, **Then** empty copy states there are no pending approvals (not a blank card).

---

### User Story 3 - Scan recent operations and changes, then hand off (Priority: P1)

As an operator, I see a short sample of recent changes and operations in progress, open a run or the Activity/Operations destination, and never see invented historical charts when data is only a snapshot.

**Why this priority**: §8.1 requires recent changes and recent operations without making Overview a full Activity page (owned by another agent).

**Independent Test**: Fixture activity + running/queued runs → Recent changes and Operations in progress lists render; open callbacks fire; no fake multi-day history chart is introduced.

**Acceptance Scenarios**:

1. **Given** activity fixtures, **When** Overview renders Recent changes, **Then** a short sample (e.g. first few events) is listed with open/hand-off to Activity.
2. **Given** running/queued runs, **When** Overview renders Operations in progress, **Then** each item shows run name and StatusChip (or StatusChip-equivalent labeled status grammar) for run state and open hand-off to Operations detail.
3. **Given** empty activity or no in-progress runs, **When** those sections render, **Then** honest empty copy appears.
4. **Given** only snapshot fixture counts (no time series), **When** Overview renders, **Then** the page MUST NOT invent multi-period history charts from synthetic series.

---

### User Story 4 - Jump via high-frequency shortcuts (Priority: P1)

As an operator, I use Overview shortcuts to open Architecture, Resources, Operations, Inbox, and Terminal without using the sidebar.

**Why this priority**: §8.1 high-frequency shortcuts; shell already owns navigate callbacks.

**Independent Test**: Click each shortcut → corresponding `onQuickAction` id fires (`architecture`, `resources`, `operations`, `inbox`, `terminal` or established shell mapping).

**Acceptance Scenarios**:

1. **Given** Overview is mounted with `onQuickAction`, **When** I activate Architecture shortcut, **Then** `onQuickAction("architecture")` is called.
2. **Given** the same, **When** I activate Resources / Operations / Inbox / Terminal shortcuts, **Then** the matching quick-action ids fire.
3. **Given** shortcuts render, **When** I inspect chrome, **Then** they are operator navigation controls (not settings forms).

---

### User Story 5 - Trust data-source honesty (Priority: P1)

As an operator, I always know whether Overview is fixture demo data, empty, or offline — never a silent “live telemetry” claim.

**Why this priority**: Product law forbids claiming unimplemented runtime as live; parallel mock quality features require fixture labels.

**Independent Test**: Default fixture mode shows fixture banner/label; empty mode empty honesty; offline mode offline honesty.

**Acceptance Scenarios**:

1. **Given** `dataSource` is fixture (default), **When** Overview renders, **Then** a fixture/demo honesty cue appears in header or footer (banner or labeled chip).
2. **Given** `dataSource` is empty, **When** Overview renders, **Then** empty-state honesty is visible and sections do not invent non-zero live counts from nowhere.
3. **Given** `dataSource` is offline, **When** Overview renders, **Then** offline honesty is visible and the page does not claim live host telemetry.
4. **Given** any mode, **When** I read Overview, **Then** copy MUST NOT claim “live production telemetry stream” for fixture data.

---

### Edge Cases

- All health counts zero: system health may read Healthy or Unknown per model rules; no divide-by-zero; empty lists stay empty.
- Pending approvals present but attention total from summary mismatches list length: list filter (pending only) is authoritative for the Needs attention list; summary count may be used for metric tiles but helpers stay consistent when built from same fixtures.
- Missing optional summary fields / undefined props: overview-model returns safe zeros; page still renders.
- Operations with progress undefined: progress clamped to 0.
- Parallel Activity agent: Overview may deep-link to Activity but MUST NOT rewrite `activity-page.tsx`.
- Integration mocks empty array: integrations health shows empty honest state.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Overview MUST show current project name and environment label as context (command center header), not as a Settings form.
- **FR-002**: System health MUST use StatusChip for the primary health label/tone (healthy / degraded / down / unknown as applicable).
- **FR-003**: At least **two** distinct health surfaces on Overview MUST use StatusChip (examples: system health; nodes health; integrations health; health-event severity). StatusDot alone is insufficient for a labeled health surface that operators read as status.
- **FR-004**: Overview MUST summarize nodes health (connected vs disconnected / availability) from the overview model.
- **FR-005**: Overview MUST summarize integrations health from fixture/integration data (connected / error / disconnected style counts) with StatusChip-backed status grammar.
- **FR-006**: Overview MUST show pending approvals with count and a control that navigates to Inbox (pending) via existing shell quick-action / navigation callbacks.
- **FR-007**: Overview MUST show a sample of recent changes (activity fixtures) with open hand-off to Activity.
- **FR-008**: Overview MUST show operations in progress (running/queued) with open hand-off to Operations; run status uses StatusChip grammar for labeled status.
- **FR-009**: Overview MUST expose high-frequency shortcuts: Architecture, Resources, Operations, Inbox, Terminal.
- **FR-010**: Overview MUST surface data-source honesty for fixture / empty / offline modes in header or footer.
- **FR-011**: Overview MUST NOT invent multi-period history charts or live telemetry series when only snapshot fixture data exists. Proportion/snapshot visualizations already driven by real counts (e.g. component waffle from counts) MAY remain; synthetic history MUST NOT be added.
- **FR-012**: Overview MUST NOT require Summary/Activity page tabs.
- **FR-013**: Testable pure helpers in `overview-model` MUST cover health tone→StatusChip mapping, attention/pending filters, operations-in-progress filter, node/component/integration aggregates, data-source labels, and shortcut contract ids; tests MUST pass under the shell package vitest config.
- **FR-014**: Implementation scope is limited to shell overview page/cards/model, fixtures/types as needed, and tests — MUST NOT rewrite Activity page, Knowledge modules, Spec Kit modules, audit-detail, or unrelated P0/P1 features.
- **FR-015**: UI MUST NOT claim live production telemetry while using fixture/mock data.

### Key Entities

- **OverviewSummary**: Snapshot of project/environment context and aggregate counts (components, nodes, operations, pending approvals, resources by environment, optional integration aggregates, health label/tone).
- **OverviewDataSource**: `fixture` | `empty` | `offline` (honesty modes for demo vs empty vs offline).
- **AttentionSummary**: Pending approvals + operations-in-progress counts for attention metric.
- **ComponentSegments / NodeAvailability / IntegrationHealth**: Derived health aggregates with tone for StatusChip mapping.
- **OverviewShortcut**: High-frequency destination id + label (architecture, resources, operations, inbox, terminal).
- **PendingApproval / InProgressRun / RecentChange**: Filtered samples for list cards with hand-off ids.

### Constitution Constraints *(mandatory)*

- **I. Operational product**: Strengthens architecture understanding + safe execution awareness via command center snapshot — pass.
- **II. Domain before framework**: Aggregates and tone maps live in pure `overview-model` helpers — pass.
- **III. Runtime separation**: Renderer fixture UI only; no live telemetry claim — pass.
- **IV. Monorepo deps**: shell → design-system/charts; no cycles — pass.
- **V. Contracts-first**: Local UI contract note only; no new IPC — pass.
- **VI. Security**: No unrestricted renderer OS; no secrets in overview chrome — pass.
- **VII. Auditable mutations**: Overview is read-only navigation/hand-off; approvals open Inbox — pass.
- **VIII. Local-first**: Fixture/offline honesty explicit — pass.
- **IX–XI**: Small snapshot lists; no Main work — pass.
- **XII. Tests**: Model + UI contract tests required — pass.
- **XIII–XIV**: Extend existing overview surfaces; single mock model — pass.
- **Non-goals**: Activity page rewrite; Knowledge/Spec Kit modules; audit-detail depth; live metrics backends; Settings redesign; mandatory Summary/Activity tabs.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can identify project, environment, and overall system health within **5 seconds** of landing on Overview (visible without scrolling past one screen height on a standard desktop layout).
- **SC-002**: **100%** of labeled health statuses shown on Overview primary health surfaces use StatusChip grammar (no hand-rolled status pills for those surfaces).
- **SC-003**: From Overview, an operator can reach Inbox pending, Operations, Resources, Architecture, and Terminal with **one activation each** via attention control and/or shortcuts.
- **SC-004**: Empty, offline, and fixture modes each show a distinct honesty cue; fixture mode never reads as silent live telemetry.
- **SC-005**: Unit tests for overview-model helpers achieve green results for health mapping, filters, aggregates, data-source labels, and shortcut ids before the feature is marked done.
- **SC-006**: Overview remains a single command-center page without mandatory Summary/Activity tabs.

### Rejection Rubric (definition of done)

- [ ] StatusChip on ≥2 health surfaces of Overview
- [ ] Pending approvals clickable → Inbox
- [ ] Empty/offline/fixture state honest
- [ ] Tests of overview-model/cards contract pass
- [ ] MUST NOT rewrite Activity page (other agent)
- [ ] FORBIDDEN: claim “already implemented” because overview-cards is large

## Assumptions

- Shell App wiring already maps `onQuickAction` ids to sections; Overview may add `terminal` if not already mapped (App.tsx may need a one-line handler if missing — still Overview-owned navigation contract).
- Fixture data from workspace-mocks remains the default authority for this mock phase.
- Parallel agents own Activity actor inspection, Knowledge document modules, Spec Kit document modules, and Audit detail depth — Overview only deep-links where needed.
- Existing proportion charts driven by real snapshot counts (waffle/segmented bars) are allowed under FR-011; history charts are not.
