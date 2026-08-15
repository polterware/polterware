# Tasks: Architecture Big Bang Refactor

**Input**: [spec.md](./spec.md) (amended) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md) · [migration-map.md](./migration-map.md)

**Completion set (FR-063)**: Desktop + `polterware-core` + CLI + MCP only. No scenic web/server/node/PostgreSQL packages.

**Tests**: Required for domain, contracts, session (SC-022), roles (SC-007), audit fields (SC-006), FR-064 e2e, security, legacy absence.

**Format**: `- [ ] Tnnn [P?] [USn?] Description with path`

---

## Phase 1: Setup (monorepo foundation)

**Goal**: Workspace ready; no product deletes yet. **FR-001 file-level inventory before structural deletes.**

- [x] T001 Capture baseline tree to `docs/migration/baseline/tree.txt` listing `apps/`, `packages/`, `legacy/` (relevant files only)
- [x] T001a Write `docs/migration/baseline/file-register.md` with rows: path · role · disposition for **FR-001 categories**: entrypoints, Main, preload, renderer, workers/async, services, persistence, IPC, integrations, scripts, configs, tests, updater, functional assets, legacy stores, React entry, preserve/migrate/remove targets (SC-001)
- [x] T001b Write `docs/migration/baseline/workers.md` applying research R18 dispositions (PRESERVE / MIGRATE TO WEB WORKER / MIGRATE TO POLTERWARE-CORE / MIGRATE TO CHILD PROCESS / REMOVE / NOT APPLICABLE) for each async mechanism found
- [x] T002 [P] Write `docs/migration/baseline/runtimes.md` (Main/Preload/Renderer present; core/CLI/MCP/web/server/node status)
- [x] T003 [P] Write `docs/migration/baseline/desktop-features.md` for `apps/desktop/src/renderer/features/*`
- [x] T004 [P] Write `docs/migration/baseline/persistence.md` with DISCARD table matching [migration-map.md](./migration-map.md)
- [x] T005 [P] Write `docs/migration/baseline/ipc-contracts.md` from `apps/desktop/src/shared/ipc.ts`
- [x] T006 [P] Write `docs/migration/baseline/integrations.md` from `packages/core/src/data/commands/*`
- [x] T007 [P] Write `docs/migration/baseline/commands.md` recording current `pnpm typecheck`/`test`/`build` results
- [x] T008 Write `docs/migration/baseline/disposition-register.md` merging tree + file-register + workers → P/M/R/C/X/DISCARD/DEFERRED per spec (100% rows)
- [x] T009 [P] Characterization keep-green: `apps/desktop/src/renderer/features/shell/desktop-shell.test.tsx`
- [x] T010 [P] Characterization keep-green: `apps/desktop/src/renderer/features/commander/command-search.test.ts`
- [x] T011 Create `packages/configuration/typescript/tsconfig.base.json` shared TS base
- [x] T012 [P] Create `packages/configuration/eslint/` + `packages/configuration/prettier/` configs including **boundary rules** (CHK049): no-restricted-imports / eslint-plugin-boundaries for apps↛feature internals, features↛features internals, frontend↛apps, non-desktop↛electron, renderer↛node:*, ban deep generated edits
- [x] T013 [P] Create `packages/configuration/tailwind/base.css` token shell for Tailwind v4
- [x] T014 Update root `package.json` scripts: `generate`, `verify` (includes boundary + domain-import + scenic checks), `dev:desktop`, `lint`, `format:check`, `typecheck`, `test`, `test:integration`, `test:e2e`, `test:security`, `test:performance`, `build`, `package`
- [x] T015 Update `turbo.json` so `typecheck`/`build` depend on `generate` where needed; cache only deterministic tasks
- [x] T016 Update `pnpm-workspace.yaml` globs for `packages/frontend/*`, `packages/features/*`, `packages/contracts`, `packages/generated`, `packages/configuration`, `packages/testing` (packages added as created)
- [x] T017 Create `backend/go.work` (extend when modules appear)
- [x] T018 Write `docs/architecture/dependency-boundaries.md` (TS zones + Go import rules; apps↛domains; domains↛concrete providers/UI)
- [x] T019 Create `tooling/check-no-scenic.sh` failing on empty `apps/web`, `backend/runtimes/server`, `backend/runtimes/node`
- [x] T020 Create `tooling/check-domain-imports.sh` ensuring Go domains do not import sql drivers/connect handlers/electron; cycles fail
- [x] T020a Create `tooling/check-ts-boundaries.sh` or wire ESLint boundaries into `pnpm lint`/`verify` (CHK049)
- [x] T021 Write `docs/migration/deferred-cloud.md` (web/server/node/org/memberships/teams/PostgreSQL/pgx/sqlc/NATS/cloud sync DEFERRED; no implementation; no empty packages)
- [x] T022 Write `docs/development/local-setup.md` stub linking planned commands + session file path

---

## Phase 2: Foundational (contracts, domains, SQLite, core shell)

**Goal**: Blocking foundation before story UIs. **Exit**: core builds, health+session work, domain unit tests green.

### Contracts & generate

