# Feature Specification: Remaining Operator Surfaces

**Feature Branch**: `007-remaining-operator-surfaces`

**Created**: 2026-08-03

**Status**: Draft

**Input**: User description: "Complete remaining operator product surfaces already implied by Polter architecture—observability, provisioning, synchronization, notifications, organizations, agent sessions, webhooks, billing, onboarding, and related deep flows—while adding only Observability to the sidebar; all other screens via tabs, contextual actions, breadcrumbs, global chrome, and settings."

**Depends on / extends**: `004-operator-product-surfaces` (seven-destination shell, in-page tabs, detail pages, process drawer, command palette, shared states) and `003-operator-design-system` (shell anatomy, tokens, quiet operator language). This feature **extends** the product map rather than replacing core destinations already specified in 004.

**Supersedes within scope**: The hard “exactly seven primary destinations” rule from 004 for the product map going forward. Primary destinations become **eight**: Architecture, Resources, Operations, Observability, Knowledge, Access, Integrations, Settings. Group label **KNOWLEDGE** becomes **INSIGHTS** (Observability, Knowledge, Access).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Extended shell navigation and Project Overview (Priority: P1)

As an operator, I open Polter and see the updated sidebar: **CORE** (Architecture, Resources, Operations), **INSIGHTS** (Observability, Knowledge, Access), **CONFIGURATION** (Integrations, Settings). Clicking the Polter logo/name opens **Project Overview** (`/overview`) for the current system: system status, attention required, recent activity, and quick actions. A chevron next to the project name opens a **project/workspace switcher** popover (not a full page) to search, switch project, create a project, or manage organizations.

**Why this priority**: Overview and project switching are the entry spine for multi-project operators; Observability must appear in the IA without bloating the sidebar with secondary capabilities.

**Independent Test**: Launch shell; verify eight destinations and three group labels; click logo → Overview; open switcher, search, switch project (or cancel); confirm notifications, search, and sync are not permanent sidebar items.

**Acceptance Scenarios**:

1. **Given** the application shell is open, **When** the operator views the sidebar, **Then** they see CORE (Architecture, Resources, Operations), INSIGHTS (Observability, Knowledge, Access), and CONFIGURATION (Integrations, Settings)—exactly eight primary destinations and no permanent nested nav for secondary surfaces.
2. **Given** the operator clicks the Polter logo or project name, **When** navigation completes, **Then** Project Overview opens for the current project with system status, attention-required items, recent activity, and quick actions (open architecture, create resource, run command, connect integration as available).
3. **Given** the operator opens the project switcher, **When** the popover appears, **Then** they can search projects, see projects grouped by organization, identify the current project, switch project, create a project, and open manage-organizations—without navigating to a full switcher page.
4. **Given** any primary destination, **When** the operator looks for notifications, full search results, or synchronization, **Then** those capabilities live in global chrome (header, palette, footer status)—not as extra sidebar destinations.
5. **Given** Overview shows attention items, **When** an item is activated, **Then** the operator is taken to the relevant surface (approval, integration, node, etc.) with enough context to act.

---

### User Story 2 - Global notifications, search results, and synchronization center (Priority: P1)

As an operator, I use global chrome for ephemeral and coordination work: a **notification center** drawer from the bell, a **full search results** page after command-palette “show all results”, and a **synchronization center** opened from local-core status in the footer. I can filter notifications, mark them read, open deep links; search across architecture, resources, operations, and knowledge; and inspect local core, cloud, and provider sync health, offline queue, conflicts, and history.

**Why this priority**: Operators need continuous awareness and coordination without expanding sidebar density; these surfaces close critical gaps called out for observability of product state itself (sync, alerts, discovery).

**Independent Test**: Open notifications drawer; filter All/Unread/Approvals; mark all read; run palette search and open full results; open `/sync` from footer status; open offline queue and a conflict.

**Acceptance Scenarios**:

1. **Given** the operator opens the notification center, **When** the drawer appears, **Then** notifications list with filters (All, Unread, Approvals), relative time, type-appropriate primary action (review approval, open node, open integration), close control, and mark-all-as-read.
2. **Given** the notification center is open, **When** the operator dismisses it or navigates away via an action, **Then** the drawer closes without permanent layout chrome remaining.
3. **Given** the operator searches via the global palette and chooses show-all-results, **When** the results page loads, **Then** the query is editable, category chips with counts (All, Architecture, Resources, Operations, Knowledge, and other in-scope categories) filter results, and each hit shows type and key status/context.
4. **Given** a search result is activated, **When** navigation occurs, **Then** the operator lands on the correct entity surface (not a dead end).
5. **Given** the operator opens Synchronization from footer core status, **When** the center loads, **Then** Status shows local core, Polter Cloud (when configured), and connected providers with health, last heartbeat/sync, pending upload count, and conflict count; Offline queue, Conflicts, and History tabs (or equivalent) are available; Sync now and Settings entry points appear when permitted.
6. **Given** a sync conflict exists, **When** the operator opens it, **Then** they see side-by-side Polter vs provider state, choose a resolution (keep Polter / accept provider / ignore until next sync), and the resolution is recorded for audit.

