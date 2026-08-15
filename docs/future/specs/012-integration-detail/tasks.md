# Tasks: Integration Detail Panel

**Input**: [spec.md](./spec.md) · [plan.md](./plan.md) · [research.md](./research.md) · [data-model.md](./data-model.md) · [contracts/](./contracts/) · [quickstart.md](./quickstart.md)

**Prerequisites**: plan.md (required), spec.md (user stories), research.md, data-model.md, contracts/

**Tests**: Risk-proportional — fixture richness, secret non-display, detail-fixture-coverage. Agents.md: no `pnpm dev`/full build unless operator asks.

**Organization**: Tasks grouped by user story (US1–US7). Mock-first production-ready detail only.

**Constitution / guardrails**:

- Global DetailPanel only (no second page/inspector)
- Page tabs Connected|Catalog; detail sections local only
- Secrets write-only never re-shown
- Disconnect non-destructive to external resources
- StatusChip for status; DataTable for collections; no Actions columns
- No new packages; shell mocks SSOT

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no incomplete dependencies)
- **[Story]**: US1–US7 maps to [spec.md](./spec.md) user stories
- Include exact file paths in every task description

## Path Conventions

- Integrations: `packages/features/integrations/src/`
- Shell mocks: `packages/features/shell/src/workspace-mocks/`
- Desktop fixture coverage: `apps/desktop/src/renderer/detail-fixture-coverage.test.ts`
- App wire: `apps/desktop/src/renderer/App.tsx`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm touch list; no scenic packages

- [x] T001 Re-read touch list in `specs/012-integration-detail/plan.md`; confirm owner packages `packages/features/integrations` and `packages/features/shell` exist — no new packages
- [x] T002 [P] Confirm Connected/Catalog list baseline in `packages/features/integrations/src/index.tsx` and App wiring in `apps/desktop/src/renderer/App.tsx`

**Checkpoint**: Inventory confirmed

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Rich DTOs + fixture SSOT + exports — MUST complete before story UI polish

- [x] T003 Extend `WorkspaceIntegration` and add credential/capability/sync/webhook types in `packages/features/shell/src/workspace-mocks/types.ts` per [data-model.md](./data-model.md)
- [x] T004 Enrich `INTEGRATION_MOCKS` (scopes, lastSuccessfulSync, rate limit, console URL, freshness, environments) in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T005 [P] Add `INTEGRATION_RESOURCE_MOCKS` inventory hand-off ids + additional sample rows in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T006 [P] Add `INTEGRATION_CREDENTIAL_MOCKS` (refs only, no secret fields) in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T007 [P] Add `INTEGRATION_CAPABILITY_MOCKS` per integration in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T008 [P] Add `INTEGRATION_SYNC_RUN_MOCKS` including success/partial/failed in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T009 [P] Add `INTEGRATION_WEBHOOK_MOCKS` with signingSecretConfigured boolean only in `packages/features/shell/src/workspace-mocks/data.ts`
- [x] T010 Export new mocks/types from `packages/features/shell/src/workspace-mocks/index.ts` and `packages/features/shell/src/index.tsx`
- [x] T011 [P] Fixture unit tests for richness + secret non-display in `packages/features/integrations/src/integration-detail.test.tsx`
- [x] T012 Extend `apps/desktop/src/renderer/detail-fixture-coverage.test.ts` for integration credential/capability/sync/webhook coverage

**Checkpoint**: Fixtures compile and tests assert no secret material

---

## Phase 3: User Story 1 — Inspect health and connection facts (P1) 🎯 MVP

**Goal**: Overview in DetailPanel with honest health/stale/error

**Independent Test**: Select GitHub vs Grafana; Overview differs; no second page

- [x] T013 [US1] Create `IntegrationDetail` shell with loading/error/notFound and header icon parity in `packages/features/integrations/src/integration-detail.tsx`
- [x] T014 [US1] Add local detail `Tabs` rail (7 sections) under header before grid in `packages/features/integrations/src/integration-detail.tsx`
- [x] T015 [US1] Implement Overview metrics + Summary StatusChip + Connection metadata + scopes + error section in `packages/features/integrations/src/integration-detail.tsx`
- [x] T016 [US1] Wire `IntegrationsView` collection register + active target → `IntegrationDetail` in `packages/features/integrations/src/index.tsx`
- [x] T017 [US1] Keep page tabs Connected|Catalog only (normalize legacy credentials/sync/webhooks tabs to connected) in `packages/features/integrations/src/index.tsx`

**Checkpoint**: MVP Overview opens from Connected row

---

## Phase 4: User Story 2 — Imported resources (P1)

**Goal**: Resources section DataTable + optional inventory hand-off

