# Feature Specification: Architecture Big Bang Refactor

**Feature Branch**: `002-architecture-big-bang`

**Created**: 2026-08-02

**Status**: Draft (amended 2026-08-02 — post-analyze normative fix)

**Input**: Big-bang architectural refactor to a modular, local-first, secure system control plane. Spec-only amendments from `/speckit-analyze` Critical/High findings; no code; plan/tasks not modified in this amend.

## Initiative Summary

Reorganize the existing Polterware codebase into a modular, **local-first**, **single-operator**, secure **system control plane**.

Polterware connects architecture, resources, integrations, operations, workflows, terminal, logs, knowledge (light), policies, approvals, and audit. The canvas represents real architecture and is the entry point to observe, create, and operate resources.

Polterware MUST NOT replace specialized platforms; it organizes context, runs common governed operations, and handoffs advanced work externally.

This initiative is an **authorized big-bang architectural refactor** on a dedicated branch: temporary dual paths during implementation are allowed; the finished base MUST run only the approved architecture, with legacy paths removed after replacements are validated.

### Completion set (normative for this feature)

**In scope (MUST deliver):**

- Monorepo orchestration (pnpm, Turborepo, Go workspace)
- Desktop Electron (Main / Preload / **single** SolidJS renderer)
- Tailwind CSS v4 (presentation)
- `polterware-core` (Go) + SQLite local + ConnectRPC local + Protobuf/Buf
- CLI local + MCP local (same domain rules and contracts)
- Operational architecture, resources, operations, local workflows/pipelines, terminal, logs
- Local approvals, policies, audit
- Light knowledge (notes/links/runbook refs on architecture entities)
- Local integrations already in scope (real local ops; vendor handoff/status)
- Preserve **approved product capabilities and journeys**
- Full removal of the previous architecture from active paths

**Out of implementation for this feature (DEFERRED — not MUST, no empty scaffolding):**

- Functional web app
- Functional `polterware-server`
- Functional `polterware-node`
- Cloud multi-tenant organizations, memberships, teams
- Remote collaboration / cloud sync
- PostgreSQL / pgx / sqlc
- NATS / distributed bus
- Billing / cloud entitlements
- Remote node coordination

Future architecture MAY document these; they MUST NOT appear as acceptance requirements for this feature.

---

## Clarifications

### Session 2026-08-02 (original)

Preserved evidence and product decisions that remain valid:

- Desktop today is shell-real + mock workbench; IPC operational mostly UI-only.
- `packages/core` real but unwired; `legacy/tui` unused by desktop.
- Completion runtimes: Desktop + local backend + CLI + MCP (no web/server/node for acceptance).
- Greenfield for listed legacy stores; git-only rollback.
- Local workspace ops real; vendors handoff/status.
- Invalid local data: reject write / isolate corrupt read / explicit reset.
- Humans and agents share policy path; distinct identities.
- User-authored map is canonical; discovery does not silently rewrite intent.
- No Polterware multi-user auth to preserve.

### Session 2026-08-02 (analyze amend — supersedes conflicts)

These decisions **replace** any prior wording that required multi-tenant org, cloud authz, or automatic preservation of all persisted config:

| Topic | Decision |
| --- | --- |
| Delivery | Local-first, single-operator |
| Multi-tenancy | **Not applicable** this feature |
| Cloud organizations / memberships / teams | **DEFERRED — future cloud capability** |
| Web / server / node functional | **Not this feature**; no scenic packages |
| Persistence | **Capabilities preserved; legacy stores DISCARD (greenfield)** unless listed PRESERVE/CONVERT |
| Roles | Owner, Operator, Viewer, Agent (local) |
| Knowledge | Light notes/links/runbooks only — **in scope** |
| Skills preview/setup helpers | **REMOVE** from executable product path |
| Updater | **PRESERVE** (Main; functional when packaged) |
| Renderer | **Single product entrypoint**; atomic cutover React→Solid |
| Local session | Loopback core + ephemeral token + OS runtime session file |

---

## Canonical local terminology