- [x] T023 Create `schemas/buf.yaml` and `schemas/buf.gen.yaml` for Go+TS plugins
- [x] T024 [P] Author `schemas/polterware/common/v1/error.proto` (ErrorDetail + retryable)
- [x] T025 Author `schemas/polterware/local/v1/*.proto` per [contracts/local-api.md](./contracts/local-api.md) (Health, Identity, Project, Architecture, Knowledge, Integration, Operation, Workflow, Process, Log, Policy, Approval, Audit, Maintenance)
- [x] T026 Author `schemas/polterware/events/v1/events.proto` + `schemas/polterware/events/v1/limits.md` matching [contracts/events.md](./contracts/events.md) defaults (64KiB stdout/stderr chunk, op buffer 256, log UI 10k, render batch 200, line 1MiB, RPC 4MiB, op timeout 10m, consumer idle 30s, concurrent ops 4, terminal sessions 4, no heartbeat)
- [x] T027 Write `schemas/README.md` stating public/internal/node protos NOT created this feature
- [x] T028 Wire `pnpm generate` → `buf generate` into `packages/generated/` and `backend/platform/gen/`
- [x] T029 Configure buf breaking detection documented in `schemas/README.md`
- [x] T030 [P] Contract test `tests/contract/health_test.go` against generated Health types
- [x] T031 [P] Contract test `tests/contract/operation_test.go` Operation message required fields FR-027a/b
- [x] T032 [P] Contract test `tests/contract/project_test.go` System/Environment messages (no Organization)
- [x] T033 [P] Contract test `tests/contract/knowledge_test.go` KnowledgeItem light fields
- [x] T034 [P] Contract test `tests/contract/approval_test.go` approval states

### Domains

- [x] T035 Create `backend/domains/identity` Principal+Role Owner/Operator/Viewer/Agent + unit tests in `backend/domains/identity/*_test.go`
- [x] T036 [P] Create `backend/domains/projects` Workspace/System/Environment + tests
- [x] T037 [P] Create `backend/domains/architecture` layout revision FR-059 + tests (no silent rewrite)
- [x] T038 [P] Create `backend/domains/resources` external_key uniqueness + tests
- [x] T039 [P] Create `backend/domains/operations` state machine + capability field + tests
- [x] T040 [P] Create `backend/domains/workflows` pipeline steps must call policy + tests
- [x] T041 [P] Create `backend/domains/policies` Authorize(role,caps,capability,ctx) + tests
- [x] T042 [P] Create `backend/domains/approvals` + agent self-approve deny test
- [x] T043 [P] Create `backend/domains/audit` redaction + FR-027a/b field builders + tests
- [x] T044 [P] Create `backend/domains/logs` buffer bounds from limits.md + tests
- [x] T045 [P] Create `backend/domains/integrations` execute|status|handoff + tests
- [x] T046 [P] Create `backend/domains/agents` agent principal rules + tests
- [x] T047 [P] Create `backend/domains/knowledge` CRUD+associate rules + tests (no RAG)
- [x] T048 Create `backend/platform/errors` Connect code mapping + tests
- [x] T049 [P] Create `backend/platform/observability` correlation ID helpers + tests
- [x] T050 [P] Create `backend/platform/resilience/retry_class.go` retry taxonomy + tests
- [x] T051 Run `tooling/check-domain-imports.sh` green on domains

### SQLite

- [x] T052 Create `backend/platform/db/sqlite` open WAL/FK/busy_timeout (pure-Go driver) + open test
- [x] T053 Author migrations `backend/platform/db/sqlite/migrations/0001_init_principals.sql` … `0008_workspace_layout.sql` per [data-model.md](./data-model.md) (knowledge + principal.role + secret_reference)
- [x] T054 Implement `backend/providers/sqlite/*` port adapters for identity/projects/architecture/resources/ops/approvals/audit/knowledge/workflows
- [x] T055 Integration test `tests/integration/sqlite_migrations_test.go`
- [x] T056 Implement quarantine + health in `backend/providers/sqlite/health.go` + FR-065 test
- [x] T057 Write `docs/migration/greenfield.md` listing DISCARD stores; no dual-read
- [x] T058 Test `tests/integration/no_legacy_conf_test.go` core ignores old conf keys

### polterware-core foundation

- [x] T059 Create `backend/runtimes/core/main.go` and `backend/compositions/local-core` wiring
- [x] T060 Implement loopback Connect server + bearer auth interceptor in `backend/compositions/local-core/server.go` + unauth test
- [x] T061 Implement session mint + descriptor write/delete in `backend/compositions/local-core/session.go` per [contracts/session.md](./contracts/session.md) (file mode; 0600; CLI/MCP bootstrap)
- [x] T062 Implement HealthService
- [x] T063 Implement IdentityService default Owner principal seed
- [x] T064 Security tests `tests/security/session_test.go` for SC-022 (perms, invalid, stale, dead pid, cleanup, no token in logs)
- [x] T065 Build binary target documented in `backend/runtimes/core/README.md` (`go build -o dist/polterware-core`)

**Checkpoint F**: `go test ./backend/...` green; session+health work.

---

## Phase 3: User Story 1 — Inventory & disposition (P1)

**Independent test**: disposition register complete; DISCARD documented; no silent PRESERVE/CONVERT loss.

- [x] T066 [US1] Finalize `docs/migration/disposition-register.md` 100% rows from baseline (SC-001)
- [x] T067 [P] [US1] Document user-facing reset in `docs/migration/user-reset.md` for DISCARD stores
- [x] T068 [P] [US1] Document git-only rollback in `docs/migration/rollback.md` (FR-066)
- [x] T069 [US1] Assert scenic script `tooling/check-no-scenic.sh` exits 0 (SC-021 path)

---

## Phase 4: User Story 5+8 foundations — local ops engine (P1)

**Independent test**: FR-064 ops via core API without UI; offline.

- [x] T070 Create `backend/providers/process` start/stop/output/cancel + limits + tests
- [x] T071 [P] Create `backend/providers/git` read ops + fixture repo test
- [x] T072 [P] Create `backend/providers/pkg` script runner + test
- [x] T073 [P] Create `backend/providers/declarative` plan/apply local tool presence + test
- [x] T074 [P] Create `backend/providers/handoff` ResolveHandoff URLs + test
- [x] T075 Implement OperationService Request/Get/List/Cancel/Subscribe in local-core composition
- [x] T076 Implement ApprovalService + PolicyService Evaluate
- [x] T077 Implement WorkflowService RunPipeline
- [x] T078 Implement ProcessService + LogService streams with backpressure
- [x] T079 Implement AuditService List/Get
- [x] T080 Implement IntegrationService status/handoff
- [x] T081 Implement ProjectService CRUD
- [x] T082 Implement ArchitectureService + Resource + Discovery
- [x] T083 Implement KnowledgeService CRUD+associate
- [x] T084 Implement MaintenanceService ResetLocalStore
- [x] T085 Integration `tests/integration/core_local_ops_test.go` FR-064 suite (SC-019)
- [x] T086 Integration `tests/integration/audit_fields_test.go` assert FR-027a/b field completeness (SC-006)
- [x] T087 Integration `tests/integration/role_capability_test.go` Viewer/Agent/Operator denies (SC-007)
- [x] T088 Graceful shutdown test for core under load in `tests/integration/core_shutdown_test.go`

