# Feature Specification: Access Audit Event Detail

**Feature Branch**: `019-audit-detail`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Access → Audit → detail do Audit event (read-only). Extrair composição dedicada production-ready no global DetailPanel, enriquecer WorkspaceAuditEvent fixtures, manter collection virtualized/filtros intactos. Irmãos Agents/Policies/Agent-session detail em paralelo — NÃO implementar."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect an audit event (Priority: P1)

As a project admin or operator with Access privileges, I select an event in Access → Audit and open the global detail panel to understand what happened: result, when, where (environment/origin), who acted (person or agent), which action and entity were involved, correlation identifiers when present, related policy/approval controls, and any context notes — without leaving Access and without any mutation path.

**Why this priority**: Audit is the append-only governance trail. Without a complete inspection surface, operators cannot verify accountability or link events to policies and approvals.

**Independent Test**: With Access → Audit open and fixture events loaded, activating any audit row shows a complete DetailPanel for that event with result, actor, action/entity, origin/time, and related controls when present; the panel is read-only.

**Acceptance Scenarios**:

1. **Given** Access → Audit lists events, **When** the operator activates an audit row, **Then** the global DetailPanel shows a clear title (short action or `Audit / {id}`), an actor-kind identity icon (person vs agent), and sectioned body content for the event.
2. **Given** an audit event is open, **When** the operator reviews metrics and overview, **Then** they see result as a status chip, environment, origin, timestamp, actor name/kind (and actor id when known), action (mono), entity, and entity kind when present.
3. **Given** the event is found in fixtures, **When** the panel opens, **Then** loading is not required in the mock stage, and the panel never invents live backend audit authority.
4. **Given** collection filters or virtualization are active, **When** the operator opens detail, **Then** the Audit table filters and virtualization continue to work unchanged.

---

### User Story 2 - Trace related controls and correlation ids (Priority: P1)

As an operator investigating a governance decision, I open an audit event that references a policy and/or approval and see those related controls clearly labeled. Optional header actions allow copying the event id or opening a related policy/approval via mock navigation callbacks — never Approve, Delete, or other mutating controls.

**Why this priority**: Audit value is correlation — linking the trail to the policy that gated the action and the approval that authorized it.

**Independent Test**: Open fixture events with and without `policyId`/`approvalId`/`requestId`/`correlationId`/`runId`; related sections and optional copy/open actions appear only when data exists; no mutation actions appear.

**Acceptance Scenarios**:

1. **Given** an event with policy and approval references (e.g. deployment pending), **When** detail opens, **Then** Related controls shows policy name/id and approval id without building Policy or Approval detail inside this composition.
2. **Given** an event with request/correlation/run identifiers, **When** detail opens, **Then** those identifiers appear in a dedicated metadata section (mono where appropriate).
3. **Given** detail is open, **When** the operator uses Copy event id (if offered), **Then** mock feedback acknowledges the copy without implying server mutation.
4. **Given** detail is open with a related policy, **When** the operator uses Open policy (if offered), **Then** navigation uses the existing Access policy activation callback path and does not invent a second inspector chrome.
5. **Given** any audit detail, **When** header actions are reviewed, **Then** Approve, Reject, Delete, Edit, and similar mutations are absent.

---

### User Story 3 - Honest empty, missing, and partial states (Priority: P2)

As an operator, when an event has no context notes, no related policy/approval, or the target id is missing from fixtures, the detail remains honest: empty optional sections show empty states or are omitted; missing entities show not-found; the UI never fabricates live audit authority or fake timeline steps.

**Why this priority**: Audit must never invent history. Partial fixtures and unknown ids are production realities even in mock stage.

**Independent Test**: Force not-found target and open events without optional fields; panel shows PageError / honest omission without crash.

**Acceptance Scenarios**:

1. **Given** an activation target for an audit id not in fixtures, **When** the panel resolves, **Then** it shows a not-found state with a clear message.
2. **Given** an event without context notes/payload, **When** Context is reviewed, **Then** an honest empty message appears (not invented payload).
3. **Given** an event without policy/approval links, **When** detail opens, **Then** Related controls is omitted or shows an explicit empty message — never placeholder invents.
4. **Given** mock stage, **When** any optional copy/open action fires, **Then** feedback is labeled as mock and does not claim backend mutation success.

