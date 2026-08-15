# Feature Specification: Operations Run Detail

**Feature Branch**: `012-operations-run-detail`

**Created**: 2026-08-11

**Status**: Draft

**Input**: User description: "Following docs/spec.md, develop the Operations Runs detail screen as production-ready mocked UI in the global DetailPanel pattern — full operator run inspection (status, steps, output, governance, result) without claiming live polter-core execution."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect a run from the Operations list (Priority: P1)

An operator opens **Operations → Runs**, selects a run row, and the global bottom **DetailPanel** shows a production-quality run detail: identity, status, timing, target/context, step progress, output, and result. The list remains visible under the open panel; maximizing covers MainSurface only.

**Why this priority**: Run inspection is the primary operations loop after requesting or discovering an execution. Without a complete detail surface, the Runs list is a dead-end.

**Independent Test**: With fixture runs only, open Operations → Runs, activate any row, and verify DetailPanel content matches the selected run with no empty structural sections when the fixture supplies data.

**Acceptance Scenarios**:

1. **Given** the Runs tab with at least one fixture run, **When** the operator activates a row, **Then** DetailPanel shows that run’s title, domain icon, status, and core metadata without leaving the Operations page.
2. **Given** an open run detail, **When** the operator activates another run row, **Then** DetailPanel content switches to the new run without unmounting the panel host or inventing a second inspector.
3. **Given** a run with steps and output, **When** the operator opens detail (open or maximized), **Then** steps appear as a sortable table with status chips and durations, and output appears as a readable stream with honest empty state when none exists.

---

### User Story 2 - Understand governance and execution honesty (Priority: P1)

The operator can see who requested the run (human, agent, or system), the correlation identity for audit, where execution is bound (local core, node, or provider), whether approval is linked, and the explicit result — including partial, denied, cancelled, or lost outcomes without false “failed” labeling.

**Why this priority**: docs/spec.md §113 and §24 require actor, target, correlation ID, explicit execution location, immutable approval binding, and honest outcomes. Operators must not be misled about authority or state.

**Independent Test**: Open fixtures covering success, running, failed, waiting approval, cancelled, and lost/unknown-outcome shapes; each shows required governance fields or an explicit “—” / unavailable reason, never fabricated success.

**Acceptance Scenarios**:

1. **Given** a completed production deploy run, **When** detail opens, **Then** actor, correlation ID, execution location, environment, target, and result are visible.
2. **Given** a run that required approval, **When** detail opens, **Then** approval linkage and decision state are shown; approved is not presented as “already executed” without a separate execution result.
3. **Given** a cancelled or lost run, **When** detail opens, **Then** status and result messaging stay honest (cancellation best-effort / outcome unknown) and do not claim provider rollback when unsupported.

---

### User Story 3 - Act on a live or terminal run (Priority: P2)

From detail header actions, the operator can cancel a non-terminal run or request retry on a failed run. Actions use fixture-honest feedback (toast or disabled with reason) until core execution is wired; UI never pretends the action succeeded against a live runtime.

**Why this priority**: Cancel/retry close the operator loop, but honesty about mock vs core is mandatory.

**Independent Test**: On a running fixture, Cancel is available and feedback is fixture-honest; on a successful terminal run, Cancel is unavailable or clearly inert; on a failed run, Retry is available with honest feedback.

**Acceptance Scenarios**:

1. **Given** a running run, **When** the operator chooses Cancel, **Then** feedback is explicit and audit-oriented wording is used; UI does not claim the underlying action fully stopped unless the model says so.
2. **Given** a failed run, **When** the operator chooses Retry, **Then** the product acknowledges a new request path (fixture toast) without rewriting the historical run as success.
3. **Given** a terminal success run, **When** detail is open, **Then** Cancel is not offered as a primary live control.

---

### User Story 4 - Cover edge states without scenic emptiness (Priority: P3)

Sparse runs (minimal metadata), empty steps, empty output, denied/waiting-approval, and not-found selection all render usable detail chrome with honest empty sections — never a blank panel or a second full-page route.

