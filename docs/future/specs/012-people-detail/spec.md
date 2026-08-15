# Feature Specification: Access People Detail

**Feature Branch**: `012-people-detail`

**Created**: 2026-08-11

**Status**: Implemented

**Input**: User description: "Seguindo o padrão de docs/spec.md e o DetailPanel de production-ready (Resources / Architecture), desenvolver a tela de details de People no Access — entrega completa, mockada, production-ready. Extrair composição dedicada, enriquecer fixtures de pessoa (permissions, sessions, activity, auth) e alinhar com Access UI contract (permissions, auth, activity; edit/remove)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect a project member (Priority: P1)

As a project admin or operator with Access privileges, I select a person in Access → People and open their detail panel to understand who they are, what role and environments they hold, which teams they belong to, how they authenticate, what capabilities their membership grants, their active sessions, and their recent project activity — without leaving the Access surface.

**Why this priority**: People detail is the primary human-governance inspection surface for membership. Without it, operators cannot verify access posture or act on members with confidence.

**Independent Test**: With Access → People open and fixture members loaded, activating any person row shows a complete DetailPanel for that member with identity, access, permissions, teams, sessions, and activity sections; mock governance actions are reachable from the header.

**Acceptance Scenarios**:

1. **Given** Access → People lists members, **When** the operator activates a person row, **Then** the global DetailPanel shows that person's name and a domain identity icon, with body content for overview/access, permissions, teams, sessions, and recent activity.
2. **Given** a member detail is open, **When** the operator reviews Overview, **Then** they see email, project role, presence status, environments, join context, authentication method, MFA state, last activity, and last seen — without a second page or modal.
3. **Given** a member has multiple teams and environments in fixtures, **When** detail opens, **Then** Teams and environment scope are listed as structured collections (not only a single comma-separated string).
4. **Given** the member is found in fixtures, **When** the panel opens, **Then** loading is not required in the mock stage, and the panel never invents live backend authority for permissions.

---

### User Story 2 - Govern membership from detail (Priority: P1)

As a project admin, I use header actions on a person detail to change role, add the person to a team, or revoke project access. In this mock stage, actions confirm with honest mock feedback and do not claim real authorization enforcement.

**Why this priority**: Access contract requires edit/remove when permitted; operators expect governance actions adjacent to the inspected identity.

**Independent Test**: Open any member detail and invoke Change role, Add to team, and Revoke access; each produces clear mock feedback without navigating away or opening a forbidden second inspector.

**Acceptance Scenarios**:

1. **Given** member detail is open, **When** the operator triggers Change role, **Then** a mock success toast acknowledges the action and the panel remains on that member.
2. **Given** member detail is open, **When** the operator triggers Add to team, **Then** mock feedback acknowledges the team picker intent without claiming persistence.
3. **Given** member detail is open, **When** the operator triggers Revoke access, **Then** mock feedback acknowledges revocation intent; the UI does not imply a silent privilege path outside the shared mutation model.
4. **Given** a session row is shown, **When** the operator revokes a non-current session (mock), **Then** mock feedback acknowledges session revocation without claiming real session invalidation.

---

### User Story 3 - Understand permissions and auth posture (Priority: P2)

As an operator reviewing least-privilege posture, I open a member detail and see the capability effects derived from their project role (allow / deny / request-only), plus authentication and MFA posture, so I can judge whether the membership matches operational expectations.

**Why this priority**: docs/spec.md requires humans and agents under the same governance model; role chips alone are not authorization, but the detail MUST surface the effective capability picture operators expect when inspecting membership.

**Independent Test**: Open Admin, Operator, and Viewer fixture members; each shows a non-empty permissions collection with distinct effects appropriate to the role, plus auth method and MFA.

**Acceptance Scenarios**:

1. **Given** an Admin fixture member, **When** detail opens, **Then** permissions include elevated project governance capabilities marked allow (and any blocked high-risk paths marked deny/request-only as defined in fixtures).
2. **Given** a Viewer fixture member, **When** detail opens, **Then** write/mutation capabilities are deny or request-only, never presented as unrestricted allow.
3. **Given** any member, **When** Auth / Sessions is reviewed, **Then** authentication method, MFA enabled/disabled, and at least one session row (or an explicit empty state) are visible.

