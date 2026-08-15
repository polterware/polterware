# Feature Specification: Access Policies Detail

**Feature Branch**: `018-policies-detail`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Seguindo o padrão de docs/spec.md, AGENTS.md e details production-ready (member/team/run/schedule), desenvolver a tela de details de Policies no Access — production-ready, mockada, completa. Extrair composição dedicada, enriquecer WorkspacePolicy/POLICY_MOCKS, related audit links (read-only), sem implementar Agents/Audit/Session details paralelos."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect a project policy (Priority: P1)

As a project admin or operator with Access privileges, I select a policy in Access → Policies and open its detail panel to understand what the policy governs: status, scope, enforcement, approval posture, description, actors (people / agents / roles), environments, allowed and blocked actions, risk/approval requirements, update history, optional activity trail, and related audit events — without leaving Access.

**Why this priority**: Policy detail is the primary governance inspection surface for authorization rules shown in Access. Without it, operators cannot verify policy posture or act on policies with confidence.

**Independent Test**: With Access → Policies open and fixture policies loaded, activating any policy row shows a complete DetailPanel for that policy with metrics, description, structured collections, and related audit list; mock governance actions are reachable from the header.

**Acceptance Scenarios**:

1. **Given** Access → Policies lists policies, **When** the operator activates a policy row, **Then** the global DetailPanel shows that policy's name and a ShieldCheck / security domain icon, with body content for overview metrics, description, actors, environments, allowed/blocked actions, approval/risk, update history, and related audit links.
2. **Given** a policy detail is open, **When** the operator reviews Overview metrics, **Then** they see status (`StatusChip`), scope, enforcement, and requiresApproval — without a second page or modal.
3. **Given** a policy has structured actors and action lists in fixtures, **When** detail opens, **Then** Actors, Allowed actions, and Blocked actions render as structured collections (not only a single comma-separated string).
4. **Given** the policy is found in fixtures, **When** the panel opens, **Then** the panel never invents live backend policy enforcement or core authorization authority.

---

### User Story 2 - Govern policy from detail (Priority: P1)

As a project admin, I use header actions on a policy detail to edit, duplicate, enable/disable, or delete the policy. In this mock stage, actions confirm with honest mock feedback and do not claim real policy enforcement or persistence.

**Why this priority**: Operators expect governance actions adjacent to the inspected policy; Access contract requires mock edit/duplicate/enable/delete affordances.

**Independent Test**: Open any policy detail and invoke Edit, Duplicate, Enable/Disable, and Delete; each produces clear mock feedback without navigating away or opening a forbidden second inspector.

**Acceptance Scenarios**:

1. **Given** policy detail is open, **When** the operator triggers Edit, **Then** a mock toast acknowledges editor intent and the panel remains on that policy.
2. **Given** policy detail is open, **When** the operator triggers Duplicate, **Then** mock feedback acknowledges duplication intent without claiming persistence.
3. **Given** an Active policy, **When** the operator triggers Disable (or Enable for Disabled), **Then** mock feedback acknowledges the state change intent; UI does not claim live enforcement.
4. **Given** policy detail is open, **When** the operator triggers Delete, **Then** mock danger-tone action acknowledges deletion intent with mock-labeled feedback.

---

### User Story 3 - Understand approval, risk, and related audit (Priority: P2)

As an operator reviewing governance posture, I open a policy detail and see approval requirements / risk posture plus related audit events linked by policy id or name, so I can judge how the policy participates in operational control without opening a separate audit product redesign.

**Why this priority**: docs/spec.md ties policies to approvals and audit; operators need cross-links without building Audit detail in this slice.

**Independent Test**: Open Production operations approval (`p1`) fixture; detail shows approval/risk section and a non-empty related audit list derived from audit fixtures that reference that policy. Activating a related audit row may call a callback; this feature MUST NOT implement Audit detail UI.

**Acceptance Scenarios**:

1. **Given** a policy with `requiresApproval` true and risk/approval fields, **When** detail opens, **Then** Approval requirements / risk posture is visible with StatusChip for risk or approval semantics where status-like.
2. **Given** audit fixtures include events with matching `policyId` or `policyName`, **When** policy detail opens, **Then** Related audit events lists those events (read-only) with empty state when none match.
3. **Given** a related audit row is activatable, **When** the operator activates it, **Then** an optional `onOpenAudit` callback fires; this feature does not mount Audit detail composition.

---

### User Story 4 - Honest empty, missing, and partial states (Priority: P2)