---

### User Story 3 - Observability hub (Priority: P1)

As an operator, I open **Observability** from the sidebar and work Overview, Logs, Metrics, Traces, and Monitors as in-page tabs. I get a control-plane health picture, filterable live logs, service metrics with external dashboard hand-off, trace list and waterfall detail, and monitors/alerts with mute/edit/resolve—without turning Polter into a full APM/logs product clone.

**Why this priority**: Observability is the only new primary destination; it completes the Observe leg of the product loop for operators who stay in Polter for triage and deep-link out for advanced analysis.

**Independent Test**: Open Observability → each tab; open a log event dialog; open a trace detail; open a monitor detail; use Open Grafana/Datadog when configured; change time range.

**Acceptance Scenarios**:

1. **Given** Observability is selected, **When** the page loads, **Then** environment context, time-range control, refresh, and tabs Overview / Logs / Metrics / Traces / Monitors are present.
2. **Given** Overview, **When** data is available, **Then** overall health cards (services, error rate, active alerts), a service health list (status, latency, errors or queue as applicable), and external dashboard hand-offs appear when integrations exist.
3. **Given** Logs, **When** the stream is active, **Then** operators can search, filter by service/level/time, toggle live updates, pause/clear/export/download as permitted, open provider logs when integrated, and open a log event for structured fields plus links to resource and trace when present.
4. **Given** Metrics, **When** a service is selected, **Then** key charts (request rate, latency, error rate, CPU or equivalent) for the time range are shown, with hand-off to the complete external dashboard when available.
5. **Given** Traces, **When** traces are listed, **Then** filters for all/errors/slow/successful work; activating a row opens trace detail with service waterfall, selected span facts, and open-resource / view-logs when linked.
6. **Given** Monitors, **When** monitors exist, **Then** list shows target, status, last check, alert count; Create monitor when permitted; detail shows condition, current value trend, related resource/traces/logs/runbook links, and mute/edit/resolve when permitted.
7. **Given** advanced analysis is required beyond hub capabilities, **When** the operator uses Open provider / Open dashboard, **Then** Polter hands off rather than embedding a full specialized observability console.

---

### User Story 4 - Provisioning, drift, and node registration (Priority: P1)

As an operator, under **Resources** I use a **Provisioning** tab (alongside Inventory, Nodes, Processes) with Catalog, Executions, and Drift. I pick a catalog item, configure a multi-step plan (including estimated cost when known), review policy checks, request approval when required, watch execution steps, and resolve infrastructure drift. Separately, **Add node** runs a registration wizard with install token and heartbeat wait.

**Why this priority**: Provisioning and nodes close the path from intent to infrastructure under the same plan → review → approve → apply → logs → audit discipline as other mutations.

**Independent Test**: Open Resources → Provisioning catalog; start a provision flow through plan review; open an execution; open Drift and resolve one item; start Add node wizard and copy install command.

**Acceptance Scenarios**:

1. **Given** Resources is open, **When** tabs are inspected, **Then** Inventory, Nodes, Processes, and **Provisioning** are available as in-page tabs (Provisioning is not a sidebar item).
2. **Given** Provisioning Catalog, **When** browsed, **Then** templates are grouped (Compute, Databases, Messaging, Applications, etc.) and Provision resource / catalog item starts configuration.
3. **Given** provisioning configuration, **When** the operator completes required fields, **Then** environment, naming, capacity, HA/backup options as applicable, and estimated cost when known are shown before review.
4. **Given** plan review, **When** the plan is shown, **Then** resources to create, policy check results (including production approval when required), draft save, and request-approval-and-provision (or equivalent) respect permissions and governance.
5. **Given** a provisioning execution, **When** opened, **Then** step progress, live output, cancel when allowed, and provider-operation hand-off are available; successful completion imports resources into Polter and offers architecture linking when applicable.
6. **Given** Drift, **When** differences exist, **Then** list shows resource, difference, severity, detected time; detail compares desired vs actual; actions Accept actual, Restore desired, or Ignore are available when permitted and audited.
7. **Given** Add node, **When** the wizard runs, **Then** the operator chooses target type (local, remote, Docker, Kubernetes), receives an install command with expiring token, can copy and regenerate token, sees waiting-for-heartbeat state, and can confirm installation.

