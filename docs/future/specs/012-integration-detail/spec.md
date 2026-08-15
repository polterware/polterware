# Feature Specification: Integration Detail Panel

**Feature Branch**: `012-integration-detail`

**Created**: 2026-08-11

**Status**: Draft

**Input**: User description: "Seguindo o padrão de docs/spec.md, desenvolver a tela de details de Integration no padrão production-ready, mockado. Detail sections locais Overview · Resources · Credentials · Sync · Permissions · Webhooks · Advanced (não page tabs). Conectar/revogar/reconnect/sync honestos; credentials never re-shown; disconnect does not destroy external resources."

**Related product docs**: `docs/spec.md` (§8.9 Integrations surface, §43 Integrations domain services, §112 Integrations rules, hub de integrações), `AGENTS.md` (Integration detail sections local to detail; global DetailPanel only), `DESIGN.md` (sections layout + optional Tabs rail under header), `specs/004-operator-product-surfaces` (Integrations list + early detail contract).

**Extends**: Integrations Connected/Catalog list surfaces already delivered under `packages/features/integrations` — this feature elevates **entity detail** to product quality. Does **not** implement live provider OAuth, real secret storage, or a second Integrations page for detail.

---

## Summary

Operators inspect a **connected integration** in the single global bottom DetailPanel and understand connection health, scopes/capabilities, last successful sync, imported resources, credential references, permissions, webhooks, and advanced revoke/hand-off — without leaving the Connected collection and without inventing a second inspector or page.

Detail is **mock-backed** for this delivery: every section is production-ready in structure, copy, a11y, and design-system grammar; values come from workspace fixtures and local mock mutations (enable/disable, sync toast, credential create write-only). No claim of live provider state.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inspect integration health and connection facts (Priority: P1)

As an operator, when I select a connected integration, the detail panel opens with Overview showing provider identity, health, account, environments affected, scopes/capabilities summary, last successful sync, and honest stale/error state so I know whether the connection is trustworthy.

**Why this priority**: §8.9 requires provider, health, scopes/capabilities, last successful sync, stale/error, and environments as the primary operator read path.

**Independent Test**: Open Connected → select GitHub (healthy) and Grafana (error). Overview shows different health, last sync, and error copy for Grafana; GitHub shows healthy without inventing a live provider call.

**Acceptance Scenarios**:

1. **Given** Connected has at least one integration, **When** I activate a row, **Then** the global DetailPanel shows that integration’s name and provider-domain icon matching the table row, with no second page, drawer, or modal for entity detail.
2. **Given** detail is open, **When** Overview is selected (default), **Then** I see status/health, provider, account, environment(s), last successful sync (or Never), resource count summary, and a short connection summary.
3. **Given** an integration with auth failure (e.g. token expired), **When** I open Overview, **Then** status is error/degraded and the error reason is visible — last-known summary and resource counts remain; the product does not pretend full health.
4. **Given** a disconnected or disabled integration, **When** I open Overview, **Then** health is explicit (disconnected/disabled) and primary mutation actions reflect that state (e.g. reconnect/enable rather than “all healthy”).
5. **Given** detail is open, **When** I switch among local detail sections via the in-detail Tabs rail, **Then** section state is local to detail (not Integrations page tabs) and the collection remains Connected | Catalog only.

---

### User Story 2 - Review imported resources and jump when available (Priority: P1)

As an operator, I open the Resources section to see resources synchronized from this provider (name, kind, environment, sync status) so I understand what the integration brought into the project.

**Why this priority**: Integrations feed inventory; operators need an honest inventory slice per connection.

**Independent Test**: Select AWS → Resources shows RDS/SQS fixtures; select Datadog (no resources) → empty state without fake rows.

**Acceptance Scenarios**:

1. **Given** an integration with imported resources, **When** I open Resources, **Then** a DataTable lists name, kind, environment, and status for that integration only.
2. **Given** resource rows exist and navigation hand-off is available, **When** I activate a resource row, **Then** the product navigates toward that resource context (or clearly toasts when hand-off is mock-only).
3. **Given** an integration with zero resources, **When** I open Resources, **Then** an empty state states no synchronized resources — no placeholder fake inventory.
4. **Given** a resource in error status, **When** listed, **Then** status uses StatusChip with non-healthy tone (not green “synced” for failures).

---

### User Story 3 - Manage credential references without secret re-display (Priority: P1)

As an operator, Credentials shows only **references** (name, scope, status, last rotated) and allows adding a credential with a write-only secret that is never shown again after submit.

