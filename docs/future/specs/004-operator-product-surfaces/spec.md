# Feature Specification: Operator Product Surfaces

**Feature Branch**: `004-operator-product-surfaces`

**Created**: 2026-08-03

**Status**: Draft

**Input**: User description: "Build all operator product interfaces following the current design system: seven sidebar destinations (Architecture, Resources, Operations, Knowledge, Access, Integrations, Settings) with in-page tabs for subdivisions, detail pages, focused dialogs, one justified process drawer, command palette, and shared empty/loading/offline/permission states."

**Depends on / extends**: `003-operator-design-system` (shell anatomy, tokens, Resources/Architecture foundation, no permanent inspectors). This feature completes the full product information architecture and remaining surfaces under that visual language.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Seven-destination shell with in-page tabs (Priority: P1)

As an operator, I open Polter and see one consistent shell: title bar with history and profile, integrated sidebar with three labeled groups (**CORE**, **KNOWLEDGE**, **CONFIGURATION**) and exactly **seven** primary destinations, a single main surface for page content, core/node status near the sidebar footer, and identity (name + role). Subdivisions of each destination appear as **tabs inside the page**, never as extra sidebar items.

**Why this priority**: Navigation is the spine of the product. Without a stable seven-destination map and tab grammar, every surface reintroduces inspector sprawl and sidebar bloat.

**Independent Test**: Launch the product shell, visit each of the seven destinations, confirm group labels and order, confirm tabs appear inside the main surface (not in the sidebar), confirm back/forward history and shell status/identity remain present on every page.

**Acceptance Scenarios**:

1. **Given** the application window is open, **When** the operator views the sidebar, **Then** they see groups CORE (Architecture, Resources, Operations), KNOWLEDGE (Knowledge, Access), and CONFIGURATION (Integrations, Settings)—exactly seven destinations, no deeper nested nav items.
2. **Given** any primary destination is open, **When** the operator inspects subdivision controls, **Then** subdivisions are page tabs (or equivalent in-page tab strip), not additional sidebar entries.
3. **Given** the operator navigates between destinations and detail pages, **When** they use history back/forward, **Then** navigation history moves as expected within the product shell.
4. **Given** any primary page, **When** the operator views shell chrome, **Then** they see core connectivity status, connected-node summary when available, and current user identity/role without leaving the shell.
5. **Given** a destination is active, **When** selection styling is inspected, **Then** active nav uses neutral raised treatment consistent with the established design system—not brand-green fill.

---

### User Story 2 - Architecture: canvas, lists, views, changes, and node detail (Priority: P1)

As an operator, I work Architecture as a multi-tab surface: **Canvas**, **Components**, **Dependencies**, **Views**, and **Changes**. I pan/zoom the map, select a node with quiet neutral emphasis, press Enter (or equivalent open action) to open **Architecture node detail** as a full page, manage saved views, and review a change timeline. I can create nodes via a focused dialog.

**Why this priority**: Architecture is the structural entry point of the control plane; tabs complete the map beyond a single canvas screen.

**Independent Test**: Open Architecture → each tab; select a canvas node (neutral border only); open node detail; return; open Components/Dependencies tables; open a saved view; browse Changes; open Create node dialog.

**Acceptance Scenarios**:

1. **Given** Architecture is selected, **When** the page loads, **Then** the operator sees environment context, search/filter or view controls as applicable, and tabs: Canvas, Components, Dependencies, Views, Changes.
2. **Given** Canvas is active, **When** the map loads with data, **Then** layered/grouped structure, relationship edges, view switcher, save-layout affordance (when layout is dirty/editable), and zoom/fit/lock controls are available.
3. **Given** a node is selected on the canvas, **When** selection is shown, **Then** emphasis is a quiet neutral border/surface—not brand glow—and Enter (or explicit open) navigates to Architecture node detail as a full page.
4. **Given** Components tab, **When** data is present, **Then** a dense table lists components with name, type, owner, environment, status, and recency, with search/filter/create entry points.
5. **Given** Dependencies tab, **When** data is present, **Then** source, relationship, target, protocol, and status columns appear; operators can filter direction, reveal external dependencies, and export when permitted.
6. **Given** Views tab, **When** saved views exist, **Then** cards show name, description, node count, and recency with Open; Create view is available when permitted.
7. **Given** Changes tab, **When** history exists, **Then** a chronological list groups events (for example Today/Yesterday) with actor and summary; compare-snapshots is available when the product supports it.
8. **Given** Architecture node detail, **When** the page opens, **Then** the operator sees identity, status chips, overview facts, relationship mini-view, connected architecture links, quick actions (open resource, logs, run, etc. as available), operational notes, and activity—without a permanent side inspector on the canvas.

