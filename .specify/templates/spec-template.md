# Feature Specification: [FEATURE NAME]

**Feature Branch**: `[###-feature-name]`

**Created**: [DATE]

**Status**: Draft

**Input**: User description: "$ARGUMENTS"

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.

  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - [Brief Title] (Priority: P1)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently - e.g., "Can be fully tested by [specific action] and delivers [specific value]"]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]
2. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

### User Story 2 - [Brief Title] (Priority: P2)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

### User Story 3 - [Brief Title] (Priority: P3)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right edge cases.
-->

- What happens when [boundary condition]?
- How does system handle [error scenario]?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: System MUST [specific capability, e.g., "allow users to create accounts"]
- **FR-002**: System MUST [specific capability, e.g., "validate email addresses"]
- **FR-003**: Users MUST be able to [key interaction, e.g., "reset their password"]
- **FR-004**: System MUST [data requirement, e.g., "persist user preferences"]
- **FR-005**: System MUST [behavior, e.g., "log all security events"]

*Example of marking unclear requirements:*

- **FR-006**: System MUST authenticate users via [NEEDS CLARIFICATION: auth method not specified - email/password, SSO, OAuth?]
- **FR-007**: System MUST retain user data for [NEEDS CLARIFICATION: retention period not specified]

### Key Entities *(include if feature involves data)*

Use Polterware domain language when applicable:

- **Organization / Workspace**: Collaboration and grouping boundaries.
- **System**: Domain root being modeled (desktop UI may say Project as alias).
- **Environment**: Named context such as local, preview, staging, production.
- **Target**: Where operations run (machine, Compose, K8s scope, provider resource, optional Node).
- **Resource**: Application or infrastructure node (Card = visual only).
- **Relationship**: Declared/linked/configured/observed/managed/drifted edge.
- **Evidence**: Proof or challenge of the model.
- **Operation / Approval / Audit Event**: Safe mutation path artifacts.
- **Workbench panel**: Dockable operator surface (map, inspector, terminal, …).

Replace placeholders with feature-specific entities:

- **[Entity 1]**: [What it represents, key attributes without implementation]
- **[Entity 2]**: [What it represents, relationships to other entities]

### Constitution Constraints *(mandatory)*

Map the feature to `.specify/memory/constitution.md` (v2.x). Analysis MUST block
implementation on critical unresolved conflicts.

- **Operational architecture goal (I)**: [which of: understand / discover-organize / safe execution / collaboration / audit / external tools]
- **Runtime owner (III)**: [Main / Preload / Renderer / Web / polter-core / polter-server / polter-node / CLI / MCP]
- **Domain vs adapter (II, IV)**: [which domain/ports; which providers/platform adapters; no domain→concrete imports]
- **Contracts (V)**: [shared contracts/events/commands/streams touched; versioning if breaking]
- **Product loop impact**: [System Map / Inspector / Evidence / Operations / Approvals / Audit / Knowledge / other]
- **Evidence expectations**: [what proves linked/observed/verified/drifted state, if any]
- **Mutation safety (VII)**: [read-only | plan-only | full plan→review→approve→apply→logs→audit]
- **Operation audit fields**: [actor, target, scope, environment, policy, approval, correlation/operation ID]
- **Security boundary (VI)**: [unprivileged surfaces; authz scope; secrets as references; MCP cannot bypass]
- **Local vs cloud authority (VIII)**: [local | cloud | synchronized; offline behavior; conflict handling]
- **Performance (IX)**: [no Main/UI blocking; queues/backpressure/timeouts/virtualization as applicable]
- **Resilience (X)**: [timeouts, retries, idempotency, offline, recovery]
- **Observability (XI)**: [correlation identifiers required]
- **Test plan (XII)**: [unit / contract / integration / e2e / security / performance proportional to risk]
- **Structure created now (XIII)**: [exact packages only if real API + consumer; none if extending existing]
- **Single source of truth (XIV)**: [no permanent dual implementation; migration dual-path exit criteria if any]
- **Big-bang / refactor context (XV)**: [n/a | part of authorized architecture refactor branch]
- **Non-goals (I, XIII)**: [explicit cuts — no specialized-platform clone, no scenic scaffolds]

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: [Measurable metric, e.g., "Users can complete account creation in under 2 minutes"]
- **SC-002**: [Measurable metric, e.g., "System handles 1000 concurrent users without degradation"]
- **SC-003**: [User satisfaction metric, e.g., "90% of users successfully complete primary task on first attempt"]
- **SC-004**: [Business metric, e.g., "Reduce support tickets related to [X] by 50%"]

## Assumptions

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right assumptions based on reasonable defaults
  chosen when the feature description did not specify certain details.
-->

- [Assumption about target users, e.g., "Users have stable internet connectivity"]
- [Assumption about scope boundaries, e.g., "Mobile support is out of scope for v1"]
- [Assumption about data/environment, e.g., "Existing authentication system will be reused"]
- [Dependency on existing system/service, e.g., "Requires access to the existing user profile API"]
