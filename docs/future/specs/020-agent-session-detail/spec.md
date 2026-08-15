# Feature Specification: Access Agent Session Detail

**Feature Branch**: `020-agent-session-detail`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Following docs/spec.md, AGENTS.md, and production-ready detail patterns (member-detail, team-detail, run-detail timeline/steps), develop Access → Agents → Agent Session detail — production-ready, mocked, complete. Dedicated composition for Recent sessions table; enrich session/step fixtures; do not implement Agent identity detail (017)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect an agent session from Recent sessions (Priority: P1)

As a project admin or operator with Access privileges, I open **Access → Agents**, select a row in **Recent sessions**, and the global bottom **DetailPanel** shows a production-quality **agent session** detail: session title, status, environment, timing, objective, owning agent identity, step progress, and ordered steps. The Agents list remains under the panel; maximizing covers MainSurface only.

**Why this priority**: Recent sessions are the operational trace of agent work. Without a complete session detail, operators cannot understand what an agent is doing, where it stalled, or how far it progressed.

**Independent Test**: With fixture sessions only, open Access → Agents → Recent sessions, activate any row, and verify DetailPanel content matches the selected session with no empty structural chrome when the fixture supplies data.

**Acceptance Scenarios**:

1. **Given** the Agents tab with at least one fixture session in Recent sessions, **When** the operator activates a session row, **Then** DetailPanel shows that session’s title, domain icon (activity/bot family), status chip, and core overview metrics without leaving Access.
2. **Given** an open session detail, **When** the operator activates another session row, **Then** DetailPanel content switches to the new session without inventing a second inspector or full-page route.
3. **Given** a session with steps, **When** the operator opens detail (open or maximized), **Then** steps appear as a sortable DataTable with status chips and optional tool/timing columns, and empty steps show an honest empty state.

---

### User Story 2 - Understand agent identity, progress, and waiting/governance (Priority: P1)

The operator can see which agent identity owns the session (name via agent lookup, open via hand-off), step progress (counts + segmented progress visualization when steps exist), objective text, environment and actor context, and — when status is Waiting — waiting reason and optional approval linkage. Related operation/audit ids appear when fixtures provide them. Fixture source honesty is preserved.

**Why this priority**: docs/spec.md requires humans and agents under the same governance model; sessions must not look like an opaque privileged path. Waiting-for-approval sessions must show linkage without claiming execution completed.

**Independent Test**: Open Running, Completed, Waiting, and Failed fixtures; each shows objective, agent link, status chip, step progress (or honest empty), and Waiting-specific governance when applicable.

**Acceptance Scenarios**:

1. **Given** a running production deploy session, **When** detail opens, **Then** status Running, environment, started time, objective, agent name, step progress bar, and multi-status steps are visible.
2. **Given** a waiting session with approval linkage, **When** detail opens, **Then** waiting reason and approval id/status are shown; the UI does not claim the blocked step already executed.
3. **Given** a completed or failed session, **When** detail opens, **Then** duration/ended timing (when present) and terminal step outcomes stay honest (including Failed steps).

---

### User Story 3 - Act on a live session and hand off to agent identity (Priority: P2)

From detail header actions, the operator can **Terminate** a non-terminal session (Running or Waiting) with honest mock feedback, **Open agent** to hand off to the agent identity detail path via callback, and optionally **Open related operation** when a related operation id is present. Actions never claim live session kill or core mutation.

**Why this priority**: Terminate closes the control loop for runaway sessions; Open agent connects operational session trace to identity/governance without reimplementing agent detail.

**Independent Test**: On a Running/Waiting fixture, Terminate is available and mock-labeled; on Completed/Failed, Terminate is not offered as a primary live control; Open agent invokes the provided callback with agentId.

**Acceptance Scenarios**:

1. **Given** a Running or Waiting session, **When** the operator chooses Terminate, **Then** feedback is explicit fixture/mock wording and does not claim durable session invalidation against core.
2. **Given** a Completed or Failed session, **When** detail is open, **Then** Terminate is not offered as an active control.
3. **Given** any session with a known agentId, **When** the operator chooses Open agent, **Then** the host receives `onOpenAgent(agentId)` (or equivalent) and this feature does not render a second agent-identity detail composition.

---

### User Story 4 - Cover edge states without scenic emptiness (Priority: P3)

Sparse sessions (minimal metadata), empty steps, missing agent name fallback, not-found selection, loading, and error all render usable detail chrome with honest empty sections — never a blank panel or a second full-page route.

