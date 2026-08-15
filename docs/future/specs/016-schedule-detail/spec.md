# Feature Specification: Operations Schedule Detail

**Feature Branch**: `016-schedule-detail`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Following docs/spec.md and the established production-ready DetailPanel pattern (Run / People / Resource details), develop the Operations Schedules detail screen — complete mocked delivery, production-ready UI, wired from Operations → Schedules list, no live polter-core scheduler claimed."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect a schedule from the Operations list (Priority: P1)

An operator opens **Operations → Schedules**, selects a schedule row, and the global bottom **DetailPanel** shows a production-quality schedule detail: name, status, cadence (cron/recurrence + timezone), next and last fire times, linked workflow, target/environment, service identity for scheduled execution, recent run history, and governance notes. The list remains visible under the open panel; maximizing covers MainSurface only.

**Why this priority**: Schedule inspection closes the operator loop for recurring operations. Without a complete detail surface, the Schedules list is a dead-end and operators cannot verify what will fire next or under which identity.

**Independent Test**: With fixture schedules only, open Operations → Schedules, activate any row, and verify DetailPanel content matches the selected schedule with no empty structural chrome when the fixture supplies data.

**Acceptance Scenarios**:

1. **Given** the Schedules tab with at least one fixture schedule, **When** the operator activates a row, **Then** DetailPanel shows that schedule’s name, domain icon, status chip, and core cadence/next-run metadata without leaving Operations.
2. **Given** an open schedule detail, **When** the operator activates another schedule row, **Then** DetailPanel content switches to the new schedule without inventing a second inspector or full-page route.
3. **Given** a schedule with recent run history, **When** the operator opens detail (open or maximized), **Then** recent runs appear as a sortable DataTable with status chips and timestamps, and empty history shows an honest empty state.

---

### User Story 2 - Understand service identity and governance honesty (Priority: P1)

The operator can see which **service identity** will own scheduled fires (docs/spec.md: scheduled run uses service identity with explicit capabilities), which workflow/version is bound, which environment and target scope apply, whether approval or policy gates apply on fire, and the schedule status (active, paused, disabled, error) without false claims of live scheduler authority.

**Why this priority**: Scheduled execution must not look like an anonymous privileged path. Operators need identity, capability, and policy posture before trusting a recurring fire.

**Independent Test**: Open active, paused, and error/disabled fixtures; each shows service identity, capabilities (or honest empty), workflow binding, and status; `dataSource: "fixture"` honesty is preserved.

**Acceptance Scenarios**:

1. **Given** an active production backup schedule, **When** detail opens, **Then** service identity name, capability list, workflow name, environment, cron expression, timezone, next run, and last run outcome are visible.
2. **Given** a paused schedule, **When** detail opens, **Then** status is Paused, next run reflects “not scheduled while paused” (or equivalent honest copy), and Resume is offered as a header action with fixture-honest feedback.
3. **Given** a schedule that requires approval on fire, **When** detail opens, **Then** governance section shows approval/risk/policy linkage and does not claim fires bypass policy.

---

### User Story 3 - Act on a schedule from detail (Priority: P2)

From detail header actions, the operator can pause an active schedule, resume a paused one, or request “Run now” (ad-hoc fire). Actions use fixture-honest feedback (toast) until core scheduling is wired; UI never pretends the action succeeded against a live runtime or that a fire completed.

**Why this priority**: Pause/resume/run-now close the operator control loop for schedules, but honesty about mock vs core is mandatory.

**Independent Test**: On an active fixture, Pause and Run now are available with mock feedback; on a paused fixture, Resume is available; disabled/error fixtures gate actions appropriately.

**Acceptance Scenarios**:

1. **Given** an active schedule, **When** the operator chooses Pause, **Then** feedback is explicit fixture/mock wording and does not claim durable persistence.
2. **Given** a paused schedule, **When** the operator chooses Resume, **Then** mock feedback acknowledges re-enable intent without inventing a new next-run from core.
3. **Given** any non-disabled schedule, **When** the operator chooses Run now, **Then** feedback describes a new ad-hoc operation request path (fixture toast), not a silent rewrite of history.

---

### User Story 4 - Cover edge states without scenic emptiness (Priority: P3)

Sparse schedules (minimal metadata), empty run history, disabled/error schedules, and not-found selection all render usable detail chrome with honest empty sections — never a blank panel or a second full-page route.

**Why this priority**: Production readiness includes degraded and incomplete fixtures; operators hit these states early.