---

### User Story 4 - Honest empty, missing, and partial states (Priority: P2)

As an operator, when a member has no teams, no sessions, no activity, or the target is missing from fixtures, the detail remains honest: empty collections show empty states; missing entities show not-found; the UI never fabricates live runtime authority.

**Why this priority**: Explicit state beats magic; production-ready detail must handle incomplete membership data.

**Independent Test**: Force not-found and empty collection fixtures; panel shows PageError / empty table states without crashing.

**Acceptance Scenarios**:

1. **Given** an activation target for a member id not in fixtures, **When** the panel resolves, **Then** it shows a not-found state with a clear message.
2. **Given** a member with empty activity or sessions arrays, **When** those sections render, **Then** each shows an explicit empty message, not placeholder invents.
3. **Given** mock stage, **When** any governance action fires, **Then** feedback is labeled as mock and does not claim backend success beyond the toast.

---

### Edge Cases

- Member with status offline vs online — presence is labeled via StatusChip, not ad-hoc pills.
- Member with status invited (if present) — treated as non-active membership posture; still inspectable.
- Member with zero teams or zero environments — sections remain and show empty/em dash honestly.
- Long email / long capability names — stay single-line where PrimaryCell/metadata rules apply; tables scroll with the DetailPanel body only.
- Simultaneous People tab entities (invitation/team) — member detail only mounts for `entityKind === "member"`; no dual content clash.
- Role chips and permission tables are **not** authorization enforcement; real authz remains backend-owned in future stages.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Access → People MUST open person detail in the single global DetailPanel hosted by MainSurface; MUST NOT invent a second entity-detail page, drawer, or modal inspector for members.
- **FR-002**: Person detail MUST use `DetailPanel.Body layout="sections"` + `DetailPanel.Grid` for operational membership content (not document layout).
- **FR-003**: Person detail MUST show identity header: domain User icon (same family as the People table PrimaryCell), single-line person name, and feature-owned header actions via `DetailPanel.Action` only.
- **FR-004**: Person detail MUST expose mock governance actions: Change role, Add to team, Revoke access (and session revoke when sessions exist). Actions MUST use honest mock feedback in this stage.
- **FR-005**: Person detail MUST present Overview/access metadata: email, project role, presence status (`StatusChip`), environments, last activity, last seen, joined-at, invited-by when known, authentication method, and MFA state.
- **FR-006**: Person detail MUST present Permissions as a structured collection of capability rows with effect allow | deny | request-only (fixture-derived from role). Effects MUST use `StatusChip` for status semantics.
- **FR-007**: Person detail MUST present Teams as a structured collection (DataTable) resolved from the member's team membership, with empty state when none.
- **FR-008**: Person detail MUST present Sessions as a structured collection (device/client, location or origin, last active, current flag) with empty state when none; revoke is mock-only.
- **FR-009**: Person detail MUST present recent Activity as a structured collection (event title, when, optional result) with empty state when none.
- **FR-010**: Person detail MUST support loading, error, and not-found honest states consistent with other operational details (Resources / Architecture).
- **FR-011**: Fixture data for people MUST be rich enough for complete detail compositions (permissions, sessions, activity, auth) and covered by the desktop detail-fixture-coverage suite.
- **FR-012**: Implementation MUST extract a dedicated member detail composition (not only an inline thin block in the Access index) following ResourceDetail / ArchitectureNodeDetail quality.
- **FR-013**: Role chips, permission tables, and disabled controls MUST NOT claim frontend authorization. Mock stage MUST remain honest that enforcement is not live.
- **FR-014**: Person detail MUST remain scoped to project Access People (human members). Agent identities remain under Access → Agents; invitations and teams keep their own detail branches.
- **FR-015**: Collection activation remains independent of panel snap mode; selecting a row updates the target without forcing maximize.

### Key Entities

