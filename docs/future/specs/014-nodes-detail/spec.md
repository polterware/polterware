# Feature Specification: Resources Nodes Detail

**Feature Branch**: `014-nodes-detail`

**Created**: 2026-08-11

**Status**: Implemented

**Input**: User description: "Seguindo o padrão de docs/spec.md, desenvolver a tela de details de nodes no padrão production-ready, mockado. Detail no global DetailPanel; entrega completa de UX/a11y/design-system; dados fixture."

**Related product docs**: `docs/spec.md` (§8.3 Resources tabs, §10.3 polter-node, §28 nodes domain, §54–55 node security, §111/resources linkage, Nodes data model), `AGENTS.md` (global DetailPanel only; StatusChip; DataTable; no second inspector), `DESIGN.md` (sections layout), `specs/004-operator-product-surfaces/contracts/resources-ui.md` (Nodes list → node detail).

**Extends**: Resources Nodes list already delivered under `packages/features/resources` — this feature elevates **node entity detail** from a thin overview to production operator quality. Does **not** implement live polter-node protocol, real heartbeats, real dispatch, or a second node detail page/route.

---

## Summary

Operators select a Polter Node on **Resources → Nodes** and inspect a complete, honest node record in the single global bottom DetailPanel: identity (node id ≠ display name), health/connection lifecycle, machine facts, announced capabilities, assigned resources, recent dispatch operations, upgrade posture, and capability-gated actions — without leaving the Nodes collection and without inventing a second inspector or page.

Detail is **mock-backed** for this delivery: structure, copy, a11y, and design-system grammar are production-ready; values come from node fixtures and local mock action feedback (toasts / disabled reasons). No claim of live outbound node connection, live dispatch, or live certificate rotation.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect node identity and health (Priority: P1)

As an operator, when I select a node row, the detail panel opens with display name + domain icon matching the table, health status (online / degraded / offline / draining / upgrading / registering / revoked / unknown), last heartbeat, agent version, location, and source honesty so I know whether the record is trustworthy.

**Why this priority**: §28 makes health, heartbeats, and identity first-class for distributed execution. Operators must not confuse heartbeat loss with process death or treat display name as node id.

**Independent Test**: Open Nodes → select nitro (online rich) and legacy-node (offline). Metrics show different StatusChip tones; identity shows stable node id distinct from hostname/display name; fixture source is explicit.

**Acceptance Scenarios**:

1. **Given** Nodes has at least one fixture node, **When** I activate a row, **Then** the global DetailPanel shows that node’s display name and domain icon matching the table row, with no second page, drawer, or modal for entity detail.
2. **Given** detail is open, **When** I read the summary metrics, **Then** I see status (StatusChip), last heartbeat, version, and capability count without leaving the first fold of detail body content.
3. **Given** an offline or degraded node, **When** I open detail, **Then** status is explicit and a health note can explain that heartbeat loss does not prove remote processes died — product does not pretend full health.
4. **Given** detail is open, **When** I activate another node row, **Then** content switches to the new node without unmounting the panel host or inventing a second inspector.

---

### User Story 2 - Understand connection, machine, and identity security (Priority: P1)

As an operator, I can see connection model facts (outbound, last connected, connection state), machine inventory (hostname, OS, arch, CPU, memory, storage, IP when known), and security posture (authenticated identity, revoked/draining/upgrading) without secrets or credential material in the UI.

**Why this priority**: §10.3 / §28 / §55 require outbound connection model, rotatable credentials (referenced, not displayed), and revoked nodes that accept no new dispatch.

**Independent Test**: Open a rich online fixture vs revoked/draining fixture; connection and machine sections populate when present; revoked never offers Run command as a live success path.

**Acceptance Scenarios**:

1. **Given** a connected online node, **When** Overview / Connection / Machine sections render, **Then** node id (mono), hostname, OS, arch, IP (when known), location, version, and last heartbeat are visible.
2. **Given** a revoked or draining node, **When** detail opens, **Then** health state is explicit and dispatch-oriented actions are unavailable or clearly inert with honest reason.
3. **Given** dataSource is fixture, **When** Source field renders, **Then** it is labeled as fixture/mock honesty, never as live polter-node stream.
4. **Given** credentials/certs exist conceptually, **When** UI renders, **Then** no secret, private key, or full cert body is shown — only non-sensitive status labels if present.

---

### User Story 3 - Review capabilities, assigned resources, and recent operations (Priority: P1)

As an operator, Capabilities list announced execution capabilities, Assigned resources show inventory tied to this node, and Recent operations show recent dispatch/command outcomes so I understand what this node can do and has done.

**Why this priority**: §28 — node only executes announced and authorized capabilities; operators need capability advertisement and recent dispatch evidence before trusting a target.

**Independent Test**: Select nitro with full tables; select sparse prod-vps with empty optional tables — honest empty states.

**Acceptance Scenarios**:

1. **Given** capabilities exist, **When** the section renders, **Then** a DataTable shows capability names (and authorized/status when modeled); no Actions column.
2. **Given** assigned resources exist, **When** the section renders, **Then** a DataTable shows name, type, status with StatusChip and domain icons; no Actions column.
3. **Given** recent operations exist, **When** the section renders, **Then** a DataTable shows command, result (StatusChip), actor, and when; empty when none.
4. **Given** empty capabilities / resources / operations, **When** sections render, **Then** empty copy is honest — no fabricated rows.

---

### User Story 4 - Act only through honest capability gates (Priority: P1)

As an operator, header actions surface Open terminal and Run command only when the node model allows them (typically online + relevant capability). Drain, revoke, or upgrade-related actions, if shown, use fixture-honest feedback and never claim live protocol success.

**Why this priority**: Mutation safety and §28 invariants — revoked/offline nodes must not fake successful dispatch.

**Independent Test**: On nitro online, Terminal and Run command available; on offline/revoked, unavailable or disabled with reason; mock toasts never claim live node ACK.

**Acceptance Scenarios**:

1. **Given** an online node with Terminal capability (or product default allow for mock online), **When** I choose Open terminal, **Then** product navigates or mock-toasts honestly without claiming SSH established if not wired.
2. **Given** an offline or revoked node, **When** detail is open, **Then** Run command is not offered as a primary live success control (hidden or disabled with reason).
3. **Given** any mock mutation action, **When** clicked, **Then** feedback is fixture-honest and does not claim live dispatch ACK or certificate rotation.

---

### User Story 5 - Cover sparse and lifecycle edge states without scenic emptiness (Priority: P2)

As an operator, sparse fixtures (minimal metadata), degraded/offline/draining/upgrading/registering/revoked states, and not-found/loading still render usable detail chrome with honest empty sections.

**Why this priority**: Production readiness includes partial registry data and lifecycle transitions; operators hit these early.

**Independent Test**: Activate sparse and lifecycle fixtures; loading/not-found match other operational details.

**Acceptance Scenarios**:

1. **Given** a node with only core list fields, **When** detail opens, **Then** optional sections show — / empty states, not a broken layout.
2. **Given** loading props, **When** detail mounts, **Then** loading chrome appears without blank panel.
3. **Given** not-found or missing entity, **When** detail mounts, **Then** error/not-found copy is honest.
4. **Given** status degraded or upgrading, **When** metrics render, **Then** StatusChip uses non-healthy tones, not online/healthy green.

---

### Edge Cases

- Heartbeat loss / offline does not imply remote process death — health note or copy MUST allow this distinction when status is offline/degraded.
- Node id differs from machine display name / hostname — both visible when known.
- Revoked node accepts no new dispatch — UI must not offer Run command as enabled live path.
- Upgrading / draining — status explicit; optional note about dispatch restriction.
- Long capability/resource/operation lists scroll only inside DetailPanel.Body.
- IP / hostname unknown — show —; do not invent network facts.
- Catalog-like OS labels do not branch layout via `if os ===`; layout is shape-driven.
- Fixture `dataSource: "fixture"` always honest.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Node detail MUST render exclusively in the global DetailPanel hosted by MainSurface; features MUST NOT create a fixed inspector page, modal-as-detail, drawer, or second panel for node entity detail.
- **FR-002**: Selecting a node in Nodes MUST activate detail target `scopeId: resources:nodes`, `entityKind: node`.
- **FR-003**: Detail MUST use operational `DetailPanel.Body layout="sections"` with Grid / Section / Metric / Metadata patterns (not knowledge `document` layout).
- **FR-004**: Detail header MUST show a 16px domain icon consistent with the Nodes PrimaryCell (`HardDrive` / infrastructure tone family), a single-line title (display name), and header actions via `DetailPanel.Action` only.
- **FR-005**: Status / health everywhere in node detail (and Nodes list chips for expanded statuses) MUST use StatusChip (filled); no hand-rolled status pills.
- **FR-006**: Detail MUST surface summary metrics for status, last heartbeat, agent version, and capabilities count (or honest —).
- **FR-007**: Detail MUST show identity fields: stable node id (mono, distinct from display name), hostname, location, OS, version, architecture, IP when known, source/dataSource honesty.
- **FR-008**: Detail MUST show machine facts when present: CPU, memory, storage (and optional connection summary).
- **FR-009**: Detail MUST list capabilities in design-system DataTable; no Actions column; honest empty state.
- **FR-010**: Detail MUST list assigned resources in design-system DataTable (name, type, status chip); no Actions column; honest empty state.
- **FR-011**: Detail MUST list recent operations in design-system DataTable (command, result chip, actor, when); no Actions column; honest empty state.
- **FR-012**: Open terminal / Run command MUST be capability- and health-gated; offline/revoked MUST NOT present them as live success paths.
- **FR-013**: Mock actions MUST use fixture-honest feedback; UI MUST NOT claim live polter-node protocol, live heartbeat stream, or live credential rotation.
- **FR-014**: Health vocabulary MUST support at least: online, offline, degraded, unknown; SHOULD also support registering, draining, upgrading, revoked for fixture coverage aligned with docs/spec.md §28.
- **FR-015**: Loading / not-found / error detail states MUST remain honest when supplied.
- **FR-016**: Implementation MUST keep a dedicated `NodeDetail` composition module plus status helper module for reuse and testing.
- **FR-017**: Collection registration, open/collapsed/maximized snaps, and Escape/maximize behavior remain owned by existing DetailPanel / shell laws — this feature MUST NOT reimplement sizing chrome.
- **FR-018**: At least one fixture MUST be a rich online node (capabilities, assigned resources, recent operations, machine facts, connection/identity fields); fixtures MUST also cover offline and at least one additional non-online lifecycle among degraded/draining/upgrading/registering/revoked; sparse minimal rows remain valid.

