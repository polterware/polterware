# Feature Specification: Integrations Secondary Flows

**Feature Branch**: `029-integrations-secondary-flows`

**Created**: 2026-08-12

**Status**: Implemented

**Input**: User description: "Elevate setup-wizard, webhooks, sync-center, and conflict-resolve to professional mock quality aligned with docs/spec.md §8.9 and §22 integrations. Do not rewrite IntegrationDetail. Wire from integrations/index without regressing detail sections."

**Related product docs**: `docs/spec.md` (§8.9 Integrations surface, §22 integrations domain, §34 synchronization conflict policy), `AGENTS.md` (design-system DataTable/StatusChip/Dialog/Field; no hand-rolled card chrome), `specs/012-integration-detail` (primary connected-detail already delivered).

**Extends**: Thin satellite surfaces under `packages/features/integrations/src/` that today exist as fragile stubs. Does **not** reimplement IntegrationDetail sections, live OAuth, real provider webhooks, or real local↔cloud sync transport.

---

## Summary

Operators connect providers through a guided **setup wizard**, inspect and create **webhooks**, monitor **sync freshness** and history, and **resolve conflicts** with an explicit choice — all mock-honest, design-system compliant, and able to hand off into Connected + IntegrationDetail without inventing a second inspector.

This delivery raises four satellite flows to production-ready mock structure, empty/error honesty, StatusChip status grammar, and critical helper tests. Existing thin files are not “done”; they must meet the rejection rubric.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Complete setup wizard with clear steps (Priority: P1)

As an operator, I connect a catalog provider through a wizard with clear steps (provider → auth/scopes → environments → confirm), honest progress status, safe cancel, and finish that lands me on Connected with detail hand-off for the new connection.

**Why this priority**: Connect is the entry path for every integration; incomplete step chrome blocks trust in the rest of the hub.

**Independent Test**: Open catalog → Connect available provider → walk four steps → Finish opens Connected and activates detail when a matching fixture exists; Cancel at any step returns without creating a connection.

**Acceptance Scenarios**:

1. **Given** I open setup for a provider, **When** the wizard appears, **Then** I see exactly four ordered steps (provider, auth/scopes, environments, confirm) with the current step and completion progress visible via StatusChip (or equivalent filled status grammar), not a single orphan form.
2. **Given** I am on auth/scopes, **When** I choose a method and continue, **Then** scopes/permissions requested are listed honestly and secrets (if any) are write-only and never re-displayed after continue/finish.
3. **Given** I am on environments, **When** I select one or more environments, **Then** confirm summarizes provider, method, scopes, and environments before finish.
4. **Given** I finish successfully (mock), **When** the wizard closes, **Then** I am on Connected and the product activates the matching connected integration detail when fixtures allow; toast is mock-honest (“connected (mock)”).
5. **Given** I cancel at any step, **When** cancel is confirmed by leaving the wizard, **Then** no connection is created and I return to the previous Integrations surface without silent partial state.

---

### User Story 2 - Manage webhooks list with delivery honesty (Priority: P1)

As an operator, I review webhooks in a DataTable (destination URL/host, events, status StatusChip, last delivery), open professional row detail, and create a mock webhook without exposing signing secrets.

**Why this priority**: §22 lists webhooks as a core integration responsibility; secondary surface must match detail quality.

**Independent Test**: Open WebhooksTab with fixtures → table shows status chips and last delivery; empty filter shows empty state; create mock adds a row; failed webhook is non-healthy.

**Acceptance Scenarios**:

1. **Given** webhook fixtures exist, **When** the list renders, **Then** columns include destination, events, status (StatusChip), and last delivery; no Actions column.
2. **Given** a row is active, **When** detail shows, **Then** signing secret appears only as Configured/Missing, plus retry policy and last response — never secret material.
3. **Given** zero webhooks for the filter, **When** the list is empty, **Then** an empty state is shown (not a blank table body alone).
4. **Given** I create a webhook (mock), **When** I submit destination + events, **Then** a new active row appears and success feedback is mock-honest; secret field is write-only if present.
5. **Given** a failed webhook, **When** listed, **Then** StatusChip uses a non-healthy tone (not labeled healthy).

---

### User Story 3 - Monitor sync center freshness and history (Priority: P1)

As an operator, Sync Center shows endpoint health, last success/fail via StatusChip, stale state, offline queue, conflicts entry, history table, and a mock Sync now trigger without claiming live provider completion.

**Why this priority**: §8.9 and §34 require visible freshness, stale/error, and non-silent conflict path.

**Independent Test**: Open Sync Center status → degraded endpoint shows non-healthy chip; trigger Sync now → history gains a running/success mock row; empty history/queue shows empty state; stale summary is explicit.

**Acceptance Scenarios**:

1. **Given** sync endpoints fixtures, **When** Status tab opens, **Then** each endpoint shows health StatusChip and last activity; degraded/offline are not green healthy.
2. **Given** last overall success/fail is known, **When** Status renders, **Then** overall last success and last fail (or Never) are visible with StatusChip tones; stale condition is explicit when present.
3. **Given** Sync now is available, **When** I trigger it, **Then** mock feedback shows sync started and history reflects a new run (running then success/fail mock) without inventing real provider traffic.
4. **Given** offline queue or history has rows, **When** those tabs open, **Then** status/outcome use StatusChip; empty lists use empty state.
5. **Given** conflicts exist, **When** Conflicts tab opens, **Then** unresolved conflicts are listed and activating a row opens conflict resolve — never auto-overwrite.

---

### User Story 4 - Resolve conflicts with explicit choice (Priority: P1)