---

### User Story 5 - Architecture import, snapshot compare, and view editor (Priority: P2)

As an operator, from Architecture I **import discovered resources** into the map (link existing, create-and-link, or ignore), **compare architecture snapshots**, and **edit saved views** (included components, filters, layout preference).

**Why this priority**: Completes Architecture deep flows already hinted in 004 (compare snapshots, create view) so discovery and model maintenance stay in product without ad-hoc tools.

**Independent Test**: Open Import resources wizard; apply mapping; open Compare snapshots; open Edit view; save.

**Acceptance Scenarios**:

1. **Given** Import resources, **When** a source is selected and discovery completes, **Then** each discovered resource can be linked to an existing component, create-and-link a new component, or ignored; suggested dependencies are shown; Apply mapping updates architecture under permission rules.
2. **Given** Compare snapshots, **When** two snapshots (or snapshot vs current) are chosen, **Then** Summary, Components, Dependencies, and Configuration views summarize added/removed/changed items; visual diff and restore are available when permitted.
3. **Given** Edit view, **When** the operator changes name, description, included components, filters, and layout, **Then** Save persists the view and Cancel discards unsaved edits after confirmation if dirty.

---

### User Story 6 - Workflow builder and schedules (Priority: P2)

As an operator in Operations → Workflows, I **edit a workflow** as an ordered step graph with per-step configuration (including approval steps), test and save versions, and manage **Schedules** (create, pause, run now, edit) separately from definitions.

**Why this priority**: Completes the Operations mutation authoring surface so operators are not limited to list/detail of opaque workflows.

**Independent Test**: Open workflow editor; select a step; change configuration; save; open Schedules; pause one; run now.

**Acceptance Scenarios**:

1. **Given** workflow edit, **When** open, **Then** ordered steps list, selected step configuration panel, add-step, trigger summary, version/saved state, Test and Save actions appear as permitted.
2. **Given** an approval step is selected, **When** configured, **Then** name, condition, reviewers, and expiration (or equivalent) can be set without leaving the builder.
3. **Given** Schedules tab, **When** schedules exist, **Then** list shows schedule, workflow, next run, status, timezone; detail shows cadence, retry, notify; Run now / Edit / Pause work when permitted.

---

### User Story 7 - Access: teams, invitations, roles, audit detail, agent sessions (Priority: P2)

As an access administrator, under Access I manage **Teams** and **Invitations** as additional tabs (with Members, Agents, Policies, Audit), open **team detail**, review a **roles and permission matrix**, open **audit event detail** with integrity context, and inspect **agent session detail** (objective, tool allow/deny, terminate).

**Why this priority**: Collaboration and governance surfaces were only partially covered in 004; teams, invitations, roles matrix, and agent sessions close operator trust and onboarding loops.

**Independent Test**: Open Teams and a team detail; open Invitations and resend/revoke; open Roles matrix; open an audit event; open an agent session and terminate if allowed.

**Acceptance Scenarios**:

1. **Given** Access, **When** tabs are inspected, **Then** Members, Teams, Agents, Policies, Audit, and Invitations appear as in-page tabs.
2. **Given** Teams list, **When** data exists, **Then** team, members count, owned components, on-call, updated are shown; Create team when permitted; team detail shows members, owned systems, default escalation.
3. **Given** Invitations, **When** pending/expired invites exist, **Then** email, role, team, status, expires are listed; Invite people, Resend, and Revoke work when permitted.
4. **Given** Roles and permissions, **When** opened from Policies, **Then** a matrix of permissions vs roles (including Agent) is visible; Create role / edit custom roles when permitted.
5. **Given** Audit event detail, **When** opened, **Then** actor, action, target, time, source, request id, field-level changes, and integrity/hash verification signals are shown; export when permitted.
6. **Given** Agent session detail, **When** opened, **Then** objective, step activity, requested tools with allow/deny, open linked approval, Terminate and Export when permitted.

---

### User Story 8 - Integration setup wizard, webhooks, and conflict resolution entry (Priority: P2)

As an operator in Integrations, I complete a **multi-step setup wizard** for a provider (connection method, credentials/role, permissions, test connection), manage **Webhooks** (create, deliveries, test, edit), and resolve **sync conflicts** from Integrations Sync or the Synchronization center.

**Why this priority**: Connecting providers and outbound events is core configuration work still incomplete after Connected/Catalog/Credentials/Sync lists in 004.

**Independent Test**: Start Connect AWS (or equivalent) wizard through test connection; open Webhooks tab; create or inspect a webhook; open a conflict resolve screen.

**Acceptance Scenarios**:

1. **Given** Add integration for a known provider, **When** the wizard runs, **Then** steps cover connection method, identity material, requested permissions, and Test connection before finish; secrets are write-only after set.
2. **Given** Webhooks tab, **When** webhooks exist, **Then** destination, events, status, last call are listed; selected detail shows signing secret state, retry policy, last response; Send test, View deliveries, Edit when permitted.
3. **Given** a failed webhook, **When** inspected, **Then** failure is visible without exposing raw secrets.
4. **Given** Sync conflict entry points, **When** used, **Then** they open the same conflict resolution experience as the Synchronization center (User Story 2).

---

### User Story 9 - Settings expansions: organization, developer, billing, security, data (Priority: P3)

As an organization or project administrator, Settings includes **Organization**, **Developer** (CLI, MCP, API tokens, local endpoints), **Billing** (plan, usage, invoices), **Security** (MFA, sessions, org security policies), and **Data** (retention, export, danger-zone deletes)—in addition to Project, Environments, Notifications, and Core from prior specs.

**Why this priority**: Completes configuration breadth without promoting billing/security/data to sidebar destinations.

**Independent Test**: Visit each new Settings tab; edit org name; copy MCP config; view usage meters; revoke a session; change a retention value; confirm danger-zone requires confirmation.

**Acceptance Scenarios**:

1. **Given** Settings, **When** navigation is inspected, **Then** Project, Organization, Environments, Notifications, Core, Developer, Billing, Security, and Data (or Security→Data sub-navigation as specified) are reachable without new sidebar items.
2. **Given** Organization, **When** edited, **Then** name, slug, logo, default role, domain verification status, and domain-join provisioning toggle can be managed when permitted.
3. **Given** Developer, **When** opened, **Then** CLI install/status, MCP status/endpoint/tools, API tokens, and local endpoints are honest; copy install/config; manage tools; restart MCP when applicable—without claiming live status for unimplemented runtimes.
4. **Given** Billing, **When** a plan is active, **Then** plan name, price cadence, renewal, usage meters vs limits, estimated invoice, invoices list, and update payment method entry points appear for authorized roles.
5. **Given** Security, **When** opened, **Then** MFA status, active sessions with revoke (except current), and org policies (require MFA for admins, require approval for production, IP restrict) are available when permitted.
6. **Given** Data and retention, **When** opened, **Then** retention for audit, operation output, logs (local/cloud), architecture snapshots, export project data, and confirmed destructive deletes of retained data are available when permitted.

---

### User Story 10 - First-run onboarding (Priority: P3)

As a new user without a project, I complete **onboarding**: create or join organization, create first project (environments + local-first vs cloud storage), then continue toward connect infrastructure and build architecture.

**Why this priority**: Without onboarding, multi-entity surfaces (orgs, projects) have no entry path for greenfield users.

**Independent Test**: Open product with no project; complete Create organization → Create project; land in Overview or Architecture with empty-but-guided state.

**Acceptance Scenarios**:

1. **Given** no organization/project, **When** the app opens, **Then** onboarding presents create organization or join with invite and states that data can remain local initially.
2. **Given** project creation step, **When** filled, **Then** name, description, initial environments, and storage choice (local first vs Polter Cloud) are collected before create.
3. **Given** successful project creation, **When** onboarding continues, **Then** the operator can proceed toward connect infrastructure and architecture without re-entering the empty onboarding dead-end.

---

### Edge Cases

- Operator opens Observability with no observability integration connected: show empty/setup guidance and catalog hand-off, not fake live metrics.
- Local core offline: Overview, Sync, and mutation surfaces remain honest; cached/read-only content labeled; Sync now and provision/apply blocked or queued per product policy with clear messaging.
- Notification deep link targets a deleted entity: show not-found with path back to list/center.
- Search with zero results: empty state with refine-query guidance; category chips show zero counts without hiding the query.
- Provisioning approval required but operator cannot self-approve: request enters Approvals; UI does not bypass.
- Drift “Restore desired” fails at provider: show failure with correlation, leave conflict/drift unresolved.
- Sync conflict while offline: resolution may be queued; UI must not claim provider updated until confirmed.
- Agent session terminates mid-tool-call: session shows terminated; tools no longer accepted; audit records termination.
- Invitation expired: resend creates a new valid invite; expired cannot be accepted.
- Billing unavailable (local-only edition / no cloud account): Billing tab explains unavailability rather than fabricating plan data.
- Onboarding abandoned mid-flow: resume or restart without partial orphan projects unless explicitly created.
- Webhook signing secret: viewable once at creation only; later only rotate/replace.
- Reduced-motion / keyboard: drawers, popovers, wizards remain fully operable without color-only status.
- Permission-denied on Settings tabs or Provisioning: consistent denied state, no partial secret leakage.