**Why this priority**: Production readiness includes degraded and incomplete fixtures; operators hit these states early.

**Independent Test**: Activate fixtures with optional fields omitted; panel still shows header + sections with empty copy; invalid target shows not-found; loading/error props mirror other operational details.

**Acceptance Scenarios**:

1. **Given** a session without steps, **When** detail opens, **Then** Steps shows an empty state, not a broken table.
2. **Given** a session whose agentId is not in agent fixtures, **When** detail opens, **Then** agent identity shows a fallback label (id or “Unknown agent”) without crashing.
3. **Given** loading or not-found entity props, **When** detail mounts, **Then** loading/not-found states mirror Member/Run detail honesty.

---

### Edge Cases

- Session status Waiting with no approvalId — show waiting reason or honest “—”; never invent an approval.
- Session status Failed with mixed Done/Failed steps — progress bar includes Failed segment; chips stay honest.
- Related operations / audit ids empty — omit open-operation action and show honest empty metadata, not fabricated links.
- Fixture `dataSource: "fixture"` must remain visible as source honesty where other surfaces show source.
- Long step tables scroll inside DetailPanel body only (no competing outer scroller).
- Parallel Agent identity detail (017) may land concurrently — this feature MUST only hand off via callback; MUST NOT redesign agent-detail.
- Policies detail (018) and Audit detail (019) are out of scope; session may display related audit ids as mono metadata only.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Access → Agents → Recent sessions detail MUST render exclusively in the global `DetailPanel` hosted by MainSurface; features MUST NOT add a second inspector, drawer, modal-as-detail, or dedicated session detail route.
- **FR-002**: Detail MUST use operational `DetailPanel.Body layout="sections"` with Grid / Section / Metric / Metadata patterns (not knowledge `document` layout).
- **FR-003**: Selecting a session in the Recent sessions DataTable MUST activate detail target `scopeId: access:agents`, `entityKind: agent-session`.
- **FR-004**: Detail header MUST show a 16px domain icon consistent with session/activity family (Activity or Bot), a single-line title (session title), and header actions via `DetailPanel.Action` only.
- **FR-005**: Detail MUST surface status with shared `StatusChip` (filled semantic chip; no hand-rolled pills). Status vocabulary for this surface: Running | Completed | Waiting | Failed.
- **FR-006**: Detail MUST show overview metrics for status, environment, startedAt, and duration and/or endedAt when present.
- **FR-007**: Detail MUST show the session objective as readable operator copy.
- **FR-008**: Detail MUST show agent identity linkage (agentId resolved to name via agent fixture lookup when available) and expose **Open agent** via callback — MUST NOT reimplement agent identity detail UI.
- **FR-009**: Detail MUST show step progress (done/total or equivalent) and, when steps exist, a segmented progress visualization covering Done / Running / Waiting / Failed (and Skipped if present).
- **FR-010**: Detail MUST list session steps in a design-system `DataTable` (step label, status chip, optional tool/action, optional started/finished or duration); no Actions column; empty steps show honest empty state.
- **FR-011**: When session status is Waiting (or a waiting reason/approval is present), detail MUST show waiting reason and optional approval linkage; absence is honest empty, not fabricated approval.
- **FR-012**: Detail MUST show environment and actor context (actor label/kind when known) and identity honesty fields (session id mono, data source fixture vs core, correlation id when known).
- **FR-013**: Detail MAY list related operation ids and audit ids as mono metadata; when a primary related operation exists, header MAY offer Open related operation via callback with mock/host navigation honesty.
- **FR-014**: Terminate action MUST only be offered for Running or Waiting statuses; MUST use fixture-honest mock feedback (toast) and MUST NOT claim live process/session kill.
- **FR-015**: Mock data MUST remain labeled `dataSource: "fixture"` (or equivalent) and MUST NOT present as live polter-core agent runtime.
- **FR-016**: Fixtures MUST cover at least Running, Completed, Waiting, and Failed sessions with rich enough steps for production detail (tool/action and timing on primary fixtures).
- **FR-017**: Implementation MUST extract a dedicated `AgentSessionDetail` composition module (not only inline page markup) for reuse and testing, matching Member/Team/Run detail structure.
- **FR-018**: Collection registration, open/collapsed/maximized snaps, and Escape/maximize behavior remain owned by existing DetailPanel / shell laws — this feature MUST NOT reimplement sizing chrome.
- **FR-019**: This feature MUST NOT implement or refactor Agent identity detail (`agent-detail.tsx` / specs/017), Policy detail (018), or Audit event detail (019).