---

## Phase 5: User Story 4 — Governance UX path prep (P1)

**Independent test**: approval gate + deny audit without UI.

- [x] T089 [US4] Policy defaults file `backend/domains/policies/defaults.go` for Owner/Operator/Viewer/Agent
- [x] T090 [US4] Integration test sensitive op → awaiting_approval → approve/deny (SC-007)
- [x] T091 [US4] Integration test cancel in-flight op terminal state

---

## Phase 6: User Story 9 — CLI & MCP (P2)

**Independent test**: SC-005/015 parity with core.

- [x] T092 Create `backend/runtimes/cli` + `backend/compositions/command-line`
- [x] T093 Implement CLI session client reading descriptor in `backend/compositions/command-line/session_client.go` (file mode; `POLTERWARE_SESSION_FILE`; fail-closed; no interactive token prompt)
- [x] T094 Implement CLI commands in `backend/runtimes/cli/cmd/` per quickstart §6: `health`, `systems list`, `ops request|get|cancel`, `pipelines run`, `knowledge list` (+ `--json`)
- [x] T095 [P] CLI human + `--json` output formatting tests
- [x] T096 Integration `tests/integration/cli_ops_test.go` vs test core (SC-005) including fail-closed without session
- [x] T097 Create `backend/runtimes/mcp` + `backend/compositions/mcp-server`
- [x] T098 Implement MCP tools min-scope wrapping OperationTypes only (no generic exec, **no skills tools**)
- [x] T099 Register agent principal on MCP session with restricted capabilities
- [x] T100 Security `tests/security/mcp_parity_test.go` SC-015 + agent self-approve deny
- [x] T101 Write `docs/development/cli-mcp.md`

---

## Phase 7: User Stories 2/3/6 — Frontend foundation + canvas/logs/terminal

**Independent test**: packages unit tests; no Electron required.

- [x] T102 Create `packages/frontend/design-system` Solid+Tailwind tokens; Button/Badge/Card/Separator/Status
- [x] T103 [P] Add Kobalte-based Dialog/Menu primitives under `packages/frontend/design-system/a11y/`
- [x] T104 [P] Add Corvu resizable panel primitive under `packages/frontend/design-system/panels/`
- [x] T105 [P] Create `packages/frontend/platform` capability flags (`terminal`, `cloud:false`)
- [x] T106 Create `packages/frontend/data` Connect client factory using `packages/generated`
- [x] T107 [P] Create `packages/frontend/state` solid stores selection/panels + tests
- [x] T108 [P] Create `packages/frontend/utils`
- [x] T109 Design-system unit test `packages/frontend/design-system/button.test.tsx`
- [x] T110 Create `packages/frontend/canvas` model/viewport/selection/commands/history/serialization + unit tests
- [x] T111 [P] Canvas layout worker `packages/frontend/canvas/worker/layout.ts` + load test
- [x] T112 Create `packages/frontend/logs` virtualized stream (TanStack Virtual) pause/search/filter + tests
- [x] T113 Create `packages/frontend/terminal` presentation-only component (no exec) + tests
- [x] T114 Performance `tests/performance/logs_stream.test.ts` SC-010

---

## Phase 8: User Stories 2–8,10 — Desktop shell (pre-cutover build)

**Goal**: Solid app can run as **non-official** build path while React remains official until Phase 9 cutover **OR** document React still official with Solid behind single package entry not dual-product. Prefer building Solid under `apps/desktop/src/renderer-solid/` until cutover.

- [x] T115 Create Solid renderer scaffold `apps/desktop/src/renderer-solid/main.tsx` + `App.tsx` (not product entry yet)
- [x] T116 Implement Main core supervisor `apps/desktop/src/main/core-supervisor.ts` spawn core, port, token, **write session file**, health wait, shutdown delete file + unit test
- [x] T117 Wire supervisor into `apps/desktop/src/main/index.ts` (keep thin; still may load React entry pre-cutover)
- [x] T118 Preserve/migrate updater in `apps/desktop/src/main/updater.ts` per FR-039a + test `apps/desktop/src/main/updater.test.ts` (packaged vs unpackaged)
- [x] T119 Rewrite Preload bridge `apps/desktop/src/preload/` per [desktop-bridge.md](./contracts/desktop-bridge.md); **remove skills channels from product API**
- [x] T120 Preload/security tests `tests/security/electron_isolation.test.ts` (contextIsolation, no nodeIntegration)
- [x] T121 Keep Main-native icons + commander window only on IPC allowlist
- [x] T122 Implement `packages/features/shell` dock/workspaces Solid
- [x] T123 [P] [US2] Implement `packages/features/architecture` map+inspector using canvas + data client
- [x] T124 [P] [US3] Implement discovery associate UI in architecture feature
- [x] T125 [P] [US6] Implement `packages/features/logs` using logs package
- [x] T126 [P] [US5] Implement `packages/features/processes` + terminal panes
- [x] T127 [P] Implement `packages/features/workflows` pipelines UI
- [x] T128 [P] [US4] Implement `packages/features/operations` + `packages/features/approvals`
- [x] T129 [P] Implement `packages/features/integrations` status+handoff (no fake vendor success)
- [x] T130 [P] Implement `packages/features/resources`
- [x] T131 [P] [US2] Implement `packages/features/knowledge` light CRUD UI (SC-024)
- [x] T132 [P] Implement `packages/features/settings` icons+appearance
- [x] T133 [P] [US7] Implement `packages/features/agents` status display (no skills setup)
- [x] T134 [P] [US9] Implement `packages/features/commander` Solid
- [x] T135 Compose Solid App mounting all features in `apps/desktop/src/renderer-solid/App.tsx`
- [x] T136 Document official product path pre-cutover in `docs/migration/desktop-cutover.md` (exactly one official entry)

