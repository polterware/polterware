# Feature Specification: Access Agents Detail

**Feature Branch**: `017-agents-detail`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Seguindo o padrão de docs/spec.md, AGENTS.md e os details production-ready já feitos (member-detail, team-detail, run/schedule/resource/node), desenvolver a tela de details de Agents no Access — production-ready, mockada, completa. Extrair composição dedicada do bloco inline em Access → Agents; enriquecer fixtures do agent; hand-off de session/operation só via callback (outros agentes implementam session/policy/audit details em paralelo)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect a project agent (Priority: P1)

As a project admin or operator with Access privileges, I select an agent in Access → Agents and open its detail panel to understand identity, status, autonomy, ownership, capabilities, tools, environments, linked operations, recent sessions, and activity — without leaving the Access surface or inventing a second inspector.

**Why this priority**: Agents are first-class governed collaborators. Without a production-grade agent detail, operators cannot verify posture, scope, or tool access before pause/revoke decisions.

**Independent Test**: With Access → Agents open and fixture agents loaded, activating any agent row shows a complete DetailPanel for that agent with identity, overview metrics, structured capabilities/tools, environments, operations, sessions, and activity; no second page or modal.

**Acceptance Scenarios**:

1. **Given** Access → Agents lists agents, **When** the operator activates an agent row, **Then** the global DetailPanel shows that agent's name and domain Bot icon, with body content for overview, capabilities, tools, environments, linked operations, recent sessions, and activity when present.
2. **Given** an agent detail is open, **When** the operator reviews Overview, **Then** they see type, status (`StatusChip`), autonomy, owner, scope, and last activity — without a second page or modal.
3. **Given** an agent has multiple capabilities and tools in fixtures, **When** detail opens, **Then** Capabilities and Tools render as structured collections (not only a comma-joined string).
4. **Given** the agent is found in fixtures, **When** the panel opens, **Then** loading is not required in the mock stage, and the panel never invents live IdentityService / core agent runtime authority.

---

### User Story 2 - Govern agent from detail (Priority: P1)

As a project admin, I use header actions on an agent detail to pause/resume, edit permissions, or revoke the agent. In this mock stage, actions confirm with honest mock feedback and do not claim real authorization enforcement.

**Why this priority**: Operators expect governance actions adjacent to the inspected agent identity, matching People/Team detail quality.

**Independent Test**: Open any agent detail and invoke Pause/Resume, Edit permissions, and Revoke; each produces clear mock-labeled feedback without navigating away or opening a forbidden second inspector.

**Acceptance Scenarios**:

1. **Given** agent detail is open for an Active agent, **When** the operator triggers Pause, **Then** a mock toast acknowledges the action and the panel remains on that agent.
2. **Given** agent detail is open for a Paused agent, **When** the operator triggers Resume, **Then** mock feedback acknowledges resume intent.
3. **Given** agent detail is open, **When** the operator triggers Edit permissions, **Then** mock feedback acknowledges the permissions-editor intent without claiming persistence.
4. **Given** agent detail is open, **When** the operator triggers Revoke, **Then** mock feedback acknowledges revoke intent; the UI does not imply a silent privilege path outside the shared mutation model.

---

### User Story 3 - Navigate to sessions and operations (Priority: P2)

As an operator reviewing agent posture, I open recent sessions and linked operations from agent detail via hand-off callbacks only. Session detail UI is owned by a sibling feature; agent detail MUST NOT redesign session/policy/audit details.

**Why this priority**: docs/spec.md treats agents as collaborators with sessions and operations; deep links are required for usable Access governance without dual ownership of sibling screens.

**Independent Test**: Open Deployment Agent fixture → activate a recent session row (or Open recent session header action) → Access activates `agent-session` target via callback; open linked operation id → `onOpenOperation` fires when wired.

**Acceptance Scenarios**:

1. **Given** agent detail shows sessions filtered by `agentId`, **When** the operator activates a session row, **Then** `onOpenSession(sessionId)` is invoked (Access wires activation to `entityKind: "agent-session"`).
2. **Given** agent has at least one recent session, **When** the operator uses the Open recent session header action, **Then** the same hand-off opens the most recent related session without embedding session detail UI.
3. **Given** agent has `recentOperationIds`, **When** the operator activates a linked operation, **Then** `onOpenOperation(operationId)` is invoked when the host provides it.
4. **Given** an agent with no sessions or no operations, **When** those sections render, **Then** each shows an explicit empty message; header Open recent session is omitted when none.

---

### User Story 4 - Honest empty, missing, and partial states (Priority: P2)

As an operator, when an agent has no tools, no sessions, no operations, no activity, or the target is missing from fixtures, the detail remains honest: empty collections show empty states; missing entities show not-found; the UI never fabricates live runtime authority.

**Why this priority**: Explicit state beats magic; production-ready detail must handle incomplete agent data.

**Independent Test**: Force not-found and empty collection fixtures; panel shows PageError / empty table states without crashing.

**Acceptance Scenarios**:

1. **Given** an activation target for an agent id not in fixtures, **When** the panel resolves, **Then** it shows a not-found state with a clear message.
2. **Given** an agent with empty tools, sessions, operations, or activity, **When** those sections render, **Then** each shows an explicit empty message, not invented rows.
3. **Given** mock stage, **When** any governance action fires, **Then** feedback is labeled as mock and does not claim backend success beyond the toast.
4. **Given** Access Agents tab also lists sessions below agents, **When** an agent detail is open, **Then** agent detail only mounts for `entityKind === "agent"`; session rows remain separate activation targets.

---

### Edge Cases

- Agent status Active / Paused / Revoked / Disabled — status labeled via StatusChip, not ad-hoc pills.
- Autonomy supervised / semi-autonomous / autonomous — readable labels; not free-form only.
- Agent with zero sessions or zero operations — sections remain and show empty states honestly.
- Long agent name / long tool names — stay single-line where PrimaryCell rules apply; tables scroll with the DetailPanel body only.
- Simultaneous Agents tab entities (agent / agent-session) — agent detail only mounts for `entityKind === "agent"`; no dual content clash with session detail.
- Sibling parallel work (policy / audit / session details) MUST NOT be rewritten by this feature; hand-offs use callbacks only.
- Capability and tool tables are **not** authorization enforcement; real authz remains backend-owned in future stages.
- Fixture `dataSource` remains fixture; NEVER claim live core / IdentityService.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Access → Agents MUST open agent detail in the single global DetailPanel hosted by MainSurface; MUST NOT invent a second entity-detail page, drawer, or modal inspector for agents.
- **FR-002**: Agent detail MUST use `DetailPanel.Body layout="sections"` + `DetailPanel.Grid` for operational agent content (not document layout).
- **FR-003**: Agent detail MUST show identity header: domain Bot icon matching the Agents table PrimaryCell family, single-line agent name, and feature-owned header actions via `DetailPanel.Action` only.
- **FR-004**: Agent detail MUST expose mock governance actions: Pause/Resume (label depends on status), Edit permissions, Revoke. Actions MUST use honest mock feedback in this stage. Open recent session MUST appear only when a related session exists and MUST use callback hand-off only.
- **FR-005**: Agent detail MUST present Overview metadata: type, status (`StatusChip`), autonomy, owner, scope, last activity, and optional description/created/notes when present.
- **FR-006**: Agent detail MUST present Capabilities as a structured collection (not only a joined string) with empty state when none.
- **FR-007**: Agent detail MUST present Tools / MCP tools as a structured collection (name + kind) with empty state when none.
- **FR-008**: Agent detail MUST present Environments as a list or empty copy.
- **FR-009**: Agent detail MUST present Linked recent operations (ids) with activation via `onOpenOperation` when provided; empty state when none.
- **FR-010**: Agent detail MUST present Recent sessions as a compact collection from sessions filtered by agent id (from host/session fixtures); activating a session ONLY via callback; MUST NOT embed or redesign session detail UI.
- **FR-011**: Agent detail SHOULD present Activity / provenance notes when the model provides them, with empty omission or empty state as appropriate.
- **FR-012**: Agent detail MUST support loading, error, and not-found honest states consistent with other operational details (People / Teams / Resources).
- **FR-013**: Fixture data for agents MUST be rich enough for complete detail compositions (structured capabilities/tools, environments, operations links, sessions via agentId, optional activity/notes) and covered by desktop detail-fixture-coverage for the primary Deployment Agent fixture when fields expand.
- **FR-014**: Implementation MUST extract a dedicated agent detail composition (not only an inline thin block in the Access index) following MemberDetail / TeamDetail quality.
- **FR-015**: Status chips and capability/tool rows MUST NOT claim frontend authorization. Mock stage MUST remain honest that enforcement is not live. `dataSource` is fixture only.
- **FR-016**: Agent detail MUST remain scoped to project Access Agents. Live IdentityService, real agent runtime control, session detail UI, policy detail, and audit detail stay out of scope (sibling agents own session/policy/audit).
- **FR-017**: Collection activation remains independent of panel snap mode; selecting a row updates the target without forcing maximize.
- **FR-018**: Access index wiring MUST only replace the agent `Show` block with `<AgentDetail … />`; MUST NOT rewrite policy/audit/session/member/team details.