| Term | Meaning in this feature |
| --- | --- |
| **Principal / Actor** | Human or agent identity used for audit and authz |
| **Role** | Owner \| Operator \| Viewer \| Agent |
| **Workspace** | Local work context (install/default operational context) |
| **System** | Domain root: product/system modeled on the canvas (UI MAY still show “Project” as transitional label for System only) |
| **Environment** | Named context: local, development, staging, production, custom |
| **Resource** | Modeled app/infra entity |
| **Target** | Where work runs (local machine/workspace host for completion) |
| **Agent** | Distinct principal type using MCP/tools under capability set |
| **Organization / Membership / Team (cloud)** | **DEFERRED** — not modeled as requirements here |

---

## Roles and capabilities (normative)

Authorization model: **`role defaults + explicit capabilities + policy evaluation + approval requirement`**.

| Role | May | Must not |
| --- | --- | --- |
| **Owner** | Configure workspace; manage local identities/agents; define policies; approve ops; run allowed ops; view audit; configure integrations; change architecture; manage local secret **references** | — |
| **Operator** | View architecture; query resources; run allowed ops; allowed terminal; query logs; request approvals; view allowed results/audit | Change global policies; manage identities; alter secrets without explicit capability |
| **Viewer** | View architecture; query resources; read knowledge; query allowed logs/audit | Execute operations |
| **Agent** | Query authorized resources/knowledge; request ops in capability set; use explicitly allowed MCP tools | Self-approve; change policies; expand own capabilities; access secret values; bypass human confirmation; run out-of-scope ops |

**Tests MUST cover** critical role×capability combinations: Owner approve; Operator request+deny; Viewer exec deny; Agent self-approve deny; Agent out-of-scope deny; CLI/MCP same deny as UI.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Inventory, preserve, and retire the current base (Priority: P1)

As a maintainer, I need every responsibility classified so approved **capabilities** survive and obsolete code/data stores do not remain as second truths.

**Independent Test**: Disposition register complete; DISCARD stores documented; PRESERVE/CONVERT tested if any.

**Acceptance Scenarios**:

1. **Given** inventory, **When** complete, **Then** every active path has disposition + destination.
2. **Given** capability PRESERVE/MIGRATE, **When** done, **Then** it works via approved interface.
3. **Given** REMOVE, **When** done, **Then** unreachable on active paths.
4. **Given** DISCARD store, **When** done, **Then** app does not require legacy format readers.
5. **Given** temporary adapters, **When** done, **Then** none remain.

---

### User Story 2 - Understand a system from the operational canvas (Priority: P1)

As an operator, I open a **System** and see resources, environments, dependencies, and connections. Selecting an entity shows state, metadata, ownership text, knowledge, integrations, ops, logs, audit, external links.

**Acceptance Scenarios**:

1. Canvas shows architecture entities and relationships, not only a static picture.
2. Inspector reaches state/metadata/knowledge/ops/logs/audit when data exists.
3. Advanced work handoffs to external tools rather than cloning platforms.
4. Lifecycle honesty: edges do not claim wired reality without evidence/link state.

---

### User Story 3 - Discover and organize resources without silent duplicates (Priority: P1)

As a developer, I discover local workspace/tool resources, normalize them, and associate them to System/Environment without silent duplicates. Authority is **local**.

**Acceptance Scenarios**:

1. Discovery yields normalized candidates.
2. Re-discovery updates existing entity via stable external key — no silent duplicate.
3. Association persists under **local** authority.
4. Partial failure shows structured errors without inventing resources.

---

### User Story 4 - Create, configure, and operate under governance (Priority: P1)

As Owner/Operator or Agent, I request operations under role+capability+policy+approval. Full audit fields. Handoff for advanced external setup.

**Acceptance Scenarios**:

1. Allowed op runs with full audit field set (FR-027 / SC-006).
2. Sensitive op awaits approval or denies/expires without silent apply.
3. Cancel yields terminal cancel state without silent success.
4. Agent uses same policy path; distinct actor type in audit; cannot self-approve.
5. External advanced setup is handoff, not incomplete clone.

---

### User Story 5 - Run local operations with clear target authority (Priority: P1)

Local target only for completion. No remote node fleet.

**Acceptance Scenarios**:

1. Local op works offline from Polterware Cloud.
2. Unauthorized capability denied before execute.
3. Desktop vs CLI/MCP policy agreement (SC-005).

---

### User Story 6 - Follow logs without freezing the UI (Priority: P2)

Stream, pause, search, filter, virtualize, bounds/backpressure. Not a full observability platform.

---

### User Story 7 - Human and agent actors (Priority: P2)

Roles Owner/Operator/Viewer/Agent on local plane. No cloud org admin.

**Acceptance Scenarios**:

1. Human principal attributed correctly.
2. Agent distinct; same policy path.
3. Missing cloud org service does not fail acceptance.

---

### User Story 8 - Local-first (Priority: P1)

All completion data authority is **local**. Cloud sync out of scope.

---

### User Story 9 - Desktop, CLI, MCP same rules (Priority: P2)

No Web for acceptance. MCP min-scope tools; no policy bypass.

---

### User Story 10 - Big bang complete (Priority: P1)

Single architecture; one renderer entrypoint; no scenic packages; legacy removed; verify green; docs match.

**Acceptance Scenarios**:

1. Contributor quickstart under SC-014.
2. Verify suite green for Desktop+core+CLI+MCP.
3. Legacy imports zero.
4. Packages have API+consumer.
5. Docs describe final architecture.
6. Exactly one primary renderer product path; no React product dependency.

---

### Edge Cases

- Core startup failure: visible error, not silent success.
- Network drop mid-op: failed/cancelled/unknown structured; no silent success.
- Retry mutating op: no double apply (idempotency).
- Discovery overlap: stable key; prompt if unsafe merge.
- Corrupt store: isolate + error + explicit reset (FR-065).
- Agent self-approve: deny + audit.
- High-volume logs: UI interactive; buffers bounded.
- Capability gap CLI vs Desktop: explicit error, not forked rules.
- Stale session file after crash: detect, reject token, cleanup.
- Invalid/expired token: reject.
- Dual renderer after cutover: verification fails.
- Token in logs: must not appear.

---

## Requirements *(mandatory)*

### Functional Requirements

#### Inventory and disposition

- **FR-001**: MUST inventory all current responsibilities at **relevant file-level** (not package names only): entrypoints; Electron Main; preload; renderer; workers/async mechanisms; services; persistence; IPC; integrations; scripts; configs; tests; updater; functional assets; legacy stores; React product entry; code to preserve / migrate / remove. Trivial static assets and generated outputs MAY be omitted. Inventory MUST be written before structural deletes (SC-001).
- **FR-002**: Each responsibility MUST be classified preserve / migrate / replace / consolidate / remove with destination.
- **FR-003**: The refactor MUST preserve **approved product capabilities and user journeys** (shell, commander, processes, pipelines, terminal presentation, infrastructure status, settings, architecture canvas semantics, governed ops, local integrations in scope). It MUST NOT mean automatic preservation of all pre-refactor **persisted configuration data**.
- **FR-004**: Removed behaviors MUST be listed when user-visible.
- **FR-005**: Persistence migration for this feature is **greenfield** for stores classified DISCARD. At completion only new formats are required. Dual-format readers MUST NOT remain. See **Data disposition** table.
- **FR-005a**: Stores classified PRESERVE or CONVERT MUST NOT be discarded silently; CONVERT MUST have migration tests; PRESERVE MUST have verification tests.
- **FR-005b**: The refactor MUST preserve approved capabilities and flows, but MUST NOT automatically preserve configuration data from the previous architecture when those stores are DISCARD.
- **FR-006**: Old architecture MUST NOT remain active after completion.
- **FR-007**: No two permanent sources of truth for the same operational state.
- **FR-008**: Business rules MUST have a single implementation shared across Desktop, CLI, and MCP.

#### Product core (local)