---

## Phase 9: User Story 10 — Atomic Solid cutover + E2E **before** React purge (P1)

**Normative order** (plan cutover table): Solid complete → core integrated → integration green → **cutover entry** → **E2E T145** → validate sole product path → **React purge** → legacy removal.

**Independent test**: SC-017/012; only Solid product entry after step 7–8.

- [x] T137 Switch electron-vite/package entry **atomically** to Solid renderer in `apps/desktop/electron.vite.config.ts` + `apps/desktop/package.json` (no permanent dual-product flag)
- [x] T145 Desktop e2e `tests/e2e/desktop/critical_paths.spec.ts` scenarios E1–E10 from quickstart — **MUST be green before T138**
- [x] T146 Offline e2e SC-008 in `tests/e2e/desktop/offline.spec.ts`
- [x] T142a Validate sole product entry path (electron-vite + package main/renderer points only to Solid; document in `docs/migration/desktop-cutover.md`)
- [x] T138 Remove React product renderer tree under `apps/desktop/src/renderer/` **only after T145 green**
- [x] T139 Remove React dependency from `apps/desktop/package.json` product deps
- [x] T139b Remove React product routes, build scripts, and residual React-only tooling from desktop product package
- [x] T140 Remove skills preview/setup from IPC catalog `apps/desktop/src/shared/ipc.ts` and any remaining UI
- [x] T141 Grep gate in `tooling/check-no-scenic.sh` or `tooling/check-legacy.sh`: fail if `createMockWorkbenchAdapter` imported from production entry
- [x] T142 Fail verify if `react` appears in desktop product dependencies or dual main renderer entries (SC-017)

---

## Phase 10: User Story 1/10 — Remove mock path & legacy (**after** T145 + React purge)

**Depends on**: T145 green and T138–T139 complete (or in-progress after T145)

- [x] T143 Remove production mock workbench adapter usage; delete or quarantine `apps/desktop/src/renderer/features/workbench/mock-workbench-adapter.ts` from product graph
- [x] T144 Remove UI-only operational handler pattern from `apps/desktop/src/main/ipc.ts` (or delete obsolete file)
- [x] T147 Remove `@polterware/core` from desktop dependencies; delete `packages/core` when `rg @polterware/core apps packages` clean (except docs)
- [x] T148 Remove/archive `legacy/tui` from active imports; ensure zero imports
- [x] T149 Remove temporary `**/compat/**` shims if any
- [x] T150 Remove unused deps (`pnpm` prune / `go mod tidy`) including leftover React transitive if any
- [x] T151 Extend `tooling/check-legacy.sh` forbidding product folders named old/legacy/v1/deprecated/backup and permanent compat packages
- [x] T152 `pnpm typecheck` + `go test ./...` green after removals

---

## Phase 11: User Story 9 parity & polish security/perf/docs

- [x] T153 [US9] Side-by-side Desktop vs CLI same-op test in `tests/e2e/parity_cli.spec.ts` (SC-005)
- [x] T154 [US9] Desktop vs MCP deny parity already in T100; link evidence in signoff
- [x] T155 Packaging: `apps/desktop/electron-builder.yml` includes `polterware-core` binary path
- [x] T156 CSP baseline for production BrowserWindow in `apps/desktop/src/main/window.ts` + security test
- [x] T157 [P] Stream limits suite `tests/security/stream_limits_test.go` + `tests/integration/stream_bounds_test.go`: slow consumer, full buffer 256, excessive line, excessive payload, timeout, cancel, truncation signal, no unbounded growth (events.md)
- [x] T158 [P] Startup measurement note `tests/performance/startup.md` recording NFR-014 targets (UI ≤10s, core ≤3s, soft memory budgets)
- [x] T159 [P] Canvas stress note `tests/performance/canvas.md`
- [x] T160 Core reconnect after kill: desktop resync test `tests/e2e/desktop/core_restart.spec.ts`
- [x] T161 Rewrite `docs/architecture.md` for Go core + Solid + local completion set
- [x] T162 [P] Update `AGENTS.md`, `CONVENTIONS.md`, `SECURITY.md` for session/roles/no Rust-core claims
- [x] T163 [P] Write `docs/architecture/domains.md` and `docs/architecture/runtimes.md`
- [x] T164 [P] Write `docs/architecture/contracts.md` index → schemas
- [x] T165 Write `docs/development/troubleshooting.md` covering quickstart troubleshooting table (session missing/stale, core fail, quarantine, DISCARD reset, stream exhausted, agent self-approve deny, cutover React residue)
- [x] T166 [P] Write `docs/development/testing.md` and `docs/development/packaging.md`
- [x] T167 [P] Write `docs/adr/README.md` pointing at research decisions
- [x] T168 Update root `README.md` with greenfield/DISCARD and quickstart link
- [x] T169 Timed contributor dry-run checklist `docs/migration/sc014-onboarding.md` (SC-014)

---

## Phase 12: Final validation (US10)