---

### User Story 3 - Resources: inventory, nodes, processes, detail pages, process drawer (Priority: P1)

As an operator, I manage Resources through tabs **Inventory**, **Nodes**, and **Processes**. Inventory and Nodes open full detail pages; Processes open a **temporary drawer** only (ephemeral runtime data), which is the sole justified drawer in this product map.

**Why this priority**: Resources are the live inventory and target fabric; completing Nodes/Processes and the process drawer finishes the Resources surface begun in the design-system feature.

**Independent Test**: Browse Inventory with category chips; open resource detail; browse Nodes; open node detail; browse Processes live table; open process drawer; use pause updates; dismiss drawer cleanly.

**Acceptance Scenarios**:

1. **Given** Resources is selected, **When** the page loads, **Then** tabs Inventory, Nodes, and Processes appear with environment context and search/filter as applicable.
2. **Given** Inventory, **When** data loads, **Then** a dense table shows name, type, environment, provider, status, last sync; category filters (All, Cloud, Local, Databases, Queues, etc.) scope the list; row activation opens Resource detail as a full page.
3. **Given** Resource detail, **When** reviewed, **Then** overview, linked architecture, notes/metadata, activity, and quick actions (provider hand-off, logs, health) are present without permanent inspectors.
4. **Given** Nodes tab, **When** data loads, **Then** nodes list location, OS, version, status, last heartbeat with online/offline summary; Add node is available when permitted; row activation opens Node detail.
5. **Given** Node detail, **When** reviewed, **Then** overview (host facts), capabilities, assigned resources, recent operations, and actions (terminal, run command) appear as available and permissioned.
6. **Given** Processes tab, **When** live data updates, **Then** process, PID, node, CPU, memory, port, status are shown with recent update time and pause/resume updates control.
7. **Given** a process row is activated, **When** detail opens, **Then** it opens as a temporary drawer (not a full page and not a permanent inspector) with stop/restart/logs/terminal actions as permitted, and closes without leaving residual layout chrome.

---

### User Story 4 - Operations: runs, workflows, approvals, and terminal (Priority: P1)

As an operator, I execute and govern work under Operations tabs **Runs**, **Workflows**, **Approvals**, and **Terminal**. I inspect run progress and logs, define/run workflows, approve or reject pending requests with impact and policy context, and use multi-session terminal work against nodes—always under permission and approval rules.

**Why this priority**: Safe execution is the product’s mutation spine (plan → review → approve → apply → logs → audit). Operations is where that loop becomes visible.

**Independent Test**: List runs with status filters; open a running run detail (steps + output); list workflows and open one; open Approvals pending list and an approval detail (approve/reject UI); open Terminal with session tabs.

**Acceptance Scenarios**:

1. **Given** Operations is selected, **When** the page loads, **Then** tabs Runs, Workflows, Approvals, and Terminal are available with environment context.
2. **Given** Runs tab, **When** data loads, **Then** operation, target, actor, status, started time appear; filters (All/Running/Pending/Failed/Completed) work; Run command entry exists when permitted; row opens Run detail.
3. **Given** Run detail, **When** a multi-step run is open, **Then** summary, context, step list with states, streaming or appended output, and actions (cancel/retry as applicable) are shown.
4. **Given** Workflows tab, **When** data loads, **Then** workflow, trigger, last run, status, and run/open actions appear with trigger filters; Create workflow when permitted; detail shows step graph and recent runs.
5. **Given** Approvals tab, **When** pending items exist, **Then** request, requester, target, risk, wait time appear with status sub-filters; detail shows impact analysis, policy checks, comments, and Approve/Reject for authorized actors.
6. **Given** Terminal tab, **When** the operator opens sessions, **Then** multiple named sessions (node + shell) can exist; connection status and clear/close controls are available; terminal is an Operations surface, not a primary sidebar destination.
7. **Given** an action requires approval or permission the operator lacks, **When** they attempt it, **Then** the product blocks execution and surfaces a clear permission or approval path—not a silent failure.