---

### Edge Cases

- Actor kind person vs agent — icon and kind label must match table PrimaryCell family (User vs Bot).
- Result Success / Pending / Failed / Denied — always `StatusChip` with semantic tone; never hand-rolled pills.
- Long action / entity strings — mono where appropriate; single-line metadata; body is the only vertical scroller.
- Events spanning All environments vs a named environment.
- Simultaneous Access tabs — audit detail only mounts for `scopeId === "access:audit"` and `entityKind === "audit-event"`; no clash with Agents/Policies/People details.
- Timeline mini (created → decided → applied) is **out of scope** unless multi-step timestamps exist on the event model; current fixtures are point-in-time events, so timeline is omitted.
- Collection remains virtualized; detail extraction must not rewrite the Audit DataTable.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Access → Audit MUST open audit event detail in the single global DetailPanel hosted by MainSurface; MUST NOT invent a second entity-detail page, drawer, or modal inspector for audit events.
- **FR-002**: Audit detail MUST use `DetailPanel.Body layout="sections"` + `DetailPanel.Grid` for operational inspection content (not document layout).
- **FR-003**: Audit detail MUST show identity header: actor-kind domain icon (person → User, agent → Bot, same family as Audit table PrimaryCell), single-line title (short action preferred, or `Audit / {id}`), and feature-owned header actions via `DetailPanel.Action` only when non-mutating actions are offered.
- **FR-004**: Audit detail MUST be read-only. MUST NOT expose Approve, Reject, Delete, Edit, or other mutation actions. Optional actions limited to Copy event id and Open related policy/approval (mock callbacks only).
- **FR-005**: Audit detail MUST present result metrics with `StatusChip` (Success / Pending / Failed / Denied), environment, origin, and timestamp.
- **FR-006**: Audit detail MUST present Actor metadata: name, kind, optional actorId.
- **FR-007**: Audit detail MUST present Action (mono), entity, and entityKind when present.
- **FR-008**: Audit detail MUST present requestId, correlationId, and runId when present on the fixture model; omit empty identifier rows honestly.
- **FR-009**: Audit detail MUST present Related controls for policy (policyId/policyName) and approvalId when present; open-related uses callbacks only — MUST NOT implement Policy or Approval detail compositions in this feature.
- **FR-010**: Audit detail MUST present Context notes/payload when present, or an honest empty state when absent.
- **FR-011**: Audit detail MUST support loading, error, and not-found honest states consistent with MemberDetail / TeamDetail / RunDetail.
- **FR-012**: Fixture data for audit events MUST be rich enough for complete detail compositions (including at least one event with policy + approval, one denied, one without related controls, and correlation identifiers where meaningful) and covered by desktop detail-fixture-coverage for audit.
- **FR-013**: Implementation MUST extract a dedicated `AuditDetail` composition (not only the thin inline block in Access index) following MemberDetail / TeamDetail / RunDetail quality.
- **FR-014**: Access → Audit collection filters, virtualization, and empty filter messaging MUST remain intact; detail extraction is composition-only wiring.
- **FR-015**: Collection activation remains independent of panel snap mode; selecting a row updates the target without forcing maximize.
- **FR-016**: Read-only honesty MAY appear as a muted empty/footer note inside the detail body (or remain on the collection footer already present); MUST NOT invent ugly chrome banners that compete with DetailPanel structure.
- **FR-017**: Parallel sibling features (Agents detail, Policies detail, Agent session detail) are out of scope; this feature MUST NOT implement or refactor those compositions.

### Key Entities