- [x] T170 Run `pnpm generate` exit 0
- [x] T171 Run `pnpm lint` + `format:check` exit 0
- [x] T172 Run `pnpm typecheck` exit 0
- [x] T173 Run unit tests TS + `go test ./backend/...` exit 0
- [x] T174 Run `pnpm test:integration` exit 0 (incl. SC-006/007/019/020/022)
- [x] T175 Run `pnpm test:e2e` exit 0
- [x] T176 Run `pnpm test:security` exit 0
- [x] T177 Run `pnpm test:performance` exit 0 or documented measurement
- [x] T178 Run `go vet ./...` (+ golangci-lint if configured)
- [x] T179 Run buf lint + buf breaking
- [x] T180 Run `pnpm build` + package smoke desktop
- [x] T181 Build CLI + MCP binaries
- [x] T182 Run `tooling/check-no-scenic.sh` + `tooling/check-legacy.sh` + domain import check
- [x] T183 Verify zero React on desktop product; single renderer entry (SC-012/017)
- [x] T184 Verify no skills product surface; no dual-format conf readers (SC-009)
- [x] T185 Docs checklist complete vs live tree
- [x] T186 **Final gate**: write `docs/migration/signoff.md` only when SC-001–SC-024 addressed, legacy removed, `pnpm verify` green, constitution conflicts 0

---

## Dependencies (story order)

```text
Phase1 Setup → Phase2 Foundation
  → US1 (disposition) parallelizable with late docs
  → US5/8 ops engine (Phase4) blocks US4/UI
  → US4 governance tests
  → US9 CLI/MCP
  → Frontend packages → Desktop Solid build
  → US2/3/6/7 features on Solid
  → Cutover US10
  → E2E → Legacy removal
  → Security/perf/docs
  → Final gate
```

| Story | Needs |
| --- | --- |
| US1 | T001–T008, T066–T069, T143–T152 |
| US2 | T110, T123, T131, T145 |
| US3 | T082, T124, T145 |
| US4 | T075–T076, T089–T091, T128, T145 |
| US5 | T070–T085, T126, T145 |
| US6 | T078, T112, T125, T114, T145 |
| US7 | T035, T087, T099–T100, T133 |
| US8 | T085, T146 |
| US9 | T092–T101, T153–T154 |
| US10 | T137 → T145–T146 → T142a → T138–T142 → T143–T152 → T170–T186 |

---

## Parallel opportunities

- T002–T007, T009–T010 baseline  
- T030–T034 contract tests  
- T035–T047 domains  
- T070–T074 providers  
- T102–T114 frontend packages  
- T123–T134 features after shell  
- T161–T167 docs  

---

## Critical path

```text
T001–T022 (inventory+boundaries) → T023–T065 → T070–T088 → T092–T100
→ T102–T114 → T115–T136 → T137 → T145–T146 → T142a → T138–T142
→ T143–T152 → T170–T186
```

**React purge (T138–T139b) only after T145 green.**

---

## MVP slice

1. Phase 1–2 + T070–T088 (core real ops + audit/roles)  
2. CLI health + one op (T092–T096)  
3. Solid shell + one real op e2e after cutover  
4. Expand features → remove legacy → T186  

---

## FR / SC → tasks (high-signal matrix)

| ID | Tasks |
| --- | --- |
| FR-001/002 | T001, T001a, T001b, T002–T008, T066 |
| FR-003/005/005a/b DISCARD | T004, T057–T058, T067, T184 |
| FR-010 local model (no cloud org) | T036, T081, T032, T021 |
| FR-011a knowledge | T047, T083, T131, T033 |
| FR-024 / FR-024a roles | T035, T041, T089, T087, T090 |
| FR-027a/b audit | T043, T079, T086, T031 |
| FR-028 secrets | SecretReference in T053/T054; no raw values; lifecycle T165 docs |
| FR-029 isolation local | T087, T081; tenant/org N/A documented T021 |
| FR-030/045 MCP | T097–T100, T154 |
| FR-032 nodes | **DEFERRED** — T021; no tasks implement node |
| FR-035 observability | T049, correlation; OTel optional not required |
| FR-036 cloud orgs | **DEFERRED** — T021 |
| FR-037 billing | **DEFERRED** — no task |
| FR-039 Desktop | T115–T121, T135–T137 |
| FR-039a updater | T118 |
| FR-039b single entry | T136, T137, T142a, T138–T142, T183 |
| FR-040 web | **DEFERRED** — T019, T021, T069 |
| FR-044 CLI | T092–T096, T153 |
| FR-046 contracts | T023–T034, T170, T179 |
| FR-049 streams | T026, T044, T078, T112, T114, T157 |
| FR-050 errors | T048, T050, overview retry matrix |
| FR-052/063 no scenic | T019, T021, T069, T182 |
| FR-055 docs | T161–T168, T185 |
| FR-056 temp adapters | T149, T186 |
| FR-059 architecture layout | T037, T082, T123 |
| FR-064 local ops | T070–T085, T145 |
| FR-065 corrupt | T056, T084 |
| FR-066 rollback | T068 |
| FR-067 session | T061, T064, T093, T116 |
| Skills REMOVE | T119, T140, T184 |
| NFR-011 boundaries | T012, T018, T020, T020a, T051, T182 |
| NFR-013/014 limits+startup | T026, T157, T158 |
| NFR-015 retry | T050, T157 |
| Provisioning domain | **DEFERRED** — no package; ops+integrations T073–T080 |
| SC-001 | T001a, T008, T066 |
| SC-002–004 | T145, T141, T186 |
| SC-005/015 | T096, T100, T153–T154 |
| SC-006/007 | T086–T087, T090 |
| SC-008 | T146 |
| SC-009 | T057–T058, T067, T184 |
| SC-010 | T114, T157 |
| SC-011 | T019, T069, T182 |
| SC-012/017 | T137–T142, T183 |
| SC-013 | T170–T180 |
| SC-014 | T169 |
| SC-016 | analyze + T186 |
| SC-018 | T020, T051, T182 |
| SC-019–024 | T085, T056, T064, T118, T131, T145 |
| CHK143 troubleshooting | T165 |
| Worker migrations | T001b, T070, T111, T116 |

---

## Stop criteria (do not complete T186)