---

### User Story 5 - Knowledge: docs, decisions, runbooks, ownership (Priority: P2)

As an operator, I treat Knowledge as the operational memory: **Docs**, **Decisions**, **Runbooks**, and **Ownership**, with document detail as a full page and create/edit via focused flows.

**Why this priority**: Knowledge links architecture and operations into durable context; it is essential but secondary to inventory and execution for MVP sequencing.

**Independent Test**: Browse each Knowledge tab; open a document detail; open decision/runbook lists; review ownership matrix; create entry points visible when permitted.

**Acceptance Scenarios**:

1. **Given** Knowledge is selected, **When** the page loads, **Then** tabs Docs, Decisions, Runbooks, Ownership are present.
2. **Given** Docs, **When** content exists, **Then** recently updated documents list linked entities, owner, and recency; categories filter; document detail shows body content, linked architecture/knowledge, and edit/share when permitted.
3. **Given** Decisions, **When** ADRs exist, **Then** ID, title, status, owner, date appear with status filters.
4. **Given** Runbooks, **When** runbooks exist, **Then** service linkage, last tested, owner, and health-of-coverage summary appear.
5. **Given** Ownership, **When** matrix loads, **Then** component, primary/secondary owners, and escalation contacts appear with unowned count highlighted when non-zero.

---

### User Story 6 - Access: members, agents, policies, audit (Priority: P2)

As an admin or authorized operator, I manage **Members**, **Agents**, **Policies**, and **Audit** under Access—who and what may act, under which rules, with an exportable audit trail.

**Why this priority**: Governance and human–agent collaboration require first-class access surfaces; depth can follow Operations core.

**Independent Test**: List members and open member detail; list agents and open agent detail (capabilities); list policies and open policy detail; browse/filter audit and export when permitted.

**Acceptance Scenarios**:

1. **Given** Access is selected, **When** the page loads, **Then** tabs Members, Agents, Policies, Audit appear.
2. **Given** Members, **When** data loads, **Then** member, role, teams, status, last seen appear; Invite member when permitted; member detail shows permissions and recent activity with edit/remove for authorized actors.
3. **Given** Agents, **When** data loads, **Then** agent identity, type, scope, status, last activity appear; agent detail lists allowed/denied capabilities and recent activity; disable/rotate token when permitted.
4. **Given** Policies, **When** data loads, **Then** policy, scope, enforcement, status, updated appear; policy detail shows applies-to, conditions, requirements, exceptions.
5. **Given** Audit, **When** events load, **Then** time, actor, action, target, result appear with filters (actor/action/result/time) and export when permitted.

---

### User Story 7 - Integrations: connected, catalog, credentials, sync (Priority: P2)

As an operator, I connect external systems under Integrations tabs **Connected**, **Catalog**, **Credentials**, and **Sync**, open integration detail for imported counts and permissions, and never see secret values again after creation.

**Why this priority**: Integrations feed Resources and Architecture; credential hygiene is non-negotiable. Full provider consoles remain out of scope.

**Independent Test**: View connected cards; browse catalog categories; open integration detail; list credentials (masked); review sync table and sync-all/configure.

**Acceptance Scenarios**:

1. **Given** Integrations is selected, **When** the page loads, **Then** tabs Connected, Catalog, Credentials, Sync appear.
2. **Given** Connected, **When** providers are linked, **Then** cards show provider, resource counts, connection status, and last sync; Add integration is available when permitted.
3. **Given** Catalog, **When** browsed, **Then** providers are grouped (Cloud, Platforms, Observability, Infrastructure, Messaging, etc.) with search.
4. **Given** Integration detail, **When** opened, **Then** account/scope facts, imported resource counts by type, granted permissions, and actions (sync now, open external console) appear—hand-off preferred over cloning the provider.
5. **Given** Credentials, **When** listed, **Then** name, provider, scope, status, rotated time appear; secret material is never displayed after creation; Add credential uses a focused dialog.
6. **Given** Sync, **When** history loads, **Then** provider, last sync, duration, imported counts, changes, result appear; automatic sync interval is visible and configurable when permitted.