- **FR-009**: MUST support human and agent principals as distinct auditable actors.
- **FR-010**: MUST support the **local** model: Workspace, System (domain root), Environment, Resource, Target, Actor/Principal. **MUST NOT** require cloud Organization, Membership, or Team management in this feature. Those are **DEFERRED — future cloud capability**.
- **FR-011**: MUST model architecture entities: resources, relationships, ownership text, and **light knowledge** (notes, descriptions, links, runbook/ADR references) associated to system/environment/component/resource.
- **FR-011a**: Knowledge MUST support create/read/update/delete and association to an architecture entity. MUST NOT include semantic search, embeddings, vector DB, RAG, distributed indexing, or cloud collaboration in this feature.
- **FR-012**: Domain root MUST be **System**. UI MAY label System as “Project” only as transitional alias (not a separate domain entity unless later defined).
- **FR-013**: Machines MUST be Targets, never System owners.
- **FR-014**: Cards are visual-only presentations of Resources.
- **FR-059**: User-authored map is canonical; discovery MUST NOT silently rewrite intent.
- **FR-060**: Local desktop MUST operate approved local capabilities without cloud organization, cloud session, or permanent Polterware Cloud connectivity.
- **FR-061**: **DEFERRED for product sync** this feature. If sync is introduced later, conflicts MUST NOT be resolved silently. Not an acceptance requirement now.
- **FR-062**: No pre-existing Polterware IdP to preserve. Local session model is defined in FR-067. Multi-actor cloud auth is DEFERRED.
- **FR-063**: Acceptance runtimes: **Desktop (Main/Preload/Renderer)**, **polterware-core**, **CLI**, **MCP**. Web, cloud backend, node connector MUST NOT be acceptance-blocking and MUST NOT be empty scenic packages.

#### Operations

- **FR-015**: Integrations as replaceable adapters behind ports.
- **FR-016**: Discovery with normalization and non-silent-duplicate reconciliation (**local** sources for completion).
- **FR-017**: Create/configure/connect/operate with validation, policy, optional approval, execute, progress, result, state update.
- **FR-018**: Workflows/pipelines compose steps without bypassing governance.
- **FR-019**: Terminal sessions only on Desktop/local authority (not Web).
- **FR-020**: Logs: stream, pause, search, filter; bounded.
- **FR-021**: Operations cancellable with terminal state recorded.
- **FR-022**: Scheduling MAY only with concrete approved use case; no scenic scaffolding.

#### Security and governance (local)

- **FR-023**: Local session authentication MUST follow FR-067 for Desktop/CLI/MCP access to core. Cloud remote auth is DEFERRED.
- **FR-024**: Authorization MUST evaluate: **principal, actor type, role defaults, explicit capabilities, workspace, system, environment, resource, operation, policy, approval requirement**. MUST NOT require cloud organization scope.
- **FR-024a**: Roles MUST be Owner, Operator, Viewer, Agent as defined in Roles matrix; critical role×capability combinations MUST be tested.
- **FR-025**: Policies, allowlists, denylists enforceable before sensitive execution.
- **FR-026**: Approvals gate ops when policy requires; recorded.
- **FR-027**: Every executed or denied operation MUST produce structured audit per FR-027a/b.
- **FR-027a (executed audit fields)**: audit event ID; operation ID; correlation ID; actor ID; actor type; workspace ID; system ID when applicable; environment ID when applicable; target type; target ID; requested capability; policy decision; approval ID when applicable; sanitized parameters; status; requested/started/finished timestamps; structured result summary; structured error when applicable.
- **FR-027b (denied audit fields)**: actor; target; capability; policy decision; denial reason; timestamp; correlation ID.
- **FR-028**: Secrets as references by default; never in logs, renderer storage, or audit payloads beyond redaction.
- **FR-029**: **Tenant isolation: NOT APPLICABLE** this feature. **Organization isolation: NOT APPLICABLE**. MUST isolate between systems, environments, and resources. MUST isolate human vs agent principals and enforce agent capability ceilings. Unauthorized ops denied before execution. CLI/MCP MUST NOT bypass policy/approval.
- **FR-030**: MCP/agents MUST NOT bypass human policies.
- **FR-031**: AI MAY propose; MUST NOT autonomously mutate production without same policy/approval path.

#### Distributed / cloud (deferred)