## Requirements *(mandatory)*

### Functional Requirements

#### Shell and information architecture

- **FR-001**: Product MUST present exactly **eight** primary sidebar destinations: Architecture, Resources, Operations, Observability, Knowledge, Access, Integrations, Settings.
- **FR-002**: Sidebar MUST group destinations as **CORE** (Architecture, Resources, Operations), **INSIGHTS** (Observability, Knowledge, Access), and **CONFIGURATION** (Integrations, Settings) in that order.
- **FR-003**: Secondary capabilities listed in this feature (notifications, search results, sync center, overview, switcher, provisioning subflows, wizards, settings sub-pages, etc.) MUST NOT appear as permanent extra sidebar destinations.
- **FR-004**: Clicking the product/project identity in the sidebar header MUST navigate to Project Overview for the current project.
- **FR-005**: Project switcher MUST open as a large popover (or equivalent non-page overlay) supporting search, organization grouping, current-project indicator, switch, create project, and manage organizations.
- **FR-006**: Global chrome MUST provide entry points for search (including palette), notifications, and synchronization center (footer core status is a valid sync entry).
- **FR-007**: In-page tabs remain the subdivision grammar for primary destinations; this feature MUST NOT reintroduce permanent nested sidebar trees.
- **FR-008**: Requirements from `004-operator-product-surfaces` remain in force except where this feature explicitly supersedes destination count/group labels or adds tabs/surfaces.

#### Project Overview

- **FR-009**: Project Overview MUST show system status summary, attention-required items, recent activity feed, and quick actions into Architecture, Resources, Operations, and Integrations as available.
- **FR-010**: Overview status and attention items MUST reflect honest live/cached/unavailable state and MUST NOT invent healthy services when data is missing.

#### Notifications

- **FR-011**: Notification center MUST open as a temporary drawer from global chrome with filters (at least All, Unread, Approvals), per-item primary actions, dismiss/close, and mark-all-as-read.
- **FR-012**: Notification actions MUST deep-link to the relevant entity or flow when the target still exists.

#### Global search results

- **FR-013**: Full search results page MUST accept a query parameter or equivalent, allow refining the query, show category chips with counts, group or filter hits by category, and open entities on activation.
- **FR-014**: Search MUST cover at least Architecture, Resources, Operations, and Knowledge entities available to the operator; additional categories MAY be included when data exists.

#### Synchronization center

- **FR-015**: Synchronization center MUST expose Status, Offline queue, Conflicts, and History (as tabs or equivalent).
- **FR-016**: Status MUST list local core, Polter Cloud when configured, and connected providers with health and last sync/heartbeat.
- **FR-017**: Operators MUST be able to trigger Sync now and open sync-related settings when permitted.
- **FR-018**: Conflict resolution MUST present Polter state vs provider state, require an explicit resolution choice, write an audit record, and avoid silent overwrite.

#### Observability

- **FR-019**: Observability MUST be a primary destination with tabs Overview, Logs, Metrics, Traces, Monitors.
- **FR-020**: Overview MUST summarize overall health, service health rows, and external dashboard hand-offs when integrations exist.
- **FR-021**: Logs MUST support search, service/level/time filters, live pause/resume, export/download when permitted, event detail with structured fields, and links to resource/trace when present.
- **FR-022**: Metrics MUST support service and time-range selection and key metric charts with external full-dashboard hand-off.
- **FR-023**: Traces MUST list traces with status filters and open waterfall detail with span selection and related resource/logs links.
- **FR-024**: Monitors MUST list monitors with status and alert counts; detail MUST show condition, current value, related context (resource, traces, logs, runbook), and mute/edit/resolve when permitted.
- **FR-025**: Observability MUST favor hub + hand-off to specialized tools over cloning full APM/log/metrics platforms.

#### Resources: Provisioning and nodes

- **FR-026**: Resources MUST add a Provisioning tab with Catalog, Executions, and Drift subdivisions.
- **FR-027**: Catalog MUST group provisionable templates and start a multi-step configuration → plan review → approve/apply flow.
- **FR-028**: Plan review MUST show resources to create, policy checks, and approval requirements before apply.
- **FR-029**: Provisioning executions MUST show step progress, output, cancel when allowed, and completion paths that import resources into Polter.
- **FR-030**: Drift MUST list desired-vs-actual differences with severity and support Accept actual, Restore desired, and Ignore when permitted, with audit.
- **FR-031**: Node registration wizard MUST support target types (local, remote, container, Kubernetes), expiring install token, copy/regenerate, and heartbeat waiting state.
- **FR-032**: Provisioning and node registration mutations MUST honor plan → review → approve → apply → logs → audit when governance requires it.