**Independent Test**: Activate fixtures with optional fields omitted; panel still shows header + sections with empty copy; invalid target keeps or collapses per existing DetailPanel collection rules.

**Acceptance Scenarios**:

1. **Given** a schedule without recent runs, **When** detail opens, **Then** Recent runs shows an empty state, not a broken table.
2. **Given** a schedule without optional governance fields, **When** detail opens, **Then** Governance shows honest empty copy, not fabricated approvals.
3. **Given** loading or missing entity props (if used by host), **When** detail mounts, **Then** loading/not-found states mirror other operational details (Run/Resource).

---

### Edge Cases

- Paused or disabled schedule — next run MUST NOT look like an imminent live fire.
- Missed-run policy (skip / catch-up / alert) when present MUST be explicit; when absent, show “—” or default copy, never invent behavior.
- Service identity without capabilities listed — honest empty capabilities list, never imply full privilege.
- Schedule bound to a workflow that failed last fire — last outcome chip remains honest (failed), not repainted as healthy because the schedule is Active.
- Fixture `dataSource: "fixture"` must remain visible as source honesty where other surfaces show source.
- Long recent-run tables scroll inside DetailPanel body only (no competing outer scroller).
- Create-schedule toolbar control remains out of redesign scope except that list activation wires detail (create may stay toast/no-op until a later authoring slice).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Operations Schedules detail MUST render exclusively in the global `DetailPanel` hosted by MainSurface; features MUST NOT add a second inspector, drawer, modal-as-detail, or dedicated schedule detail route.
- **FR-002**: Detail MUST use operational `DetailPanel.Body layout="sections"` with Grid / Section / Metric / Metadata patterns (not knowledge `document` layout).
- **FR-003**: Selecting a schedule in the Operations Schedules DataTable MUST activate detail target `scopeId: operations:schedules`, `entityKind: schedule`.
- **FR-004**: Detail header MUST show a 16px domain icon consistent with the Schedules list PrimaryCell (`CalendarClock` / execution family), a single-line title (schedule name), and header actions via `DetailPanel.Action` only.
- **FR-005**: Detail MUST surface status with shared `StatusChip` (filled semantic chip; no hand-rolled pills).
- **FR-006**: Detail MUST show summary metrics for status, next run, last run outcome (or last fire time), and a fourth metric useful for operators (e.g. success rate of recent fires, or timezone).
- **FR-007**: Detail MUST show cadence fields: human-readable recurrence and/or cron expression, timezone, and optional description.
- **FR-008**: Detail MUST show identity and honesty fields: schedule id (mono), data source (`fixture` vs `core`), created/updated when known.
- **FR-009**: Detail MUST show execution context: linked workflow name/id/version when present, environment, target/scope, and execution location family when known.
- **FR-010**: Detail MUST show **service identity** for scheduled fires (name + kind) and explicit capability labels when present, aligning with docs/spec.md “scheduled run uses service identity with explicit capabilities”.
- **FR-011**: Detail MUST show governance linkage when present: approval required on fire, risk, policy notes; absence is honest empty, not fabricated approval.
- **FR-012**: Detail MUST list recent scheduled runs in a design-system `DataTable` (run title or id, status chip, started/finished); no Actions column.
- **FR-013**: Detail MAY show a compact timeline of schedule lifecycle events (created, paused, resumed, last fire) when fixture data provides them.
- **FR-014**: Pause action MUST only be offered for active schedules; Resume for paused; Run now for active or paused (not for permanently disabled/error unless product allows); all MUST use fixture-honest feedback until core is live.
- **FR-015**: Mock data MUST remain labeled `dataSource: "fixture"` (or equivalent source field) and MUST NOT present as live polter-core scheduler.
- **FR-016**: At least one fixture MUST cover each of: active rich schedule with history, paused schedule, sparse minimal schedule, and a schedule with governance/approval-on-fire and/or last-failed outcome.
- **FR-017**: Implementation MUST extract a dedicated schedule detail composition module (not only inline page markup) for reuse and testing, matching Run/Resource detail structure.
- **FR-018**: Collection registration for `operations:schedules`, open/collapsed/maximized snaps, and Escape/maximize behavior remain owned by existing DetailPanel / shell laws — this feature MUST NOT reimplement sizing chrome.
- **FR-019**: Schedules list MUST use StatusChip for status column (not raw text) and MUST NOT use Actions columns; PrimaryCell uses the domain schedule icon.

### Key Entities