- **FR-032**: Node capabilities are **DEFERRED** (not acceptance). No empty node packages.
- **FR-033**: If node introduced later: no mandatory public inbound port default.
- **FR-034**: Cloud sync/notifications **DEFERRED** (not acceptance).
- **FR-035**: Internal observability MUST correlate ops across Desktop, core, CLI, MCP (not external APM replacement).

#### Cloud business

- **FR-036**: Cloud multi-tenant orgs/memberships/teams/shared audit aggregation **DEFERRED** (not acceptance). No scenic packages.
- **FR-037**: Billing/entitlements MUST NOT be scenic packages this initiative.
- **FR-038**: Desktop local-first complete without Polterware Cloud.

#### Runtime-specific

- **FR-039 Desktop**: Reliable startup; secure bridge; supervise core; visible startup failures; clean shutdown; non-blocking ops; **auto-update hooks in Main (FR-039a)**; local resources; secret handling; no unrestricted host access from renderer.
- **FR-039a Updater (PRESERVE)**: Electron Main owns auto-update (`electron-updater` path). Check MUST NOT block startup. When not packaged / not configured: no-op or structured skip. Download/install only per approved defaults (current: autoDownload when packaged; install on quit). Errors logged sanitized (no secrets). No update logic in renderer. MUST have test or contract-level verification of hook registration/behavior in packaged vs unpackaged modes.
- **FR-039b Single renderer entrypoint**: Final architecture MUST have exactly one primary product renderer entrypoint (SolidJS). During migration, temporary co-existence of React code is allowed **only** if a single official product path is designated per checkpoint. Cutover MUST switch the official entrypoint atomically to Solid. After cutover: React MUST NOT remain reachable via route, flag, script, or alternate product build. Final verification MUST fail if more than one primary product renderer is active.
- **FR-040 Web**: DEFERRED; if later: no desktop host APIs; no raw terminal; no local secret authority.
- **FR-041 Local backend (polterware-core)**: Local state, SQLite, local ops, tool adapters, authenticated Desktop/CLI/MCP channel, offline useful.
- **FR-042 Cloud backend**: DEFERRED; no scenic packages.
- **FR-043 Node**: DEFERRED; no scenic packages.
- **FR-044 CLI**: Same contracts/use cases; no policy bypass; structured errors; identity+activity.
- **FR-045 MCP**: Same contracts/use cases; min-scope tools/resources; no policy bypass; structured errors; identity+activity.
- **FR-067 Local session**: Desktop starts and supervises polterware-core. Core listens on **loopback only**, **dynamic port**, creates **ephemeral session token** valid for process lifetime. Token MUST NOT be logged; MUST NOT be permanent config; MUST NOT be exposed to renderer beyond necessity (renderer uses bridge, not raw token storage). For CLI/MCP: write **session descriptor** to OS-appropriate runtime directory with minimal fields (endpoint, token, pid, started_at). Unix file mode **0600**; Windows restricted ACL for current user. Remove on normal shutdown; detect and reject **stale** files (dead pid / health fail). Invalid/expired token rejected. Tests MUST cover: permissions, invalid token, stale token, dead process, cleanup, no token in logs.

#### Contracts, quality, monorepo

- **FR-046**: Shared versioned typed contracts; no hand twin types across runtimes.
- **FR-047**: Breaking changes declare versioning/migration strategy.
- **FR-048**: Integrations replaceable without domain change.
- **FR-049**: Streams define limits and backpressure.
- **FR-050**: Structured understandable errors.
- **FR-051**: Central monorepo generate/lint/typecheck/test/build/verify for **active** runtimes.
- **FR-052**: No packages without real API + consumer.
- **FR-053**: Not done on scaffolding alone.
- **FR-054**: Replaced legacy removed after validation.
- **FR-055**: Docs match final architecture at completion.
- **FR-056**: Temp adapters only on branch; removed before done.
- **FR-057**: Mock not presented as live product behavior after completion.
- **FR-058**: No false/incomplete provider integrations as functional.
- **FR-064**: Real local ops: package scripts, local git, process lifecycle/logs, local pipelines, declarative plan/apply when tools present. Vendor catalog entries handoff/status or non-complete; not fake success.
- **FR-065**: Validate writes fail-closed; corrupt reads isolated; structured error; explicit operator reset.
- **FR-066**: Post-merge rollback is **git revision only**. No dual-stack/dual-format rollback product.