- [x] T018 [US2] Resources DataTable (name/kind/environment/status StatusChip) filtered by integrationId in `packages/features/integrations/src/integration-detail.tsx`
- [x] T019 [US2] Row activate calls `onOpenResource(inventoryResourceId)` when provided in `packages/features/integrations/src/integration-detail.tsx`
- [x] T020 [US2] App navigates Resources inventory detail on hand-off in `apps/desktop/src/renderer/App.tsx`

**Checkpoint**: AWS shows resources; Datadog empty honest

---

## Phase 5: User Story 3 — Credential references (P1)

**Goal**: Refs only; write-only create; secret cleared

- [x] T021 [US3] Credentials DataTable (name/scope/status/rotated) in `packages/features/integrations/src/integration-detail.tsx`
- [x] T022 [US3] Add credential dialog with Field + password Input; clear secret on submit/cancel in `packages/features/integrations/src/index.tsx`
- [x] T023 [US3] Session-local mock credential refs appear after create without storing secret in `packages/features/integrations/src/index.tsx` + `integration-detail.tsx`
- [x] T024 [P] [US3] Secret non-display invariant tests in `packages/features/integrations/src/integration-detail.test.tsx`

**Checkpoint**: Create credential never re-shows secret; list gains ref

---

## Phase 6: User Story 4 — Sync freshness (P1)

**Goal**: Sync metrics, runs table, rate-limit honesty, Sync now mock

- [x] T025 [US4] Sync metrics (last attempt/success/interval) + rate-limit section in `packages/features/integrations/src/integration-detail.tsx`
- [x] T026 [US4] Recent sync runs DataTable with result StatusChip tones in `packages/features/integrations/src/integration-detail.tsx`
- [x] T027 [US4] Sync now header + section actions with mock toast in `packages/features/integrations/src/integration-detail.tsx`

**Checkpoint**: Vercel partial/backoff visible; Grafana failed not healthy

---

## Phase 7: User Story 5 — Permissions (P2)

**Goal**: Capability allowed/denied table + coverage bar when meaningful

- [x] T028 [US5] Permissions DataTable + Allowed/Denied StatusChip in `packages/features/integrations/src/integration-detail.tsx`
- [x] T029 [US5] DitherSegmentedBar coverage only when capability count > 0 in `packages/features/integrations/src/integration-detail.tsx`

**Checkpoint**: GitHub/AWS capabilities differ

---

## Phase 8: User Story 6 — Webhooks (P2)

**Goal**: Per-integration webhooks; signing configured/missing only

- [x] T030 [US6] Webhooks section tables + endpoint detail (no secret material) in `packages/features/integrations/src/integration-detail.tsx`
- [x] T031 [P] [US6] Align secondary `WebhooksTab` to workspace mocks in `packages/features/integrations/src/webhooks.tsx`

**Checkpoint**: GitHub webhooks listed; Datadog empty; secret never printed

---

## Phase 9: User Story 7 — Reconnect / disable / revoke / hand-off (P1)

**Goal**: Header actions + Advanced non-destructive disconnect + console hand-off

- [x] T032 [US7] Header actions Sync / Reconfigure|Reconnect / Enable|Disable / Open console in `packages/features/integrations/src/integration-detail.tsx`
- [x] T033 [US7] Local mock disable + disconnect overlays in `packages/features/integrations/src/index.tsx`
- [x] T034 [US7] Advanced disconnect copy non-destructive + console hand-off in `packages/features/integrations/src/integration-detail.tsx`
- [x] T035 [US7] Catalog connected provider → request Connected target in `packages/features/integrations/src/index.tsx`

**Checkpoint**: Disable/disconnect mock; disconnect copy clear

---

## Phase 10: Polish & Cross-Cutting

- [x] T036 [P] Export `IntegrationDetail` from `packages/features/integrations/src/index.tsx`
- [x] T037 Reset detail section to overview when active integration id changes in `packages/features/integrations/src/integration-detail.tsx`
- [x] T038 Avoid double toast on credential create (feature owns toast; App may stay silent or feature skips when parent handles) in `packages/features/integrations/src/index.tsx` / `apps/desktop/src/renderer/App.tsx`
- [x] T039 Run quickstart automated checks (vitest UI config) for integrations + detail-fixture-coverage
- [x] T040 Confirm contracts/docs remain accurate in `specs/012-integration-detail/contracts/integration-detail-ui.md`

---

## Dependencies

```text
Phase 1 → Phase 2 → US1 (MVP)
US1 → US2, US3, US4, US7 (parallel after Overview shell)
US5, US6 parallel after Phase 2
Polish after stories
```

## MVP

US1 (Overview + DetailPanel host) is MVP. Full mock production delivery = US1–US7 + Polish.

## Parallel opportunities

- T005–T009 fixture sets
- US5/US6 after fixtures
- T011/T024/T031 tests and secondary webhooks