- **Operation schedule**: Recurring intent that, when due, creates operation requests / workflow runs under a bound service identity. Attributes: identity, name, status, cadence (cron/recurrence + timezone), next/last fire, workflow binding, environment/target, service identity + capabilities, governance flags, recent runs, timeline, data source honesty.
- **Schedule status**: Active | Paused | Disabled | Error (operator-facing; maps to product lifecycle without claiming live tick state).
- **Service identity**: Non-human identity used for scheduled execution, with explicit capability labels (never implied full admin).
- **Scheduled fire / recent run**: Summary of a past (or in-flight) execution spawned by the schedule — id, title, status, timing — distinct from the schedule definition itself.
- **Workflow binding**: Optional link to a workflow definition/version that the schedule triggers.
- **Missed-run policy**: Optional operator-visible policy for missed windows (skip, catch-up, alert).

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: Safe observation and control of scheduled operational intents + audit continuity.
- **Runtime owner (III)**: Renderer feature package only for this mocked UI; no Main privilege; no claimed live core scheduler.
- **Domain vs adapter (II, IV)**: Operations domain model in feature types/fixtures; no provider-specific hardcoding beyond fixture labels.
- **Contracts (V)**: Feature UI contract for schedule detail view-model; no protobuf change required for mocked surface.
- **Product loop impact**: Operations Schedules list → DetailPanel inspect → pause/resume/run-now (mock) → future core scheduling; Approvals remain Inbox for decisions; Workflow builder remains authoring surface.
- **Evidence expectations**: Recent runs and timeline are operator-visible evidence; fixture source is explicit.
- **Mutation safety (VII)**: Pause/resume/run-now UI is request-oriented with honest mock feedback; no silent destructive apply.
- **Operation audit fields**: service identity, target/environment, workflow binding, schedule id required in view model when known.
- **Security boundary (VI)**: No secrets in fixtures; no MCP bypass; renderer remains typed bridge only; scheduled path never implies anonymous privilege.
- **Local vs cloud authority (VIII)**: Fixture-local; offline-friendly mock UI.
- **Performance (IX)**: Detail body is sole vertical scroller; recent runs stay bounded by panel.
- **Resilience (X)**: Honest paused/disabled/error/empty states.
- **Observability (XI)**: Schedule id and recent run correlation (when present) visible for hand-off.
- **Test plan (XII)**: Unit tests for status mapping/helpers and fixture completeness; desktop detail-fixture-coverage extended.
- **Structure created now (XIII)**: Extend `packages/features/operations` only (types, fixtures, schedule-detail module, list wiring, tests); no scenic new package.
- **Single source of truth (XIV)**: One schedule detail composition; replace minimal inline schedule table with fixture-backed list + detail.
- **Big-bang / refactor context (XV)**: Continues operator product surfaces on authorized refactor branch.
- **Non-goals (I, XIII)**: No live cron engine, no full CI/CD scheduler clone, no calendar heatmap product, no separate schedule full-page editor, no claiming durable pause/resume persistence.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can select any fixture schedule and identify status, next run, workflow, and service identity in under 10 seconds without leaving Operations.
- **SC-002**: 100% of fixture schedules used in the Schedules list resolve to a detail view that never shows blank structural chrome (header + at least summary sections or honest empty states).
- **SC-003**: Service identity and capability posture are present on the primary rich fixtures; reviewers can verify alignment with docs/spec.md scheduled-execution identity rule without code spelunking.
- **SC-004**: Pause/Resume/Run now actions never claim live core success; mock feedback is explicit in operator-facing copy.
- **SC-005**: Visual and interaction patterns match Run/Resource operational details (sections grid, StatusChip, DataTable history, DetailPanel actions) with zero parallel field/menu/card chrome.
- **SC-006**: Fixture coverage test asserts expanded schedule shape (cadence, timezone, service identity, recent runs, dataSource) for at least the primary active fixture.

## Assumptions

- Schedules live under **Operations → Schedules** (already routed); they are operation/workflow schedules, not calendar/UI appointments.
- Workflow builder remains the workflow authoring surface; schedule detail may show workflow id/name/version read-only and MAY offer a future jump later — not required in this slice.
- Create schedule toolbar button may remain non-authoring (no full create form) in this slice; detail for existing fixtures is in scope.
- No backend/API wiring — fixtures only, labeled honestly.
- DESIGN.md and AGENTS.md DetailPanel laws fully apply.
- Existing Operations tabs (Runs · Workflows · Schedules) stay; only Schedules list + detail are upgraded.
- Portuguese chat with Erick; English for code, UI strings, and this specification.
