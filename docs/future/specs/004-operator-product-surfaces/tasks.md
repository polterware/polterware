# Tasks: Operator Product Surfaces

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/

**Tests**: Risk-proportional per Constitution XII — unit tests for pure route/tab helpers, filters, and design-system Tabs/Drawer. Manual validation in [quickstart.md](./quickstart.md). Do not mark a task complete while required typecheck/lint/tests fail for the changed surface. Agents.md: do not run `pnpm dev` / full build unless the user asks.

**Organization**: Tasks grouped by user story for independent implementation and testing.

**Constitution**: Dependency direction IV; contracts-first V; security VI (no secret re-display); safe execution VII; SSoT XIV (seven-destination IA only after cutover); real code only XIII; no permanent inspectors; brand green restricted.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: US1–US9 maps to spec user stories
- Include exact file paths in every task description

## Path Conventions (this feature)

- Design SSOT updates: `DESIGN.md` (navigation/blueprints only; tokens from 003)
- Design system: `packages/frontend/design-system/src/`
- Shell/route: `packages/features/shell/src/`
- Features: `packages/features/{architecture,resources,operations,knowledge,access,integrations,settings,commander}/`
- Composition: `apps/desktop/src/renderer/App.tsx`
- Data: `@polterware/frontend-data` + per-feature `fixtures.ts`
- Contracts reference: `specs/004-operator-product-surfaces/contracts/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Align inventory and package boundaries before code changes

- [x] T001 Inventory current nav and section Switch in `packages/features/shell/src/index.tsx` (`DEFAULT_NAV`), `packages/features/shell/src/route.ts`, and `apps/desktop/src/renderer/App.tsx`; list legacy section ids (processes, logs, approvals, workflows, commander, agents) and target redirects in `specs/004-operator-product-surfaces/notes-nav-inventory.md`
- [x] T002 [P] Inventory stub feature packages (`packages/features/{operations,knowledge,integrations,settings,workflows,approvals,agents,processes,commander}/src/index.tsx`) and note absorb vs expand decisions matching [research.md](./research.md) R5 in the same notes file
- [x] T003 [P] Confirm only one new package is allowed (`packages/features/access`); document in notes that no scenic packages will be added

**Checkpoint**: Inventory complete; package map clear

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Route model, shared DS primitives, page states, Access package scaffold, DESIGN nav update — **MUST complete before story UI depth**

**⚠️ CRITICAL**: No US1–US9 product surface depth until this phase completes (except notes)

### Route + types

- [x] T004 Extend `ClientRoute` with `tab`, `entityKind`, `drawer` (`DrawerState`), expanded `DialogState` variants, and helpers (`setTab`, `openDrawer`, `closeDrawer`, `openDetail` with entityKind, `historySnapshot` includes tab) in `packages/features/shell/src/route.ts` per [data-model.md](./data-model.md)
- [x] T005 [P] Define `SectionId`, default tabs map, and legacy section→section+tab redirect map in `packages/features/shell/src/route.ts` (or `packages/features/shell/src/nav.ts`) per [contracts/shell-ia.md](./contracts/shell-ia.md)
- [x] T006 [P] Unit tests for route helpers (goSection resets tab, setTab clears detail, drawer excluded from history equality as designed) in `packages/features/shell/src/route.test.ts`

### Design-system primitives

- [x] T007 Implement keyboardable `Tabs` (`TabList`/`Tab`/`TabPanel` or single component API) in `packages/frontend/design-system/src/tabs.tsx` per [contracts/page-tabs-states.md](./contracts/page-tabs-states.md)
- [x] T008 [P] Implement temporary right `Drawer` with focus trap, Escape, overlay in `packages/frontend/design-system/src/drawer.tsx` (process-only product consumer later)
- [x] T009 [P] Ensure dialog primitive is exportable and sufficient for multi-field forms (create-node, run-command, credential) — evolve `packages/frontend/design-system/src/` dialog/a11y path and export from `packages/frontend/design-system/src/index.ts`
- [x] T010 [P] Add `PageOffline` and `PageForbidden` (permission message + optional request access) in `packages/frontend/design-system/src/feedback.tsx`; keep existing PageLoading/PageEmpty/PageError/FixtureBanner
- [x] T011 [P] Add dense table utility classes or thin presentational `DataTable` shell (header row + body rows, no domain logic) in `packages/frontend/design-system/src/table.tsx` (or `table.css` + helpers)
- [x] T012 Export new primitives from `packages/frontend/design-system/src/index.ts`
- [x] T013 [P] Unit tests for Tabs keyboard basics and Drawer open/close in `packages/frontend/design-system/src/tabs.test.tsx` and `packages/frontend/design-system/src/drawer.test.tsx`

### Package scaffold + docs

- [x] T014 Create real `packages/features/access` package (`package.json`, `tsconfig.json`, `src/index.tsx` exporting AccessView stub) with workspace wiring matching sibling features; no empty scenic extras
- [x] T015 Update `DESIGN.md` navigation model and screen blueprint sections for seven destinations, in-page tabs, process-drawer exclusivity, and command palette (do not rewrite tokens/identity from 003)

**Checkpoint**: Foundation ready — route model, Tabs/Drawer/states, Access package exists; stories can start

---

## Phase 3: User Story 1 — Seven-destination shell with in-page tabs (Priority: P1) 🎯 MVP

**Goal**: Sidebar shows exactly seven destinations in CORE / KNOWLEDGE / CONFIGURATION; history works; shell status/identity; each destination hosts in-page tabs (minimal placeholder tabs OK until later stories fill content)

**Independent Test**: Open Desktop shell; count sidebar items (7); verify groups; switch destinations; confirm tabs inside MainSurface not sidebar; back/forward; core status + actor footer; active nav not brand green

### Implementation for User Story 1

- [x] T016 [US1] Replace `DEFAULT_NAV` with seven items and groups CORE / KNOWLEDGE / CONFIGURATION in `packages/features/shell/src/index.tsx` per [contracts/shell-ia.md](./contracts/shell-ia.md)
- [x] T017 [US1] Update `packages/features/shell/src/sidebar.tsx` to render only the seven destinations with group labels KNOWLEDGE (not INSIGHTS) and neutral active styling
- [x] T018 [US1] Wire shell footer status (core online, nodes connected, actor name/role) from `ShellStatus` in `packages/features/shell/src/index.tsx` / sidebar footer
- [x] T019 [US1] Apply legacy section redirects and default `tab` on `goSection` in `packages/features/shell/src/route.ts` helpers used by App
- [x] T020 [US1] Refactor `apps/desktop/src/renderer/App.tsx` section `Switch` to seven destinations only; map legacy ids; pass `route`/`setTab`/status; remove primary rendering of processes/logs/approvals/workflows/commander/agents as top-level sections
- [x] T021 [US1] Add minimal in-page `Tabs` shell on each destination host (Architecture/Resources already richer later) so US1 acceptance “tabs not in sidebar” is true — e.g. default tab labels from data-model tab catalog even if panels are short placeholders in `packages/features/{architecture,resources,operations,knowledge,access,integrations,settings}/src/`
- [x] T022 [US1] Ensure history push/replace includes `tab` changes in `apps/desktop/src/renderer/App.tsx` (`applyRoute` / `historySnapshot` usage)
- [x] T023 [US1] Verify brand-green active nav does not return; status dots remain restricted brand use only in shell chrome

**Checkpoint**: US1 independently testable — seven-destination IA + tab strip grammar

---

## Phase 4: User Story 2 — Architecture tabs, canvas, node detail (Priority: P1)

**Goal**: Architecture tabs Canvas / Components / Dependencies / Views / Changes; canvas selection + detail page; create-node dialog; no permanent inspector

**Independent Test**: Open Architecture → each tab; select node (neutral border); open detail; Components/Dependencies tables; Views cards; Changes feed; Create node dialog

### Tests for User Story 2

- [x] T024 [P] [US2] Unit tests for component/dependency filter helpers in `packages/features/architecture/src/filter.test.ts` (create helpers if needed)

### Implementation for User Story 2

- [x] T025 [US2] Expand Architecture view-model types (ArchitectureComponent, DependencyEdge, ArchitectureView, ArchitectureChangeEvent) in `packages/features/architecture/src/types.ts` per data-model
- [x] T026 [P] [US2] Expand fixtures for components, dependencies, views, changes with `dataSource: "fixture"` in `packages/features/architecture/src/fixtures.ts`
- [x] T027 [US2] Wire Architecture page tabs (canvas/components/dependencies/views/changes) in `packages/features/architecture/src/index.tsx` using design-system Tabs and `route.tab`
- [x] T028 [US2] Keep/enhance canvas full-width behavior and neutral selection + Enter/open detail in `packages/features/architecture/src/architecture-canvas.tsx` (no permanent inspector)
- [x] T029 [P] [US2] Implement Components dense table in `packages/features/architecture/src/components-table.tsx`
- [x] T030 [P] [US2] Implement Dependencies table (direction filter, external toggle, export stub/action) in `packages/features/architecture/src/dependencies-table.tsx`
- [x] T031 [P] [US2] Implement Views card grid + open/create entry in `packages/features/architecture/src/views-grid.tsx`
- [x] T032 [P] [US2] Implement Changes chronological feed in `packages/features/architecture/src/changes-feed.tsx`
- [x] T033 [US2] Enhance node detail page sections (overview, relationship mini-view, connected, activity, notes, quick actions) in `packages/features/architecture/src/node-detail.tsx`
- [x] T034 [US2] Implement Create architecture node dialog in `packages/features/architecture/src/create-node-dialog.tsx` bound to `dialog.type === "arch-node-create"`
- [x] T035 [US2] Wire Architecture data + dialogs + navigation in `apps/desktop/src/renderer/App.tsx` with fixture fallback and FixtureBanner when not live
- [x] T036 [US2] Ensure Architecture loadState empty/loading/error/offline presentations on index tabs

**Checkpoint**: US2 independently testable — full Architecture tab surface

---

## Phase 5: User Story 3 — Resources inventory, nodes, processes, drawer (Priority: P1)

**Goal**: Resources tabs Inventory / Nodes / Processes; resource & node detail pages; process **drawer only**

**Independent Test**: Inventory → resource detail; Nodes → node detail; Processes → drawer open/close; pause updates; no full-page process route

### Tests for User Story 3

- [x] T037 [P] [US3] Unit tests for nodes/processes filter helpers in `packages/features/resources/src/filter.test.ts` (extend existing)

### Implementation for User Story 3

- [x] T038 [US3] Add TargetNode and ProcessRow types in `packages/features/resources/src/types.ts` per data-model
- [x] T039 [P] [US3] Expand fixtures for nodes and processes in `packages/features/resources/src/fixtures.ts`
- [x] T040 [US3] Wire Resources page tabs (inventory/nodes/processes) in `packages/features/resources/src/index.tsx` / `resources-index.tsx` composition
- [x] T041 [US3] Polish Inventory table + category filters remaining from 003 in `packages/features/resources/src/resources-index.tsx` if gaps vs contract
- [x] T042 [P] [US3] Implement Nodes table in `packages/features/resources/src/nodes-index.tsx`
- [x] T043 [P] [US3] Implement Node detail page in `packages/features/resources/src/node-detail.tsx`
- [x] T044 [US3] Implement Processes live table with pause/resume in `packages/features/resources/src/processes-table.tsx` (optionally reuse helpers from `packages/features/processes`)
- [x] T045 [US3] Implement Process drawer content + wire design-system Drawer in `packages/features/resources/src/process-drawer.tsx`; open only via `route.drawer`
- [x] T046 [US3] Enforce product rule: process activation MUST NOT set `page: "detail"` — only drawer — in resources navigation helpers
- [x] T047 [US3] Wire Resources nodes/processes data adapters + drawer state in `apps/desktop/src/renderer/App.tsx` (core ProcessService when available, else fixtures)
- [x] T048 [US3] Loading/empty/error/offline states for Resources tabs

**Checkpoint**: US3 independently testable — Resources complete with process drawer exclusivity

---

## Phase 6: User Story 4 — Operations runs, workflows, approvals, terminal (Priority: P1)

**Goal**: Operations tabs Runs / Workflows / Approvals / Terminal; run detail; run-command dialog; approval gates honesty

**Independent Test**: Runs list → run detail; Workflows list → detail; Approvals pending → detail Approve/Reject UI; Terminal multi-session chrome; Run command dialog

### Tests for User Story 4

- [x] T049 [P] [US4] Unit tests for run status filter helpers in `packages/features/operations/src/filter.test.ts`

### Implementation for User Story 4

- [x] T050 [US4] Add OperationRun, RunStep, WorkflowDef, ApprovalRequest, TerminalSession types in `packages/features/operations/src/types.ts`
- [x] T051 [P] [US4] Create fixtures for runs/workflows/approvals/sessions in `packages/features/operations/src/fixtures.ts`
- [x] T052 [US4] Implement Operations tab host in `packages/features/operations/src/index.tsx`
- [x] T053 [P] [US4] Implement Runs list + filters in `packages/features/operations/src/runs-index.tsx`
- [x] T054 [P] [US4] Implement Run detail (steps + output + cancel/retry UI) in `packages/features/operations/src/run-detail.tsx`
- [x] T055 [P] [US4] Implement Workflows list + detail in `packages/features/operations/src/workflows-index.tsx` and `packages/features/operations/src/workflow-detail.tsx` (may wrap `feature-workflows`)
- [x] T056 [P] [US4] Implement Approvals list + detail (impact, policy checks, comments, approve/reject) in `packages/features/operations/src/approvals-index.tsx` and `packages/features/operations/src/approval-detail.tsx` (may wrap `feature-approvals`)
- [x] T057 [US4] Implement Terminal multi-session UI in `packages/features/operations/src/terminal-sessions.tsx` using `@polterware/frontend-terminal`
- [x] T058 [US4] Implement Run command dialog in `packages/features/operations/src/run-command-dialog.tsx` (`dialog.type === "run-command"`); primary CTA respects approval-required messaging
- [x] T059 [US4] Wire Operations to App + core OperationService/ApprovalService/WorkflowService/ProcessService with fixture fallback in `apps/desktop/src/renderer/App.tsx`
- [x] T060 [US4] Forbidden path: when permission missing, show PageForbidden instead of silent apply for run/approve actions
- [x] T061 [US4] Loading/empty/error/offline states for Operations tabs

**Checkpoint**: US4 independently testable — Operations destination complete

---

## Phase 7: User Story 5 — Knowledge docs, decisions, runbooks, ownership (Priority: P2)

**Goal**: Knowledge tabs Docs / Decisions / Runbooks / Ownership + document detail page

**Independent Test**: Browse each Knowledge tab; open document detail; list decisions/runbooks; ownership matrix

### Implementation for User Story 5

- [x] T062 [US5] Add KnowledgeDocument, DecisionRecord, Runbook, OwnershipRow types in `packages/features/knowledge/src/types.ts`
- [x] T063 [P] [US5] Create knowledge fixtures in `packages/features/knowledge/src/fixtures.ts`
- [x] T064 [US5] Implement Knowledge tab host in `packages/features/knowledge/src/index.tsx`
- [x] T065 [P] [US5] Implement Docs list + document detail in `packages/features/knowledge/src/docs-index.tsx` and `packages/features/knowledge/src/document-detail.tsx`
- [x] T066 [P] [US5] Implement Decisions list in `packages/features/knowledge/src/decisions-index.tsx`
- [x] T067 [P] [US5] Implement Runbooks list in `packages/features/knowledge/src/runbooks-index.tsx`
- [x] T068 [P] [US5] Implement Ownership matrix in `packages/features/knowledge/src/ownership-matrix.tsx`
- [x] T069 [US5] Wire Knowledge data in `apps/desktop/src/renderer/App.tsx` (core KnowledgeService when available, else fixtures)
- [x] T070 [US5] Empty/loading/error presentations for Knowledge tabs

**Checkpoint**: US5 independently testable

---

## Phase 8: User Story 6 — Access members, agents, policies, audit (Priority: P2)

**Goal**: Access tabs Members / Agents / Policies / Audit + detail pages; agents capabilities; no fake authz

**Independent Test**: Members list/detail; Agents list/detail; Policies list/detail; Audit list + filters; invite/register entry points when permitted

### Implementation for User Story 6

- [x] T071 [US6] Add Member, AgentActor, PolicyDef, AuditEvent types in `packages/features/access/src/types.ts`
- [x] T072 [P] [US6] Create access fixtures in `packages/features/access/src/fixtures.ts`
- [x] T073 [US6] Implement Access tab host in `packages/features/access/src/index.tsx`
- [x] T074 [P] [US6] Implement Members list + detail in `packages/features/access/src/members-index.tsx` and `packages/features/access/src/member-detail.tsx`
- [x] T075 [P] [US6] Implement Agents list + detail (capabilities matrix) in `packages/features/access/src/agents-index.tsx` and `packages/features/access/src/agent-detail.tsx` (wrap or absorb `packages/features/agents`)
- [x] T076 [P] [US6] Implement Policies list + detail in `packages/features/access/src/policies-index.tsx` and `packages/features/access/src/policy-detail.tsx`
- [x] T077 [P] [US6] Implement Audit list with filters + export action stub/honest UI in `packages/features/access/src/audit-index.tsx`
- [x] T078 [US6] Wire Access section in `apps/desktop/src/renderer/App.tsx` (Identity/Policy/Audit services when available)
- [x] T079 [US6] Ensure disable/remove/invite actions show forbidden state when not permitted
- [x] T080 [US6] Empty/loading/error presentations for Access tabs

**Checkpoint**: US6 independently testable

---

## Phase 9: User Story 7 — Integrations connected, catalog, credentials, sync (Priority: P2)

**Goal**: Integrations tabs + detail; credentials never re-show secrets; catalog honesty

**Independent Test**: Connected cards; catalog groups; integration detail; credentials list without secrets; sync table

### Implementation for User Story 7

- [x] T081 [US7] Add IntegrationConnection, CatalogEntry, CredentialRef, SyncRow types in `packages/features/integrations/src/types.ts` (CredentialRef MUST NOT include secret field on read models)
- [x] T082 [P] [US7] Create integrations fixtures in `packages/features/integrations/src/fixtures.ts`
- [x] T083 [US7] Implement Integrations tab host in `packages/features/integrations/src/index.tsx`
- [x] T084 [P] [US7] Implement Connected cards grid in `packages/features/integrations/src/connected-grid.tsx`
- [x] T085 [P] [US7] Implement Catalog grouped browser in `packages/features/integrations/src/catalog-browser.tsx` (coming-soon honesty)
- [x] T086 [P] [US7] Implement Integration detail page in `packages/features/integrations/src/integration-detail.tsx`
- [x] T087 [P] [US7] Implement Credentials list + create dialog (write-only secret, clear after submit) in `packages/features/integrations/src/credentials-index.tsx` and `packages/features/integrations/src/credential-create-dialog.tsx`
- [x] T088 [P] [US7] Implement Sync table in `packages/features/integrations/src/sync-table.tsx`
- [x] T089 [US7] Wire Integrations in `apps/desktop/src/renderer/App.tsx` with SecretService reference-only behavior
- [x] T090 [US7] Security smoke: ensure no secret values rendered in list/detail after create in integrations UI

**Checkpoint**: US7 independently testable

---

## Phase 10: User Story 8 — Settings project, environments, notifications, core, account (Priority: P2)

**Goal**: Settings five tabs; Project=System alias; Core honesty; danger zone confirm

**Independent Test**: Project form; Environments table; Notifications matrix; Core status; Account profile/appearance

### Implementation for User Story 8

- [x] T091 [US8] Add settings view-model types in `packages/features/settings/src/types.ts` per data-model
- [x] T092 [P] [US8] Create settings fixtures in `packages/features/settings/src/fixtures.ts`
- [x] T093 [US8] Implement Settings tab host in `packages/features/settings/src/index.tsx`
- [x] T094 [P] [US8] Implement Project (System) settings form + danger zone confirm in `packages/features/settings/src/project-settings.tsx`
- [x] T095 [P] [US8] Implement Environments table + protection rules in `packages/features/settings/src/environments-settings.tsx`
- [x] T096 [P] [US8] Implement Notifications matrix + quiet hours in `packages/features/settings/src/notifications-settings.tsx`
- [x] T097 [P] [US8] Implement Core settings panel (status, endpoint, startup toggles, restart/logs actions when bridged) in `packages/features/settings/src/core-settings.tsx`
- [x] T098 [P] [US8] Implement Account settings in `packages/features/settings/src/account-settings.tsx`
- [x] T099 [US8] Wire Settings in `apps/desktop/src/renderer/App.tsx` with honest core health reflection
- [x] T100 [US8] Disable or honestly message unwired destructive project delete / notification channels

**Checkpoint**: US8 independently testable

---

## Phase 11: User Story 9 — Global dialogs, command palette, system states (Priority: P2)

**Goal**: Command palette overlay; shared empty/loading/offline/permission consistency; remaining global dialogs wired

**Independent Test**: ⌘K/Ctrl+K palette jumps to all 7 destinations; create-node and run-command from actions; force offline/empty/forbidden presentations

### Implementation for User Story 9

- [x] T101 [US9] Convert `packages/features/commander/src/index.tsx` into CommandPalette overlay (Navigation / Actions / Recent) per [contracts/overlays.md](./contracts/overlays.md) — not a sidebar destination
- [x] T102 [US9] Wire palette open shortcut and state in `apps/desktop/src/renderer/App.tsx`; actions open dialogs or navigate sections
- [x] T103 [US9] Ensure create-node and run-command dialogs are invocable from palette actions and from Architecture/Operations headers
- [x] T104 [US9] Audit all seven destinations for consistent PageEmpty / PageLoading / PageError / PageOffline / PageForbidden / FixtureBanner usage; fix gaps in feature packages
- [x] T105 [US9] Document keyboard shortcuts and palette behavior in `DESIGN.md` navigation section
- [x] T106 [US9] Verify process drawer remains the only drawer usage (grep product features for Drawer misuse) and fix violations

**Checkpoint**: US9 independently testable — overlays + system states

---

## Phase 12: Polish & Cross-Cutting Concerns

**Purpose**: Convergence, dual-path removal, performance, security, quickstart

- [x] T107 Remove temporary dual navigation paths and dead top-level section code from `apps/desktop/src/renderer/App.tsx` and `packages/features/shell/src/` (XIV)
- [x] T108 [P] Virtualize long tables (resources inventory, audit, processes, components) with `@tanstack/solid-virtual` where row counts warrant in respective feature files
- [x] T109 [P] Performance pass: process table polling must not block shell; canvas remains non-blocking (reuse 003 canvas worker patterns)
- [x] T110 Security review checklist: credentials secrets, approval self-approve policy UI, renderer has no Node/OS, MCP not bypassed — note results in `specs/004-operator-product-surfaces/notes-security-smoke.md`
- [x] T111 [P] Risk-proportional tests remaining for pure helpers added during implementation
- [x] T112 Update feature package exports and workspace consumers so Access/Operations/etc. compile from Desktop
- [x] T113 Convergence check: code ↔ [spec.md](./spec.md) ↔ [plan.md](./plan.md) ↔ this tasks file (Governance §6)
- [x] T114 Run static portions of [quickstart.md](./quickstart.md) (rg/nav checks, typecheck/tests on changed packages); interactive Desktop only if user requests
- [x] T115 Mark feature ready for `/speckit-implement` handoff when US1–US4 MVP+CORE depth claimed or full US1–US9 as scoped

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 Setup**: Immediate
- **Phase 2 Foundational**: After Setup — **BLOCKS** all user stories
- **Phase 3 US1**: After Foundational — 🎯 MVP navigation
- **Phases 4–6 US2–US4**: After Foundational; prefer after US1 for App Switch stability; can parallelize across people on different packages
- **Phases 7–10 US5–US8**: After Foundational; best after US1; independent of each other by package
- **Phase 11 US9**: After US1 (palette targets); best after US2–US4 dialogs exist
- **Phase 12 Polish**: After desired stories complete

### User Story Dependencies

| Story | Depends on | Notes |
| --- | --- | --- |
| US1 Shell IA | Phase 2 | MVP |
| US2 Architecture | Phase 2, ideally US1 | CORE |
| US3 Resources | Phase 2, ideally US1 | CORE; drawer uses DS Drawer |
| US4 Operations | Phase 2, ideally US1 | CORE; mutation honesty |
| US5 Knowledge | Phase 2, US1 | P2 |
| US6 Access | Phase 2, US1, T014 Access package | P2 |
| US7 Integrations | Phase 2, US1 | P2; secrets |
| US8 Settings | Phase 2, US1 | P2 |
| US9 Overlays/states | US1 + dialogs from US2/US4 | Cross-cutting |

### Within Each Story

1. Types + fixtures  
2. Tab host  
3. List/detail panels (parallel [P] where different files)  
4. App wiring  
5. States + honesty  

### Parallel Opportunities

- After Phase 2: US2 / US3 / US4 can proceed in parallel on different packages
- After US1: US5–US8 parallel by package
- Within a story, all [P] tasks with different files can run together

---

## Parallel Example: User Story 2

```bash
# After T025 types exist:
Task: "T026 Expand architecture fixtures"
Task: "T029 Components table"
Task: "T030 Dependencies table"
Task: "T031 Views grid"
Task: "T032 Changes feed"
```

## Parallel Example: User Stories 5–8 (post-US1)

```bash
Task: "US5 Knowledge tabs in packages/features/knowledge"
Task: "US6 Access tabs in packages/features/access"
Task: "US7 Integrations tabs in packages/features/integrations"
Task: "US8 Settings tabs in packages/features/settings"
```

---

## Implementation Strategy

### MVP First (US1 only)

1. Phase 1 Setup  
2. Phase 2 Foundational  
3. Phase 3 US1 — seven-destination shell + tab grammar  
4. **STOP and VALIDATE** via quickstart §5.1  

### Recommended CORE slice (demo-ready control plane)

1. MVP (US1)  
2. US2 Architecture + US3 Resources + US4 Operations  
3. Validate quickstart §5.2–5.4  

### Full product map

1. CORE slice  
2. US5–US8 in any order  
3. US9 palette + state audit  
4. Polish T107–T115  

### Parallel Team Strategy

1. Team completes Phase 1–2 together  
2. Dev A: US1 → US9 polish  
3. Dev B: US2 Architecture  
4. Dev C: US3 Resources  
5. Dev D: US4 Operations  
6. Then split US5–US8  

---

## Notes

- [P] = different files, no incomplete dependencies  
- [USn] maps to spec user stories  
- Process detail is **drawer only** — never a permanent inspector or full page  
- Credentials: secret write-once, never re-display  
- Fixtures always labeled (`FixtureBanner` / `dataSource`)  
- No React; no scenic packages beyond `feature-access`  
- Do not complete while required checks for changed surface fail  
- Commit after each task or logical group when user requests git ops  
- Implementation: `/speckit-implement` or manual execution of this list  

## Task Count Summary

| Phase | Tasks | Story |
| --- | --- | --- |
| Phase 1 Setup | T001–T003 (3) | — |
| Phase 2 Foundational | T004–T015 (12) | — |
| Phase 3 US1 | T016–T023 (8) | Shell IA |
| Phase 4 US2 | T024–T036 (13) | Architecture |
| Phase 5 US3 | T037–T048 (12) | Resources |
| Phase 6 US4 | T049–T061 (13) | Operations |
| Phase 7 US5 | T062–T070 (9) | Knowledge |
| Phase 8 US6 | T071–T080 (10) | Access |
| Phase 9 US7 | T081–T090 (10) | Integrations |
| Phase 10 US8 | T091–T100 (10) | Settings |
| Phase 11 US9 | T101–T106 (6) | Overlays/states |
| Phase 12 Polish | T107–T115 (9) | — |
| **Total** | **115** | |

**Suggested MVP**: Phase 1–3 (T001–T023) — seven-destination shell.  
**Suggested demo CORE**: through Phase 6 (US1–US4).  
**Format validation**: All tasks use `- [ ]`, sequential IDs T001–T115, [P]/[USn] where required, and explicit file paths.
