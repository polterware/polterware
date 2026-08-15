# Feature Specification: Access Invitation Detail

**Feature Branch**: `024-invitation-detail`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Seguindo o padrão de docs/spec.md e o DetailPanel de production-ready (People / Teams), desenvolver a tela de details de Invitations no Access → People — entrega completa, mockada, production-ready. Extrair composição dedicada, enriquecer fixtures de invitation (email, role, team, status, expires, invitedBy, optional message/environments) e alinhar com governança de membership."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect a project invitation (Priority: P1)

As a project admin on Access → People, I select an invitation row (pending or expired) and open its detail panel to understand who was invited, to which role and team, who invited them, when it expires, and the current invitation status — without leaving Access.

**Why this priority**: Invitations are the pre-membership gate for People. Operators need more than a table row to resend, cancel, or re-invite confidently.

**Independent Test**: With Access → People open and fixture invitations loaded, activating any invitation row shows a complete DetailPanel with identity, assignment, provenance, and status.

**Acceptance Scenarios**:

1. **Given** Access → People lists invitations, **When** the operator activates an invitation row, **Then** the global DetailPanel shows the invitee email and a domain person/invite icon, with body content for status, role, team, expiry, and invited-by.
2. **Given** an invitation detail is open, **When** the operator reviews Assignment, **Then** role and team are explicit; optional environments/message appear when fixture-enriched.
3. **Given** the invitation is found in fixtures, **When** the panel opens, **Then** loading is not required in the mock stage and the UI never invents live email delivery.

---

### User Story 2 - Govern invitation lifecycle from detail (Priority: P1)

As a project admin, I use header actions appropriate to status: Resend / Cancel for Pending, Re-invite for Expired. Actions produce honest mock feedback only.

**Why this priority**: Lifecycle actions belong next to invitation identity; table-only actions are forbidden by product DataTable LAW.

**Independent Test**: Open pending invitation → Resend and Cancel toasts; open expired → Re-invite toast; Cancelled (if present) shows no destructive-redundant actions or honest disabled state.

**Acceptance Scenarios**:

1. **Given** a Pending invitation detail, **When** the operator triggers Resend, **Then** mock feedback acknowledges resend without claiming SMTP success.
2. **Given** a Pending invitation detail, **When** the operator triggers Cancel, **Then** mock feedback acknowledges cancellation (danger tone).
3. **Given** an Expired invitation detail, **When** the operator triggers Re-invite, **Then** mock feedback acknowledges re-invite intent.
4. **Given** mock stage, **When** any lifecycle action fires, **Then** no claim of real identity-provider or mailbox integration.

---

### User Story 3 - Relate invitation to team and future member (Priority: P2)

As an operator preparing access, I open an invitation and understand the target team (and optional open-team hand-off) and that acceptance will create a person membership — without inventing a person record that does not exist yet.

**Why this priority**: Invitations sit between anonymous email and full People member; navigation to team helps governance review.

**Independent Test**: Invitation with known team name/id can open team detail when team id is resolvable; no fabricated person detail for unaccepted invites.

**Acceptance Scenarios**:

1. **Given** invitation references a known team, **When** the operator activates open-team (if wired), **Then** team detail opens under Access People scope.
2. **Given** invitation is not yet accepted, **When** detail opens, **Then** there is no fake “member profile” pretending the person already joined.
3. **Given** optional invite message/note exists in fixtures, **When** detail opens, **Then** it is shown in a Notes/Message section.

---

### User Story 4 - Honest empty, missing, and partial states (Priority: P2)

As an operator, when invitation id is unknown or optional fields are empty, the detail remains honest.

**Independent Test**: Force not-found; sparse message omitted or empty; statuses Pending / Expired / Cancelled covered.

**Acceptance Scenarios**:

1. **Given** unknown invitation id, **When** panel resolves, **Then** not-found is shown.
2. **Given** no message field, **When** detail opens, **Then** message section is omitted or empty — not invented copy.
3. **Given** only Pending and Expired appear in the index filters, **When** Cancelled fixtures exist for detail coverage, **Then** they remain valid for direct target activation tests.