As an operator, when a policy has empty blocked actions, empty actors, empty activity, no related audit, or the target is missing from fixtures, the detail remains honest: empty collections show empty states; missing entities show not-found; the UI never fabricates live enforcement.

**Why this priority**: Explicit state beats magic; production-ready detail must handle incomplete policy data.

**Independent Test**: Force not-found and empty collection fixtures; panel shows PageError / empty table states without crashing.

**Acceptance Scenarios**:

1. **Given** an activation target for a policy id not in fixtures, **When** the panel resolves, **Then** it shows a not-found state with a clear message.
2. **Given** a policy with empty blocked actions or empty activity arrays, **When** those sections render, **Then** each shows an explicit empty message, not placeholder invents.
3. **Given** mock stage, **When** any governance action fires, **Then** feedback is labeled as mock and does not claim backend success beyond the toast.

---

### Edge Cases

- Policy status Active vs Disabled — status labeled via StatusChip, not ad-hoc pills.
- Policy with zero blocked actions (e.g. Staging operator access) — Blocked actions section remains with empty state.
- Policy with optional enforcement and requiresApproval false — metrics show Optional / Not required honestly.
- Long action names — mono cells stay readable; tables scroll with the DetailPanel body only.
- Simultaneous Access tabs — policy detail only mounts for `scopeId === "access:policies"` + `entityKind === "policy"`; no dual content clash with Agents / Audit / Sessions (owned by parallel agents).
- Policy tables and StatusChips are **not** authorization enforcement; real authz remains backend-owned in future stages.
- Related audit is read-only listing; Audit detail is out of scope for this feature.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Access → Policies MUST open policy detail in the single global DetailPanel hosted by MainSurface; MUST NOT invent a second entity-detail page, drawer, or modal inspector for policies.
- **FR-002**: Policy detail MUST use `DetailPanel.Body layout="sections"` + `DetailPanel.Grid` for operational policy content (not document layout).
- **FR-003**: Policy detail MUST show identity header: ShieldCheck / security domain icon (same family as Policies table PrimaryCell), single-line policy name, and feature-owned header actions via `DetailPanel.Action` only.
- **FR-004**: Policy detail MUST expose mock governance actions: Edit, Duplicate, Enable/Disable (status-dependent label/icon), Delete (danger). Actions MUST use honest mock feedback in this stage.
- **FR-005**: Policy detail MUST present Overview metrics: status (`StatusChip`), scope, enforcement, requiresApproval (Required / Not required or equivalent StatusChip).
- **FR-006**: Policy detail MUST present Description as readable body text.
- **FR-007**: Policy detail MUST present Actors as a structured collection of people / agents / roles (fixture-derived), with empty state when none.
- **FR-008**: Policy detail MUST present Environments as a structured list, with empty state when none.
- **FR-009**: Policy detail MUST present Allowed actions and Blocked actions as structured collections (mono action identifiers), with empty states when none.
- **FR-010**: Policy detail MUST present Approval requirements / risk posture when fixtures provide those fields (risk StatusChip + approval requirement summary).
- **FR-011**: Policy detail MUST present Update history (updated, updatedBy) and optional activity trail when fixtures provide activity events.
- **FR-012**: Policy detail MUST present Related audit events as a read-only structured list filtered from existing audit fixtures by `policyId` / `policyName`; MAY invoke `onOpenAudit` callback; MUST NOT implement Audit detail UI.
- **FR-013**: Policy detail MUST support loading, error, and not-found honest states consistent with other operational details (People / Teams / Resources).
- **FR-014**: Fixture data for policies MUST be rich enough for complete detail compositions and covered by the desktop detail-fixture-coverage suite for the primary policy fixture.
- **FR-015**: Implementation MUST extract a dedicated `PolicyDetail` composition (not only an inline thin block in the Access index) following MemberDetail / TeamDetail quality.
- **FR-016**: Status / result / risk / severity chips MUST use filled `StatusChip`; MUST NOT hand-roll status pills.
- **FR-017**: Role of this UI is display/governance inspection only. Mock stage MUST remain honest that policy enforcement is not live in the core.
- **FR-018**: Implementation MUST NOT modify Agents detail, Audit detail, Agent session detail, or rewrite Agents/Audit/Sessions collection DataTables. File boundaries limited to Policy detail + policy fixtures + minimal Access wiring.
- **FR-019**: Collection activation remains independent of panel snap mode; selecting a row updates the target without forcing maximize.

### Key Entities

