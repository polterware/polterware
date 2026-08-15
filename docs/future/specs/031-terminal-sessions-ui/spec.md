# Feature Specification: Terminal Sessions UI (Mock)

**Feature Branch**: `031-terminal-sessions-ui`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Professional Terminal page + side panel mock — session list/switcher with StatusChip (active/idle/closed), target health honesty, new/close session mock, correlation/session id audit display, empty/disabled states, helper tests. No real SSH/PTY/main spawn. Aligned to docs/spec.md §8.6."

**Related product docs**: `docs/spec.md` (§8.6 Terminal, §56 Terminal e comandos, §64 Terminal UI, §65 screen states), `AGENTS.md` (StatusChip grammar; no live runtime claims; Terminal is a project page with mock sessions until SSH).

**Extends**: Thin Terminal surfaces under `packages/features/shell/src/` (`terminal-page.tsx`, `terminal-side-panel.tsx`, `target-picker.tsx`, types/fixtures/workspace-mocks). Does **not** implement SSH, PTY, Electron main spawn, or live host connectivity.

---

## Summary

Operators manage **mock terminal sessions** against project **targets** with professional session chrome: status StatusChip, target name, shell, started time, and visible correlation/session identifiers for audit honesty. Opening a session against an unusable target (offline, revoked, lifecycle) is blocked with explicit feedback. Empty states for no targets and no sessions are clear. The product never claims a live host connection while data remains fixture/mock.

This delivery raises the current thin Terminal page and side panel to production-ready mock structure. Existing thin files are **not** “already done.”

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Switch and inspect mock sessions (Priority: P1)

As an operator, I open the project Terminal page, see a professional session list/switcher, select a session, and read status, target, shell, and started time at a glance without believing the stream is a live SSH session.

**Why this priority**: Session identity and status are the primary operator loop on Terminal; without them the page is a raw log strip.

**Independent Test**: Load Terminal with fixture sessions → each session exposes StatusChip (active/idle/closed), target name, shell, startedAt; switching sessions updates the active output surface; a fixture/mock data-source cue remains visible.

**Acceptance Scenarios**:

1. **Given** fixture sessions exist, **When** the Terminal page renders, **Then** the session switcher shows each session with status StatusChip (active / idle / closed), target name, shell, and startedAt (or equivalent started-time label).
2. **Given** multiple sessions, **When** I select another session, **Then** the output surface and session metadata update to that session without leaving the page.
3. **Given** any session is shown, **When** I read status chrome, **Then** status uses StatusChip (not a hand-rolled pill or leading-dot-only badge for labeled status).
4. **Given** sessions are mock/fixture, **When** the page renders, **Then** copy does not claim “connected to a real host”; a fixture/mock data-source label is present where session honesty is shown.

---

### User Story 2 - Open session only on usable targets (Priority: P1)

As an operator, I pick a target to open a new mock session; online/degraded targets open a session, while offline/revoked/lifecycle targets are blocked with honest feedback (never silent fail).

**Why this priority**: Target health gates interactive sessions per product honesty and §8.6 / §65 offline/unavailable states.

**Independent Test**: Open target picker → usable targets show StatusChip health and open a session; unusable targets show StatusChip health and do not create a session; feedback names the status.

**Acceptance Scenarios**:

1. **Given** targets exist, **When** the target picker opens, **Then** each row shows target name and target-health StatusChip via established target status tone/label grammar.
2. **Given** a usable target (online or degraded), **When** I select it and no session already exists for that target, **Then** a new mock session is created and selected.
3. **Given** an unusable target (offline, revoked, registering, draining, upgrading, unknown), **When** I attempt to open a session, **Then** no new session is created and I receive explicit feedback naming the target and reason (not silent no-op).
4. **Given** a session already exists for a target, **When** I select that target again, **Then** the existing session is selected rather than silently duplicated (or duplication is prevented with clear UX).
5. **Given** zero targets in the project, **When** I open the picker or land with no targets, **Then** empty copy states that no targets are available.

