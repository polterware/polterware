# Implementation Plan: Integration Detail Panel

**Branch**: `012-integration-detail` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-11 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/012-integration-detail/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `004-operator-product-surfaces` (Integrations Connected/Catalog + early detail), `010-compound-data-table` (DataTable law), design-system `DetailPanel` / `StatusChip` / `Tabs`, shell workspace-mocks

**Risk control**: No live OAuth. No real secret vault. No second Integrations detail page. No provider-console clone. Mock-first honesty only.

**Implementation status (as of plan)**: Mock production-ready UI landed in-tree during specify session — plan freezes design SSOT and remaining polish/live-swap gaps for `/speckit-tasks` / converge.

---

## Summary

Elevate **Integration entity detail** to production operator quality inside the single global bottom `DetailPanel`: seven **local** sections (Overview · Resources · Credentials · Sync · Permissions · Webhooks · Advanced), fixture-backed data, write-only credential create, honest health/stale/error/rate-limit states, and disconnect that does **not** claim destruction of external provider resources.

**Technical approach** (see research):

1. **Keep page tabs** `connected` | `catalog` only — detail sections never become PageToolbar tabs.
2. **Extract `IntegrationDetail`** composition (`DetailPanel.Content` + Tabs rail + sectional grid) consumed by `IntegrationsView`.
3. **Workspace mock SSOT** for connection, resources, credentials (refs only), capabilities, sync runs, webhooks — extend types in `feature-shell` workspace-mocks.
4. **Design-system grammar only** — DetailPanel sections/metrics/metadata, DataTable, StatusChip, Field+Input for secret create.
5. **Mock mutations** — enable/disable, disconnect, sync toast, credential create (clear secret); no fake live provider success without toast honesty.
6. **Hand-off** — external console URL / callback; resource row → inventory id when present.
7. **Tests** — fixture richness + secret non-display invariants + detail-fixture-coverage.

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9 (renderer features)

**Primary Dependencies**:
- `@polterware/feature-integrations` (owner)
- `@polterware/feature-shell` (workspace-mocks types + fixtures)
- `@polterware/frontend-design-system` (DetailPanel, DataTable, StatusChip, Tabs, Field, Dialog, Button, Badge)
- `@polterware/frontend-charts` (DitherSegmentedBar for coverage distributions only)
- `@polterware/frontend-state` (detailPanelStore collection registration)
- `@polterware/feature-architecture/icons` (NodeIcon + provider icon key)

**Storage**: Fixture/mock only for this delivery. No new SQLite tables, protobuf, or secret persistence. Future live swap targets integrations domain queries without a second detail UX.

**Testing**: Vitest (via Desktop `vitest.ui.config.ts` aliases): integration fixture unit tests; `apps/desktop` detail-fixture-coverage; static typecheck on feature packages. No `pnpm dev` unless operator requests visual check.

**Target Platform**: Desktop Electron renderer — Integrations project destination; browser-safe feature packages only

**Project Type**: Desktop product feature (Solid feature package)

**Performance Goals**:
- Detail open on row activate feels immediate (&lt; 100ms perceived with fixtures)
- Section tab switches instant (local signal)
- Tables OK for fixture sizes (&lt; ~50 rows per section without mandatory virtualization)

**Constraints**:
- Renderer: no unrestricted Node/OS; secrets write-only never re-shown
- AGENTS/DESIGN LAW: global DetailPanel only; StatusChip for status; no Actions columns; no Card chrome in operational detail sections
- Integration Tabs rail allowed under header before grid (DESIGN exception)
- Mock honesty: never claim live OAuth, live vault, or full health after partial/failed sync
- Disconnect copy non-destructive to external resources (`docs/spec.md` §112)

**Scale/Scope**: One entity detail surface for Integrations Connected collection; Catalog remains connect/discover; Sync Center / Conflict Resolve remain secondary (out of detail completeness requirement)

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Integrations hub inspect + limited operate (sync/reconnect); not provider console clone |
| **II. Domain Before Framework** | PASS | Fixture DTOs stand in for domain queries; layout capability-oriented, not `if provider ===` for structure |
| **III. Runtime Separation** | PASS | Renderer feature only; no Main domain rules |
| **IV. Monorepo deps** | PASS | integrations → shell mocks + design-system + state + architecture icons; no cycles |
| **V. Contracts-First** | PASS | UI contract in `contracts/integration-detail-ui.md`; no dual protobuf for mock |
| **VI. Security** | PASS | Credential refs only; write-only secret; never re-display; signing secret configured/missing only |
| **VII. Safe execution** | PASS | Mock mutations + toasts; disconnect non-destructive; no silent cloud destroy |
| **VIII. Local-first** | PASS | Full mock detail offline without live providers |
| **IX. Performance** | PASS | No Main work; scroll in DetailPanel.Body only |
| **X. Resilience** | PASS | error/disabled/disconnected/rate-limit/stale first-class UI states |
| **XI. Observability** | PASS | Operator toasts for mock actions; no fake APM claims |
| **XII. Testability** | PASS | Fixture + secret invariant tests without Electron runtime |
| **XIII. Real code** | PASS | Extend existing packages; no scenic new package |
| **XIV. SSOT** | PASS | One IntegrationDetail composition; no dual page+panel |
| **XV. Big-bang** | PASS | Incremental on refactor branch |
| **XVI. Docs** | PASS | Spec + plan + contracts + data model |

**Hard stops**: No security/dependency/SSOT violations.  
**Post-Phase 1 re-check**: PASS — design keeps mock ports, secret hygiene, single detail host, local sections only.

---

## Project Structure

### Documentation (this feature)

```text
specs/012-integration-detail/
├── plan.md                 # This file
├── research.md             # Phase 0
├── data-model.md           # Phase 1
├── quickstart.md           # Phase 1
├── contracts/
│   └── integration-detail-ui.md
├── checklists/requirements.md
└── tasks.md                # /speckit-tasks — not this command
```

### Source Code (touch list)

```text
packages/features/shell/src/workspace-mocks/
├── types.ts                # WorkspaceIntegration* DTOs
├── data.ts                 # INTEGRATION_*_MOCKS
└── index.ts                # re-exports

packages/features/shell/src/index.tsx   # public exports

packages/features/integrations/src/
├── index.tsx               # IntegrationsView: list + collection register + wire detail
├── integration-detail.tsx  # DetailPanel composition (7 sections)
├── integration-detail.test.tsx
├── webhooks.tsx            # secondary surface; detail owns primary webhook section
├── setup-wizard.tsx        # reconfigure/reconnect entry (existing)
├── provider-icon.ts        # provider mark key for table + header parity
└── package.json

apps/desktop/src/renderer/
└── detail-fixture-coverage.test.ts   # rich fixture assertions

# Out of scope this plan
backend/ polter-core integration APIs
protobuf / Connect-ES generation
live OAuth / secret storage adapters
```

**Structure Decision**:
- **Runtime owner**: Renderer (`packages/features/integrations`)
- **Data**: shell workspace-mocks as temporary adapter/port implementation of “integration detail queries”
- **Contracts**: UI contract only; no new shared protocol package until live API
- **No new packages** — Constitution XIII

---

## Complexity Tracking

> No constitution violations requiring justification.

| Item | Notes |
| --- | --- |
| Mock vs live dual path | Temporary fixture SSOT only; live queries must replace fixture injection without a second detail UX (XIV exit criteria when core lands) |
| Secondary WebhooksTab | Kept for non-detail surfaces; entity detail must not depend on nested page chrome inside DetailPanel |