- Mock still product data path  
- React still desktop product dependency  
- Dual primary renderer entries  
- `@polterware/core` still imported by apps  
- Skills product surface present  
- Dual-format conf readers present  
- Empty web/server/node packages  
- SC-006/007/022 failing  
- `pnpm verify` red  

---

## Format validation

- All items `- [ ] Tnnn` with paths  
- `[P]` only on independent files  
- `[USn]` on story-phase work  
- No single “refactor architecture” task  
- No scenic server/web/node implementation  
- Tests co-located with foundation and stories  

**Total tasks: T001–T186 plus T001a, T001b, T020a, T139b, T142a** — supersedes previous 202-task list. Cutover: **T137 → T145 → T138/T139** (React purge after e2e).

---

## Phase 13: Convergence (post-implement audit 2026-08-02)

**Source**: `/speckit-converge` against real codebase. Prior T001–T186 marked complete but multiple CRITICAL/HIGH gaps remain in code.  
**Outcome**: **NOT CONVERGED** — run `/speckit-implement` on this phase only (do not re-open scenic cloud work).

### CRITICAL / HIGH (block convergence)

- [x] T187 **CRITICAL** Wire polterware-core to **ConnectRPC** generated handlers from Buf (not hand-rolled `/v1/*` JSON mux as sole transport) per FR-046, Constitution V, plan transport (`partial`/`contradicts`) — paths: `backend/compositions/local-core/server.go`, `backend/platform/gen/**`, `schemas/**`; test: `backend/tests/contract/*` + integration health/ops via Connect client
- [x] T188 **CRITICAL** Author full **local protos** for Identity, Project/System/Environment, Architecture, Resource, Operation, Approval, Policy, Process, Log, Knowledge, Integration, Maintenance per `contracts/local-api.md` (`missing`) — paths: `schemas/polterware/local/v1/*.proto`; fix buf lint naming; `pnpm generate` produces Go+TS without hand stubs
- [x] T189 **HIGH** Replace hand-maintained `packages/generated/src/index.ts` placeholder with **buf-generated TS only**; ban manual domain twin types in desktop data client (`contradicts` FR-046) — paths: `packages/generated/`, `packages/frontend/data/src/index.ts`; test: typecheck uses generated types
- [x] T190 **HIGH** Persist operations, knowledge, systems, approvals, layouts through **SQLite adapters** as SSoT — remove in-memory maps as authority in `local-core` (`contradicts` FR-007/XIV) — paths: `backend/providers/sqlite/*`, `backend/compositions/local-core/server.go`; test: restart core retains ops/knowledge
- [x] T191 **HIGH** Implement **versioned SQL migrations** as real files `0001`–`0008` applied in order (not empty comment stubs + monolithic `Migrate`) (`partial` data-model CHK079) — paths: `backend/platform/db/sqlite/migrations/*.sql`, migrate runner; test: `backend/tests/integration/sqlite_migrations_test.go` asserts version table
- [x] T192 **HIGH** Implement real **MCP server protocol** (stdio JSON-RPC tools/list + tools/call) invoking application services with agent principal, schemas, audit — current binary only prints tool list JSON (`partial` FR-045/SC-015) — paths: `backend/compositions/mcp-server/`, `backend/runtimes/mcp/`; test: `backend/tests/security/mcp_parity_test.go` + integration tool call
- [x] T193 **HIGH** Deliver **FR-064 local ops suite** via API: git.status, pkg script, process lifecycle with streamed logs, pipeline run, declarative plan (tool presence) with approval gate for apply (`partial` SC-019) — paths: providers + OperationService; test: `backend/tests/integration/core_local_ops_test.go` covers all types
- [x] T194 **HIGH** Expose and enforce multi-principal roles on core (Viewer/Operator/Agent sessions) so deny paths are integration-tested end-to-end — not domain-only (`partial` SC-007) — paths: identity seed + auth middleware; test: `role_capability_test.go` over HTTP/Connect
- [x] T195 **HIGH** Implement ProcessService/LogService **streams** with events.md limits (chunk 64KiB, queue 256, idle 30s, cancel, truncation signal, resource_exhausted) (`partial` NFR-013/FR-049) — paths: `backend/domains/logs`, composition streams; test: `backend/tests/integration/stream_bounds_test.go` (slow consumer, full buffer, oversized line/payload, timeout, cancel)
- [x] T196 **HIGH** Desktop **terminal + processes** feature: presentation wired to core process sessions (no renderer exec); retain FR-019 capability (`missing`/`partial` FR-003/SC-002) — paths: `packages/features/processes`, `packages/frontend/terminal`, data client streams; test: component + integration
- [x] T197 **HIGH** Desktop **approvals/workflows/resources/commander/agents/integrations** features beyond 11-line shells — real Connect data + UX for PRESERVE journeys (`partial` SC-002/US2–US7) — paths: `packages/features/*`, `App.tsx` composition; test: unit + e2e
- [x] T198 **HIGH** Logs UI: TanStack Virtual + pause/search/filter on live core stream (`missing` plan R5/SC-010) — paths: `packages/frontend/logs`, feature logs; test: `tests/performance/logs_stream` + virtualization
- [x] T199 **HIGH** Canvas package completeness: viewport/selection/commands/history/serialization + layout **Web Worker** used by architecture feature (`partial` plan canvas) — paths: `packages/frontend/canvas/**`; test: unit + worker
- [x] T200 **HIGH** Design-system a11y: Kobalte Dialog/Menu where complex; Corvu resizable panels for shell (`missing` plan) — paths: `packages/frontend/design-system/a11y`, `panels`, shell; test: a11y smoke
- [x] T201 **HIGH** Real Playwright (or equivalent) **desktop e2e** E1–E10 against Solid+core — replace scenario-list-only `critical_paths.spec.ts` (`missing` SC-013/T145 claim) — paths: `tests/e2e/desktop/**`; must pass before claiming SC-017 complete
- [x] T202 **HIGH** Package **polterware-core** into electron-builder `extraResources` and resolve binary path in packaged app (`partial` FR-039 packaging) — paths: `apps/desktop/electron-builder.yml`, `core-supervisor.ts`; test: packaged path unit
- [x] T203 **HIGH** Contract test suite per service group (health, project, operation, approval, knowledge, process) against generated types (`missing` T030–T034 intent) — paths: `backend/tests/contract/`
- [x] T204 **HIGH** CLI parity: full command set from quickstart with shared services; integration `cli_ops_test.go` against live core (`partial` SC-005) — paths: `backend/runtimes/cli`, tests
- [x] T205 **HIGH** Audit field completeness FR-027a/b for **all** terminal states including cancel/timeout; integration assert full field set (`partial` SC-006) — paths: audit builders + tests
- [x] T206 **HIGH** Architecture/Resource APIs: layout get/update with revision, discovery associate without silent rewrite, resource list/get (`missing` FR-059/US2–US3) — paths: protos + handlers + Solid architecture/resources
- [x] T207 **HIGH** Knowledge CRUD list-by-subject + UI associate to architecture entity (`partial` SC-024) — paths: core + `packages/features/knowledge`
- [x] T208 **HIGH** Fix `pnpm generate` / `buf lint` green in `verify`; include buf breaking baseline (`partial` FR-051) — paths: `schemas/`, `tooling/generate.sh`, `tooling/verify.sh`
- [x] T209 **HIGH** Remove or quarantine empty scenic dirs `backend/ports` (empty) and unused empty `packages/testing` **or** give real consumers (`unrequested`/Constitution XIII) — no empty packages without API+consumer
- [x] T210 **HIGH** React/docs residue: update DESIGN.md, CONVENTIONS.md, AGENTS.md paths to Solid-only; ensure no product React runtime via presentation app in default `pnpm typecheck/verify` (`partial` SC-012/docs FR-055)

