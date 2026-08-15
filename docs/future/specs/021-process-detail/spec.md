# Feature Specification: Resources Process Detail

**Feature Branch**: `021-process-detail`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Seguindo o padrão de docs/spec.md e o DetailPanel de production-ready (Nodes / Inventory / Runs), desenvolver a tela de details de Processes no Resources — entrega completa, mockada, production-ready. Extrair/enriquecer composição dedicada, enriquecer fixtures de process (node host, command, user, ports, resource usage, related component/runbook when known) e alinhar com o inventário operacional de nodes/processes."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect a running process (Priority: P1)

As an operator on Resources → Processes, I select a process row and open its detail panel to understand which process it is, on which node it runs, its health status, resource usage (CPU/memory), command line, user, ports, and how long it has been up — without leaving the Resources surface.

**Why this priority**: Process detail is the primary runtime inspection surface for host-level workload identity. Without production-grade process detail, operators cannot diagnose noisy or failed processes next to node inventory.

**Independent Test**: With Resources → Processes open and fixture processes loaded, activating any process row shows a complete DetailPanel with identity, host context, runtime metrics, command/user/port metadata, and honest empty/missing fields.

**Acceptance Scenarios**:

1. **Given** Resources → Processes lists processes, **When** the operator activates a process row, **Then** the global DetailPanel shows that process name and a domain process/CPU icon, with body content for status, host node, runtime metrics, and command metadata.
2. **Given** a process detail is open, **When** the operator reviews Overview, **Then** they see status, PID, node name, CPU, memory, started-at / uptime when known, user, port, and command — without a second page or modal.
3. **Given** a process fixture includes sparse fields (no port or no command), **When** detail opens, **Then** missing values show honest em dash / empty copy, not invented data.
4. **Given** the process is found in fixtures, **When** the panel opens, **Then** loading is not required in the mock stage, and the panel never invents live node/agent telemetry.

---

### User Story 2 - Take operational actions from process detail (Priority: P1)

As an operator, I use header actions on a process detail to view logs, open terminal context for the host node, restart, or stop the process. In this mock stage, actions confirm with honest mock feedback and do not claim real process control.

**Why this priority**: Process workbench actions must sit next to identity; operators expect control affordances adjacent to inspection.

**Independent Test**: Open any process detail and invoke View logs, Open terminal, Restart, and Stop; each produces clear mock feedback without navigating to a second inspector.

**Acceptance Scenarios**:

1. **Given** process detail is open, **When** the operator triggers View logs, **Then** mock feedback acknowledges log intent (or routes to an existing logs surface if already wired) without claiming live stream authority.
2. **Given** process detail is open, **When** the operator triggers Open terminal, **Then** mock feedback or existing terminal hand-off targets the host node context without inventing SSH.
3. **Given** process detail is open and status is Running, **When** the operator triggers Restart or Stop, **Then** mock feedback acknowledges intent; Stop is presented as destructive (danger tone).
4. **Given** a process is Stopped or Unknown, **When** detail opens, **Then** unavailable control actions are either hidden or disabled with an honest reason — not silently active as if live.

---

### User Story 3 - Relate process to host node and optional system context (Priority: P2)

As an operator diagnosing an issue, I open a process detail and understand its host node, with optional links to node detail and, when fixture data provides them, related architecture component or recent operation identity.

**Why this priority**: Processes are host-scoped; navigation back to Nodes is essential for operational flow.

**Independent Test**: Open a process whose nodeId matches a Nodes fixture; host section shows node name and allows open-node hand-off when wired; sparse fixtures without component links remain honest.

**Acceptance Scenarios**:

1. **Given** process fixture references a known node, **When** detail opens, **Then** host section shows node name (and node id when distinct) without requiring the operator to leave detail for basic identity.
2. **Given** open-node is wired, **When** the operator activates the host node link, **Then** Resources → Nodes detail for that node becomes the active target (or existing hand-off runs).
3. **Given** optional related component / operation fields are absent, **When** detail opens, **Then** those sections are omitted or show empty states — never fabricated names.

---

### User Story 4 - Honest empty, missing, and partial states (Priority: P2)

As an operator, when a process is missing from fixtures, stopped, or has incomplete telemetry, the detail remains honest: not-found, sparse metrics, and empty optional relations never crash or invent live authority.

**Why this priority**: Explicit state beats magic; production-ready detail must handle incomplete process samples.

**Independent Test**: Force not-found target and sparse process fixture; panel shows not-found / em dash metrics without crash.

**Acceptance Scenarios**:

1. **Given** an activation target for a process id not in fixtures, **When** the panel resolves, **Then** it shows a not-found state with a clear message.
2. **Given** a process with null CPU or memory, **When** metrics render, **Then** values show honest placeholders and no fake charts invent capacity history.
3. **Given** mock stage, **When** any control action fires, **Then** feedback is labeled as mock and does not claim backend success beyond the toast/hand-off.

---

### Edge Cases