### FR change log (this amend)

| Previous | Replacement | Reason |
| --- | --- | --- |
| FR-003 preserve “behaviors” ambiguous re data | FR-003 + FR-005a/b capabilities vs DISCARD data | C3 analyze |
| FR-010 orgs/memberships/teams MUST | Local Workspace/System/Environment; org DEFERRED | C1 |
| FR-024 org-scoped authz | Local principal/role/capability/workspace/system/env/resource/op | C2 |
| FR-029 org isolation MUST | N/A tenant/org; isolate system/env/resource/actor | C2 |
| FR-061 sync conflicts as active SC | DEFERRED product sync; not acceptance | completion set |
| FR-023 multi-actor cloud-style | Local session FR-067 | local-first |
| FR-036 soft “MAY retain Organization concepts” | Hard DEFERRED multi-tenant | C1 |
| Skills M or X | REMOVE (disposition) | H4 |
| Updater implicit | FR-039a PRESERVE | inventory functional |
| Dual renderer risk | FR-039b | H3 style |
| Audit fields thin | FR-027a/b | H6/audit |
| Knowledge vague | FR-011a light scope | knowledge |
| — | FR-024a roles matrix | H7 |
| — | FR-067 session | H8 |

### Non-Functional Requirements

- **NFR-001**: No multi-second UI freeze during log stream fixture.
- **NFR-002**: No heavy work on Main lifecycle process.
- **NFR-003**: Timeouts; structured timeout errors.
- **NFR-004**: Retries must not double-apply mutations.
- **NFR-005**: Virtualization/pagination; bounded log memory per [contracts/events.md](./contracts/events.md) defaults (10_000 visible entries; drop-oldest signalled).
- **NFR-006**: Offline local workflows without cloud.
- **NFR-007**: Renderer cannot arbitrary exec or read secret stores.
- **NFR-008**: Audit queryable by correlation/operation ID.
- **NFR-009**: Documented setup without tribal knowledge.
- **NFR-010**: Central verify for active runtimes (includes domain import + scenic package + boundary lint).
- **NFR-011**: Dependency direction; no domain→UI/provider concrete; no cycles; **enforced** by tooling (ESLint zones + Go import checks) in `verify`.
- **NFR-012**: Correlation across Desktop/core/CLI/MCP.
- **NFR-013**: Stream/queue defaults from events.md (64 KiB chunks; buffer 256; timeouts; concurrency caps) are configurable but MUST remain bounded and tested.
- **NFR-014**: Measurement targets — Desktop interactive ≤10s cold (document host); core Health ≤3s; soft memory budgets renderer ≤512 MiB / core ≤128 MiB idle.
- **NFR-015**: Retry classification per contracts/overview; mutations require idempotency where retryable.

### Key Entities (completion)

Principal/Actor · Role · Workspace · System · Environment · Target · Resource · Relationship · Evidence · Integration · Discovery Result · Operation · Approval · Policy · Audit Event · Workflow/Pipeline · Terminal Session · Log Stream · Secret Reference · Knowledge Item (light) · Workbench Panel · Card  

**DEFERRED entities (not required):** cloud Organization, Membership, Team, Node fleet, Sync Conflict product, Entitlement, cloud billing.

### Constitution Constraints

- Goals I for local control plane (collaboration via local human+agent, not cloud multi-tenant).
- Runtimes **active**: Main, Preload, Renderer, polterware-core, CLI, MCP. Web/server/node documented DEFERRED only.
- II/IV domain→ports; V contracts; VI security (local session, MCP no bypass); VII audit fields FR-027a/b; VIII local-first; IX–XII quality; XIII no scenic; XIV no dual final; XV big bang; XVI docs.

---

## Current State Inventory *(baseline)*

### Functional reality

