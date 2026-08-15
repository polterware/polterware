# Tasks: Remaining Operator Surfaces

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/

**Tests**: Risk-proportional per Constitution XII — unit tests for route helpers, pure filters/mappers, and critical UI state machines. Manual validation in [quickstart.md](./quickstart.md). Do not mark a task complete while required typecheck/lint/tests fail for the changed surface. Agents.md: do not run `pnpm dev` / full build unless the user asks.

**Organization**: Tasks grouped by user story for independent implementation and testing.

**Constitution**: Dependency direction IV; contracts-first V; security VI (no secret re-display); safe execution VII; SSoT XIV (eight-destination INSIGHTS IA only after cutover); real code only XIII; process drawer exclusivity for domain entities + notifications drawer for inbox only; Observability hub+hand-off not APM clone.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: US1–US10 maps to [spec.md](./spec.md) user stories
- Include exact file paths in every task description

## Path Conventions (this feature)

- Design SSOT updates: `DESIGN.md` (navigation/blueprints; tokens stay from 003)
- Design system: `packages/frontend/design-system/src/`
- Shell/route/chrome: `packages/features/shell/src/`
- Observability (new): `packages/features/observability/`
- Features: `packages/features/{architecture,resources,operations,access,integrations,settings,commander,logs}/`
- Composition: `apps/desktop/src/renderer/App.tsx`
- Data: `@polterware/frontend-data` + per-feature `fixtures.ts`
- Contracts reference: `specs/007-remaining-operator-surfaces/contracts/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Align inventory and package boundaries before code changes

- [x] T001 Inventory current `DEFAULT_NAV`, `SectionId`, `SECTION_TABS`, and App `Switch` in `packages/features/shell/src/index.tsx`, `packages/features/shell/src/route.ts`, and `apps/desktop/src/renderer/App.tsx`; record seven-destination baseline and target eight-destination + chrome sections in `specs/007-remaining-operator-surfaces/notes-nav-inventory.md`
- [x] T002 [P] Inventory existing feature depth (tabs already shipped vs placeholders) for resources/operations/access/integrations/settings/architecture in `packages/features/*/src/` and note gaps vs 007 wireframes in the same notes file
- [x] T003 [P] Confirm only one new package is allowed (`packages/features/observability`); document that overview/switcher/notifications live under shell and sync page under integrations per [research.md](./research.md) R5

**Checkpoint**: Inventory complete; package map clear

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Route model v2, nav types, observability package scaffold, DESIGN nav update, shared dialog/drawer types — **MUST complete before story UI depth**

**⚠️ CRITICAL**: No US1–US10 product surface depth until this phase completes (except notes)

### Route + types

- [x] T004 Extend `SectionId` with nav `observability` and chrome sections `overview` | `search` | `sync` | `onboarding`; split helpers `isNavSectionId` / `isSectionId` in `packages/features/shell/src/route.ts` per [data-model.md](./data-model.md) and [contracts/shell-ia.md](./contracts/shell-ia.md)
- [x] T005 [P] Expand `EntityKind`, `DialogState`, and `DrawerState` (`process` | `notifications`) plus optional `wizardStep` on `ClientRoute` in `packages/features/shell/src/route.ts` per data-model
- [x] T006 [P] Update `DEFAULT_TABS` / `SECTION_TABS` for observability tabs; resources `provisioning`; access `teams`/`invitations`; integrations `webhooks`; settings organization/developer/billing/security/data; sync chrome tabs in `packages/features/shell/src/route.ts`
- [x] T007 Update `resolveSection` legacy map: prefer `logs` → observability+logs; keep 004 redirects; add chrome section resolution in `packages/features/shell/src/route.ts`
- [x] T008 Unit tests for new section/tab/drawer/history behavior (chrome sections not in nav set; notifications drawer excluded from history; goSection to overview clears detail) in `packages/features/shell/src/route.test.ts`

### Package scaffold + docs

- [x] T009 Create real `packages/features/observability` package (`package.json` as `@polterware/feature-observability`, `tsconfig.json`, `src/index.tsx` exporting ObservabilityView stub, workspace wiring matching sibling features); no empty scenic extras
- [x] T010 [P] Add design-system `Popover` (or confirm existing primitive path) sufficient for project switcher focus trap / Escape in `packages/frontend/design-system/src/popover.tsx` and export from `packages/frontend/design-system/src/index.ts` if missing
- [x] T011 Update `DESIGN.md` navigation model for eight destinations, INSIGHTS group, Project Overview logo entry, global chrome (notifications/search/sync), and non-goals (no sidebar inflation)

**Checkpoint**: Foundation ready — route v2, observability package exists, DESIGN IA updated; stories can start

---

## Phase 3: User Story 1 — Extended shell navigation and Project Overview (Priority: P1) 🎯 MVP

**Goal**: Sidebar shows eight destinations in CORE / INSIGHTS / CONFIGURATION; logo opens Project Overview; project switcher popover works; chrome entry points exist without extra sidebar items

**Independent Test**: Open shell; count sidebar items (8); verify INSIGHTS includes Observability; click logo → Overview; open switcher search/select/cancel; confirm notifications/search/sync are not permanent sidebar items

### Tests for User Story 1

- [x] T012 [P] [US1] Unit tests for DEFAULT_NAV length/groups and overview route helpers in `packages/features/shell/src/route.test.ts` (or `nav.test.ts`)

### Implementation for User Story 1

- [x] T013 [US1] Replace `DEFAULT_NAV` with eight items and groups CORE / INSIGHTS / CONFIGURATION in `packages/features/shell/src/index.tsx` per [contracts/shell-ia.md](./contracts/shell-ia.md)
- [x] T014 [US1] Update `packages/features/shell/src/sidebar.tsx` for eight destinations + INSIGHTS label and neutral active styling; no chrome sections in nav
- [x] T015 [US1] Make logo/project name keyboard-activatable control that navigates to overview in `packages/features/shell/src/index.tsx` (wire `onOpenOverview`)
- [x] T016 [US1] Implement project switcher popover UI in `packages/features/shell/src/project-switcher.tsx` per [contracts/global-chrome.md](./contracts/global-chrome.md)
- [x] T017 [P] [US1] Add switcher fixtures/types in `packages/features/shell/src/fixtures.ts` and `packages/features/shell/src/types.ts` (ProjectSwitcherItem, ProjectOverviewSnapshot)
- [x] T018 [US1] Implement Project Overview page in `packages/features/shell/src/overview-page.tsx` (system status, attention, recent activity, quick actions) with empty/offline/fixture honesty
- [x] T019 [US1] Wire App composition: Match `overview` + `observability` stub; pass shell callbacks for overview/switcher; history includes chrome sections in `apps/desktop/src/renderer/App.tsx`
- [x] T020 [US1] Wire quick actions from Overview (architecture / create resource / run command / connect integration) via existing dialogs/sections in `apps/desktop/src/renderer/App.tsx`
- [x] T021 [US1] Ensure chrome sections clear sidebar `aria-current` (no false active nav) in `packages/features/shell/src/sidebar.tsx` / shell props

**Checkpoint**: US1 independently testable — eight-destination IA + Overview + switcher

---

## Phase 4: User Story 2 — Notifications, search results, synchronization center (Priority: P1)

**Goal**: Notification drawer, full search results page, sync center with queue/conflicts/history and shared conflict resolve

**Independent Test**: Open notifications drawer; filter All/Unread/Approvals; mark all read; palette search → show all results; open sync from footer; open a conflict and choose resolution

### Tests for User Story 2

- [x] T022 [P] [US2] Unit tests for notification filter (all/unread/approvals) and conflict resolution choice validation in `packages/features/shell/src/notifications.test.ts` and `packages/features/integrations/src/conflict.test.ts`

### Implementation for User Story 2

- [x] T023 [US2] Implement notification drawer UI in `packages/features/shell/src/notification-drawer.tsx` per [contracts/global-chrome.md](./contracts/global-chrome.md); bind `drawer.type === "notifications"`
- [x] T024 [P] [US2] Add notification fixtures and types in `packages/features/shell/src/fixtures.ts` / `types.ts`
- [x] T025 [US2] Add title-bar bell control opening notifications drawer in `packages/features/shell/src/title-bar.tsx` and wire in `packages/features/shell/src/index.tsx`
- [x] T026 [US2] Implement full search results page in `packages/features/commander/src/search-results.tsx` (query, category chips, hits, open hit) per contracts
- [x] T027 [P] [US2] Add search hit fixtures/types and “Show all results” action in `packages/features/commander/src/index.tsx` / fixtures
- [x] T028 [US2] Implement Synchronization center page in `packages/features/integrations/src/sync-center.tsx` (Status / Offline queue / Conflicts / History tabs)
- [x] T029 [P] [US2] Implement shared conflict resolve view in `packages/features/integrations/src/conflict-resolve.tsx` (side-by-side + keep-polter / accept-provider / ignore)
- [x] T030 [P] [US2] Add sync fixtures (endpoints, queue, conflicts, history) with `dataSource` in `packages/features/integrations/src/fixtures.ts`
- [x] T031 [US2] Wire footer core status click → `section=sync` in `packages/features/shell/src/index.tsx` and App Match for `search` + `sync` in `apps/desktop/src/renderer/App.tsx`
- [x] T032 [US2] Wire notification deep links and conflict open detail (`entityKind: sync-conflict`) in `apps/desktop/src/renderer/App.tsx`
- [x] T033 [US2] Ensure empty/loading/offline/permission presentations on search and sync pages in `packages/features/commander/src/search-results.tsx` and `packages/features/integrations/src/sync-center.tsx`

**Checkpoint**: US2 independently testable — global chrome awareness surfaces

---

## Phase 5: User Story 3 — Observability hub (Priority: P1)

**Goal**: Observability primary destination with Overview / Logs / Metrics / Traces / Monitors, details, and external hand-off

**Independent Test**: Open Observability → each tab; open log event dialog; open trace detail; open monitor detail; see Open Grafana/Datadog when fixture integrations present; change time range

### Tests for User Story 3

- [x] T034 [P] [US3] Unit tests for log filter helpers and trace status filters in `packages/features/observability/src/filter.test.ts`

### Implementation for User Story 3

- [x] T035 [US3] Define observability types (service health, log event, trace/span, monitor) in `packages/features/observability/src/types.ts` per data-model
- [x] T036 [P] [US3] Create labeled fixtures for overview/logs/metrics/traces/monitors in `packages/features/observability/src/fixtures.ts` with honest `dataSource`
- [x] T037 [US3] Implement Observability shell with environment, time range, tabs in `packages/features/observability/src/index.tsx` per [contracts/observability-ui.md](./contracts/observability-ui.md)
- [x] T038 [P] [US3] Implement Overview tab (health cards, service list, external dashboards) in `packages/features/observability/src/overview-tab.tsx`
- [x] T039 [P] [US3] Implement Logs tab (search/filters/live pause/virtualizable list) in `packages/features/observability/src/logs-tab.tsx`; reuse `packages/features/logs` helpers only if beneficial
- [x] T040 [P] [US3] Implement log event dialog in `packages/features/observability/src/log-event-dialog.tsx` (structured fields + resource/trace links)
- [x] T041 [P] [US3] Implement Metrics tab with lightweight SVG sparklines (no new chart lib) in `packages/features/observability/src/metrics-tab.tsx`
- [x] T042 [P] [US3] Implement Traces list tab in `packages/features/observability/src/traces-tab.tsx`
- [x] T043 [US3] Implement Trace detail waterfall page in `packages/features/observability/src/trace-detail.tsx`
- [x] T044 [P] [US3] Implement Monitors list in `packages/features/observability/src/monitors-tab.tsx`
- [x] T045 [US3] Implement Monitor detail (condition, sparkline, related context, mute/edit/resolve stubs) in `packages/features/observability/src/monitor-detail.tsx`
- [x] T046 [US3] Wire Observability section Match + detail navigation (`entityKind` trace/monitor) + FixtureBanner in `apps/desktop/src/renderer/App.tsx`
- [x] T047 [US3] Empty state when no observability integration: setup/catalog CTA without fake live APM claims in `packages/features/observability/src/overview-tab.tsx`

**Checkpoint**: US3 independently testable — Observability hub

---

## Phase 6: User Story 4 — Provisioning, drift, and node registration (Priority: P1)

**Goal**: Resources → Provisioning (Catalog / Executions / Drift); configure → plan review → approval/apply; execution detail; node registration wizard

**Independent Test**: Open Resources → Provisioning catalog; run configure→plan; open execution; open Drift and resolve one item; start Add node wizard and copy install command

### Tests for User Story 4

- [x] T048 [P] [US4] Unit tests for plan policy gate (block apply when fail checks without override) and drift action enum in `packages/features/resources/src/provisioning.test.ts`

### Implementation for User Story 4

- [x] T049 [US4] Add provisioning types (template, plan, execution, drift, node install session) in `packages/features/resources/src/types.ts` (or `provisioning-types.ts`)
- [x] T050 [P] [US4] Expand fixtures for catalog/plans/executions/drift/node install in `packages/features/resources/src/fixtures.ts`
- [x] T051 [US4] Add `provisioning` tab to Resources host in `packages/features/resources/src/index.tsx` with sub-views catalog|executions|drift via `route.filter`
- [x] T052 [P] [US4] Implement provisioning catalog UI in `packages/features/resources/src/provisioning-catalog.tsx` per [contracts/provisioning-ui.md](./contracts/provisioning-ui.md)
- [x] T053 [US4] Implement multi-step configure page in `packages/features/resources/src/provisioning-configure.tsx` (wizardStep)
- [x] T054 [US4] Implement plan review page in `packages/features/resources/src/provisioning-plan-review.tsx` with policy checks and approval-aware CTA
- [x] T055 [P] [US4] Implement execution detail in `packages/features/resources/src/provisioning-execution.tsx`
- [x] T056 [P] [US4] Implement drift list/detail actions in `packages/features/resources/src/provisioning-drift.tsx`
- [x] T057 [US4] Implement node registration wizard in `packages/features/resources/src/node-registration-wizard.tsx` (token expiry, copy, regenerate, heartbeat wait)
- [x] T058 [US4] Wire provisioning navigation, dialogs/wizards, and Add node entry from Nodes tab in `apps/desktop/src/renderer/App.tsx` / resources index
- [x] T059 [US4] Ensure mutations show request-approval path when production policy requires it (no silent apply UI) in `packages/features/resources/src/provisioning-plan-review.tsx`

**Checkpoint**: US4 independently testable — provisioning + node registration

---

## Phase 7: User Story 5 — Architecture import, snapshot compare, view editor (Priority: P2)

**Goal**: Import resources mapping; compare snapshots; edit saved views

**Independent Test**: Open Import resources wizard; apply mapping; open Compare snapshots; open Edit view; save/cancel dirty confirm

### Implementation for User Story 5

- [x] T060 [P] [US5] Add import/compare/view-edit types + fixtures in `packages/features/architecture/src/types.ts` and `fixtures.ts`
- [x] T061 [US5] Implement import resources wizard in `packages/features/architecture/src/import-resources.tsx` per [contracts/architecture-deep-ui.md](./contracts/architecture-deep-ui.md)
- [x] T062 [P] [US5] Implement snapshot comparison UI in `packages/features/architecture/src/snapshot-compare.tsx`
- [x] T063 [P] [US5] Implement view editor page in `packages/features/architecture/src/view-editor.tsx` with dirty cancel confirm
- [x] T064 [US5] Wire entry points: Architecture ⋯ → Import; Changes → Compare; Views → Edit in `packages/features/architecture/src/index.tsx` / related components
- [x] T065 [US5] Wire architecture deep routes/dialogs in `apps/desktop/src/renderer/App.tsx` as needed

**Checkpoint**: US5 independently testable — architecture deep flows

---

## Phase 8: User Story 6 — Workflow builder and schedules (Priority: P2)

**Goal**: Workflow edit builder with step config; schedules secondary strip under Workflows

**Independent Test**: Open workflow editor; select approval step; change config; save; open Schedules; pause; run now

### Implementation for User Story 6

- [x] T066 [P] [US6] Add workflow builder/schedule types + fixtures in `packages/features/operations/src/types.ts` and `fixtures.ts`
- [x] T067 [US6] Implement workflow builder two-pane UI in `packages/features/operations/src/workflow-builder.tsx` per [contracts/workflow-builder-ui.md](./contracts/workflow-builder-ui.md)
- [x] T068 [P] [US6] Implement schedules strip/list actions in `packages/features/operations/src/workflow-schedules.tsx`
- [x] T069 [US6] Wire edit/create workflow navigation and schedules secondary strip on Workflows tab in `packages/features/operations/src/index.tsx`
- [x] T070 [US6] Wire App detail route for workflow edit (`entityKind: workflow`, page detail or filter=edit) in `apps/desktop/src/renderer/App.tsx`

**Checkpoint**: US6 independently testable — workflow authoring + schedules

---

## Phase 9: User Story 7 — Access teams, invitations, roles, audit detail, agent sessions (Priority: P2)

**Goal**: Access tabs Teams + Invitations; team detail; roles matrix; audit event detail; agent session detail

**Independent Test**: Open Teams and team detail; Invitations resend/revoke; Roles matrix; audit event detail; agent session terminate (confirm)

### Implementation for User Story 7

- [x] T071 [US7] Expand Access tabs to members|teams|agents|policies|audit|invitations in `packages/features/access/src/index.tsx`
- [x] T072 [P] [US7] Add access expansion types + fixtures in `packages/features/access/src/types.ts` and `fixtures.ts`
- [x] T073 [P] [US7] Implement teams list + team detail in `packages/features/access/src/teams-index.tsx` and `team-detail.tsx` per [contracts/access-expansions-ui.md](./contracts/access-expansions-ui.md)
- [x] T074 [P] [US7] Implement invitations list + invite/resend/revoke in `packages/features/access/src/invitations.tsx`
- [x] T075 [P] [US7] Implement roles permission matrix in `packages/features/access/src/roles-matrix.tsx` (entry from Policies)
- [x] T076 [P] [US7] Implement audit event detail in `packages/features/access/src/audit-event-detail.tsx`
- [x] T077 [P] [US7] Implement agent session detail (tools allow/deny, terminate confirm) in `packages/features/access/src/agent-session-detail.tsx`
- [x] T078 [US7] Wire Access detail navigation for team/invitation/audit-event/agent-session entityKinds in `apps/desktop/src/renderer/App.tsx`

**Checkpoint**: US7 independently testable — Access expansions

---

## Phase 10: User Story 8 — Integration setup wizard, webhooks, conflict entry (Priority: P2)

**Goal**: Multi-step connect wizard; Webhooks tab; conflicts open shared resolver

**Independent Test**: Start Connect provider wizard through test connection; open Webhooks; inspect/test; open conflict from Integrations Sync

### Implementation for User Story 8

- [x] T079 [US8] Add `webhooks` tab to Integrations host in `packages/features/integrations/src/index.tsx`
- [x] T080 [P] [US8] Add webhook + setup wizard types/fixtures in `packages/features/integrations/src/types.ts` and `fixtures.ts` (secret = configured boolean only)
- [x] T081 [US8] Implement integration setup wizard in `packages/features/integrations/src/setup-wizard.tsx` per [contracts/integrations-expansions-ui.md](./contracts/integrations-expansions-ui.md)
- [x] T082 [P] [US8] Implement webhooks list/detail actions in `packages/features/integrations/src/webhooks.tsx` (test, deliveries stub, edit; no secret re-display)
- [x] T083 [US8] Wire Catalog/Connected “Add integration” → setup wizard and Sync rows → `conflict-resolve.tsx` in `packages/features/integrations/src/index.tsx` and App
- [x] T084 [US8] Security pass: ensure credential/webhook secret fields never rehydrate after create in `packages/features/integrations/src/setup-wizard.tsx` and `packages/features/integrations/src/webhooks.tsx`

**Checkpoint**: US8 independently testable — integration connect + webhooks

---

## Phase 11: User Story 9 — Settings expansions (Priority: P3)

**Goal**: Organization, Developer, Billing, Security, Data settings surfaces with honesty for local-only billing and MCP/CLI status

**Independent Test**: Visit each new Settings tab; edit org name; copy MCP config; view usage or unavailable; revoke a non-current session; change retention; danger-zone confirm

### Implementation for User Story 9

- [x] T085 [US9] Expand Settings tabs catalog and host routing in `packages/features/settings/src/index.tsx` per [contracts/settings-expansions-ui.md](./contracts/settings-expansions-ui.md)
- [x] T086 [P] [US9] Add settings expansion types + fixtures in `packages/features/settings/src/types.ts` and `fixtures.ts`
- [x] T087 [P] [US9] Implement Organization settings in `packages/features/settings/src/organization.tsx`
- [x] T088 [P] [US9] Implement Developer settings (CLI/MCP/API tokens/local endpoints) with honest status in `packages/features/settings/src/developer.tsx`
- [x] T089 [P] [US9] Implement Billing (plan/usage/invoices or unavailable state) in `packages/features/settings/src/billing.tsx`
- [x] T090 [P] [US9] Implement Security (MFA, sessions revoke, org policies) in `packages/features/settings/src/security.tsx`
- [x] T091 [P] [US9] Implement Data/retention + export + confirmed destructive delete in `packages/features/settings/src/data-retention.tsx`
- [x] T092 [US9] Wire Settings tab navigation and confirm dialogs in `apps/desktop/src/renderer/App.tsx` if composition owns dialogs; ensure Account tab does not permanently dual-home sessions (migrate or document temporary retain)

**Checkpoint**: US9 independently testable — settings expansions

---

## Phase 12: User Story 10 — First-run onboarding (Priority: P3)

**Goal**: Create/join organization and first project with local-first storage when no project exists

**Independent Test**: Launch with no project; complete Create organization → Create project (local-first); land Overview or Architecture with guided empty state

### Implementation for User Story 10

- [x] T093 [US10] Implement onboarding types/state machine helpers in `packages/features/shell/src/onboarding-types.ts` (or under shell types)
- [x] T094 [US10] Implement onboarding UI steps in `packages/features/shell/src/onboarding.tsx` per [contracts/global-chrome.md](./contracts/global-chrome.md)
- [x] T095 [US10] Detect no-project condition and force `section=onboarding` until create completes in `apps/desktop/src/renderer/App.tsx` (fixture flag OK for demo)
- [x] T096 [US10] On success navigate to overview/architecture with local-first default (no cloud payment required) in `apps/desktop/src/renderer/App.tsx` and `packages/features/shell/src/onboarding.tsx`

**Checkpoint**: US10 independently testable — greenfield entry

---

## Phase 13: Polish & Cross-Cutting Concerns

**Purpose**: IA cutover, honesty, a11y, docs, static validation

- [x] T097 Remove any temporary dual seven/eight destination flags; ensure SSoT is eight + INSIGHTS only in `packages/features/shell/src/index.tsx` and App
- [x] T098 [P] Commander palette navigation groups include Observability + Overview/Sync/Search actions in `packages/features/commander/src/index.tsx`
- [x] T099 [P] Cross-surface FixtureBanner / dataSource honesty for overview, observability, sync, billing, developer in `packages/features/shell/src/overview-page.tsx`, `packages/features/observability/src/index.tsx`, `packages/features/integrations/src/sync-center.tsx`, `packages/features/settings/src/billing.tsx`, `packages/features/settings/src/developer.tsx`
- [x] T100 [P] Keyboard smoke notes for switcher, notification drawer, and one multi-step wizard (focus trap/Escape) documented in `specs/007-remaining-operator-surfaces/notes-a11y-smoke.md` after verifying `packages/features/shell/src/project-switcher.tsx`, `notification-drawer.tsx`, and `packages/features/resources/src/node-registration-wizard.tsx`
- [x] T101 [P] Status not color-only audit on new status chips (sync endpoints, monitors, provision steps) in `packages/features/integrations/src/sync-center.tsx`, `packages/features/observability/src/monitors-tab.tsx`, `packages/features/resources/src/provisioning-execution.tsx`
- [x] T102 Run static checks from [quickstart.md](./quickstart.md) sections 1–3; record results in `specs/007-remaining-operator-surfaces/notes-static-checks.md` and fix failures in shell/route sources
- [x] T103 [P] Update stale “exactly seven destinations” SSoT copy in `DESIGN.md` (and shell comments in `packages/features/shell/src/index.tsx`) without rewriting full 004 history docs
- [x] T104 Final self-check: no scenic empty packages; no secret re-display; no APM/console clone scope creep; run cheap typecheck on changed feature packages under `packages/features/{shell,observability,resources,access,integrations,settings,operations,commander}/` per Agents.md

**Checkpoint**: Feature ready for `/speckit-implement` increments and manual quickstart

---

## Dependencies & Execution Order

### Phase dependencies

```text
Phase 1 Setup
    ↓
Phase 2 Foundational (blocks all stories)
    ↓
Phase 3 US1 (MVP shell + overview) ──┬──→ Phase 4 US2 (chrome)
                                      ├──→ Phase 5 US3 (observability)  [needs US1 nav item]
                                      └──→ later stories
Phase 6 US4 provisioning   ─┐
Phase 7 US5 architecture   ├─ can proceed after Foundational (+ US1 for nav polish)
Phase 8 US6 workflows      │
Phase 9 US7 access         │
Phase 10 US8 integrations ─┤  (US8 conflict UI should reuse US2 conflict-resolve)
Phase 11 US9 settings      │
Phase 12 US10 onboarding  ─┘
    ↓
Phase 13 Polish
```

### User story dependency notes

| Story | Depends on | Notes |
| --- | --- | --- |
| US1 | Foundational | MVP |
| US2 | Foundational; benefits from US1 shell callbacks | Conflict component required before US8 polish |
| US3 | Foundational + US1 nav | Observability package |
| US4 | Foundational | Independent of US3 |
| US5 | Foundational | Independent |
| US6 | Foundational | Independent |
| US7 | Foundational | Independent |
| US8 | Foundational; reuses US2 conflict-resolve | Wizard + webhooks |
| US9 | Foundational | Independent |
| US10 | Foundational; exits to US1 overview | Can stub overview if US1 partial |

### Parallel opportunities

Within a story, tasks marked **[P]** can proceed in parallel when they touch different files (e.g. US3 tabs T038–T042; US7 list/detail files T073–T077; US9 settings panels T087–T091).

Across stories after Foundational + US1: US3, US4, US5, US6, US7, US9 can fan out; US2 before or with US8; US10 late or early with stub overview.

### Parallel example (US3)

```text
T035 types
T036 fixtures          } parallel after types
T038 overview-tab
T039 logs-tab
T041 metrics-tab       } parallel UI tabs
T042 traces-tab
T044 monitors-tab
→ then T043/T045 details + T046 App wire
```

---

## Implementation Strategy

### MVP (recommended first delivery)

1. Phase 1–2 complete  
2. **US1 only** (eight destinations + Overview + switcher)  
3. Stop and validate Independent Test for US1  

### Incremental value order

1. US1 shell IA + overview  
2. US2 notifications + search + sync (operator awareness)  
3. US3 Observability (new primary destination depth)  
4. US4 Provisioning (mutation path)  
5. US5–US8 (deep flows) in any order after foundation  
6. US9–US10 (config + onboarding)  
7. Polish  

### Suggested agent batching

- Batch A: T001–T011 (setup + foundation)  
- Batch B: T012–T021 (US1 MVP)  
- Batch C: T022–T033 (US2)  
- Batch D: T034–T047 (US3)  
- Batch E: T048–T059 (US4)  
- Batch F: T060–T084 (US5–US8 parallelizable)  
- Batch G: T085–T096 (US9–US10)  
- Batch H: T097–T104 (polish)

---

## Task summary

| Phase | Story | Task IDs | Count |
| --- | --- | --- | --- |
| 1 Setup | — | T001–T003 | 3 |
| 2 Foundational | — | T004–T011 | 8 |
| 3 | US1 MVP | T012–T021 | 10 |
| 4 | US2 | T022–T033 | 12 |
| 5 | US3 | T034–T047 | 14 |
| 6 | US4 | T048–T059 | 12 |
| 7 | US5 | T060–T065 | 6 |
| 8 | US6 | T066–T070 | 5 |
| 9 | US7 | T071–T078 | 8 |
| 10 | US8 | T079–T084 | 6 |
| 11 | US9 | T085–T092 | 8 |
| 12 | US10 | T093–T096 | 4 |
| 13 Polish | — | T097–T104 | 8 |
| **Total** | | **T001–T104** | **104** |

### Format validation

- All tasks use `- [ ]`, sequential IDs, file paths  
- Story tasks include `[USn]`  
- Setup/Foundational/Polish have no story label  
- `[P]` only on parallelizable tasks  

### Independent test criteria (from spec)

| Story | Independent test |
| --- | --- |
| US1 | 8 destinations + Overview + switcher |
| US2 | Notifications + search results + sync/conflict |
| US3 | Observability five tabs + details + hand-off |
| US4 | Provisioning flow + drift + node wizard |
| US5 | Import + compare + view editor |
| US6 | Workflow builder + schedules |
| US7 | Teams, invitations, roles, audit, agent session |
| US8 | Setup wizard + webhooks + conflict entry |
| US9 | Org/Developer/Billing/Security/Data |
| US10 | Onboarding org+project local-first |

### MVP scope

**US1 only** after Foundational: eight-destination shell, Project Overview, project switcher — delivers navigable product spine without requiring Observability depth yet (Observability can remain stub tab until US3).