As an operator, I see a conflict list, open a conflict, compare local vs remote values, and choose keep-local, keep-remote, or merge (mock) — never silent overwrite.

**Why this priority**: §34 forbids silent overwrite of significant conflicts; resolution must be explicit and auditable in copy.

**Independent Test**: Open conflict → three choices required before resolve; Resolve disabled without choice; cancel returns to list; resolved conflict leaves list (mock).

**Acceptance Scenarios**:

1. **Given** unresolved conflicts, **When** the list shows, **Then** each conflict has resource, field, and unresolved StatusChip (or equivalent).
2. **Given** I open a conflict, **When** compare view shows, **Then** local and remote values and timestamps are both visible using design-system Card (no hand-rolled card chrome).
3. **Given** I have not selected a resolution, **When** I try to resolve, **Then** action stays disabled or is blocked — no default silent overwrite.
4. **Given** I choose keep-local, keep-remote, or merge, **When** I resolve, **Then** mock success feedback names the choice and the conflict is removed from the open list; audit copy states the resolution will be logged.
5. **Given** I cancel, **When** I leave the resolver, **Then** the conflict remains unresolved.

---

### Edge Cases

- Wizard finish when no matching connected fixture exists: still closes to Connected with honest toast; no crash selecting detail.
- Webhooks filtered to an integration with zero rows: empty state, not error.
- Sync now while a run is already “running” mock: disable or queue feedback without double-claiming success.
- Conflict resolve with submitting=true: primary action disabled.
- Offline / empty history and empty queue: dedicated empty states.
- Secret fields on wizard or webhook create: cleared after submit; never stored in list models.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Setup wizard MUST present four steps in order: provider → auth/scopes → environments → confirm, with visible progress/status.
- **FR-002**: Setup wizard MUST support safe cancel without creating a connection.
- **FR-003**: Setup wizard finish MUST hand off to Connected and activate IntegrationDetail when a matching connected fixture exists.
- **FR-004**: Auth/scopes step MUST treat secrets as write-only (never re-displayed after save/continue).
- **FR-005**: Webhooks list MUST use DataTable with destination, events, status StatusChip, and last delivery.
- **FR-006**: Webhooks MUST provide professional row detail (inline or panel) and a mock create flow.
- **FR-007**: Webhooks MUST show empty state when no rows match and MUST never show signing secret values.
- **FR-008**: Sync Center MUST show endpoint health, overall last success/fail, and stale state with StatusChip grammar.
- **FR-009**: Sync Center MUST support mock Sync now and a history table with outcome StatusChip.
- **FR-010**: Sync Center MUST provide empty states for empty queue/history/conflicts and MUST NOT auto-resolve conflicts.
- **FR-011**: Conflict resolve MUST list conflicts and require explicit keep-local | keep-remote | merge before resolve.
- **FR-012**: Conflict resolve MUST never silently overwrite; cancel leaves conflict unresolved.
- **FR-013**: All four flows MUST reuse design-system DataTable, StatusChip, Dialog/A11yDialog, Button, Field, and Card where nested panels are needed — zero hand-rolled card chrome.
- **FR-014**: Critical pure helpers MUST have unit tests (at least sync status helpers, conflict choice helpers, and one wizard step helper).
- **FR-015**: `integration-detail.tsx` MUST NOT be rewritten; only minimal index wiring for hand-off is allowed.
- **FR-016**: Integrations index wiring MUST preserve IntegrationDetail section behavior (Overview…Advanced).

### Key Entities

- **SetupWizardSession**: provider, method, scopes, environments, step index, connection test result (mock).
- **WebhookRegistration**: id, integrationId, name, destination, events, status, last delivery, signing configured, retry, last response.
- **SyncEndpoint**: id, name, health, last activity, stale flag.
- **SyncHistoryEntry**: id, summary, at, outcome (success|failed|running|partial).
- **SyncConflict**: id, resource, field, local value/meta, remote value/meta, resolution choice optional.
- **ConflictChoice**: `keep-local` | `keep-remote` | `merge`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can complete mock connect in under 2 minutes with all four wizard steps visible and finish landing on Connected.
- **SC-002**: 100% of status/health/result fields in the four flows use StatusChip (or documented StatusDot only for compact indicators — not raw text status pills).
- **SC-003**: Zero of the four flows present a bare list without empty/error handling for empty fixture sets.
- **SC-004**: At least two test files new or expanded pass for helpers (sync + conflict + wizard step coverage across files).
- **SC-005**: `integration-detail.tsx` line-level rewrite is avoided (no greenfield rewrite); detail section tests still pass.
- **SC-006**: Operators never see signing secrets or access-key secrets re-displayed after submit in these flows.

## Assumptions

- Mock-only: no live provider OAuth, webhook delivery, or sync transport in this feature.
- Shell workspace mocks remain SSOT for integration webhooks where shared; satellite-local fixtures allowed for sync-center/conflict when not already in shell.
- Conflict vocabulary is product-facing **keep-local / keep-remote / merge** (maps to Polter vs provider vs explicit merge mock).
- Sync Center remains a satellite export; primary product tabs stay Connected | Catalog; detail Sync/Webhooks sections stay on IntegrationDetail.
- Parallel P1 agents own other packages; this feature only touches integrations secondary files, helpers/tests, and minimal index wiring.

## Out of Scope

- Rewriting `integration-detail.tsx` or its seven detail sections.
- Architecture, operations workflows, org-team, settings, terminal, access.
- Real credential vault, OAuth device flow, or backend sync protocol.
- Generic project-level webhooks under Settings.
- Live rate-limit enforcement beyond honest fixture display.