---

### User Story 3 - Close sessions and empty honesty (Priority: P1)

As an operator, I can close a mock session and land on a clear empty state when none remain; I am never left with a blank, unexplained surface.

**Why this priority**: Close is a core session lifecycle action (§8.6 close); empty states are mandatory screen states (§65).

**Independent Test**: Close sessions until none remain → empty “no sessions” copy with guidance to open a target; reopen creates a new mock session.

**Acceptance Scenarios**:

1. **Given** an open session, **When** I close it, **Then** it leaves the switcher and another open session becomes active if any remain.
2. **Given** the last session is closed, **When** the page updates, **Then** an empty state explains there are no terminal sessions and how to open one (not a blank main area alone).
3. **Given** no sessions and usable targets exist, **When** I open a new session from the picker, **Then** a mock session appears and becomes active.

---

### User Story 4 - Audit identifiers on panel and page (Priority: P1)

As an operator, I can see session id and correlation id (monospace) on the Terminal surface and/or side panel even when data is fixture — so audit correlation is honest before real SSH lands.

**Why this priority**: §8.6 and §56 require correlation/audit on terminal sessions; mock UI must already surface identifiers.

**Independent Test**: Open side panel or active session chrome → session id and correlationId are visible in mono; neither invents a live host claim.

**Acceptance Scenarios**:

1. **Given** an active mock session, **When** session chrome (page and/or side panel) is visible, **Then** session id and correlation id are displayed in monospace.
2. **Given** the side panel is open for a target/session, **When** footer or metadata renders, **Then** target health uses StatusChip (or established tone mapping) and connection copy is mock-honest (fixture/mock, not “Connected to real host”).
3. **Given** fixture data only, **When** panel/page render, **Then** a fixture or mock data-source cue is visible.

---

### Edge Cases

- Unusable target selected from picker: no session row created; feedback shown; picker may close or stay open as long as feedback is not silent.
- All targets already have sessions: picker empty or “no available targets” with clear copy (not a broken list).
- Zero targets fixture set: empty targets copy; new-session control does not pretend a host opened.
- Closed session re-entry: closed sessions are not presented as active; reopening uses a new mock session when target is usable.
- Side panel without matching target: honest empty / select-target copy.
- Session with missing shell/startedAt fields: helpers supply safe display defaults without crashing.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Terminal session switcher MUST show session status via StatusChip with labels **Active**, **Idle**, and **Closed** (tones mapped consistently).
- **FR-002**: Session switcher / active chrome MUST show target name, shell, and startedAt for the active (and list) sessions.
- **FR-003**: Target picker MUST show target-health StatusChip using established terminal target status tone/label helpers.
- **FR-004**: Opening a session MUST succeed only for usable targets (online, degraded); unusable targets MUST be blocked with explicit operator feedback naming status.
- **FR-005**: Operators MUST be able to create a new mock session and close an existing mock session without claiming real SSH/PTY execution.
- **FR-006**: Active session chrome and/or side panel MUST display session id and correlation id in monospace for audit honesty.
- **FR-007**: Empty states MUST cover (a) no sessions and (b) no targets with clear operator copy.
- **FR-008**: Disabled / blocked interactions for offline or otherwise unusable targets MUST be honest (disabled control and/or feedback), never silent success.
- **FR-009**: UI MUST NOT claim connection to a real host while using fixture/mock data; a fixture/mock data-source label MUST appear where connection honesty is communicated.
- **FR-010**: Pure helpers for session status tone/label, mock session create/close model, unusable-target feedback, and target usability MUST be unit-tested; existing `isTerminalTargetUsable` coverage MUST remain and expand toward UI contract helpers.
- **FR-011**: Implementation MUST NOT add SSH, native PTY, Electron main-process spawn, or unrestricted Node/OS execution from the renderer.
- **FR-012**: Scope is limited to shell terminal page, side panel, target picker, types/helpers, fixtures, and tests — not architecture, operations workflows, org-team, integrations, project settings, or Electron main.

