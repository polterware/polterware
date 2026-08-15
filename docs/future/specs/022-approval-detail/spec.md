# Feature Specification: Inbox Approval Detail

**Feature Branch**: `022-approval-detail`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Seguindo o padrão de docs/spec.md e o DetailPanel de production-ready (Runs / People), desenvolver a tela de details de Approvals no Inbox — entrega completa, mockada, production-ready. Extrair composição dedicada, enriquecer fixtures de approval (risk, requester kind person/agent, related run/policy/component/node/resource, justification, decision trail) e alinhar com o loop plan→review→approve→apply."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect a pending approval (Priority: P1)

As a project admin or operator with approval rights, I open Inbox → Pending, select an approval request, and use the detail panel to understand what is requested, who requested it (person or agent), risk, environment, related system entities, governing policy, and the justification — before deciding.

**Why this priority**: Approvals are the human gate in the operator mutation loop. Thin detail blocks prevent confident approve/reject decisions.

**Independent Test**: With Inbox → Pending open and fixture approvals loaded, activating any pending row shows a complete DetailPanel with request context, risk/status, justification, related entities, and approve/reject actions.

**Acceptance Scenarios**:

1. **Given** Inbox → Pending lists approvals, **When** the operator activates a row, **Then** the global DetailPanel shows the request title and a domain inbox/security icon, with body content for risk, status, environment, requester, related entities, policy, and justification.
2. **Given** a pending approval detail is open, **When** the operator reviews Request context, **Then** they see requested-by (and kind person/agent), component/node/resource when known, policy, when, and related run id when present — without a second page or modal.
3. **Given** the approval is pending, **When** detail opens, **Then** Approve and Reject header actions are available.
4. **Given** fixtures only, **When** the panel opens, **Then** the UI never claims live policy engine or real apply pipeline completion.

---

### User Story 2 - Decide approve or reject (Priority: P1)

As an approver, I use header actions to approve or reject a pending request. In mock stage, the decision updates local fixture/session state (or shows honest mock feedback) including decision note and decided-by context where the model supports it.

**Why this priority**: Decision is the primary job of Inbox; detail without decide actions is incomplete.

**Independent Test**: Open a pending approval; Approve and Reject produce explicit feedback and move the item out of pending posture in the mock model (or document honest toast-only if session mutation is not yet wired).

**Acceptance Scenarios**:

1. **Given** pending approval detail is open, **When** the operator triggers Approve, **Then** mock feedback acknowledges approval; Reject uses non-primary/danger-appropriate presentation relative to Approve primary.
2. **Given** pending approval detail is open, **When** the operator triggers Reject, **Then** mock feedback acknowledges rejection without silent disappearance without feedback.
3. **Given** a decision completes in mock stage, **When** the operator opens the item under Approved or Rejected tabs, **Then** Decision section shows decided-by, decided-at, and decision note when fixture fields exist.
4. **Given** mock stage, **When** decide fires, **Then** the UI does not claim remote apply finished unless a related run result is already fixture-backed.

---

### User Story 3 - Inspect decided approvals and related run outcome (Priority: P2)

As an operator reviewing history, I open Approved or Rejected items and see the decision trail plus optional operation result when a related run exists.

**Why this priority**: Auditability of who decided what is part of governance; Inbox is not only a pending queue.

**Independent Test**: Open an approved fixture with decision fields and optional runResult; Decision section is complete; approve/reject actions are absent.

**Acceptance Scenarios**:

1. **Given** an approved/rejected approval, **When** detail opens, **Then** header does not offer Approve/Reject as live primary actions.
2. **Given** decision fields exist, **When** Decision section renders, **Then** decided-by, decided-at, and decision note are visible.
3. **Given** runResult or runId exists, **When** detail opens, **Then** related run identity/result is visible; optional open-run hand-off may be provided without building a second run detail implementation.

---

### User Story 4 - Honest empty, missing, and partial states (Priority: P2)

As an operator, when related component/node/resource fields are missing, justification is short, or the target id is unknown, the detail remains honest.

**Independent Test**: Force not-found and sparse related-entity fixtures; panel shows not-found / em dash / empty without crash.

**Acceptance Scenarios**:

1. **Given** an unknown approval id, **When** the panel resolves, **Then** not-found is shown.
2. **Given** optional related entity fields are absent, **When** Request context renders, **Then** missing values are honest placeholders.
3. **Given** search filters empty a tab, **When** no row is active, **Then** the panel collapses or stays empty without stale content from another tab.

---

### Edge Cases

- Risk Low / Medium / High — StatusChip tones, never hand-rolled risk pills.
- Requester kind person vs agent — both visible; agent requests are first-class (docs/spec.md human+agent co-management).
- Pending vs decided — actions only for pending.
- Long justification — scrolls inside DetailPanel body only.
- Related run may be missing even when approved (decision without execution).
- Approvals are governance intents; they are not silent privilege bypass.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Inbox approval detail MUST open in the single global DetailPanel; MUST NOT invent a second entity page, drawer, or modal for approvals.
- **FR-002**: Approval detail MUST use `DetailPanel.Body layout="sections"` + `DetailPanel.Grid`.
- **FR-003**: Approval detail MUST show identity header: domain inbox/security icon, single-line request title, and `DetailPanel.Action` only for feature actions.
- **FR-004**: For pending items, detail MUST expose Approve (primary) and Reject actions with honest mock or session-local decision feedback.
- **FR-005**: Approval detail MUST present Overview metrics: risk (`StatusChip`), status (`StatusChip`), environment.
- **FR-006**: Approval detail MUST present Request context: requested-by, requested-by kind, component/node/resource when known, policy, when, related run id when known.
- **FR-007**: Approval detail MUST present Justification as readable body text.
- **FR-008**: For non-pending items, detail MUST present Decision: decided-by, decided-at, decision note, optional operation result.
- **FR-009**: Approval detail MUST support loading, error, and not-found honest states.
- **FR-010**: Fixture data for approvals MUST cover pending, approved, and rejected with at least one high-risk and one agent-requested example; desktop detail-fixture-coverage remains green for approval fields already asserted.
- **FR-011**: Implementation MUST extract a dedicated approval detail composition (not only an inline thin block in Inbox).
- **FR-012**: Decision UI MUST NOT claim live policy engine enforcement or remote apply success beyond fixture-backed run result.
- **FR-013**: Collection activation remains independent of panel snap mode.
- **FR-014**: Scope is Inbox approval detail only. Full audit product rewrite, policy editor, and live workflow apply pipeline stay out of scope.

### Key Entities

- **Approval Request**: Human or agent request requiring decision before privileged work proceeds.
- **Requester**: Person or agent identity that submitted the request.
- **Risk Posture**: Low / Medium / High classification for operator attention.
- **Related System Context**: Optional component, node, resource, policy, and run references.
- **Decision Trail**: decided-by, decided-at, decision note, optional run result after approval.

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: governance gate for privileged operations.
- **Runtime owner (III)**: Renderer `@polterware/feature-shell` Inbox + workspace approval mocks.
- **Mutation safety (VII)**: mock/session decision only; real plan→approve→apply remains future core work.
- **Security boundary (VI)**: UI is not privilege escalation; approve does not bypass future policy engine.
- **Contracts (V)**: `approval-detail-ui.md` under this feature directory.
- **Test plan (XII)**: composition tests + existing APPROVAL_MOCKS fixture coverage.
- **Single source of truth (XIV)**: one ApprovalDetail composition.
- **Non-goals**: no live Connect approval API, no policy editor, no redesign of Notifications drawer.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can open any fixture approval from Inbox tabs and fully inspect request, risk, justification, and decision trail without another page.
- **SC-002**: 100% of pending fixtures show Approve and Reject; 100% of decided fixtures hide those live actions and show Decision content when fields exist.
- **SC-003**: At least one agent-requested and one high-risk fixture are inspectable end-to-end.
- **SC-004**: Not-found and sparse related-entity states are non-crashing.
- **SC-005**: Detail matches production operational pattern (sections grid, StatusChip, DetailPanel.Action) with zero hand-rolled inspector chrome.
- **SC-006**: Dedicated ApprovalDetail module replaces the thin inline Inbox block.

## Assumptions

- Mock/session stage only for decide semantics.
- Inbox collection and a basic inline approval detail already exist; this feature deepens production quality.
- Runs detail already exists; approval may deep-link by id without reimplementing run detail.
- Portuguese chat / English UI strings remain project law.
- Human and agent requesters are both first-class per `docs/spec.md`.