- **Audit Event**: Append-only project governance trail record — who (actor + kind), what (action + entity), where (environment + origin), when (timestamp), result, optional correlation/request/run ids, optional related policy/approval, optional context notes.
- **Actor Kind**: `person` | `agent` — drives identity icon and labeling; not an authorization engine.
- **Audit Result**: Success | Pending | Failed | Denied — displayed via StatusChip tones.
- **Related Control Reference**: Optional policy and/or approval identifiers linking the event to governance controls without embedding those detail UIs.

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: audit / governance inspection for project actions by humans and agents.
- **Runtime owner (III)**: Renderer feature package `@polterware/feature-access` + shell workspace mocks; no Main/Node privilege for this UI stage.
- **Domain vs adapter (II, IV)**: UI composition over audit trail concepts; no new authorization engine; fixtures stand in for core audit store.
- **Contracts (V)**: UI contract `audit-detail-ui.md` under this feature directory; extends Access Audit surface.
- **Product loop impact**: Access / DetailPanel / audit inspection; no System Map change.
- **Evidence expectations**: n/a for live evidence; mock fixtures must be complete and honest.
- **Mutation safety (VII)**: audit is append-only inspection; optional copy/open are non-mutating; no plan→approve→apply claims from this panel.
- **Operation audit fields**: detail presents actor, target/entity, policy, approval, timestamps, result, correlation/request ids when fixture-supplied.
- **Security boundary (VI)**: unprivileged renderer; no secrets in audit DTOs; no silent policy bypass.
- **Local vs cloud authority (VIII)**: mock local fixtures only; offline/cloud sync out of scope.
- **Performance (IX)**: DetailPanel body sole vertical scroller; Audit collection keeps virtualization; fixtures modest size.
- **Resilience (X)**: not-found / empty / optional-section honesty without crash.
- **Observability (XI)**: correlation/request ids displayed when present; not invented.
- **Test plan (XII)**: unit tests for audit detail helpers + composition contract + fixture coverage expansion for audit.
- **Structure created now (XIII)**: extend `packages/features/access` + shell workspace-mocks (audit types/fixtures only); no scenic package.
- **Single source of truth (XIV)**: one `AuditDetail` composition; remove thin inline duplicate once extracted.
- **Big-bang / refactor context (XV)**: continues operator product surfaces on `refactor/new`.
- **Non-goals (I, XIII)**: no live audit store API, no SIEM product, no Agents/Policies/Agent-session detail work, no collection rewrite, no mutation of audit trail, no fake multi-step timeline without fixture support.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can open any fixture audit event from Access → Audit and fully inspect result, actor, action/entity, environment/origin/time, related controls when present, and context without navigating to another page.
- **SC-002**: 100% of primary fixture events used in the Audit list resolve to a detail that never blanks required metrics (result, environment, actor, action, entity, origin, time).
- **SC-003**: At least one fixture event exposes both policy and approval related controls; at least one is Denied; at least one has correlation/request identifiers; at least one has no related controls.
- **SC-004**: Not-found and empty-context states are visible and non-crashing when fixtures omit data or the id is unknown.
- **SC-005**: Detail composition matches production operational detail pattern (sections grid, StatusChip for result, DetailPanel.Action only for non-mutating optional actions) with zero hand-rolled inspector chrome.
- **SC-006**: Desktop detail-fixture-coverage treats Audit event fixtures as first-class complete compositions (enriched fields + related controls coverage).
- **SC-007**: Audit collection filters and virtualization remain functional after detail extraction (no regression of list behavior).

## Assumptions

- Mock stage only: no ConnectRPC / polter-core audit APIs in this feature slice.
- Access → Audit tab already lists events via `AUDIT_EVENT_MOCKS`, filters, virtualized DataTable, and detail target registration; this feature deepens detail quality, not the index IA.
- Point-in-time audit events do not include multi-step lifecycle timestamps; timeline mini is omitted.
- Policy open uses existing Access policy activation when `onOpenPolicy` / local `activate(..., "policy")` is available; Approval open is mock toast or callback only (Inbox owns approvals).
- Portuguese chat / English UI strings remain project law.
- `docs/spec.md` Access + audit guidance is the product source; DESIGN.md + DetailPanel LAW govern chrome.
- Parallel agents own Agents / Policies / Agent session details; file boundaries in the implementation plan MUST be respected.