| Capability | Status | Evidence |
| --- | --- | --- |
| Window, shortcuts, icons, **auto-updater when packaged** | Functional | `main/*`, `updater.ts` |
| Preload bridge surface | Present; UI mostly mock | preload |
| Operational IPC | UI-only errors | `ipc.ts` |
| Workbench data | Mock | mock-workbench-adapter |
| Core catalogs/runner/process/pipeline/declarative/pins/repos/skills helpers | Implemented, unwired | packages/core |
| Skill preview/setup | Core helpers exist; UI mock / not product-critical control-plane path | skillSetup + mock |
| Cloud/Web/Node | Absent | — |

### Persistence observed (nominal)

| Store | Location | Classification (this feature) |
| --- | --- | --- |
| `globalPipelinesV1` | user conf `polterware` | **DISCARD** |
| `desktopRepositoriesV1` | user conf | **DISCARD** |
| `pinnedCommandBasesV2` / `pinnedCommandRunsV1` | user conf | **DISCARD** |
| Legacy `pinnedCommands` | user conf | **DISCARD** / N/A |
| `.polterware/config.json` | project root | **DISCARD** |
| `polterware.yaml` (old declarative file as required format) | workspace | **DISCARD** as required reader; new declarative path is greenfield |
| In-memory processes | runtime | **NOT APPLICABLE** (ephemeral) |
| Mock workbench state | renderer | **DISCARD** |
| Prototype/mock fixtures as product truth | various | **DISCARD** |
| App icon selection preference | desktop Main | **PRESERVE** (capability/UX; re-store in new prefs OK without old conf key compat) |
| Frosted appearance / local UI prefs | renderer | **DISCARD** keys if legacy; capability re-implemented greenfield |
| Cloud data | — | **NOT APPLICABLE** |
| New SQLite (post-refactor) | polterware-core | **PRESERVE** going forward (new) |

No store is CONVERT for this feature (greenfield). PRESERVE applies to **capabilities**, not dual-format readers.

### Product surfaces retained (capabilities, not mock data)

Dock/workspaces, commander, processes/commands/pipelines, terminal panes, infrastructure/status, settings (icons + appearance capability), architecture/map semantics, repository concepts, approvals/audit concepts, local integrations (git/pkg real; vendors handoff).

---

## Capability Disposition (final)

| Area | Disposition | Notes |
| --- | --- | --- |
| Desktop shell / shortcuts / icons | P/M | Main + shell |
| **Auto-updater** | **P** | FR-039a Main only |
| Preload bridge | M/R | Contracts |
| Mock workbench product path | **X** | After real adapters |
| Workbench UX panels | P/M | Solid features |
| Command catalog local git/pkg | M | Real |
| Vendor gh/vercel/supabase | M handoff/status | Not full clone |
| Process/pipelines/declarative local | M | Real FR-064 |
| Pins/repos/old project config stores | **DISCARD** data; capability re-greenfield | FR-005 |
| MCP install/status helpers | M/R | Governed MCP runtime |
| **Skill preview/setup helpers** | **REMOVE** | Prototype/secondary; not completion control-plane journey; remove from executable product path |
| Shared UI (tokens intent) | P → Solid redesign | React package removed after cutover |
| Presentation app | P isolate | Non-runtime |
| Legacy TUI | **X** | Remove active path |
| Duplicate types | C | Generated contracts |
| UI-only IPC | R/X | Real core |
| Web/server/node/org cloud | **DEFERRED** | No packages |
| Full specialized platforms | X scope | Handoff |

---

## Runtime Boundary Requirements

| Runtime | Must own | Must not own |
| --- | --- | --- |
| Desktop Main | Lifecycle, windows, menus, permissions, **updates**, supervise core, session bootstrap file | Domain engine, bulk streams, policy authority |
| Preload | Minimal typed bridge | Business rules, raw token dump to storage |
| Renderer (Solid only as product) | Presentation/interaction | Node, secrets authority, exec, update logic |
| polterware-core | Local authority, SQLite, ops, policy, audit, streams | UI, cloud multi-tenant root |
| CLI | Terminal UX over same contracts | Forked rules |
| MCP | Min-scope tools over same contracts | Policy bypass |
| Web / server / node | DEFERRED | Not built this feature |

---

## Big Bang Migration Requirements