---

### User Story 8 - Settings: project, environments, notifications, core, account (Priority: P2)

As an operator or admin, I configure workspace defaults under Settings tabs **Project**, **Environments**, **Notifications**, **Core**, and **Account**—including local core health, environment protection levels, notification channels, and personal appearance/security.

**Why this priority**: Configuration closes the product map; less frequent than day-to-day operate loops but required for a complete control plane.

**Independent Test**: Edit project fields and save; review environments and protection; toggle notification matrix; view core status and startup options; edit account profile/appearance.

**Acceptance Scenarios**:

1. **Given** Settings is selected, **When** the page loads, **Then** tabs Project, Environments, Notifications, Core, Account appear.
2. **Given** Project, **When** edited, **Then** name, description, default environment, and project ID (copyable) can be managed; destructive delete is isolated in a danger zone with confirmation.
3. **Given** Environments, **When** listed, **Then** resource/node counts, protection level, and default marker appear; production protection rules are visible.
4. **Given** Notifications, **When** configured, **Then** event × channel matrix and quiet hours can be saved.
5. **Given** Core, **When** viewed, **Then** local core status, version, endpoint, data location, restart/logs/data actions, startup toggles, and updates status are honest about connectivity.
6. **Given** Account, **When** edited, **Then** profile fields, theme/density preferences, MFA/session summary, and manage-sessions path are available.

---

### User Story 9 - Global dialogs, command palette, and system states (Priority: P2)

As an operator, I complete short tasks in focused dialogs (create architecture node, run command), jump via command palette, and always understand empty, loading, offline/cached, and permission-denied states with clear recovery actions.

**Why this priority**: Shared interaction grammar prevents each feature inventing incompatible overlays and error patterns.

**Independent Test**: Open create-node and run-command dialogs; open command palette navigation/actions/recent; force or simulate empty, loading, offline, and permission-denied presentations.

**Acceptance Scenarios**:

1. **Given** create architecture node is invoked, **When** the dialog opens, **Then** name, type, environment, owner, description, optional linked resource fields and cancel/create actions appear; validation errors keep the dialog open.
2. **Given** run command is invoked, **When** the dialog opens, **Then** target, command, working directory, environment appear; if approval is required, the primary action requests execution rather than silently applying.
3. **Given** command palette is invoked, **When** opened, **Then** the operator can search navigation, actions, and recent entities without permanently consuming layout.
4. **Given** a list or map has no data, **When** empty state shows, **Then** a short explanation and primary next actions appear (create/import/connect as applicable).
5. **Given** data is loading, **When** skeleton/placeholder shows, **Then** it does not look like a finished empty product.
6. **Given** local core is offline, **When** content depends on core, **Then** offline messaging states that cached data may be shown and offers retry—shell status remains honest.
7. **Given** the operator lacks permission, **When** an action is denied, **Then** a clear permission message names the needed capability and offers request-access when that flow exists.

---

### Edge Cases

- What if the operator has view-only role? Tabs remain navigable for read-allowed data; mutate actions are hidden or disabled with permission messaging—not broken buttons.
- What if Processes list updates while a drawer is open for a dead PID? Drawer shows terminated/not-found state without crashing; list row may disappear on next refresh.
- What if Approvals expire while the detail page is open? Status updates to Expired; Approve/Reject become unavailable with explanation.
- What if Terminal session disconnects mid-command? Session status becomes disconnected; operator can reconnect or close without freezing other sessions.
- What if Integrations sync partially fails? Sync row shows failed/partial result; connected card does not claim full health.
- What if Credentials rotation fails? Status reflects invalid/error; secrets still never re-display.
- What if Architecture graph is very large? Canvas and shell remain interactive for ordinary pan/zoom/select; list tabs remain usable (virtualization allowed).
- What if Resources inventory is very large? Tables remain scrollable/selectable without shell lockup.
- What if environment context is switched? In-scope lists and maps re-scope to the selected environment without losing shell navigation state.
- What if a detail entity was deleted? Not-found state with path back to the parent index/tab.
- What if offline with no cache? Empty/offline composition explains unavailability without inventing fake “live” catalogs.
- What about light theme? Dark quiet operator language remains the product default for these surfaces unless Account theme already supports additional modes without forking a second design system.

## Requirements *(mandatory)*