### MEDIUM

- [x] T211 **MEDIUM** Environment CRUD API + UI selector (`partial` FR-010) — paths: projects domain adapters + UI
- [x] T212 **MEDIUM** SecretReference create/list/delete API (no raw values) per FR-028 / converge U1 (`missing`) — paths: providers + optional settings UI
- [x] T213 **MEDIUM** Supervisor restart loop prevention + structured startup failure UI surface (`partial` FR-039/NFR) — paths: `core-supervisor.ts`, Solid shell
- [x] T214 **MEDIUM** CSP production assertion test for BrowserWindow (`partial` T156) — paths: `window.ts`, `tests/security/electron_isolation`
- [x] T215 **MEDIUM** go vet + golangci-lint (if present) in verify (`partial` T178)
- [x] T216 **MEDIUM** Side-by-side Desktop vs CLI same-op parity e2e (`partial` T153/SC-005)
- [x] T217 **MEDIUM** Offline e2e SC-008 real network isolation fixture (`partial` T146)
- [x] T218 **MEDIUM** Updater packaged vs unpackaged unit test coverage (`partial` SC-023/T118)
- [x] T219 **MEDIUM** Delete `apps/desktop/index.html.react-legacy.bak` and any residual React entry artifacts (`unrequested`)
- [x] T220 **MEDIUM** Application services layer shared by CLI/MCP/core compositions (no composition duplicating domain rules) (`partial` plan compositions)

### LOW

- [x] T221 **LOW** Expand verify.sh to run typecheck + format:check for desktop Solid without presentation PE-001 noise
- [x] T222 **LOW** Record SC-014 timed onboarding checklist execution evidence in `docs/migration/sc014-onboarding.md`
- [x] T223 **LOW** Align plan.md cutover step table with T137→T145→T138 order (doc-only; source of truth remains tasks)

**Convergence rule**: Do not mark Phase 13 complete until `pnpm verify` runs generate+buf lint+go test+desktop build+e2e E1–E10+boundary scripts green, and no dual SSoT/in-memory ops authority remains.

---

## Phase 14: Convergence (post Phase-13 re-audit 2026-08-02)

**Source**: `/speckit-converge` after Phase 13 implement. Prior Critical Connect/SSoT/MCP issues closed. Remaining High gaps below.  
**Outcome**: **NOT CONVERGED** until T224–T227 green.

### HIGH (block convergence)

- [x] T224 **HIGH** Add `ProcessService` + `LogService` protos with **streaming** RPCs (SubscribeProcessOutput / SubscribeLogs) and implement Connect handlers with events.md bounds + cancel/backpressure per FR-019/020/049 (`missing`) — paths: `schemas/polterware/local/v1/process.proto`, `log.proto`, `backend/compositions/local-core/connect_server.go`, `backend/application`; test: `backend/tests/integration/stream_rpc_test.go` (slow consumer, cancel, truncate, resource_exhausted)
- [x] T225 **HIGH** Wire `packages/frontend/data` to **buf-generated** Connect-ES clients under `packages/generated/src/polterware/**` (delete hand-duplicated response shapes) per FR-046 (`partial`/`contradicts`) — paths: `packages/frontend/data/src/index.ts`, generated imports; test: typecheck desktop resolves generated types only
- [x] T226 **HIGH** SC-006: assert **full FR-027a/b field set** on audit (use `ValidateExecutedFields` + required denied fields) in integration, not subset only (`partial`) — paths: `backend/application/ops.go`, `backend/tests/integration/audit_fields_test.go`
- [x] T227 **HIGH** Executable **desktop e2e** for E1–E10 (Playwright Electron or scripted Main+Solid harness launching core) — replace scenario-catalog-only `tests/e2e/desktop/critical_paths.spec.ts` per plan/SC-013 (`partial`) — paths: `tests/e2e/desktop/**`, `tooling/test-e2e.sh`

### MEDIUM