### Key Entities

- **Polter Node (TargetNode)**: Registered execution agent on a machine — stable node id, display name, location, OS, agent version, health status, last heartbeat, optional machine facts, capabilities, assigned resources, recent operations, connection summary, data source honesty.
- **Health status**: Operational visibility of node registry/connection lifecycle (online, degraded, offline, draining, upgrading, registering, revoked, unknown) — heartbeat loss ≠ process death.
- **Announced capability**: Named execution capability the node advertises (and optionally authorization flag).
- **Assigned resource**: Inventory object observed or managed via this node (name, type, status).
- **Recent operation**: Recent dispatch/command evidence (time, command, actor, result).
- **Connection summary**: Non-secret connection posture (state, last connected, outbound model note).

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: Discover and operate distributed execution targets; understand node health before dispatch.
- **Runtime owner (III)**: Renderer feature package `packages/features/resources` + desktop composition; no Main/Node unrestricted access; no claimed live node protocol.
- **Domain vs adapter (II, IV)**: UI consumes normalized TargetNode shapes from fixtures (stand-in for domain queries); no layout branching on OS vendor strings.
- **Contracts (V)**: Desktop UI contract for Node detail; no protobuf change required for mock delivery.
- **Product loop impact**: Nodes list → DetailPanel inspect → Terminal / Run command entry (mock) → future real dispatch via operations.
- **Evidence expectations**: Heartbeat, capabilities, recent operations act as operator-visible evidence stand-ins; fixture source explicit.
- **Mutation safety (VII)**: Mock-only actions with honest feedback; real plan→approve→apply not claimed for drain/revoke/upgrade.
- **Operation audit fields**: Mock actions identify node target and action in toast/copy; no fake audit persistence required this increment.
- **Security boundary (VI)**: No secrets/certs/private keys in fixtures or UI; renderer unprivileged; MCP not in scope.
- **Local vs cloud authority (VIII)**: Fixtures are local mock authority; real node identity remains protocol-owned when wired later.
- **Performance (IX)**: Detail content scroll only in DetailPanel.Body; tables use existing DataTable patterns.
- **Resilience (X)**: Offline/degraded/revoked/draining/upgrading/unknown are first-class UI states.
- **Observability (XI)**: Node id and source honesty visible for hand-off; no fake telemetry claims.
- **Test plan (XII)**: Unit tests for status mapping/helpers, composition contract, fixture richness; desktop detail-fixture-coverage extended.
- **Structure created now (XIII)**: Extend `packages/features/resources` (+ desktop fixture coverage); no new scenic package.
- **Single source of truth (XIV)**: One NodeDetail composition; no dual detail page + panel.
- **Big-bang / refactor context (XV)**: Continues operator surfaces on current architecture branch.
- **Non-goals (I, XIII)**: No live polter-node protocol; no remote desktop; no full process manager clone; no real certificate UI; no Inventory/Processes redesign; no Provisioning wizard work.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: From Nodes list, an operator reaches full node context (status, identity, machine, capabilities, resources, operations) in under 15 seconds without leaving Resources.
- **SC-002**: 100% of fixture nodes used in the Nodes list resolve to a detail view that never shows blank structural chrome (header + at least summary sections or honest empty states).
- **SC-003**: Operators can distinguish online vs offline vs at least one additional lifecycle (degraded/revoked/draining/upgrading) from summary metrics without reading raw IDs.
- **SC-004**: Offline/revoked fixtures do not present Run command as an enabled live success path.
- **SC-005**: Detail uses only design-system DetailPanel + DataTable + StatusChip + Action grammar — no parallel hand-rolled detail chrome.
- **SC-006**: Fixture coverage test asserts expanded node shape (capabilities, assigned resources, recent operations, machine facts, node id ≠ name, health beyond binary online/offline for at least one fixture) for primary rich fixture.
- **SC-007**: Visual and interaction patterns match other operational details (Run/Member/Inventory) for sections grid, StatusChip, and header actions.

## Assumptions

- Resources page tabs remain Inventory · Nodes · Processes; this feature does not change tab IA.
- Terminal and Run command may already be wired as project-level mocks by App; detail only gates visibility and invokes callbacks.
- Health vocabulary expands beyond the prior online/offline list chips while remaining StatusChip-compatible.
- Live polter-node APIs are out of scope for this increment; swapping fixtures for real queries later must not force a second detail UX.
- Portuguese chat with Erick; English for code, UI strings, contracts, and this specification.