**Why this priority**: Production readiness includes degraded and incomplete fixtures; operators hit these states early.

**Independent Test**: Activate fixtures with optional fields omitted; panel still shows header + sections with empty copy; invalid target keeps or collapses per existing DetailPanel collection rules.

**Acceptance Scenarios**:

1. **Given** a run without steps, **When** detail opens, **Then** Steps section shows an empty state, not a broken table.
2. **Given** a run without output lines, **When** detail opens, **Then** Output shows “No output captured” (or equivalent).
3. **Given** loading or missing entity props (if used by host), **When** detail mounts, **Then** loading/not-found states mirror other operational details (Resource/Node).

---

### Edge Cases

- Run still `waiting_approval` / pending with no execution started — show governance, hide false progress.
- Run `lost` or indeterminate external outcome — never coerce to failed.
- Rollback section only when the model marks rollback as available; otherwise omit or show “Not supported”.
- Fixture `dataSource: "fixture"` must remain visible as source honesty where other surfaces show source.
- Long output streams must remain scrollable inside DetailPanel body only (no competing outer scroller).
- Agent-triggered runs must show agent as actor distinctly from human.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Operations Runs detail MUST render exclusively in the global `DetailPanel` hosted by MainSurface; features MUST NOT add a second inspector, drawer, modal-as-detail, or dedicated run detail route.
- **FR-002**: Detail MUST use operational `DetailPanel.Body layout="sections"` with Grid / Section / Metric / Metadata patterns (not knowledge `document` layout).
- **FR-003**: Selecting a run in the Operations Runs DataTable MUST activate detail target `scopeId: operations:runs`, `entityKind: run`.
- **FR-004**: Detail header MUST show a 16px domain icon consistent with the Runs list PrimaryCell, a single-line title (run title), and header actions via `DetailPanel.Action` only.
- **FR-005**: Detail MUST surface status with shared `StatusChip` (filled semantic chip; no hand-rolled pills).
- **FR-006**: Detail MUST show summary metrics for status, duration, and step completion progress when steps exist (visualization optional but preferred when counts allow).
- **FR-007**: Detail MUST show identity and audit fields: actor (with actor kind when known), started/finished times, duration, correlation ID, and operation/request id when present.
- **FR-008**: Detail MUST show execution context: action/kind, target, environment, and explicit execution location (`local_core` | `node` | `provider` or human labels Local core / Node / Provider).
- **FR-009**: Detail MUST show governance linkage when present: approval id/status, policy check summary, risk; absence is honest empty, not fabricated approval.
- **FR-010**: Detail MUST list run steps in a design-system `DataTable` (step name, status chip, duration); no Actions column.
- **FR-011**: Detail MUST show output/progress lines in a mono stream section with honest empty state; truncation MUST be explicit if applied.
- **FR-012**: Detail MUST show an explicit result (outcome + message); MUST NOT claim rollback when unsupported; MUST NOT map unknown external outcome to failed.
- **FR-013**: Cancel action MUST only be offered for non-terminal statuses; Retry for failed (and optionally cancelled when product allows re-request); both MUST use fixture-honest feedback until core is live.
- **FR-014**: Mock data MUST remain labeled `dataSource: "fixture"` (or equivalent source field) and MUST NOT present as live polter-core stream.
- **FR-015**: At least one fixture MUST cover each of: success with steps+output, running in-progress, failed, sparse minimal row, and waiting-approval or governance-linked run.
- **FR-016**: Implementation MUST extract a dedicated run detail composition module (not only inline page markup) for reuse and testing, matching Resource/Node detail structure.
- **FR-017**: Collection registration, open/collapsed/maximized snaps, and Escape/maximize behavior remain owned by existing DetailPanel / shell laws — this feature MUST NOT reimplement sizing chrome.

### Key Entities