#### Architecture deep flows

- **FR-033**: Architecture MUST support Import resources mapping (link existing, create-and-link, ignore) with suggested dependencies and apply under permissions.
- **FR-034**: Architecture MUST support snapshot comparison with summary of added/removed/changed components and optional restore when permitted.
- **FR-035**: Architecture view editor MUST allow name, description, included components, filters, and layout preference with save/cancel.

#### Operations: workflows

- **FR-036**: Workflow builder MUST allow ordered steps, per-step configuration (including approval steps), test, and save with version visibility.
- **FR-037**: Workflow Schedules MUST list schedules with next run/status/timezone and support create, edit, pause, and run now when permitted.

#### Access expansions

- **FR-038**: Access MUST expose tabs Members, Teams, Agents, Policies, Audit, Invitations (extending 004’s Access tabs).
- **FR-039**: Teams MUST support list, create, and detail (members, owned systems, default escalation).
- **FR-040**: Invitations MUST support list, invite, resend, and revoke with expiry status.
- **FR-041**: Roles and permission matrix MUST be reachable from Policies and show permission-by-role including Agent role semantics.
- **FR-042**: Audit event detail MUST show actor, action, target, time, source, request id, changes, and integrity verification signals when available.
- **FR-043**: Agent session detail MUST show objective, activity steps, tool allow/deny, linked approval when present, terminate, and export when permitted.

#### Integrations expansions

- **FR-044**: Integration setup wizard MUST guide connection method, identity material, requested permissions, and test connection before activation.
- **FR-045**: Integrations MUST expose a Webhooks tab (or equivalent) for list, create, test, deliveries, and edit; signing secrets MUST NOT remain readable after creation except via rotate.
- **FR-046**: Sync conflict resolution from Integrations MUST reuse the same resolution semantics as the Synchronization center.

#### Settings expansions

- **FR-047**: Settings MUST include Organization, Developer, Billing, Security, and Data/retention surfaces in addition to Project, Environments, Notifications, and Core.
- **FR-048**: Organization settings MUST manage identity, default role, domain verification display, and domain join policy when permitted.
- **FR-049**: Developer settings MUST honestly report CLI and MCP status, expose copyable configuration, API tokens management, and local endpoints without presenting unimplemented runtimes as live.
- **FR-050**: Billing MUST show plan, usage vs limits, invoices, and payment method management for authorized roles when billing applies to the edition; otherwise show a clear unavailable state.
- **FR-051**: Security MUST manage MFA configuration entry, session list with revoke, and organization security policies when permitted.
- **FR-052**: Data settings MUST manage retention policies, project export, and confirmed destructive deletion of retained data when permitted.

#### Onboarding

- **FR-053**: First-run experience MUST support create or join organization and create first project with environments and storage preference (local first vs cloud).
- **FR-054**: Onboarding MUST allow local-first operation and MUST NOT force cloud signup as the only path.

#### Cross-cutting quality

- **FR-055**: All new surfaces MUST provide empty, loading, error, offline/cached, and permission-denied presentations consistent with the design system.
- **FR-056**: Status MUST NOT rely on color alone.
- **FR-057**: Keyboard operation with visible focus MUST work for shell, tabs, tables, drawers, popovers, wizards, and primary actions.
- **FR-058**: Long lists (logs, search hits, audit, traces, notifications) MUST remain usable at high row counts.
- **FR-059**: UI MUST remain honest about live vs fixture/cached data.
- **FR-060**: Product MUST NOT expand into incomplete clones of specialized platforms; deep-link and hand-off remain the default for advanced external work.
- **FR-061**: Secret values (credentials, webhook secrets, install tokens after expiry window, API tokens after creation) MUST follow write-once / rotate patterns and MUST NOT be re-displayed in full after establishment except where a short-lived install token is intentionally shown during registration.
- **FR-062**: Notification center drawer is an additional justified temporary overlay for ephemeral inbox items; it MUST NOT become a permanent inspector pattern for domain entities (domain detail remains full pages per 004, except the existing process drawer).

### Key Entities *(include if feature involves data)*