- Process statuses Running / Stopped / Unknown — always labeled via StatusChip, never ad-hoc pills.
- Multiple processes sharing a host node — detail remains per-process; host section does not list sibling processes unless a deliberate related list is fixture-backed.
- Long command lines — mono text wraps or truncates inside DetailPanel body scroll only; no horizontal page overflow.
- PID as string or number in fixtures — display is consistent and mono when appropriate.
- Process control is **not** live node agent authority in mock stage.
- Logs and Terminal remain separate product surfaces; process detail only hands off or mocks.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Resources → Processes MUST open process detail in the single global DetailPanel hosted by MainSurface; MUST NOT invent a second entity-detail page, drawer, or modal inspector for processes.
- **FR-002**: Process detail MUST use `DetailPanel.Body layout="sections"` + `DetailPanel.Grid` for operational content (not document layout).
- **FR-003**: Process detail MUST show identity header: domain process/CPU icon matching the Processes table PrimaryCell family, single-line process name, and feature-owned header actions via `DetailPanel.Action` only.
- **FR-004**: Process detail MUST expose mock (or wired hand-off) actions: View logs, Open terminal, Restart, Stop. Stop MUST use danger tone. Actions MUST use honest mock feedback when no live runtime exists.
- **FR-005**: Process detail MUST present Overview metrics: status (`StatusChip`), PID, host node, CPU (optional visualization when percent known), memory, started-at when known.
- **FR-006**: Process detail MUST present Runtime metadata: command (mono when present), user, port, data source honesty (`fixture` / future `core`).
- **FR-007**: Process detail MUST present Host context: node name and node id when distinct; MAY offer open-node navigation when Resources node detail is available.
- **FR-008**: Process detail SHOULD present optional Related context when fixtures provide it (architecture component, resource, recent operation) with empty/omit when absent.
- **FR-009**: Process detail MUST support loading, error, and not-found honest states consistent with Nodes / Inventory details.
- **FR-010**: Fixture data for processes MUST be rich enough for complete detail compositions (at least one primary process with full runtime fields; at least one sparse process) and covered by desktop detail-fixture-coverage.
- **FR-011**: Implementation MUST keep a dedicated process detail composition (already partially present) at production People/Nodes quality — not a thin residual stub.
- **FR-012**: Status chips and control actions MUST NOT claim frontend authorization or live process kill success.
- **FR-013**: Collection activation remains independent of panel snap mode; selecting a row updates the target without forcing maximize.
- **FR-014**: Scope is project Resources Processes tab only. Full observability product, live agent process APIs, and log streaming backends stay out of scope.

### Key Entities

- **Process**: Host-scoped runtime process identity (name, PID, status, command, user, ports, resource usage, started-at).
- **Host Node**: Target machine/node on which the process runs; links to Resources Nodes inventory.
- **Runtime Metrics**: CPU percent, memory footprint, optional visualizations derived only from present numbers.
- **Process Control Intent**: Mock operator intents (logs, terminal, restart, stop) without live enforcement in this stage.
- **Related Context** (optional): Component / resource / operation references when fixtures provide them.

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: runtime inspection for processes on managed nodes.
- **Runtime owner (III)**: Renderer feature package `@polterware/feature-resources` + process fixtures; no Main/Node privilege for this UI stage.
- **Domain vs adapter (II, IV)**: UI composition over process/node concepts; fixtures stand in for node agent samples.
- **Contracts (V)**: new feature contract `process-detail-ui.md` under this feature directory.
- **Product loop impact**: Resources / DetailPanel / Terminal-logs hand-off; no System Map change required.
- **Evidence expectations**: n/a for live evidence; mock fixtures must be complete and honest.
- **Mutation safety (VII)**: mock-only control actions with toast acknowledgment; no claim of plan→approve→apply completion for stop/restart.
- **Security boundary (VI)**: unprivileged renderer; process control UI is not privilege elevation; secrets never shown in command fields beyond already-public fixture commands.
- **Local vs cloud authority (VIII)**: mock local fixtures only.
- **Performance (IX)**: DetailPanel body is sole vertical scroller.
- **Resilience (X)**: not-found / sparse / mock failure feedback without crash.
- **Test plan (XII)**: unit tests for process detail composition + fixture coverage expansion.
- **Single source of truth (XIV)**: one ProcessDetail composition; no dual process inspector.
- **Non-goals**: no live agent process API, no full log product rewrite, no second observability destination, no inventing historical CPU series.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can open any fixture process from Resources → Processes and fully inspect identity, host, status, and runtime metadata without navigating to another page.
- **SC-002**: 100% of primary process fixture fields required by FR-005–FR-006 are complete for at least one canonical process (command, user, port, CPU, memory, started-at).
- **SC-003**: Control header actions are reachable in one click and always acknowledge with explicit mock or hand-off feedback.
- **SC-004**: Not-found and sparse-metric states are visible and non-crashing.
- **SC-005**: Detail composition matches the production operational detail pattern (sections grid, StatusChip, DetailPanel.Action, optional DitherSegmentedBar only when numbers exist) with zero hand-rolled inspector chrome.
- **SC-006**: Desktop detail-fixture-coverage treats process fixtures as first-class complete compositions (full + sparse coverage).

## Assumptions

- Mock stage only: no live node-agent process list APIs in this feature slice.
- Resources → Processes collection and a basic ProcessDetail already exist; this feature deepens production quality and fixture richness.
- Terminal and logs remain separate surfaces; process detail only hands off or mocks.
- Portuguese chat / English UI strings remain project law.
- `docs/spec.md` resources/nodes/process guidance + DESIGN.md DetailPanel LAW govern chrome.
- Sibling high-priority Access details (agents/policies/audit/session) are out of scope for this feature.
