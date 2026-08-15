# Feature Specification: Organization Team

**Feature Branch**: `028-organization-team`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Organization sidebar → Team page production-ready mockada. DataTable profissional, DetailPanel for org members, ORG_TEAM_MOCKS fixtures, MFA StatusChip consistency, invite mock honesty. Scope: org-scoped Team only — NOT project Access People/Teams."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse organization members (Priority: P1)

As an organization owner or admin, I open Organization → Team and see a professional table of organization members (name, role, MFA, optional status/last active) with clear copy that these are org-level members, not project Access people.

**Why this priority**: Team is the first-class org sidebar destination for membership; the current thin hardcoded table is not production-ready.

**Independent Test**: Open org sidebar Team for a fixture organization; table shows reusable fixtures with PrimaryCell identity, MFA as StatusChip for both enabled and disabled, and org≠project Access copy.

**Acceptance Scenarios**:

1. **Given** an organization with fixture members, **When** the operator opens Team, **Then** a DataTable lists members with name (PrimaryCell + domain icon), role, and MFA StatusChip for every row.
2. **Given** members with MFA enabled and disabled, **When** the table renders, **Then** both states use StatusChip (never plain text for disabled).
3. **Given** the Team page header, **When** the operator reads the description, **Then** copy states organization members are distinct from project Access.
4. **Given** fixtures only, **When** the page loads, **Then** the UI never claims live membership sync or cloud invite delivery.

---

### User Story 2 - Inspect an organization member in DetailPanel (Priority: P1)

As an org admin, I activate a member row and inspect overview (email, role, MFA, joined) plus optional fixture counts (teams/projects) in the global DetailPanel, with mock actions that stay honest.

**Why this priority**: Collection without detail is incomplete; production entity collections always open DetailPanel.

**Independent Test**: Activate any org team member row; global DetailPanel shows identity header, overview metadata, StatusChip MFA, and mock actions with honest feedback.

**Acceptance Scenarios**:

1. **Given** Team lists members, **When** the operator activates a row, **Then** the global DetailPanel shows the member name, domain icon, and overview fields (email, role, MFA, joined).
2. **Given** fixture fields for teamCount/projectCount exist, **When** detail opens, **Then** those counts are visible as metrics or metadata.
3. **Given** detail is open, **When** the operator triggers Change role or Remove, **Then** feedback is an honest mock toast (no fake remote mutation success).
4. **Given** an unknown member id, **When** the panel resolves, **Then** not-found or loading/error states remain honest.

---

### User Story 3 - Invite members (mock only) (Priority: P2)

As an org admin, I use Invite members to start an invitation flow that does not pretend email was delivered.

**Why this priority**: Invite is the primary creation action on Team; dishonest “email sent” feedback breaks trust.

**Independent Test**: Click Invite members; mock dialog or toast clearly states invitation is mocked / not delivered.

**Acceptance Scenarios**:

1. **Given** Team page is open, **When** the operator clicks Invite members, **Then** a mock dialog or toast appears.
2. **Given** invite is triggered, **When** feedback is shown, **Then** copy does not claim real email delivery or remote invitation API success.
3. **Given** optional detail action Invite, **When** used, **Then** the same honesty rule applies.

---

### User Story 4 - Empty and loading honesty (Priority: P2)

As an operator, when the organization has no members matching filters/search or data is loading, the page and panel stay honest.

**Independent Test**: Force empty member list and loading detail states; empty table message and loading panel render without crash.

**Acceptance Scenarios**:

1. **Given** no members for the organization (or search empties the list), **When** the table renders, **Then** an empty state explains no organization members.
2. **Given** detail loading, **When** the panel is shown, **Then** PageLoading appears without inventing member data.
3. **Given** fixtures filtered by organizationId, **When** switching organizations, **Then** only that org’s members appear.

---

### Edge Cases