- **Project Overview Snapshot**: Aggregated status, attention items, and recent activity for the current System (UI: Project).
- **Notification**: In-app inbox item with category, read state, target reference, and primary action.
- **Search Hit**: Ranked reference to an entity across product domains with type and summary fields.
- **Sync Endpoint**: Local core, cloud, or provider connection with health and last successful exchange.
- **Offline Queue Item**: Local change waiting to upload or apply when connectivity allows.
- **Sync Conflict**: Divergent Polter vs external state for a resource or configuration field, with resolution choice and audit.
- **Observability Service Health**: Per-service health, latency, and error summary for the hub Overview.
- **Log Event**: Structured operational log line with service, level, message, time, and optional trace/resource links.
- **Trace / Span**: Distributed request path with duration, status, and hierarchical spans.
- **Monitor / Alert**: Threshold or condition against a target with status, mute, and related context links.
- **Provisioning Template**: Catalog entry describing a provisionable resource type and configuration schema.
- **Provisioning Plan / Execution**: Planned creates and step-wise apply progress with policy checks and outputs.
- **Infrastructure Drift**: Desired vs actual difference for a managed resource with severity and resolution.
- **Node Install Token**: Short-lived registration secret for attaching a Polter Node / Target.
- **Architecture Snapshot**: Point-in-time model used for comparison and optional restore.
- **Architecture View**: Named filtered subset of components with layout preference.
- **Workflow Definition / Step / Schedule**: Ordered automation with triggers, step config, and time-based schedules.
- **Team**: Group of members/agents with owned systems and escalation defaults.
- **Invitation**: Pending or expired access grant with role, team, and expiry.
- **Role Permission**: Matrix cell of capability vs role (including Agent).
- **Audit Event Detail**: Single immutable audit record with integrity signals.
- **Agent Session**: Time-bounded agent work unit with objective, tool grants, and termination state.
- **Integration Connection**: Provider link with method, permissions, and health.
- **Webhook**: Outbound event subscription with destination, signing, retry, and delivery history.
- **Organization**: Collaboration boundary above projects with domain and default role policy.
- **Billing Plan / Usage Meter / Invoice**: Commercial entitlement and consumption visibility when edition supports it.
- **Security Session**: Authenticated client session subject to revoke.
- **Retention Policy**: Configurable keep windows for audit, operations output, logs, and snapshots.
- **Onboarding State**: First-run progress through org/project creation.

### Constitution Constraints *(mandatory)*

Map the feature to `.specify/memory/constitution.md` (v2.x). Analysis MUST block
implementation on critical unresolved conflicts.

