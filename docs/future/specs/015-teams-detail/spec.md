# Feature Specification: Access Teams Detail

**Feature Branch**: `015-teams-detail`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Seguindo o padrão de docs/spec.md e o DetailPanel de production-ready (People / Resources), desenvolver a tela de details de Teams no Access — entrega completa, mockada, production-ready. Extrair composição dedicada, enriquecer fixtures de team (members, owned resources, policies, on-call/escalation, activity) e alinhar com governança de organizations/teams."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect a project team (Priority: P1)

As a project admin or operator with Access privileges, I select a team in Access → People (Teams table) and open its detail panel to understand what the team owns, who belongs to it, who is on-call, which environments it covers, which policies apply, and recent team activity — without leaving the Access surface.

**Why this priority**: Teams are the primary collaboration/ownership grouping for organizations and projects. Without a production-grade team detail, operators cannot verify ownership posture or navigate from people ↔ teams confidently.

**Independent Test**: With Access → People open and fixture teams loaded, activating any team row shows a complete DetailPanel for that team with identity, overview/ops posture, members, owned resources, policies, and activity; mock governance actions are reachable from the header.

**Acceptance Scenarios**:

1. **Given** Access → People lists teams, **When** the operator activates a team row, **Then** the global DetailPanel shows that team's name and a domain team icon, with body content for overview, members, owned resources, policies, and recent activity.
2. **Given** a team detail is open, **When** the operator reviews Overview / operations posture, **Then** they see description, lead, on-call, escalation, schedule when present, environments, status, and updated/created context — without a second page or modal.
3. **Given** a team has multiple members and owned resources in fixtures, **When** detail opens, **Then** Members and Owned resources are listed as structured collections (not only scalar counts).
4. **Given** the team is found in fixtures, **When** the panel opens, **Then** loading is not required in the mock stage, and the panel never invents live backend authority for membership mutations.

---

### User Story 2 - Govern team from detail (Priority: P1)

As a project admin, I use header actions on a team detail to edit the team, add a member, or archive the team. In this mock stage, actions confirm with honest mock feedback and do not claim real authorization enforcement.

**Why this priority**: Operators expect governance actions adjacent to the inspected team identity, matching People detail quality.

**Independent Test**: Open any team detail and invoke Edit team, Add member, and Archive team; each produces clear mock feedback without navigating away or opening a forbidden second inspector.

**Acceptance Scenarios**:

1. **Given** team detail is open, **When** the operator triggers Edit team, **Then** a mock toast acknowledges the action and the panel remains on that team.
2. **Given** team detail is open, **When** the operator triggers Add member, **Then** mock feedback acknowledges the member-picker intent without claiming persistence.
3. **Given** team detail is open, **When** the operator triggers Archive team, **Then** mock feedback acknowledges archive intent; the UI does not imply a silent privilege path outside the shared mutation model.

---

### User Story 3 - Navigate between team members and people detail (Priority: P2)

As an operator reviewing ownership, I open a team detail, inspect its member roster, and activate a member row to jump to that person's People detail (and can return via team links on the person detail).

**Why this priority**: docs/spec.md treats teams as membership/ownership groupings; bidirectional navigation is required for usable Access governance.

**Independent Test**: Open Platform Team fixture → activate a roster member with a known person id → People member detail opens for that id; from member detail Teams table, activate the same team → team detail reopens.

**Acceptance Scenarios**:

1. **Given** team detail shows a member whose id exists in People fixtures, **When** the operator activates that member row, **Then** the active detail target becomes that member under Access → People.
2. **Given** member detail shows a team row, **When** the operator activates that team, **Then** team detail opens for that team id (existing People → Team path remains).
3. **Given** a roster entry without a resolvable person fixture, **When** listed, **Then** it remains visible; activation may no-op or stay honest without inventing a person.

---

### User Story 4 - Honest empty, missing, and partial states (Priority: P2)

As an operator, when a team has no members, no owned resources, no policies, no activity, or the target is missing from fixtures, the detail remains honest: empty collections show empty states; missing entities show not-found; the UI never fabricates live runtime authority.