### Functional Requirements

#### Shell and information architecture

- **FR-001**: Product MUST present exactly seven primary sidebar destinations: Architecture, Resources, Operations, Knowledge, Access, Integrations, Settings.
- **FR-002**: Sidebar MUST group destinations as CORE, KNOWLEDGE, and CONFIGURATION with the order defined in the feature input wireframes.
- **FR-003**: Subdivisions of each destination MUST appear as in-page tabs (or equivalent tab strip inside the main surface), MUST NOT appear as additional permanent sidebar items.
- **FR-004**: All primary destinations MUST render inside the established global shell (title bar with history controls and profile affordance, integrated sidebar, single main surface, core/node status, user identity).
- **FR-005**: Shell MUST surface honest local-core connectivity and connected-node summary when the product has that data.
- **FR-006**: Navigation history (back/forward) MUST work across destinations and detail pages within the shell.
- **FR-007**: Active navigation and selection styling MUST follow the established design system (neutral emphasis; brand accent restricted to small signals).

#### Navigation model and overlays

- **FR-008**: Deep operational context (resource, node, architecture node, run, workflow, approval, document, member, agent, policy, integration) MUST open as full pages unless explicitly specified otherwise.
- **FR-009**: Short create/edit/confirm tasks MUST use focused dialogs (or equivalent focused overlays).
- **FR-010**: The ONLY justified temporary drawer in this product map is the **process detail drawer** for ephemeral process data; other entities MUST NOT reintroduce permanent or ad-hoc side inspectors as their primary detail pattern.
- **FR-011**: Product MUST provide a command palette for navigation, common actions, and recent entities.
- **FR-012**: Transient feedback MUST use toast or inline status; destructive/confirm flows MUST require explicit confirmation.

#### Architecture surfaces

- **FR-013**: Architecture MUST expose tabs: Canvas, Components, Dependencies, Views, Changes.
- **FR-014**: Canvas MUST support pan, zoom, fit, lock (as applicable), view switching, layout save when layouts are editable, search/filter, neutral node selection, and open-to-detail.
- **FR-015**: Components MUST provide a dense searchable/filterable table of architecture components with create entry when permitted.
- **FR-016**: Dependencies MUST list relationships with direction filtering, optional external dependencies, and export when permitted.
- **FR-017**: Views MUST list saved architecture views with open/create flows.
- **FR-018**: Changes MUST show a chronological architecture/resource-related change feed with optional snapshot compare when supported.
- **FR-019**: Architecture node detail MUST include overview, relationship context, connected entities, activity, notes/metadata, and quick actions (including hand-off to Resources/logs/run when available).

#### Resources surfaces

- **FR-020**: Resources MUST expose tabs: Inventory, Nodes, Processes.
- **FR-021**: Inventory MUST support search, category filters, dense table attributes (name, type, environment, provider, status, last sync), and full-page resource detail.
- **FR-022**: Resource detail MUST include overview, linked architecture, activity/notes, and quick actions favoring external provider hand-off over console cloning.
- **FR-023**: Nodes MUST list operational nodes with heartbeat/status summary and full-page node detail (capabilities, assigned resources, recent operations, terminal/run actions when permitted).
- **FR-024**: Processes MUST show a live-updating process table with pause/resume updates and open process detail in a temporary drawer only.
- **FR-025**: Process drawer MUST expose identity, resource usage, command, and permitted actions (logs, terminal, restart, stop).

#### Operations surfaces

- **FR-026**: Operations MUST expose tabs: Runs, Workflows, Approvals, Terminal.
- **FR-027**: Runs MUST list operations with status filters and open multi-step run detail with output.
- **FR-028**: Workflows MUST list definitions with triggers and support run/open/create as permitted; detail MUST show ordered steps and recent runs.
- **FR-029**: Approvals MUST list requests by status with detail including impact, policy checks, comments, and approve/reject for authorized non-self (when policy requires separation of duties).
- **FR-030**: Terminal MUST support multiple concurrent sessions against targets/nodes with clear connection status.
- **FR-031**: Run command and other mutations exposed in UI MUST honor plan → review → approve → apply → logs → audit when governance requires it; UI MUST not bypass policy.

#### Knowledge surfaces