- **Operation run**: One execution attempt of an operation intent (command, deploy, provision step, workflow-bound run, sync, agent action). Attributes: identity, status, actor, target, environment, execution location, correlation ID, timing, steps, output, result, optional approval/workflow links, data source honesty.
- **Run step**: Ordered unit of progress within a run (validate, approval wait, build, deploy, health check, command, notify). Status and optional duration.
- **Run result**: Terminal or intermediate outcome record (succeeded, failed, cancelled, denied, lost/partial) with operator-facing message and optional rollback support flag.
- **Approval binding**: Immutable link from intent/run to an approval decision when policy required it.
- **Actor**: Human, agent, or system identity that requested the run (distinct audit identity).

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: Safe execution observation + audit continuity for operations runs.
- **Runtime owner (III)**: Renderer feature package only for this mocked UI; no Main privilege; no claimed live core stream.
- **Domain vs adapter (II, IV)**: Operations domain model in feature types/fixtures; no provider-specific hardcoding beyond fixture labels.
- **Contracts (V)**: Feature UI contract for run detail view-model; no protobuf change required for mocked surface.
- **Product loop impact**: Operations list → DetailPanel inspect → (future) cancel/retry → audit/correlation; Approvals remain Inbox for decision work.
- **Evidence expectations**: Steps/output/result are operator-visible evidence of execution; fixture source is explicit.
- **Mutation safety (VII)**: Cancel/retry UI is plan/request oriented with honest mock feedback; no silent destructive apply.
- **Operation audit fields**: actor, target, environment, approval link, correlation/operation ID required in view model when known.
- **Security boundary (VI)**: No secrets in output fixtures; no MCP bypass; renderer remains typed bridge only.
- **Local vs cloud authority (VIII)**: Fixture-local; offline-friendly mock UI.
- **Performance (IX)**: Detail body is sole vertical scroller; long output stays bounded by panel.
- **Resilience (X)**: Honest lost/cancelled/empty states.
- **Observability (XI)**: Correlation ID visible for hand-off.
- **Test plan (XII)**: Unit tests for status mapping/helpers and fixture completeness; optional render smoke; desktop detail-fixture-coverage extended.
- **Structure created now (XIII)**: Extend `packages/features/operations` only (types, fixtures, run-detail module, tests); no scenic new package.
- **Single source of truth (XIV)**: One run detail composition; remove/replace inline minimal detail in Operations page.
- **Big-bang / refactor context (XV)**: Continues operator product surfaces on authorized refactor branch.
- **Non-goals (I, XIII)**: No live log streaming backend, no full CI/CD clone, no Datadog/Grafana embed, no Terminal drawer reuse as run detail, no separate run full-page editor.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can select any fixture run and identify status, target, actor, and timing in under 10 seconds without leaving Operations.
- **SC-002**: 100% of fixture runs used in the Runs list resolve to a detail view that never shows blank structural chrome (header + at least summary sections or honest empty states).
- **SC-003**: Governance-critical fields (correlation ID, execution location, approval when linked) are present on the primary “rich” fixtures; reviewers can verify against docs/spec.md §113 without code spelunking.
- **SC-004**: Cancel/Retry actions never claim live core success; mock feedback is explicit in operator-facing copy.
- **SC-005**: Visual and interaction patterns match Resource/Node operational details (sections grid, StatusChip, DataTable steps, DetailPanel actions) with zero parallel field/menu/card chrome.
- **SC-006**: Fixture coverage test asserts expanded run shape (steps, output, correlation, location, result) for at least the primary success fixture.

## Assumptions

- Approvals remain decided primarily in **Inbox**; run detail shows linkage and status, not a full approval workspace.
- Workflow builder remains the workflow authoring surface; run detail may link workflow id/name/version read-only.
- Status vocabulary may use product-facing labels aligned with simplified run statuses while remaining compatible with docs/spec.md lifecycle (pending/queued/waiting approval/running/success/failed/cancelled/denied/lost).
- No backend/API wiring in this feature slice — fixtures only, labeled honestly.
- DESIGN.md and AGENTS.md DetailPanel laws fully apply; no exceptions beyond documented surface-border rules.
- Existing Operations tabs (Runs · Workflows · Schedules) and Run command dialog are out of redesign scope except wiring run detail.
- Portuguese chat with Erick; English for code, UI strings, and this specification.