- **Person (Member)**: Human project collaborator identity with membership role, teams, environment scope, presence, auth posture, sessions, permissions picture, and recent activity.
- **Membership Role**: Project-scoped role label (Admin, Operator, Viewer, …) used to present capability effects; not itself an authorization engine.
- **Capability Effect**: Named capability with allow | deny | request-only for display of least-privilege posture.
- **Member Session**: Client/session record attributable to the person (device, origin, last active, current).
- **Member Activity Event**: Recent governance or operational event involving the person as actor.
- **Team Membership**: Link from person to project/org team for ownership and collaboration grouping.

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: collaboration + audit / governance inspection for human actors in Access.
- **Runtime owner (III)**: Renderer feature package `@polterware/feature-access` + shell workspace mocks; no Main/Node privilege for this UI stage.
- **Domain vs adapter (II, IV)**: UI composition over identity/organizations membership concepts; no new authorization engine inside Access; fixtures stand in for core data.
- **Contracts (V)**: extends Access UI member detail contract; new feature contract `people-detail-ui.md` under this feature directory.
- **Product loop impact**: Access / DetailPanel / Audit-adjacent inspection; no System Map change.
- **Evidence expectations**: n/a for live evidence; mock fixtures must be complete and honest.
- **Mutation safety (VII)**: mock-only actions with toast acknowledgment; no claim of plan→approve→apply pipeline completion.
- **Operation audit fields**: activity rows may show actor-related events; full audit remains Access → Audit.
- **Security boundary (VI)**: unprivileged renderer; role/permission UI is not enforcement; secrets never shown.
- **Local vs cloud authority (VIII)**: mock local fixtures only; offline/cloud sync out of scope.
- **Performance (IX)**: DetailPanel body is sole vertical scroller; tables stay modest fixture size.
- **Resilience (X)**: not-found / empty / mock failure feedback without crash.
- **Observability (XI)**: n/a for new correlation IDs in mock UI.
- **Test plan (XII)**: unit tests for member detail composition + fixture coverage expansion; access detail-targets remain green.
- **Structure created now (XIII)**: extend `packages/features/access` + shell workspace-mocks; no scenic package.
- **Single source of truth (XIV)**: one MemberDetail composition; remove thin inline duplicate once extracted.
- **Big-bang / refactor context (XV)**: continues operator product surfaces on `refactor/new`.
- **Non-goals (I, XIII)**: no live IdP, no real session kill API, no org-wide directory product, no second authz engine, no Agents detail rewrite, no invitation redesign.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can open any fixture person from Access → People and fully inspect membership posture (role, environments, teams, permissions, auth, sessions, activity) without navigating to another page.
- **SC-002**: 100% of fixture people used in People list have complete detail fields required by FR-005–FR-009 (no blank required section for the three primary fixtures).
- **SC-003**: Governance header actions are reachable in one click from the open person detail and always acknowledge with explicit mock feedback.
- **SC-004**: Not-found and empty-collection states are visible and non-crashing when fixtures omit data or the id is unknown.
- **SC-005**: Detail composition matches the production operational detail pattern used by Resources/Architecture (sections grid, StatusChip for status, DataTable for collections, DetailPanel.Action for header actions) with zero hand-rolled inspector chrome.
- **SC-006**: Desktop detail-fixture-coverage treats People member fixtures as first-class complete compositions (permissions, sessions, activity present for at least the primary Admin fixture).

## Assumptions

- Mock stage only: no ConnectRPC / polter-core membership APIs in this feature slice.
- Project Access People tab already lists members via `PEOPLE_MOCKS` and registers detail targets; this feature deepens detail quality, not the index IA.
- Roles remain Admin / Operator / Viewer in fixtures; custom roles matrix UI stays deferred (007 roles matrix).
- Permission rows are fixture-authored (or role-seeded helpers) for display; they are not computed from a live policy engine.
- Session revoke and membership mutations are toast-mock only.
- Portuguese chat / English UI strings remain project law.
- `docs/spec.md` Access + identity/organizations guidance is the product source for membership semantics; DESIGN.md + DetailPanel LAW govern chrome.