- **FR-032**: Knowledge MUST expose tabs: Docs, Decisions, Runbooks, Ownership.
- **FR-033**: Docs MUST list documents with linkage and open full-page document detail with edit when permitted.
- **FR-034**: Decisions MUST list architecture decisions with status filters.
- **FR-035**: Runbooks MUST list operational runbooks with service linkage and review freshness signals.
- **FR-036**: Ownership MUST present primary/secondary owners and escalation per component with unowned visibility.

#### Access surfaces

- **FR-037**: Access MUST expose tabs: Members, Agents, Policies, Audit.
- **FR-038**: Members and Agents MUST support list + detail with role/capability visibility and invite/register/disable/remove when permitted.
- **FR-039**: Policies MUST support list + detail describing scope, conditions, requirements, and exceptions.
- **FR-040**: Audit MUST provide filterable event history with export when permitted.

#### Integrations surfaces

- **FR-041**: Integrations MUST expose tabs: Connected, Catalog, Credentials, Sync.
- **FR-042**: Connected and Catalog MUST enable discovery and connection of providers without implementing full specialized-platform UIs.
- **FR-043**: Integration detail MUST show sync health, imported counts, and granted permissions; external console open is hand-off only.
- **FR-044**: Credentials MUST store references safely; secret values MUST NOT be displayed after creation.
- **FR-045**: Sync MUST report last sync outcomes and support manual sync / configure automatic interval when permitted.

#### Settings surfaces

- **FR-046**: Settings MUST expose tabs: Project, Environments, Notifications, Core, Account.
- **FR-047**: Project settings MUST allow editing identity fields and isolate destructive delete behind confirmation.
- **FR-048**: Environments MUST list environments with protection posture and default designation.
- **FR-049**: Notifications MUST allow per-event channel preferences and quiet hours.
- **FR-050**: Core settings MUST honestly report local core status and provide restart/logs/data/startup controls as applicable.
- **FR-051**: Account MUST allow profile and appearance preferences and surface security/session management entry points.

#### Shared system states and quality

- **FR-052**: In-scope surfaces MUST provide empty, loading, error, offline/cached, and permission-denied presentations consistent with the wireframes and design system.
- **FR-053**: Status MUST NOT rely on color alone (text and/or icon required).
- **FR-054**: Keyboard operation with visible focus MUST work on shell, tabs, primary tables/lists, dialogs, and key detail actions.
- **FR-055**: Motion MUST respect reduced-motion preferences; decorative motion gimmicks MUST NOT be introduced.
- **FR-056**: Long lists, logs, audit feeds, and process tables MUST remain usable at high row counts (virtualization allowed).
- **FR-057**: UI MUST remain honest about live vs fixture/cached data; unimplemented runtime capabilities MUST NOT be presented as live.
- **FR-058**: Visual language MUST continue the established quiet dark operator design system; this feature MUST NOT invent a second permanent design language.
- **FR-059**: Domain root remains **System** (UI may label Project as alias in Settings/copy); machines/nodes are **Targets**, not owners of Systems.
- **FR-060**: Product MUST NOT expand into incomplete clones of specialized platforms (AWS, Vercel, Grafana, Datadog, Kubernetes consoles, etc.); deep-link and hand-off are the default for advanced external work.

### Key Entities

- **App Shell**: Global chrome—title bar, sidebar groups, main surface, status, identity.
- **Primary Destination**: One of seven sidebar routes.
- **Page Tab**: In-page subdivision of a destination.
- **Main Surface**: Large content host for headers, tabs, and page body.
- **Detail Page**: Full-page deep context for durable entities.
- **Dialog**: Focused overlay for short tasks.
- **Process Drawer**: Temporary overlay exclusively for process inspection.
- **Command Palette**: Global search/run overlay.
- **System**: Domain root (Project alias in UI copy as needed).
- **Environment**: Scope for inventories, architecture, and operations.
- **Architecture Node / Component**: Structural entity on the map and in component lists.
- **Relationship / Dependency**: Edge between components with protocol and status.
- **Architecture View**: Named saved subset/layout of the map.
- **Architecture Change**: Timeline event about structure or related operational shifts.
- **Resource**: Inventory entity (service, database, queue, app, etc.).
- **Target / Node**: Where work runs (self-hosted or managed host).
- **Process**: Ephemeral runtime process on a node.
- **Operation / Run**: Execution instance with steps and output.
- **Workflow**: Reusable multi-step operation definition with trigger.
- **Approval**: Gated request before sensitive apply.
- **Knowledge Document / Decision / Runbook**: Durable operational knowledge artifacts.
- **Ownership Record**: Component-to-team/person responsibility mapping.
- **Member**: Human actor with role and permissions.
- **Agent**: Non-human actor (AI or application) with scoped capabilities.
- **Policy**: Governance rule for who/what may act under which conditions.
- **Audit Event**: Immutable-ish record of who did what to which target with result.
- **Integration**: Connected external provider/account.
- **Credential**: Secret reference for integrations (value never re-shown).
- **Sync Run**: Provider import/sync execution outcome.
- **Notification Preference**: Event × channel matrix + quiet hours.
- **Local Core Status**: Health/version/endpoint of the local authority process.