### Key Entities

- **Agent**: Project-scoped governed collaborator (AI agent or application identity) with status, autonomy, owner, scope, capabilities, tools, environments, and linked operations.
- **Capability**: Structured permission/capability granted to the agent for display (not enforcement).
- **Tool**: Named tool or MCP surface the agent may invoke (kind: mcp / builtin / integration).
- **Agent Session (reference)**: Session summary filtered by `agentId` for hand-off; full session detail is sibling-owned.
- **Linked Operation**: Operation/run id for deep-link hand-off to Operations surfaces.
- **Agent Activity Event**: Recent governance or operational event involving the agent (optional fixture enrichment).

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: governance + collaboration inspection for agents in Access.
- **Runtime owner (III)**: Renderer feature package `@polterware/feature-access` + shell workspace mocks; no Main/Node privilege for this UI stage. Never claim unimplemented agent runtime as live.
- **Security (VI)**: Capabilities/tools display-only; no secrets/tokens; mock pause/revoke is not real mutation authority.
- **SSOT (XIV)**: Single `AgentDetail` composition; replace inline agent block.
- **DetailPanel LAW**: Global panel only; StatusChip for status; DataTable for collections; no PrimaryCell descriptions; no Actions columns.

### Assumptions

- Mock stage uses shell `AGENT_MOCKS` / `AGENT_SESSION_MOCKS`; no ConnectRPC agent control plane in this slice.
- Session detail, policy detail, and audit detail are implemented in parallel by other agents; this feature only passes callbacks.
- Primary completeness fixture is Deployment Agent (`a1`) with non-empty capabilities, tools, environments, and at least one linked session (`sess-184`) and operation (`run-1`).
- Linear Bridge (`a4`, Paused) exercises Pause/Resume label flip and thinner collections honesty where applicable.
- English UI strings; Portuguese only for operator chat.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can open agent detail from Access → Agents and inspect identity + overview without leaving Access (100% of fixture agents).
- **SC-002**: Capabilities and tools are readable as structured rows for the primary Deployment Agent fixture (not only a single joined string).
- **SC-003**: Pause/Resume, Edit permissions, and Revoke are reachable from the header and produce mock-labeled feedback within one interaction each.
- **SC-004**: Session hand-off works for agents with sessions; empty session state is explicit for agents without sessions.
- **SC-005**: Not-found and loading states are present and copy-honest; package helper + composition contract tests pass for agent detail.
- **SC-006**: Single dedicated composition is wired; Access Agents no longer depends on a thin inline agent detail block for the ready state.
- **SC-007**: Fixture coverage gate remains green when agent model fields expand (desktop detail-fixture-coverage for primary agent).

## Out of Scope

- Agent session detail UI (sibling `020-agent-session-detail`)
- Policy detail UI (sibling `018-policies-detail`)
- Audit event detail UI (sibling `019-audit-detail`)
- Live IdentityService / polter-core agent control APIs
- Real pause/resume/revoke persistence or plan→approve→apply pipeline
- MCP tool invocation from the detail panel
- Org-wide agent directory product