### Key Entities

- **TerminalTarget**: Machine/target where a session may open — id, name, shell, status (online/offline/degraded/registering/draining/upgrading/revoked/unknown), optional environment.
- **TerminalSession**: Mock interactive session — id, name, target reference, shell, status (active|idle|closed), startedAt, duration/display time, correlationId, output lines, dataSource (fixture/mock).
- **SessionStatus**: active | idle | closed — operator-facing lifecycle for mock sessions.
- **TargetUsability**: Derived rule — usable when online or degraded; otherwise blocked for new sessions.
- **AuditCorrelation**: session id + correlationId pair shown in mono for operator audit trail honesty.

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: Safe execution surface honesty + auditability for terminal sessions (mock until real SSH feature).
- **Runtime owner (III)**: Renderer feature UI only; no Main/PTY/SSH claimed live.
- **Domain vs adapter (II, IV)**: Session/target pure helpers in shell feature; UI consumes helpers; no domain→Electron.
- **Contracts (V)**: Local mock DTOs / UI contract notes only; no new cross-runtime terminal IPC in this slice.
- **Product loop impact**: Terminal page + title-bar side panel workbench surfaces.
- **Evidence expectations**: N/A (no resource evidence graph).
- **Mutation safety (VII)**: Mock session create/close only; no host-side mutation.
- **Operation audit fields**: correlationId + session id visible in UI mock.
- **Security boundary (VI)**: No unrestricted renderer Node/OS; no secret material in session chrome.
- **Local vs cloud authority (VIII)**: Local fixture authority; offline targets explicit.
- **Performance (IX)**: Trivial session lists; no Main blocking.
- **Resilience (X)**: Unusable targets blocked; empty/error copy required.
- **Observability (XI)**: correlation identifiers required in session UI.
- **Test plan (XII)**: Unit tests for session helpers + target usability UI contract.
- **Structure created now (XIII)**: Extend `@polterware/feature-shell` only.
- **Single source of truth (XIV)**: One mock session model; no dual live/mock runtime claims.
- **Big-bang / refactor context (XV)**: Part of operator surfaces quality pass on refactor branch.
- **Non-goals (I, XIII)**: No real SSH/PTY; no main-process spawn; no architecture/ops/org/integrations/settings edits.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of session lifecycle statuses shown in the switcher/chrome use StatusChip with Active/Idle/Closed labels.
- **SC-002**: 100% of target rows in the picker show health via StatusChip (or StatusChip-equivalent filled status grammar — not labeled StatusDot-only status).
- **SC-003**: Attempting to open a session on an unusable target produces zero new sessions and non-empty feedback in 100% of attempts.
- **SC-004**: Active session surfaces expose both session id and correlation id in monospace whenever a session is selected.
- **SC-005**: Empty fixture sets for sessions and for targets each produce dedicated empty copy (not blank chrome alone).
- **SC-006**: Shell terminal-related unit tests pass; helper coverage includes session model + target usability UI contract.
- **SC-007**: Product copy never asserts live host connection for fixture/mock sessions.

## Assumptions

- Mock-only until a dedicated real SSH/PTY feature lands; fixture data remains authoritative in Desktop for this surface.
- Usable targets are **online** and **degraded** only (`isTerminalTargetUsable` already encodes this).
- Session statuses are simplified to **active / idle / closed** for operator grammar (map any legacy connected/connecting/disconnected/error fixtures into this set).
- One open mock session per target is sufficient; re-select focuses existing session.
- Side panel remains the title-bar quick terminal; full page remains Operations → Terminal (project Terminal destination).
- Parallel agents own other P1 surfaces; this feature must not edit architecture, operations workflows, org-team, integrations, or project settings packages.
- Toasts or inline messages are acceptable honest feedback for blocked targets as long as the operator sees them.
- No [NEEDS CLARIFICATION] markers: scope, honesty rules, and rubric were fully specified in the request.