### Constitution Constraints *(mandatory)*

Map the feature to `.specify/memory/constitution.md` (v2.x). Analysis MUST block
implementation on critical unresolved conflicts.

- **Operational architecture goal (I)**: Strengthens architecture understanding, resource discovery/organization, safe execution, human–agent collaboration, auditability, and external-tool hand-off across the full seven-destination product map.
- **Runtime owner (III)**: Desktop Renderer owns presentation of these surfaces; Main/Preload remain thin (window/chrome/bridge); privileged authority and mutations remain in local core / governed runtimes—not in presentational UI.
- **Domain vs adapter (II, IV)**: UI features are presentation adapters; domain rules for operations, approvals, policies, and inventory remain outside UI packages; no domain→provider SDK imports from feature UI.
- **Contracts (V)**: Prefer existing shared contracts/events/commands; UI MUST NOT invent parallel permanent protocol entities solely for convenience; honest fixtures allowed when core offline.
- **Product loop impact**: Full loop visibility—Map/Resources → Operate → Approve → Verify/logs → Audit; Knowledge and Integrations support link and external context.
- **Evidence expectations**: Dependency edges and “healthy” states MUST NOT claim verified live wiring without honest evidence/sync state; offline/cached labeled.
- **Mutation safety (VII)**: Operations, approvals, credential changes, destructive settings, process stop/restart, and terminal commands follow governed mutation path when policy requires; UI cannot silent-apply high-risk production changes.
- **Operation audit fields**: Actor, target, scope, environment, policy, approval, and correlation/operation identifiers required when operations are executed or approved from these surfaces.
- **Security boundary (VI)**: Renderer unprivileged; secrets as references only; MCP/agents cannot bypass policy; Access surfaces must reflect real authorization, not cosmetic role chips alone.
- **Local vs cloud authority (VIII)**: Local core is primary authority for desktop operator session; Settings/Core and shell status reflect that; cloud integrations are synchronized inputs, not fake local omnipotence.
- **Performance (IX)**: Canvas, live processes, long audit/run logs MUST NOT block shell; virtualization and non-blocking updates expected.
- **Resilience (X)**: Offline/cached, retry, session reconnect, and failed sync handling required on relevant surfaces.
- **Observability (XI)**: Failed operations and syncs SHOULD surface correlation-friendly diagnostics to operators.
- **Test plan (XII)**: Risk-proportional tests for navigation IA, mutation entry points (operations/approvals/credentials), access permission gates, and critical list/detail flows; a11y smoke on shell + one dense table + one dialog.
- **Structure created now (XIII)**: Extend existing desktop app and feature packages; no scenic empty packages; no React product renderer; no permanent dual UI stacks.
- **Single source of truth (XIV)**: One visual system (from 003/DESIGN); one navigation IA; temporary stubs only if honestly labeled and removed before claiming full delivery of a destination.
- **Big-bang / refactor context (XV)**: Continues post big-bang Solid desktop operator product on the active architecture line; completes product surface map beyond the Resources/Architecture foundation.
- **Non-goals (I, XIII)**:
  - Not full AWS/Vercel/Grafana/Datadog/K8s/GitHub Action console clones
  - Not redefining the design token system (consume established design system)
  - Not marketing/Remotion work
  - Not mobile-first redesign
  - Not multiple permanent drawers/inspectors
  - Not claiming multi-provider live sync complete if only fixtures/UI exist—label honesty required

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In a navigation audit, 100% of reviewers identify exactly seven sidebar destinations in the specified three groups, and 0 extra permanent sidebar items for page subdivisions.
- **SC-002**: For each of the seven destinations, operators can open the default tab and switch to every documented tab for that destination without leaving the main surface (checklist pass rate 100% in moderated review).
- **SC-003**: Operators complete Architecture canvas → node detail → return and Resources inventory → resource detail → return each in under 30 seconds on a populated demo catalog.
- **SC-004**: Operators complete Operations runs list → run detail and Approvals list → approve/reject decision UI review in under 45 seconds each on demo data (decision may be cancelled; path must be clear).
- **SC-005**: Process detail opens only as a dismissible temporary drawer; a UI audit finds zero permanent right inspectors on Architecture canvas, Resources inventory, and Operations lists.
- **SC-006**: Empty, loading, offline, and permission-denied presentations are present and distinguishable on at least one representative surface per destination group (CORE, KNOWLEDGE, CONFIGURATION)—checklist 100%.
- **SC-007**: Credential list review shows zero instances of raw secret values after creation (security smoke: 0 secrets leaked in UI).
- **SC-008**: Keyboard-only users can reach primary nav, tab strip, first row/action, and dismiss a dialog on CORE destinations without pointer use (internal a11y smoke task completion 100%).
- **SC-009**: On prototype-scale catalogs (dozens of resources/components, multi-session terminal UI present), ordinary navigation and list scrolling keep the shell interactive (no multi-second full-app freezes in manual smoke).
- **SC-010**: Visual consistency review against the established design system scores ≥ 90% pass on shell anatomy, neutral selection, brand restraint, and density for newly added surfaces.
- **SC-011**: 100% of high-risk mutation entry points visible in Operations/Access/Integrations either route through approval/policy messaging or are hidden/disabled when unauthorized—no silent apply path in UI review.
- **SC-012**: Command palette can jump to all seven primary destinations in under 5 seconds of user intent from any primary page in demo use.