**Why this priority**: Explicit state beats magic; production-ready detail must handle incomplete team data.

**Independent Test**: Force not-found and empty collection fixtures; panel shows PageError / empty table states without crashing.

**Acceptance Scenarios**:

1. **Given** an activation target for a team id not in fixtures, **When** the panel resolves, **Then** it shows a not-found state with a clear message.
2. **Given** a team with empty activity, policies, or owned resources arrays, **When** those sections render, **Then** each shows an explicit empty message, not invented rows.
3. **Given** mock stage, **When** any governance action fires, **Then** feedback is labeled as mock and does not claim backend success beyond the toast.

---

### Edge Cases

- Team status active vs archived — status labeled via StatusChip, not ad-hoc pills.
- Team with zero members or zero owned resources — sections remain and show empty states honestly.
- Long team name / long resource names — stay single-line where PrimaryCell rules apply; tables scroll with the DetailPanel body only.
- Simultaneous People tab entities (member / invitation / team) — team detail only mounts for `entityKind === "team"`; no dual content clash.
- Member and owned counts in the Teams index table MUST stay consistent with roster/resource array lengths for primary fixtures.
- Team membership and ownership UI are **not** authorization enforcement; real authz remains backend-owned in future stages.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Access → People (Teams table) MUST open team detail in the single global DetailPanel hosted by MainSurface; MUST NOT invent a second entity-detail page, drawer, or modal inspector for teams.
- **FR-002**: Team detail MUST use `DetailPanel.Body layout="sections"` + `DetailPanel.Grid` for operational team content (not document layout).
- **FR-003**: Team detail MUST show identity header: domain AddTeam (or Users) icon matching the Teams table PrimaryCell family, single-line team name, and feature-owned header actions via `DetailPanel.Action` only.
- **FR-004**: Team detail MUST expose mock governance actions: Edit team, Add member, Archive team. Actions MUST use honest mock feedback in this stage.
- **FR-005**: Team detail MUST present Overview / operations metadata: description, lead, on-call, escalation, schedule when known, environments, status (`StatusChip`), created-at, updated, and optional notes.
- **FR-006**: Team detail MUST present Members as a structured collection (DataTable) of roster people (name, role, presence) with empty state when none; activating a known person opens member detail when wired.
- **FR-007**: Team detail MUST present Owned resources as a structured collection (name, kind, environment) with empty state when none.
- **FR-008**: Team detail MUST present Policies as a structured collection (name, enforcement, status) with empty state when none.
- **FR-009**: Team detail MUST present recent Activity as a structured collection (event title, when, optional result) with empty state when none.
- **FR-010**: Team detail MUST support loading, error, and not-found honest states consistent with other operational details (People / Resources).
- **FR-011**: Fixture data for teams MUST be rich enough for complete detail compositions (roster, resources, policies, activity, ops posture) and covered by the desktop detail-fixture-coverage suite for the primary Platform Team fixture.
- **FR-012**: Implementation MUST extract a dedicated team detail composition (not only an inline thin block in the Access index) following MemberDetail / ResourceDetail quality.
- **FR-013**: Status chips, policy rows, and disabled controls MUST NOT claim frontend authorization. Mock stage MUST remain honest that enforcement is not live.
- **FR-014**: Team detail MUST remain scoped to project Access People teams. Org-wide directory product, live membership APIs, and Roles matrix stay out of scope.
- **FR-015**: Collection activation remains independent of panel snap mode; selecting a row updates the target without forcing maximize.
- **FR-016**: Teams index list columns (member count, owned count, on-call, updated) MUST remain populated from the enriched fixture model without breaking existing targets registration.

### Key Entities