**Why this priority**: §112 and security boundary — secrets must never appear after create; UI hygiene is non-negotiable even in mock mode.

**Independent Test**: Open Credentials for AWS; see named role ref and rotated age; open Add credential; submit secret; secret field clears and list still never shows the secret value.

**Acceptance Scenarios**:

1. **Given** an integration with credential refs, **When** Credentials opens, **Then** each row shows name, scope/role label, status, and rotated age — never a secret value or password.
2. **Given** I open Add credential, **When** I submit name + secret, **Then** the dialog closes, secret input is cleared, and success feedback is mock-honest; the new ref appears without the secret.
3. **Given** credential create is cancelled, **When** dialog closes, **Then** any typed secret is discarded from the UI state.
4. **Given** an integration with expired credentials, **When** Credentials opens, **Then** status communicates expired/needs rotation and guides reconnect/reconfigure without printing secret material.

---

### User Story 4 - Understand sync freshness and trigger sync (Priority: P1)

As an operator, Sync shows last successful sync, interval, imported counts, recent sync runs (or honest empty), rate-limit/backoff when present, and Sync now — without claiming full health after a partial/failed run.

**Why this priority**: §112 freshness/last sync and rate-limit backoff must be visible; operators act on sync from detail.

**Independent Test**: Overview + Sync for Vercel show last sync and interval; Sync now toast is mock; Grafana error does not show as fully healthy sync result.

**Acceptance Scenarios**:

1. **Given** a connected integration, **When** Sync opens, **Then** last sync, automatic interval, and imported resource count are visible.
2. **Given** recent sync runs exist in fixtures, **When** Sync opens, **Then** a table of runs shows result (success/partial/failed) with StatusChip — partial/failed never labeled as full health.
3. **Given** Sync now is available, **When** I click it (header or Sync section), **Then** mock feedback confirms sync started without inventing a real provider call.
4. **Given** rate-limit or backoff is in fixtures, **When** Sync opens, **Then** that condition is explicit (not hidden behind a green status only).

---

### User Story 5 - Inspect granted scopes and permissions (Priority: P2)

As an operator, Permissions shows capability/scopes granted vs denied for this connection so least-privilege intent is reviewable.

**Why this priority**: Connect registers scopes/capabilities (§112); operators must see what the integration can do.

**Independent Test**: Open Permissions for GitHub vs AWS; allowed/denied differ; coverage metric matches table rows.

**Acceptance Scenarios**:

1. **Given** permissions fixtures for the integration, **When** Permissions opens, **Then** each capability is listed with Allowed or Denied.
2. **Given** allowed and denied counts exist, **When** a coverage visualization is shown, **Then** it pairs with visible text counts and is omitted when counts would be meaningless (zero total).
3. **Given** permissions string summary exists, **When** Overview or Permissions renders, **Then** scope summary does not claim capabilities not listed as allowed.

---

### User Story 6 - Review webhooks and delivery honesty (Priority: P2)

As an operator, Webhooks lists delivery endpoints for this integration (destination host, events, status, last call), shows signing secret as configured/missing only, and offers mock test/send without exposing secret values.

**Why this priority**: §112 webhook verification when provider supports signature; operators need delivery visibility per connection.

**Independent Test**: Open Webhooks for GitHub (has webhooks) vs Datadog (none); failed delivery shows non-healthy status; signing secret never printed.

**Acceptance Scenarios**:

1. **Given** webhooks for the integration, **When** Webhooks opens, **Then** a DataTable lists endpoints with status chips and last call times.
2. **Given** a failed webhook, **When** selected/listed, **Then** last response reason is visible and status is failed — not active/healthy.
3. **Given** signing secret is configured, **When** detail shows secret state, **Then** only “Configured” / “Missing” appears — never the secret material.
4. **Given** no webhooks, **When** Webhooks opens, **Then** empty state is honest.

---

### User Story 7 - Reconnect, disable, revoke, and external hand-off (Priority: P1)

As an operator, header and Advanced expose Sync now, Reconfigure/Reconnect, Enable/Disable, Open provider console (external hand-off), and Disconnect — with clear copy that disconnect does **not** destroy external provider resources.

**Why this priority**: §112 revoke/disconnect rules and §8.9 reconnect/revoke + deep link.

**Independent Test**: Disable GitHub → status disabled and Enable appears; Disconnect shows non-destructive copy; Open console hand-off is external-only.

**Acceptance Scenarios**:

1. **Given** a connected integration, **When** I Disable, **Then** mock state becomes disabled and toast confirms; Enable restores mock connected (or original) without claiming live revoke APIs.
2. **Given** an error integration (auth failure), **When** header actions render, **Then** Reconfigure/Reconnect is primary recovery path.
3. **Given** Advanced, **When** I read Disconnect, **Then** copy states disconnect stops Polter synchronization and does not destroy external resources.
4. **Given** Open console / provider hand-off, **When** invoked, **Then** it is external hand-off (or mock toast), not a cloned provider admin console inside Polter.
5. **Given** Disconnect completes (mock), **When** done, **Then** the product does not claim deletion of cloud resources, repos, or projects on the provider.

---

### Edge Cases

- Catalog tab has no collection targets — detail does not invent an integration selection from catalog rows alone (opening a connected catalog provider jumps to Connected target).
- Setup wizard open — detail collection unregisters / does not fight the wizard full-page flow.
- Integration with `lastSync: Never` and zero resources — Overview and Sync remain coherent.
- Partial resource list vs advertised `resourceCount` — Resources table is source of listed rows; Overview may show fixture count honestly as summary (document assumption).
- Simultaneous disabled + error fixtures — disabled mock override applies only when originally connected; error integrations keep error unless explicitly handled.
- Webhook secret create (if offered) never re-displays secret after submit.
- Empty permissions / webhooks / credentials arrays — empty states, no fake “healthy all clear” bars with zero data.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Integration entity detail MUST open exclusively in the global DetailPanel hosted by MainSurface; features MUST NOT create a fixed inspector page, modal, drawer, or second panel for integration entity detail.
- **FR-002**: Integrations page tabs MUST remain **Connected** and **Catalog** only. Overview · Resources · Credentials · Sync · Permissions · Webhooks · Advanced MUST be **local detail sections**, not page tabs.
- **FR-003**: Detail MUST use `DetailPanel.Body layout="sections"` with optional shared Tabs rail directly under the header before the sectional grid (DESIGN exception for Integration detail).
- **FR-004**: Overview MUST present provider, health/status (StatusChip), account, environment(s), last successful sync (or Never), resource summary, scopes/capabilities summary, and error/stale reason when present.
- **FR-005**: Resources MUST list synchronized resources for the active integration only, with StatusChip status and honest empty state.
- **FR-006**: Credentials MUST show secret references only (name, scope, status, rotated); secret values MUST NOT appear after create; create dialog secret field is write-only and cleared on submit/cancel.
- **FR-007**: Sync MUST show freshness (last sync), interval when known, imported counts, optional recent runs with honest result tones, optional rate-limit/backoff, and Sync now action (mock-honest).
- **FR-008**: Permissions MUST list granted vs denied capabilities/scopes for the connection; coverage visualization only when counts are meaningful.
- **FR-009**: Webhooks MUST list endpoints for the integration with status, last call, and signing-secret configured/missing only; never print secret material.
- **FR-010**: Header actions MUST include Sync now and state-appropriate Enable/Disable or Reconfigure; Advanced MUST include external console hand-off and Disconnect with non-destructive external-resource copy.
- **FR-011**: Auth failure MUST mark health as error/degraded without erasing last-known summary/resource facts.
- **FR-012**: Disconnect/disable MUST NOT claim destruction of external provider resources.
- **FR-013**: This delivery MAY use mock fixtures and local mock mutations; the UI MUST NOT claim live provider connectivity, live secret storage, or live OAuth completion.
- **FR-014**: Detail header icon MUST reuse the same provider mark used on the Connected table row.
- **FR-015**: Status / health / result / severity everywhere in integration detail MUST use StatusChip (filled); no hand-rolled status pills.
- **FR-016**: Operational collections inside detail (resources, permissions, webhooks, sync runs, credentials) MUST use design-system DataTable when tabular; no Actions columns; row activation only for navigation/select patterns already allowed.
- **FR-017**: Loading / not-found / error detail states MUST remain honest when those states are supplied (panel keeps structure; no silent blank).

### Key Entities

- **Integration (connection)**: Authenticated/configured link to an external provider for a project context — provider, account, health/status, environments, last successful sync, summary, resource counts, optional error reason, optional external console URL, optional rate-limit state.
- **Integration resource**: Provider-imported inventory item linked to one integration (name, kind, external id, environment, sync status).
- **Credential reference**: Named secret **reference** for an integration (name, scope/role, status, rotated) — never carries secret material after create.
- **Capability / permission grant**: Normalized allowed or denied capability for the connection.
- **Sync run**: Historical or last sync attempt (started/finished, result, imported/changed counts, message).
- **Webhook registration**: Delivery endpoint (name, destination host, events, status, last call, signing configured, last response) for the integration.
- **Catalog provider**: Discoverable connector (out of primary scope except hand-off into connect or existing connection).