## Assumptions

- Design tokens, shell chrome patterns, button materials, edge-light surfaces, and no-permanent-inspector rules from `003-operator-design-system` / `DESIGN.md` remain authoritative; this feature implements product surfaces on that foundation.
- Desktop is the primary delivery surface; web may later reuse browser-safe pieces but is not required for full parity here.
- Domain data may be fixture-backed or partially live depending on core availability; honesty about source is mandatory.
- “Project” in Settings/UI copy is the transitional alias for domain **System**.
- “Node” in Resources is a **Target** in domain language; UI may say Node for operator familiarity.
- Terminal sessions and process actions require appropriate permissions; demo mode may simulate outcomes if real remote execution is unavailable, but must not fake success as live production apply.
- Notification channels (email/Slack) may show preference UI before full delivery backends exist; toggles that do not yet send MUST be labeled or no-op honestly.
- Integration catalog lists intended providers; not every catalog card must be a fully wired connector in the first delivery increment—unwired items are disabled or “coming soon” honestly rather than fake-connected.
- Command palette shortcuts shown in wireframes are illustrative; final keybindings may follow platform conventions if documented.
- Full multi-user identity provider setup may reuse local/single-operator defaults initially while Access UI is complete.
- `/speckit-plan` will sequence delivery (likely CORE first, then Operations depth, then KNOWLEDGE/CONFIGURATION) without changing this specification’s end-state map.

## Scope Boundaries

### In scope

- Full seven-destination IA and in-page tab model
- Architecture tabs + node detail + create-node dialog
- Resources tabs + resource/node detail + process drawer
- Operations tabs (runs, workflows, approvals, terminal) + run command dialog
- Knowledge, Access, Integrations, Settings tabbed surfaces and primary detail pages
- Command palette
- Shared empty/loading/offline/permission states
- Alignment with established quiet dark operator design system

### Out of scope

- Redefining the visual design system from scratch
- Full specialized-provider administration consoles
- Marketing site, pitch decks, Remotion presentation product work
- Mobile-native layouts
- Multiple permanent inspectors/drawers beyond the process drawer rule
- Claiming complete live multi-cloud automation if only UI/fixtures ship (honesty required)
- React product renderer or resurrection of removed UI stacks