---

### Edge Cases

- Status Pending / Expired / Cancelled — StatusChip only.
- Long emails — single-line title truncation rules apply; full email in metadata if needed.
- Simultaneous People tab entities (member / invitation / team) — invitation detail mounts only for `entityKind === "invitation"`.
- Invitation actions are not authorization enforcement and not email delivery guarantees.
- Must not conflict with parallel Access detail work on agents/policies/audit/session (different entity kinds).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Access → People (Invitations table) MUST open invitation detail in the single global DetailPanel; MUST NOT invent a second page/modal for invitations.
- **FR-002**: Invitation detail MUST use `DetailPanel.Body layout="sections"` + `DetailPanel.Grid`.
- **FR-003**: Invitation detail MUST show identity header: domain person/invite icon matching invitations PrimaryCell family, single-line email title, and `DetailPanel.Action` only.
- **FR-004**: Invitation detail MUST expose status-appropriate mock actions: Pending → Resend + Cancel; Expired → Re-invite; Cancelled → none or Re-invite only if product chooses — document in contract.
- **FR-005**: Invitation detail MUST present Overview metrics: status (`StatusChip`), expires, role, team.
- **FR-006**: Invitation detail MUST present Assignment metadata: role, team, optional environments.
- **FR-007**: Invitation detail MUST present Provenance: invited-by, optional invited-at when enriched.
- **FR-008**: Invitation detail SHOULD present Message/Notes when fixture-backed.
- **FR-009**: Invitation detail MUST support loading, error, and not-found honest states.
- **FR-010**: Fixture data for invitations MUST cover Pending and Expired (and ideally Cancelled) with complete assignment/provenance; desktop detail-fixture-coverage remains green and may expand for new fields.
- **FR-011**: Implementation MUST extract a dedicated invitation detail composition (not only an inline thin block in Access index).
- **FR-012**: Lifecycle actions MUST use honest mock feedback; MUST NOT claim live email delivery.
- **FR-013**: Collection activation remains independent of panel snap mode.
- **FR-014**: Scope is Access People invitations detail only. Org-level directory invites, SSO SCIM, and live identity APIs stay out of scope.

### Key Entities

- **Invitation**: Pre-membership offer to an email for a project role/team.
- **Invitee**: Email identity not yet a project member.
- **Assignment**: Role + team (+ optional environments) the invitee will receive on accept.
- **Provenance**: Who invited and when/expiry posture.
- **Lifecycle Intent**: Resend, cancel, re-invite — mock only in this stage.

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: governed membership onboarding.
- **Runtime owner (III)**: `@polterware/feature-access` + shell invitation mocks.
- **Security boundary (VI)**: invitation UI is not privilege grant until accept + future backend; no secrets.
- **Mutation safety (VII)**: mock lifecycle only.
- **Contracts (V)**: `invitation-detail-ui.md` under this feature directory.
- **Single source of truth (XIV)**: one InvitationDetail composition.
- **Test plan (XII)**: composition tests + INVITATION_MOCKS coverage.
- **Non-goals**: no live email, no SCIM, no rewrite of member/team details, no conflict with agents/policies/audit/session specs.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can open any fixture invitation from Access → People and fully inspect assignment, status, expiry, and provenance without another page.
- **SC-002**: 100% of Pending fixtures expose Resend + Cancel; Expired fixtures expose Re-invite.
- **SC-003**: Not-found and sparse optional-message states are non-crashing.
- **SC-004**: Dedicated InvitationDetail module replaces the thin inline Access block.
- **SC-005**: Detail matches production operational pattern (sections grid, StatusChip, DetailPanel.Action).
- **SC-006**: Desktop detail-fixture-coverage continues to treat invitations as first-class compositions for required fields.

## Assumptions

- Invitations table and a basic inline invitation detail already exist in Access.
- People member detail and team detail are separate features already delivered; invitation may link to team only.
- Parallel agents may work on agents/policies/audit/session details — invitation work only touches invitation entity kind and its mocks.
- Portuguese chat / English UI strings remain project law.
- Mock stage only for email lifecycle.