### Constitution Constraints *(mandatory)*

- **Operational architecture goal (I)**: Discover-organize external connections; support safe execution entry (sync/reconnect) with audit-visible mock actions; external tools via hand-off not clone.
- **Runtime owner (III)**: Renderer feature package `packages/features/integrations` + shared shell mocks; no Main/Node unrestricted access.
- **Domain vs adapter (II, IV)**: UI consumes normalized integration/capability shapes from workspace mocks (stand-in for domain queries); no provider-specific `if provider ===` branching for core layout — capability-oriented sections.
- **Contracts (V)**: Desktop UI contract for Integration detail sections; no protobuf change required for mock delivery.
- **Product loop impact**: Integrations hub feeding Resources/Architecture; detail is inspect + limited mock mutations (sync/disable/credential create).
- **Evidence expectations**: Sync health, last successful sync, webhook last response, credential status — fixtures act as evidence stand-ins; never fake “Observed live” for mock.
- **Mutation safety (VII)**: Mock-only mutations with toasts; real plan→approve→apply not claimed; disconnect non-destructive.
- **Operation audit fields**: Mock actions identify integration target and action in toast copy; no fake audit persistence required this increment.
- **Security boundary (VI)**: Secrets write-only; never re-display; renderer unprivileged; MCP not in scope.
- **Local vs cloud authority (VIII)**: Fixtures are local mock authority; providers remain authoritative for their real state (not simulated as Polter-owned).
- **Performance (IX)**: Detail content scroll only in DetailPanel.Body; tables use existing DataTable patterns.
- **Resilience (X)**: Error/disconnected/disabled/rate-limit states are first-class UI states.
- **Observability (XI)**: Mock toasts for operator feedback; no fake telemetry claims.
- **Test plan (XII)**: Unit/component tests for section presence, status mapping, secret non-display invariants, fixture coverage for rich detail models.
- **Structure created now (XIII)**: Extend `packages/features/integrations` (+ shell workspace-mocks); no new scenic package.
- **Single source of truth (XIV)**: One Integration detail composition; no dual detail page + panel.
- **Big-bang / refactor context (XV)**: Continues operator surfaces on current architecture branch.
- **Non-goals (I, XIII)**: No full provider console clone; no live OAuth; no real webhook delivery engine; no Credentials/Sync as page tabs (superseded by detail sections); no cloud billing management.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: From Connected list, an operator reaches full integration context (all seven local sections) in under 30 seconds without leaving the Integrations destination.
- **SC-002**: 100% of mock integrations in the Connected fixture open detail with Overview content (no blank panel for a selected valid target).
- **SC-003**: 100% of credential and webhook surfaces in this feature show zero secret material after create (only refs / configured|missing).
- **SC-004**: Operators can distinguish healthy vs error vs disconnected/disabled integrations from Overview without reading raw IDs (StatusChip + error copy).
- **SC-005**: Disconnect/Advanced copy is understood as non-destructive to external resources in review (no “delete AWS resources” or equivalent wording).
- **SC-006**: Detail uses only design-system DetailPanel + DataTable + StatusChip + Tabs grammar — no parallel hand-rolled detail chrome for the seven sections.
- **SC-007**: Fixture coverage test asserts rich integration detail models (credentials, capabilities, sync runs, webhooks as applicable) for at least one connected integration.

## Assumptions

- Connected and Catalog list UIs already exist and remain the only Integrations page tabs.
- Workspace mock module (`@polterware/feature-shell` workspace-mocks) is the SSOT for mock integration data for Desktop composition.
- Resource navigation hand-off may be a mock callback into Resources when wired by App; if absent, row activation may no-op or toast honestly.
- External console URLs may be fixture strings; opening may use `window.open` or a mock toast when URL absent.
- Rate-limit, sync runs, webhooks, and credentials may be empty for some providers; empty is valid.
- “Disabled” is a local mock operator control on top of fixture status for demo, not a persisted server field yet.
- Full Sync Center / Conflict Resolve standalone views remain optional secondary surfaces and are not required for detail completeness.
- Live polter-core integration APIs are out of scope for this increment; swapping fixtures for real queries later must not force a second detail UX.