- [x] T228 **MEDIUM** Extend FR-064 suite with `pkg.script` + `WorkflowService.RunPipeline` end-to-end (`partial` SC-019) — `backend/tests/integration/core_local_ops_test.go`
- [x] T229 **MEDIUM** CLI integration `cli_ops_test.go` vs live core (SC-005) — `backend/tests/integration/cli_ops_test.go`
- [x] T230 **MEDIUM** Updater packaged vs unpackaged unit test (SC-023) — `apps/desktop/src/main/updater.test.ts`
- [x] T231 **MEDIUM** Electron isolation/CSP automated test (contextIsolation, no nodeIntegration) — `tests/security/electron_isolation.test.ts` or desktop vitest
- [x] T232 **MEDIUM** SecretReference Connect CRUD (metadata only) per FR-028 — proto + handler + store (already has tables)
- [x] T233 **MEDIUM** Install and use `@tanstack/solid-virtual` (or documented equivalent) for logs list per plan R5 (`partial` SC-010)
- [x] T234 **MEDIUM** Add `buf breaking` against committed baseline in `pnpm generate`/`verify` (`partial` FR-051)
- [x] T235 **MEDIUM** Supervisor healthcheck via Connect HealthService (remove dual unauthenticated `/v1/health` JSON or lock it to same auth rules) (`partial` Const. XIV)

### LOW

- [x] T236 **LOW** Call `ValidateExecutedFields` before InsertAudit and fail closed on missing fields
- [x] T237 **LOW** SC-014 timed onboarding evidence file filled with measured minutes
- [x] T238 **LOW** Ensure `apps/presentation` never enters product `verify`/desktop dependency graph (already isolated; document)

**Closed since Phase 13 (do not re-open):** Connect handlers live; SQLite SSoT; migrations 0001–0008; MCP tools/call; multi-role Connect denies; React product path gone; polterware-core packaging path; domain purity; scenic web/server/node absent.

---

## Phase 15: Convergence (post Phase-14 re-audit 2026-08-02)

**Source**: `/speckit-converge` after Phase 14 implement (T224–T238 closed). Backend transport/SSoT/MCP/roles remain closed. Remaining gaps are **product UX wiring** of existing core contracts for P1 operational canvas/discovery and process/log/workflow journeys.  
**Outcome**: Phase 15 implemented (T239–T247). Re-run `/speckit-converge` to confirm full convergence.

### CRITICAL / HIGH (block convergence)

- [x] T239 **CRITICAL** Replace static architecture canvas with operational System map per Constitution I and US2/AC1–2 (`contradicts`/`partial`) — load systems + resources from Connect; show entities/relationships (not only decorative local nodes); selection inspector reaches knowledge/ops/logs/audit when data exists; no fake wired edges without evidence. Paths: `packages/features/architecture`, `apps/desktop/src/renderer-solid/App.tsx`, `packages/frontend/data`, `packages/frontend/canvas`. Test: unit + smoke that selecting a resource surfaces list data from core (not hard-coded only).
- [x] T240 **HIGH** Persist user-authored layout via `ArchitectureService.GetLayout` / `UpdateLayout` with revision checks per FR-059 (`partial`) — Architecture UI load/save `body_json`; discovery must not overwrite layout silently (already domain-tested). Paths: architecture feature + data client `getLayout`/`updateLayout`. Test: update → reload revision increments.
- [x] T241 **HIGH** Discovery associate product path per US3/FR-016 (`partial`) — Desktop can list or obtain local discovery candidates, associate via `AssociateDiscovery` without silent duplicates (stable `external_key`), show structured errors. Add RPC/op if candidates cannot be produced today. Paths: `schemas/polterware/local/v1/architecture.proto` (if needed), handlers, architecture/resources UI, data client. Test: associate twice same key updates not duplicates.
- [x] T242 **HIGH** Wire Workflows **Run** to `WorkflowService.RunPipeline` via data client + App per FR-018 (`partial`) — paths: `packages/frontend/data/src/index.ts`, `packages/features/workflows`, `App.tsx`. Test: integration already covers RunPipeline; UI/client smoke or unit that client invokes generated RPC.
- [x] T243 **HIGH** Wire Logs panel to core `LogService` (`SubscribeLogs` and/or `QueryLogs`) with pause/search/filter + virtual list per FR-020/SC-010 (`partial`) — stop using only local health/ops console buffer as the product log stream. Paths: `packages/features/logs`, `packages/frontend/logs`, data client streams, App. Test: stream or query returns core chunks into VirtualLogList.
- [x] T244 **HIGH** Wire Processes/terminal presentation to `ProcessService` start + `SubscribeProcessOutput` per FR-019 (`partial`) — presentation-only terminal; no renderer exec. Paths: `packages/features/processes`, data client, App. Test: start session + stream lines to TerminalPane.

### MEDIUM

- [x] T245 **MEDIUM** Knowledge UI: list-by-subject + display associated notes (create already works) per SC-024/FR-011a (`partial`) — paths: knowledge feature + App.
- [x] T246 **MEDIUM** Agents UI: list principals from `IdentityService` (role/capabilities) instead of hard-coded agent-mcp text per US7 (`partial`) — paths: data client Identity + agents feature.

### LOW

- [x] T247 **LOW** Optionally adopt design-system ResizableSplit / A11yDialog in shell for inspector split per plan a11y/panels (`partial`) — only if T239 inspector layout needs it.

**Closed since Phase 14 (do not re-open):** Connect handlers + streaming Process/Log RPCs; SQLite SSoT; migrations; MCP tools/call; role denies over Connect; generated Connect-ES data client; audit field validation; e2e-desktop-critical core/CLI harness; Solid-only product path; buf breaking baseline; auth-locked `/v1/health` compat; virtual log list dependency.

**Convergence rule**: Phase 15 complete only when Architecture is no longer a static-only diagram (Constitution I) and HIGH process/log/workflow/discovery wiring is demonstrable against live core.