- MFA enabled vs disabled — both StatusChip tones, never mixed chip/text.
- Owner / Admin / Member roles — sortable role column.
- Invited / active / suspended status when fixture-backed.
- Org member scope is organization membership, not project Access People or Access Teams.
- Last active optional — show em dash or omit when absent.
- Parallel agents own project Access details — this feature MUST NOT rewrite them.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Organization Team page MUST present organization members in design-system DataTable with PrimaryCell identity icon, role, and MFA StatusChip columns; optional lastActive/status columns when fixture-backed.
- **FR-002**: MFA enabled and disabled MUST both render as StatusChip (no plain “Disabled” text).
- **FR-003**: Activating a member row MUST open the global DetailPanel with a dedicated org-member detail composition (not a second page, modal, or drawer for entity detail).
- **FR-004**: Org member detail MUST use `DetailPanel.Body layout="sections"` + Grid with overview (email, role, MFA, joined) and optional teams/projects counts from fixtures.
- **FR-005**: Detail MUST support loading, error, and not-found honest states.
- **FR-006**: Mock actions Invite / Change role / Remove MUST use honest toasts or mock UI and MUST NOT claim remote membership mutation or email delivery.
- **FR-007**: Page-level Invite members MUST use mock dialog or toast with non-delivery honesty.
- **FR-008**: Fixtures MUST live as exported reusable `ORG_TEAM_MOCKS` in shell workspace-mocks (typed), not only an anonymous local 3-item array.
- **FR-009**: Members MUST be filterable/scoped by organizationId when multiple orgs have fixtures.
- **FR-010**: UI copy MUST clarify organization members ≠ project Access people/teams.
- **FR-011**: Empty and loading states MUST be honest for collection and detail.
- **FR-012**: Implementation MUST NOT rewrite project Access People/Teams/MemberDetail modules.
- **FR-013**: Page composition tests and detail helper tests MUST cover table MFA chips, DetailPanel activation wiring, fixture export, and helper tones.

### Key Entities

- **Organization Member**: Person identity with membership in exactly one organization context (role, MFA, join date, optional activity/status).
- **Organization Membership Role**: Owner, Admin, Member (and optional billing-oriented roles when fixture-backed).
- **MFA Posture**: Enabled or Disabled security posture for the member account.
- **Organization Invitation (mock)**: Intent to invite; mock-stage only — no email delivery claim.

### Constitution Constraints *(mandatory)*

- **Operational architecture (I)**: org membership governance surface per docs/spec.md §8.10 / §107.
- **Runtime owner (III)**: Renderer `@polterware/feature-shell` only.
- **Mutation safety (VII)**: mock/session only; no live membership API.
- **Security (VI)**: UI is not privilege escalation; remove/change-role mocks are not real revocation.
- **Contracts (V)**: `contracts/organization-team-ui.md`.
- **Test plan (XII)**: page + detail helper tests.
- **SSoT (XIV)**: single OrganizationTeamPage + OrgMemberDetail composition.
- **Non-goals**: project Access People/Teams, live invitations API, org billing rewrite, cloud membership sync.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can open Organization → Team and inspect every fixture member in the global DetailPanel without a second page.
- **SC-002**: 100% of fixture MFA states (enabled and disabled) render as StatusChip in both table and detail.
- **SC-003**: Fixtures are exported as `ORG_TEAM_MOCKS` and reused by page/detail/tests (not a private 3-row local constant only).
- **SC-004**: Invite and mutation actions never claim real email delivery or remote apply.
- **SC-005**: Shell tests for page composition and detail helpers pass for the touched surface.
- **SC-006**: Project Access People/Teams detail modules remain untouched by this feature’s diff.

## Assumptions

- Mock/session stage only for invite and membership mutations.
- Org sidebar Team currently resolves through org-settings/people; Team content is owned by OrganizationTeamPage (wired from that surface).
- Project Access People use separate PEOPLE_MOCKS / MemberDetail — intentionally distinct entity model.
- Portuguese chat / English UI strings remain project law.
- At least five org members across roles and MFA postures; cover at least one disabled MFA and one invited/suspended when status is shown.