- **Policy**: Project-scoped governance rule with status, scope, enforcement, approval posture, actors, environments, allowed/blocked actions, risk, and update history.
- **Policy Actor**: Person, agent, or role subject to the policy (structured entry for detail).
- **Policy Action**: Named capability/action identifier that is allowed or blocked.
- **Policy Activity Event**: Optional history trail entry for policy changes (title, when, actor, optional result).
- **Related Audit Event**: Existing audit fixture linked by policy id/name; listed read-only from policy detail.
- **Enforcement Mode**: Display label for how strictly the policy is applied (e.g. Required, Optional) — presentation only in mock stage.
- **Risk Posture**: Display risk level associated with the policy (low / medium / high / critical) when authored in fixtures.

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: governance + audit inspection for project policies in Access.
- **Runtime owner (III)**: Renderer feature package `@polterware/feature-access` + shell workspace mocks; no Main/Node privilege for this UI stage.
- **Domain vs adapter (II, IV)**: UI composition over access/governance policy concepts; no new authorization engine inside Access; fixtures stand in for core data.
- **Contracts (V)**: feature contract `policies-detail-ui.md` under this feature directory.
- **Product loop impact**: Access / DetailPanel / Audit-adjacent inspection; no System Map change.
- **Evidence expectations**: n/a for live evidence; mock fixtures must be complete and honest.
- **Mutation safety (VII)**: mock-only actions with toast acknowledgment; no claim of plan→approve→apply pipeline completion.
- **Operation audit fields**: related audit rows may show result StatusChip; full Audit detail remains separate (parallel agent).
- **Security boundary (VI)**: unprivileged renderer; policy UI is not enforcement; secrets never shown.
- **Local vs cloud authority (VIII)**: mock local fixtures only; offline/cloud sync out of scope.
- **Performance (IX)**: DetailPanel body is sole vertical scroller; tables stay modest fixture size.
- **Resilience (X)**: not-found / empty / mock failure feedback without crash.
- **Observability (XI)**: n/a for new correlation IDs in mock UI.
- **Test plan (XII)**: unit tests for policy detail composition + fixture coverage expansion; access detail-targets remain green.
- **Structure created now (XIII)**: extend `packages/features/access` + shell workspace-mocks (Policy only); no scenic package.
- **Single source of truth (XIV)**: one PolicyDetail composition; remove thin inline duplicate once extracted.
- **Big-bang / refactor context (XV)**: continues operator product surfaces on `refactor/new`.
- **Non-goals (I, XIII)**: no live policy engine, no real enable/delete API, no Agents/Audit/Session detail work, no rewrite of other Access collections, no frontend authz engine.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can open any fixture policy from Access → Policies and fully inspect policy posture (status, scope, enforcement, approval, actors, environments, actions, risk, history, related audit) without navigating to another page.
- **SC-002**: 100% of primary fixture policies used for completeness gates have complete detail fields required by FR-005–FR-012 (no blank required section for the primary Production operations approval fixture).
- **SC-003**: Governance header actions are reachable in one click from the open policy detail and always acknowledge with explicit mock feedback.
- **SC-004**: Not-found and empty-collection states are visible and non-crashing when fixtures omit data or the id is unknown.
- **SC-005**: Detail composition matches the production operational detail pattern used by People/Teams/Resources (sections grid, StatusChip for status, DataTable for collections, DetailPanel.Action for header actions) with zero hand-rolled inspector chrome.
- **SC-006**: Desktop detail-fixture-coverage treats Policy fixtures as first-class complete compositions (actors structured, actions, risk/approval, activity or update history, and related audit linkage present for primary fixture).
- **SC-007**: Parallel sibling surfaces (Agents / Audit / Agent session details) remain untouched by this feature's intentional file boundary.

## Assumptions

- Mock stage only: no ConnectRPC / polter-core policy enforcement APIs in this feature slice.
- Access → Policies tab already lists policies via `POLICY_MOCKS` and registers detail targets; this feature deepens detail quality, not the index IA.
- `actors` string array may remain for list compatibility; detail prefers structured actor entries when present.
- Related audit is filtered client-side from existing `AUDIT_EVENT_MOCKS` without mutating audit fixtures.
- Enable/Disable/Delete/Edit/Duplicate are toast-mock only.
- Portuguese chat / English UI strings remain project law.
- Parallel agents own Agents detail, Audit detail, and Agent session detail — this feature only reads audit fixtures for related links.
- `docs/spec.md` Access + governance guidance is the product source for policy semantics; DESIGN.md + DetailPanel LAW govern chrome.
