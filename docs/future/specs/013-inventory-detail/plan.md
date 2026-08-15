# Implementation Plan: Inventory Resource Detail

**Branch**: `013-inventory-detail` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/013-inventory-detail/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `004-operator-product-surfaces` (Inventory list + early resource detail), `010-compound-data-table` (DataTable law), design-system `DetailPanel` / `StatusChip`, shell detail collection registration via `frontend-state`

**Risk control**: No live provider discovery/sync. No second resource detail page. No cloud console clone. Mock-first honesty only. Stale/missing ≠ deleted.

**Implementation status (as of plan)**: Mock production-ready UI landed in-tree during specify session — plan freezes design SSOT and remaining polish/live-swap gaps for `/speckit-tasks` / converge.

---

## Summary

Elevate **Inventory resource entity detail** to production operator quality inside the single global bottom `DetailPanel`: identity + lifecycle metrics, provenance/provider context, capability-gated actions, linked architecture, activity, and operator annotations — fixture-backed, design-system grammar only, without leaving Resources → Inventory.

**Technical approach** (see research):

1. **Keep page tabs** Inventory · Nodes · Processes (Provisioning if present from 007) — detail is not a page tab or full-page route.
2. **Elevate `ResourceDetail`** composition (`DetailPanel.Content` + sections grid) consumed by `ResourcesView` for `scopeId: resources:inventory`.
3. **Feature-local fixture SSOT** in `@polterware/feature-resources` (`types.ts`, `fixtures.ts`, `resource-status.ts`) — not shell workspace-mocks (Resources already owns inventory DTOs).
4. **Design-system grammar only** — DetailPanel metrics/sections/metadata, DataTable, StatusChip, Badge for tags, DetailPanel.Action header actions.
5. **Mock mutations** — Refresh toast, quick-action toast/disabled reason, Edit → existing form dialog; Open provider external hand-off only.
6. **Lifecycle honesty** — status union includes `stale` / `missing` with lifecycle notes; never jump to “deleted” copy.
7. **Tests** — resource-status + fixture contract unit tests; desktop detail-fixture-coverage richness.

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9 (renderer features)

**Primary Dependencies**:
- `@polterware/feature-resources` (owner)
- `@polterware/frontend-design-system` (DetailPanel, DataTable, StatusChip, Badge, PageLoading/PageError, pushToast, curated Hugeicons)
- `@polterware/frontend-state` (detailPanelStore collection registration — existing ResourcesView wiring)
- Existing resource table icons (`resource-table-icon`) for header/table parity

**Storage**: Fixture/mock only for this delivery. No new SQLite tables, protobuf, or inventory persistence APIs. Future live swap targets resources domain queries without a second detail UX.

**Testing**: Vitest unit tests in `packages/features/resources` (node env for pure TS); Desktop `vitest.ui.config.ts` for `detail-fixture-coverage.test.ts`. No `pnpm dev` unless operator requests visual check.

**Target Platform**: Desktop Electron renderer — Resources project destination; browser-safe feature packages only

**Project Type**: Desktop product feature (Solid feature package)

**Performance Goals**:
- Detail open on row activate feels immediate (&lt; 100ms perceived with fixtures)
- Scroll only inside `DetailPanel.Body`
- Tables OK for fixture sizes (&lt; ~50 linked nodes / activity rows without mandatory virtualization)

**Constraints**:
- Renderer: no unrestricted Node/OS
- AGENTS/DESIGN LAW: global DetailPanel only; StatusChip for status; no Actions columns; no Card chrome in operational detail sections
- No local Tabs rail required (unlike Integration’s seven sections) — single scroll sections grid
- Mock honesty: never claim live discovery/sync or core inventory authority
- `docs/spec.md` §21 / §111: provenance, lifecycle before delete, annotations survive provider refresh, capabilities gate actions

**Scale/Scope**: One entity detail surface for Inventory collection; Nodes/Processes details out of redesign; no Provisioning wizard work

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Inventory inspect + architecture link + external tool hand-off; not a console clone |
| **II. Domain Before Framework** | PASS | Resource DTOs stand in for domain queries; layout shape-driven, not `if provider ===` for structure |
| **III. Runtime Separation** | PASS | Renderer feature only; no Main domain rules |
| **IV. Monorepo deps** | PASS | resources → design-system + state; no cycles |
| **V. Contracts-First** | PASS | UI contract in `contracts/inventory-detail-ui.md`; no dual protobuf for mock |
| **VI. Security** | PASS | No secrets in inventory fixtures; external hand-off only; no privileged apply |
| **VII. Safe execution** | PASS | Mock mutations + toasts; destructive ops gated unavailable with reason |
| **VIII. Local-first** | PASS | Full mock detail offline without live providers |
| **IX. Performance** | PASS | No Main work; scroll in DetailPanel.Body only |
| **X. Resilience** | PASS | stale/missing/degraded/unknown/syncing first-class UI states |
| **XI. Observability** | PASS | External ids + source honesty; mock toasts; no fake APM claims |
| **XII. Testability** | PASS | Status helpers + fixture contract tests without Electron runtime |
| **XIII. Real code** | PASS | Extend existing `feature-resources`; no scenic new package |
| **XIV. SSOT** | PASS | One ResourceDetail composition; no dual page+panel |
| **XV. Big-bang** | PASS | Incremental on refactor branch |
| **XVI. Docs** | PASS | Spec + plan + contracts + data model + quickstart |

**Hard stops**: No security/dependency/SSOT violations.  
**Post-Phase 1 re-check**: PASS — design keeps mock ports, single detail host, lifecycle honesty, capability gating.

---

## Project Structure

### Documentation (this feature)

```text
specs/013-inventory-detail/
├── plan.md                 # This file
├── research.md             # Phase 0
├── data-model.md           # Phase 1
├── quickstart.md           # Phase 1
├── contracts/
│   └── inventory-detail-ui.md
├── checklists/requirements.md
└── tasks.md                # /speckit-tasks — not this command
```

### Source Code (touch list)

```text
packages/features/resources/src/
├── types.ts                # Resource + lifecycle + capability/metric shapes
├── fixtures.ts             # RESOURCE_FIXTURES (rich + sparse + non-healthy)
├── resource-status.ts      # tone/label/provenance/source helpers
├── resource-status.test.ts
├── resource-detail.tsx     # DetailPanel composition
├── resources-index.tsx     # StatusChip tone/label via helpers
├── map.ts                  # statusTone re-export alignment
├── index.tsx               # public exports
├── package.json            # vitest devDep for unit tests
└── vitest.config.ts

apps/desktop/src/renderer/
└── detail-fixture-coverage.test.ts   # rich inventory fixture assertions

# Out of scope this plan
backend/ polter-core inventory/discovery APIs
protobuf / Connect-ES generation
live provider sync adapters
Nodes/Processes detail redesign
```

**Structure Decision**:
- **Runtime owner**: Renderer (`packages/features/resources`)
- **Data**: feature-local fixtures as temporary adapter of “inventory detail queries”
- **Contracts**: UI contract only; no new shared protocol package until live API
- **No new packages** — Constitution XIII

---

## Complexity Tracking

> No constitution violations requiring justification.

| Item | Notes |
| --- | --- |
| Mock vs live dual path | Temporary fixture SSOT only; live queries must replace fixture injection without a second detail UX (XIV exit criteria when core lands) |
| Status union extension | `stale` / `missing` added for §111 lifecycle; mapped to StatusChip tones via helpers (not raw string passthrough) |
| Architecture hand-off | Optional `onOpenArchitecture` prop; mock toast when unwired — honest until App wires navigation |
