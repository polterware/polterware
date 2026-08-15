# Feature Specification: Project Settings Environments

**Feature Branch**: `030-project-settings-environments`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Project Settings production-ready mock (General · Environments · Advanced). Environments DataTable with StatusChip kind/protection, DetailPanel for environment protection posture, General/Advanced design-system forms with honest mock save and archive confirm. Align docs/spec.md §8.12. No Access/Integrations tabs in Settings."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse project environments with protection posture (Priority: P1)

As a project operator, I open Project → Settings → Environments and see a professional table of environments (name, kind, resources, nodes, protection, default) with StatusChip for kind and protection — not plain text cells only.

**Why this priority**: Environments is the core Settings collection; the current shallow table lacks protection posture chips and inspector depth.

**Independent Test**: Open Settings Environments with fixture data; every row shows PrimaryCell identity, kind StatusChip, protection StatusChip, resource/node counts, and default flag.

**Acceptance Scenarios**:

1. **Given** fixture environments for the project, **When** the operator opens Environments, **Then** a DataTable lists them with PrimaryCell name + domain icon, kind StatusChip, protection StatusChip, resources count, nodes count, and default indicator.
2. **Given** production, staging, and development kinds, **When** the table renders, **Then** each kind uses StatusChip (not plain text only).
3. **Given** Strict, Standard, and Relaxed protection levels, **When** the table renders, **Then** each protection level uses StatusChip.
4. **Given** fixtures only, **When** the page loads, **Then** the UI never claims live core environment mutation or remote promote.

---

### User Story 2 - Inspect environment protection in DetailPanel (Priority: P1)

As a project operator, I activate an environment row and inspect protection rules summary, default flag, counts, and mock promote/demote actions in the global DetailPanel.

**Why this priority**: Collection without detail is incomplete for operational posture review.

**Independent Test**: Activate any environment row; global DetailPanel shows identity, kind/protection chips, rules summary, default flag, and honest mock promote/demote feedback.

**Acceptance Scenarios**:

1. **Given** Environments lists rows, **When** the operator activates a row, **Then** the global DetailPanel shows environment name, domain icon, kind, protection, resources, nodes, and default flag.
2. **Given** protection rules on the fixture, **When** detail opens, **Then** a rules summary section lists those rules without inventing live policy evaluation.
3. **Given** detail is open, **When** the operator triggers Promote or Demote, **Then** feedback is an honest mock toast (no fake remote apply success).
4. **Given** an unknown environment id, **When** the panel resolves, **Then** not-found or loading states remain honest.

---

### User Story 3 - Edit general project settings (Priority: P1)

As a project operator, I open General and edit name, description, and default environment with design-system Field/Input/Textarea/Select, then save with an honest mock toast.

**Why this priority**: General is the default Settings tab and currently uses ad-hoc labels without Field or save honesty.

**Independent Test**: Change name/description/default env and click Save; Field primitives are used; toast states mock save (not live core persistence).

**Acceptance Scenarios**:

1. **Given** General tab, **When** the form renders, **Then** name, description, and default environment use design-system Field + Input/Textarea/Select (no invented inputClass recipes).
2. **Given** edited fields, **When** the operator clicks Save changes, **Then** a mock toast confirms session/mock save without claiming remote project apply.
3. **Given** project ID display, **When** Copy is used, **Then** copy is mock-friendly (clipboard optional) without claiming cloud identity sync.
4. **Given** Settings tabs, **When** the operator inspects tab strip, **Then** only General · Environments · Advanced appear (no Access, Integrations, Knowledge).

---

### User Story 4 - Advanced danger zone with confirm (Priority: P2)

As a project operator, I open Advanced and see data/retention controls plus a danger zone to archive the project via a design-system confirm dialog — without claiming real core delete/wipe.

**Why this priority**: Destructive affordances must be present and honest; current Advanced danger zone is disabled text without proper confirm flow for archive mock.

**Independent Test**: Open Advanced → Archive project → confirm dialog; cancel and confirm both work; confirm toast is honest mock; no live wipe claim.

**Acceptance Scenarios**:

1. **Given** Advanced tab, **When** the operator opens danger zone Archive, **Then** a design-system Dialog confirms the action.
2. **Given** the confirm dialog, **When** the operator cancels, **Then** no archive mock fires.
3. **Given** the confirm dialog, **When** the operator confirms archive, **Then** feedback states mock/session only and does not claim core project wipe.
4. **Given** Advanced retention controls, **When** rendered, **Then** they use design-system Field/Select primitives.

---

### User Story 5 - Honest fixtures and tests (Priority: P2)

As a maintainer, fixtures carry explicit dataSource honesty and helpers/table/detail contracts are covered by package tests.

**Why this priority**: Rejection rubric requires tests green and honest fixtures.

**Independent Test**: Run settings package tests; helpers map kind/protection tones; composition contracts assert DataTable, StatusChip, DetailPanel, Field, Dialog; fixtures mark dataSource fixture.

**Acceptance Scenarios**:

1. **Given** environment fixtures, **When** loaded, **Then** each entry has dataSource fixture (or package-level honesty export) and protection rules for production-like envs.
2. **Given** helper module, **When** tested, **Then** kind and protection labels/tones map deterministically.
3. **Given** composition sources, **When** contract tests run, **Then** DataTable + StatusChip + DetailPanel + Field + Dialog requirements hold.
4. **Given** Settings tab list source, **When** inspected, **Then** Access and Integrations are absent.

---

### Edge Cases

- Exactly one default environment; changing default in General is mock-only until save toast.
- Production kind pairs with Strict protection in fixtures; development with Relaxed.
- Empty environments list shows honest empty state.
- Legacy tab ids (integrations, access, people, etc.) normalize to General without reintroducing those tabs.
- Promote/demote on default production remains mock-only and may warn in toast copy.
- Danger zone must not enable real delete; Archive is mock confirm only.
- Parallel agents own org-team, integrations secondary, architecture, operations, terminal — this feature MUST NOT rewrite them.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Project Settings MUST expose exactly three page tabs: General, Environments, Advanced. MUST NOT reintroduce Access, Integrations, Knowledge, or Account tabs under Settings.
- **FR-002**: General MUST collect project name, description, and default environment using design-system Field + Input/Textarea/Select. Save MUST emit an honest mock toast.
- **FR-003**: Environments MUST present environments in design-system DataTable with PrimaryCell identity icon, kind StatusChip, protection StatusChip, resources count, nodes count, and default indicator.
- **FR-004**: Kind values (production / staging / development / local as fixture-backed) and protection levels (Strict / Standard / Relaxed) MUST both render as StatusChip in the table.
- **FR-005**: Activating an environment row MUST open the global DetailPanel with a dedicated environment detail composition (not a second page, modal, or drawer for entity detail).
- **FR-006**: Environment detail MUST use DetailPanel.Body layout="sections" + Grid with overview (kind, protection, default, counts) and protection rules summary from fixtures.
- **FR-007**: Detail MUST support loading, error, and not-found honest states.
- **FR-008**: Mock actions Promote / Demote (and Add environment if present) MUST use honest toasts and MUST NOT claim remote environment mutation success.
- **FR-009**: Advanced MUST include a danger zone with Archive project that opens a design-system confirm Dialog; confirm feedback MUST state mock/session only and MUST NOT claim core wipe/delete.
- **FR-010**: Advanced retention/notification controls MUST use design-system Field/Select (or CheckboxRow when applicable) — no invented inputClass.
- **FR-011**: Fixtures MUST be typed reusable exports with honest dataSource (fixture) and include protection rules for at least the production environment.
- **FR-012**: Implementation MUST NOT rewrite org-team, integrations secondary flows, architecture, operations, or terminal packages.
- **FR-013**: Package tests MUST cover settings helpers (kind/protection labels+tones, tab normalize) and environments table/detail composition contracts.

### Key Entities

- **Project Settings (General)**: Display name, description, default environment reference, project/system id.
- **Project Environment**: Named deployment stage with kind, protection posture, resource/node counts, default flag, protection rules summary.
- **Environment Kind**: Production, Staging, Development (and Local when fixture-backed).
- **Protection Level**: Strict, Standard, Relaxed — operational gate posture, not live policy engine output.
- **Protection Rule (summary)**: Named rule line shown in detail (e.g. require approval, freeze window) — fixture text only.
- **Project Archive Intent (mock)**: Operator intent to archive; mock-stage only — no core delete.

### Constitution Constraints *(mandatory)*

- **Operational architecture (I)**: project settings surface per docs/spec.md §8.12.
- **Runtime owner (III)**: Renderer `@polterware/feature-settings` only.
- **Mutation safety (VII)**: mock/session only; no live project/environment API.
- **Security (VI)**: Archive mock is not real destruction; UI is not privilege escalation.
- **Contracts (V)**: `contracts/project-settings-ui.md`.
- **Test plan (XII)**: helpers + composition contracts.
- **SSoT (XIV)**: single SettingsView + EnvironmentDetail composition.
- **Non-goals**: live core settings persistence, Access/Integrations under Settings, org Team, real project delete, cloud sync of settings.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can open Settings → Environments and inspect every fixture environment in the global DetailPanel without a second page.
- **SC-002**: 100% of fixture environment rows show StatusChip for both kind and protection.
- **SC-003**: General save and Archive confirm never claim real core persistence or wipe.
- **SC-004**: Settings package tests for helpers and table/detail contracts pass.
- **SC-005**: Settings tab strip contains only General, Environments, Advanced.
- **SC-006**: Diff for this feature does not reintroduce Access/Integrations tabs and does not touch forbidden parallel-agent packages.

## Assumptions

- Mock/session stage only for save, promote/demote, add environment, and archive.
- Environments fixtures live in `@polterware/feature-settings` (package-local) to avoid conflicting with shell breadcrumb EnvironmentItem shape unless wiring reuses shell ids.
- Project sidebar Settings remains the host; App.tsx already mounts SettingsView.
- Portuguese chat / English UI strings remain project law.
- At least three environments covering Strict/Standard/Relaxed and prod/staging/dev kinds.