- **Team**: Project/organization collaboration group with ownership, on-call posture, environments, member roster, policies, and activity.
- **Team Member (Roster Entry)**: Person summary on a team (id, name, role, presence); may link to full Person detail when id is known.
- **Owned Resource**: Resource (or architecture-linked asset) attributed to the team for ownership inspection in mock stage.
- **Team Policy Link**: Policy that applies to the team as actor/scope for display (not enforcement).
- **Team Activity Event**: Recent governance or operational event involving the team.
- **On-call / Escalation Posture**: Operational contact path (primary on-call, escalation chain, optional schedule).

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: collaboration + audit / governance inspection for teams in Access.
- **Runtime owner (III)**: Renderer feature package `@polterware/feature-access` + shell workspace mocks; no Main/Node privilege for this UI stage.
- **Domain vs adapter (II, IV)**: UI composition over organizations/teams/membership concepts; no new authorization engine inside Access; fixtures stand in for core data.
- **Contracts (V)**: extends Access UI teams detail; new feature contract `teams-detail-ui.md` under this feature directory.
- **Product loop impact**: Access / DetailPanel / ownership inspection; no System Map change.
- **Evidence expectations**: n/a for live evidence; mock fixtures must be complete and honest.
- **Mutation safety (VII)**: mock-only actions with toast acknowledgment; no claim of plan→approve→apply pipeline completion.
- **Operation audit fields**: activity rows may show team-related events; full audit remains Access → Audit.
- **Security boundary (VI)**: unprivileged renderer; membership/policy UI is not enforcement; secrets never shown.
- **Local vs cloud authority (VIII)**: mock local fixtures only; offline/cloud sync out of scope.
- **Performance (IX)**: DetailPanel body is sole vertical scroller; tables stay modest fixture size.
- **Resilience (X)**: not-found / empty / mock failure feedback without crash.
- **Observability (XI)**: n/a for new correlation IDs in mock UI.
- **Test plan (XII)**: unit tests for team detail composition + fixture coverage expansion; access detail-targets remain green.
- **Structure created now (XIII)**: extend `packages/features/access` + shell workspace-mocks; no scenic package.
- **Single source of truth (XIV)**: one TeamDetail composition; remove thin inline duplicate once extracted.
- **Big-bang / refactor context (XV)**: continues operator product surfaces on `refactor/new`.
- **Non-goals (I, XIII)**: no live org directory API, no real membership mutation API, no second authz engine, no Agents/Policies detail rewrite, no invitation redesign.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can open any fixture team from Access → People Teams table and fully inspect team posture (overview, members, owned resources, policies, activity, on-call) without navigating to another page.
- **SC-002**: 100% of primary Platform Team fixture fields required by FR-005–FR-009 are complete (non-empty members, owned resources, policies, and activity for the primary fixture).
- **SC-003**: Governance header actions are reachable in one click from the open team detail and always acknowledge with explicit mock feedback.
- **SC-004**: Not-found and empty-collection states are visible and non-crashing when fixtures omit data or the id is unknown.
- **SC-005**: Detail composition matches the production operational detail pattern used by People/Resources (sections grid, StatusChip for status, DataTable for collections, DetailPanel.Action for header actions) with zero hand-rolled inspector chrome.
- **SC-006**: Desktop detail-fixture-coverage treats Platform Team fixtures as first-class complete compositions (roster, resources, policies, activity present).
- **SC-007**: Bidirectional navigation works for at least one Platform Team member ↔ team pair via fixture ids.

## Assumptions

- Mock stage only: no ConnectRPC / polter-core team membership APIs in this feature slice.
- Project Access People tab already lists teams via `TEAM_MOCKS` and registers detail targets with `entityKind === "team"`; this feature deepens detail quality, not the index IA.
- Teams remain project-scoped collaboration groups; org-level team admin product stays deferred.
- Owned resources are fixture-authored ownership links for display; they are not live inventory sync.
- Membership mutations are toast-mock only.
- Portuguese chat / English UI strings remain project law.
- `docs/spec.md` organizations/teams + Access guidance is the product source for team semantics; DESIGN.md + DetailPanel LAW govern chrome.
- People detail (`012-people-detail`) already opens teams via `onOpenTeam`; this feature completes the reverse path and replaces the thin team stub.