- **MIG-001**–**MIG-006**, **MIG-008**–**MIG-009**: unchanged intent (big bang, no permanent dual, git-only rollback, convergence).
- **MIG-007**: Greenfield DISCARD for listed stores; document user-visible reset; no dual-format readers.
- **MIG-010**: Single renderer cutover atomic (FR-039b).
- **MIG-011**: Session descriptor lifecycle (FR-067).

---

## Out of Scope

- Specialized platform clones; full APM/IaC/K8s/CI replacements; all future providers
- Microservices, NATS, empty packages
- Functional web, server, node, cloud orgs/memberships/teams, cloud sync, PostgreSQL stack, billing
- Semantic knowledge/RAG
- Autonomous AI production mutation without policy/approval
- Permanent dual React+Solid product paths
- Automatic preservation of DISCARD stores

---

## Success Criteria *(mandatory)*

| ID | Criterion | Verification method |
| --- | --- | --- |
| **SC-001** | 100% inventory rows have disposition + destination | Disposition register review |
| **SC-002** | 100% PRESERVE/MIGRATE capabilities demonstrable | Capability checklist e2e/manual |
| **SC-003** | 0 REMOVE capabilities reachable | Grep/runtime smoke |
| **SC-004** | 0 dual sources of truth for system/resource/operation/approval | Architecture audit sample |
| **SC-005** | Desktop vs CLI and Desktop vs MCP allow/deny agreement 100% on suite | Side-by-side tests |
| **SC-006** | Executed ops audit contains all FR-027a fields; denied ops all FR-027b fields; 100% suite | Integration/e2e field assert |
| **SC-007** | Role×capability denies: Viewer exec, Agent self-approve, Agent out-of-scope, Operator without capability — 100% | Security suite |
| **SC-008** | Offline local suite 100% without Polterware Cloud | e2e offline |
| **SC-009** | DISCARD stores documented; app does not require legacy readers; PRESERVE/CONVERT (if any) tested; greenfield new persistence works | Migration tests + greps |
| **SC-010** | Log stream fixture: UI not multi-second frozen; buffers bounded | Performance test |
| **SC-011** | 0 scenic empty packages | Package audit script |
| **SC-012** | 0 imports from replaced legacy; 0 React product dependency | Dependency/import audit |
| **SC-013** | generate+lint+typecheck+tests+buf+builds for Desktop/core/CLI/MCP pass | `pnpm verify` / go test |
| **SC-014** | New contributor completes quickstart and starts primary local app in ≤60 minutes | Timed checklist |
| **SC-015** | MCP vs human policy agreement 100% on suite | Security suite |
| **SC-016** | Spec/plan/tasks converge; 0 critical constitution conflicts | Analysis gate |
| **SC-017** | Exactly one primary product renderer entrypoint; cutover complete | Build/entry audit |
| **SC-018** | 0 domain modules import UI shells or concrete providers | Import boundary check |
| **SC-019** | FR-064 real local ops suite 100% | e2e/integration |
| **SC-020** | Corrupt store isolate+error; no silent rewrite; reset works | FR-065 tests |
| **SC-021** | 0 acceptance dependency on Web/server/node | Inventory + verify |
| **SC-022** | Session token security: permissions, invalid/stale/dead process, cleanup, no token in logs — 100% suite | Security tests FR-067 |
| **SC-023** | Updater: Main owns; unpackaged no-op; no renderer update API; hook testable | Unit/contract FR-039a |
| **SC-024** | Knowledge CRUD+associate works for architecture entity; no vector/RAG surface | API+UI smoke FR-011a |

---

## Assumptions

- Plan/tasks may lag this amend until regenerated; **this spec is normative**.
- Stack choices (Go core, Solid, Connect, Buf, SQLite) are plan-owned; this spec requires outcomes not library brands beyond completion set list.
- Presentation app remains non-product.
- External tools remain systems of record for advanced work.

## Dependencies

- Constitution v2.0.0
- product.md, DESIGN.md, SECURITY.md (update at completion)
- plan.md / tasks.md (must be reconciled after this amend — **not modified here**)

---

## Amendment summary (for report output)

See agent final message: FR/SC tables, dispositions, findings C1–C3/H4/H7 closed at spec layer.