- **Operational architecture goal (I)**: Observe (Observability, Overview, notifications), discover-organize (import, search, provisioning catalog), safe execution (provisioning, workflows, approvals), collaboration (teams, invitations, orgs), audit (audit detail, sync resolution, agent sessions), external tools (dashboard/provider hand-off).
- **Runtime owner (III)**: Renderer (UI surfaces); polter-core (authority for inventory, ops, sync, audit, secrets references); CLI/MCP surfaced honestly via Developer settings; no unrestricted renderer OS access.
- **Domain vs adapter (II, IV)**: Domains for system, resource, operation, integration, identity/access, observability aggregation, billing entitlement; adapters for cloud providers, log/metrics backends, payment provider—domain MUST NOT import concrete provider SDKs.
- **Contracts (V)**: Shared commands/queries/events for notifications, search, sync, observability summaries, provisioning plans/executions, teams/invites, webhooks, org/billing/settings; versioned if breaking relative to existing local API.
- **Product loop impact**: Overview + Observability strengthen Observe; Provisioning + workflows strengthen Operate/Approve; Sync/conflicts strengthen Evidence; Access/audit strengthen Audit; Integrations hand-off preserves external specialized tools.
- **Evidence expectations**: Sync health, drift desired-vs-actual, monitor conditions, audit integrity hashes, provisioning step outputs prove state; UI must label cached vs live.
- **Mutation safety (VII)**: Provisioning, drift restore, workflow save/run, webhook changes, org/security/billing changes, agent terminate, invite revoke follow plan→review→approve→apply→logs→audit when policy requires; read paths (logs/metrics browse) remain read-only.
- **Operation audit fields**: Actor, target, scope, environment, policy, approval, correlation/operation ID on provisioning, conflict resolution, role changes, webhook edits, retention deletes, session revoke.
- **Security boundary (VI)**: Secrets as references; webhook secrets and API tokens not re-displayed; MCP cannot bypass policy; install tokens short-lived; billing/payment method behind authorized roles.
- **Local vs cloud authority (VIII)**: Local-first onboarding and local core remain valid; cloud sync optional; offline queue and conflict UX required; billing may be unavailable offline/local-only edition.
- **Performance (IX)**: Virtualize long logs/search/audit/trace lists; no Main/UI blocking on sync or live log tails; backpressure on live streams (pause control).
- **Resilience (X)**: Timeouts on provider test connection and provision steps; retries visible on webhooks; offline queue; recoverable onboarding.
- **Observability (XI)**: Correlation IDs on failed sync, provision steps, webhook deliveries, and agent sessions; surface request/operation ids in audit detail.
- **Test plan (XII)**: Unit for reducers/mappers of overview/search/sync conflict resolution; contract tests for new APIs; integration for provisioning plan policy gates; e2e smoke for shell IA (8 destinations), overview entry, observability tabs, provision happy path (fixture), onboarding; security tests for secret non-echo and permission-denied settings.
- **Structure created now (XIII)**: Prefer extending existing feature packages (architecture, resources, operations, access, integrations, settings, shell) and shared frontend packages; new packages only with real API + consumer.
- **Single source of truth (XIV)**: No permanent dual navigation models (seven vs eight); once this feature lands, eight-destination INSIGHTS map is authoritative; remove temporary dual nav flags before done.
- **Big-bang / refactor context (XV)**: Extends operator surfaces on the active refactor line; not a second product UI stack.
- **Non-goals (I, XIII)**: Not a full Grafana/Datadog/AWS console; not a full billing engine UI beyond plan/usage/invoices/payment entry; not CRM; not infinite sidebar items for every capability.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In a navigation audit, 100% of reviewers identify exactly eight sidebar destinations in CORE / INSIGHTS / CONFIGURATION, and zero permanent sidebar items for notifications, search, sync, provisioning, billing, or webhooks.
- **SC-002**: Operators can open Project Overview from the product identity control and identify at least one system-status and one attention or activity signal (or a clear empty/offline state) in under 10 seconds in a populated demo project.
- **SC-003**: From an unread approval notification, operators reach the approval review surface in one primary action in under 5 seconds in moderated usability tests.
- **SC-004**: Global search “show all results” for a known entity name returns that entity in the correct category on first try for at least 95% of seeded demo queries.
- **SC-005**: Operators can open Synchronization center from footer status and correctly state local-core connectivity (online/offline/degraded) in 100% of offline-simulation checks.
- **SC-006**: Observability Overview, Logs, Metrics, Traces, and Monitors are each reachable in one click from the Observability destination; hand-off controls to external dashboards are present whenever a matching integration is connected in demo data.
- **SC-007**: A complete provisioning happy path (catalog → configure → plan review → request approval or apply per policy → execution visible) is completable without leaving Resources/Approvals surfaces in under 10 minutes in a guided demo with fixtures.
- **SC-008**: Drift and sync conflict resolution each require an explicit choice and produce an audit-visible outcome in 100% of scripted resolution tests.
- **SC-009**: Access administrators can invite a user, create a team, and open an agent session detail without using any new sidebar destination beyond Access.
- **SC-010**: New Settings surfaces (Organization, Developer, Billing, Security, Data) are reachable from Settings only; 0 of them appear in the primary sidebar.
- **SC-011**: First-run onboarding creates an organization and project with local-first storage without requiring cloud payment details.
- **SC-012**: In honesty review, 0 surfaces claim live observability, billing, or MCP/CLI status when the underlying capability is unimplemented or disconnected; empty/unavailable states are shown instead.
- **SC-013**: Keyboard-only operators can open notification drawer, project switcher, and one multi-step wizard (provisioning or integration or node registration) end-to-end without pointer use in accessibility smoke tests.

## Assumptions

- Builds on shell, design system, and primary destination surfaces from `003` and `004`; does not re-specify canvas layout engine, process drawer exclusivity for processes, or Knowledge Docs/Decisions/Runbooks/Ownership content already covered.
- “Project” in UI remains the alias for domain **System** where product copy uses Project Overview / project switcher.
- Observability data may be aggregated from connected providers and/or local core; when no source exists, empty/setup states are shown.
- Estimated provisioning cost is best-effort from provider catalogs and may be approximate or unavailable.
- Billing applies when the edition and account support Polter commercial plans; local-only usage may hide or disable Billing with explanation.
- Notification delivery channels (email/Slack) preferences remain under Settings → Notifications from 004; this feature adds the in-app notification center.
- Command palette remains the primary quick-search entry; full results page is the expanded view.
- Webhooks are outbound from Polter (or core) to customer endpoints; inbound provider webhooks may appear later and are out of scope unless already implied by a specific integration adapter.
- Agent “Request only” style permissions in the roles matrix align with existing approval-gated operations policy.
- Snapshot compare and import mapping operate on architecture model data already available to the desktop/local authority; large imports may be asynchronous with progress.
- Node install token lifetime on the order of minutes (e.g. ~15 minutes) is acceptable; exact duration is an implementation detail within a short-lived window.
- Out of scope for this feature: full multi-region billing tax compliance UI, full SIEM product, full Terraform/K8s IDE, mobile-native apps, and permanent dual seven/eight destination modes after release.