### Key Entities

- **Agent session**: One operational execution trace of an agent identity for a concrete objective. Attributes: identity, agentId, title, objective, status, environment, timing, steps, optional waiting/approval linkage, related operation/audit ids, actor context, data source honesty.
- **Session step**: Ordered unit of progress within a session (read, plan, wait approval, tool call, execute, publish). Status and optional tool/action/timing.
- **Agent identity (link only)**: Project agent registry entity referenced by agentId; full posture lives under Agent detail (out of scope).
- **Approval binding (optional)**: Immutable link from session wait state to an approval decision when policy required it.
- **Related operation / audit reference**: Optional ids connecting the session to Operations runs and Access audit for hand-off.

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: Safe observation of agent operational sessions + governance continuity (waiting/approval) under Access.
- **Runtime owner (III)**: Renderer feature package `@polterware/feature-access` + shell workspace mocks; no Main/Node privilege for this UI stage; no claimed live agent runtime.
- **Domain vs adapter (II, IV)**: Session semantics presented from fixtures; no new authorization or agent orchestration engine inside Access UI.
- **Contracts (V)**: Feature UI contract `agent-session-detail-ui.md`; extends Access Agents session inspection.
- **Product loop impact**: Access Agents list → DetailPanel session inspect → Terminate (mock) / Open agent / optional Open operation; Approvals remain Inbox for decision work.
- **Evidence expectations**: Steps/progress/waiting reason are operator-visible evidence of agent work; fixture source is explicit.
- **Mutation safety (VII)**: Terminate is mock-only with toast honesty; no silent destructive apply or real kill path.
- **Operation audit fields**: correlation/session id, actor, environment, approval link when known.
- **Security boundary (VI)**: No secrets/tool credentials in session fixtures; renderer remains unprivileged; UI is not enforcement.
- **Local vs cloud authority (VIII)**: Fixture-local; offline-friendly mock UI.
- **Performance (IX)**: Detail body is sole vertical scroller; fixture step tables stay modest.
- **Resilience (X)**: Honest not-found / empty / waiting / failed states.
- **Observability (XI)**: Correlation/session ids visible for hand-off when present.
- **Test plan (XII)**: Unit tests for status/helpers + composition contract; desktop detail-fixture-coverage for enriched session shape.
- **Structure created now (XIII)**: Extend `packages/features/access` + shell workspace-mocks; no scenic package.
- **Single source of truth (XIV)**: One `AgentSessionDetail` composition; replace thin inline session detail in Access index.
- **Big-bang / refactor context (XV)**: Continues operator product surfaces on authorized refactor branch.
- **Non-goals (I, XIII)**: No live agent runtime, no real terminate/kill API, no agent identity detail rewrite, no policy/audit detail rewrite, no second full-page session editor, no MCP bypass.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can select any fixture agent session and identify status, objective, agent, environment, and step progress in under 10 seconds without leaving Access → Agents.
- **SC-002**: 100% of fixture sessions used in Recent sessions resolve to a detail view that never shows blank structural chrome (header + at least overview sections or honest empty states).
- **SC-003**: Running, Completed, Waiting, and Failed statuses are each represented by at least one fixture; Waiting fixtures expose waiting reason and/or approval linkage for review.
- **SC-004**: Terminate and Open agent never claim live core success beyond explicit mock feedback / host callback; Terminate is unavailable on terminal Completed/Failed sessions.
- **SC-005**: Visual and interaction patterns match Member/Team/Run operational details (sections grid, StatusChip, DataTable steps, DitherSegmentedBar progress, DetailPanel actions) with zero parallel field/menu/card chrome.
- **SC-006**: Desktop detail-fixture-coverage asserts enriched session shape (objective, multi-status steps, optional tool/timing on primary fixture, status coverage across the set).

## Assumptions

- Agent identity detail is owned by a parallel feature (017); this slice only links out via `onOpenAgent`.
- Approvals remain decided primarily in **Inbox**; session detail shows linkage and waiting reason, not a full approval workspace.
- Operations run detail remains the primary run inspector; session detail may surface related operation ids for host navigation only.
- No backend/API wiring in this feature slice — fixtures only, labeled honestly.
- DESIGN.md and AGENTS.md DetailPanel laws fully apply.
- Existing Access Agents agent-identity table and inline agent detail remain out of redesign scope for this feature.
- Portuguese chat with Erick; English for code, UI strings, and this specification.
