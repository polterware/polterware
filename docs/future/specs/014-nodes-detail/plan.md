# Implementation Plan: Resources Nodes Detail

**Branch**: `014-nodes-detail` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/014-nodes-detail/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `004-operator-product-surfaces` (Resources Nodes list + thin node detail), `010-compound-data-table` (DataTable law), design-system `DetailPanel` / `StatusChip`, `frontend-state` detail panel collection registration

**Risk control**: No live polter-node protocol. No real heartbeats or dispatch ACK. No secrets/certs in fixtures. No second node detail page. Mock-first honesty only. Heartbeat loss ≠ process death.

**Implementation status (as of plan)**: Mock production-ready UI landed in-tree during specify session — plan freezes design SSOT and remaining polish/live-swap gaps for `/speckit-tasks` / converge.

---

## Summary

Elevate **Polter Node entity detail** to production operator quality inside the single global bottom `DetailPanel`: identity (node id ≠ display name), health/connection lifecycle aligned with `docs/spec.md` §28, machine facts, announced capabilities, assigned resources, recent operations, upgrade posture, and health-gated Terminal / Run command actions — fixture-backed, without inventing a second inspector.

**Technical approach** (see research):

1. **Keep Resources page tabs** `inventory` | `nodes` | `processes` only — node depth lives in DetailPanel, not new page tabs.
2. **Extract / harden `NodeDetail`** composition (`DetailPanel.Content` + sections grid) consumed by `ResourcesView` when `scopeId: resources:nodes`.
3. **Expand `TargetNode` + `NODE_FIXTURES`** in `feature-resources` with lifecycle statuses, connection summary, capabilities (string or object), health notes, upgrade fields.
4. **Status helpers** (`node-status.ts`) for labels, tones, terminal/dispatch gates, capability normalization.
5. **Design-system grammar only** — DetailPanel sections/metrics/metadata, DataTable, StatusChip; HardDrive header icon matching Nodes PrimaryCell.
6. **Mock actions** — Open terminal / Run command only when health allows; callbacks already owned by App/shell; no claim of live node session.
7. **Tests** — fixture richness + composition contract + detail-fixture-coverage + status gate unit tests.

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9 (renderer features)

**Primary Dependencies**:
- `@polterware/feature-resources` (owner)
- `@polterware/frontend-design-system` (DetailPanel, DataTable, StatusChip, icons)
- `@polterware/frontend-state` (detailPanelStore collection registration)
- Desktop composition root (`apps/desktop`) for Terminal / Run command callbacks and fixture coverage

**Storage**: Fixture/mock only for this delivery. No new SQLite tables, protobuf, or node credential persistence. Future live swap targets nodes domain queries / registry without a second detail UX.

**Testing**: Vitest package config `packages/features/resources/vitest.config.ts` (includes `node-detail.test.ts`); `apps/desktop` detail-fixture-coverage; static typecheck optional. No `pnpm dev` unless operator requests visual check.

**Target Platform**: Desktop Electron renderer — Resources → Nodes project destination; browser-safe feature packages only

**Project Type**: Desktop product feature (Solid feature package)

**Performance Goals**:
- Detail open on row activate feels immediate (&lt; 100ms perceived with fixtures)
- Nested tables OK for fixture sizes (&lt; ~50 rows per section without mandatory virtualization)
- Single vertical scroller = DetailPanel.Body only

**Constraints**:
- Renderer: no unrestricted Node/OS; no cert/private key material in UI
- AGENTS/DESIGN LAW: global DetailPanel only; StatusChip for status; no Actions columns; no Card chrome in operational detail sections
- Mock honesty: never claim live outbound connection stream, live dispatch ACK, or live certificate rotation
- §28 health vocabulary: online / degraded / offline / draining / upgrading / registering / revoked / unknown
- Heartbeat loss copy must not imply remote process death
- Revoked / offline / draining / upgrading: no Run command as live success path

**Scale/Scope**: One entity detail surface for Resources Nodes collection; Processes remain a separate tab; Inventory resource detail is a sibling surface (013); no Provisioning work in this feature

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Nodes registry inspect + safe entry to terminal/command; not remote desktop / full process manager |
| **II. Domain Before Framework** | PASS | TargetNode DTOs stand in for domain queries; layout shape-driven, not `if os ===` |
| **III. Runtime Separation** | PASS | Renderer feature only; no Main node protocol; unimplemented live node not claimed |
| **IV. Monorepo deps** | PASS | resources → design-system + state; no cycles; no scenic package |
| **V. Contracts-First** | PASS | UI contract in `contracts/node-detail-ui.md`; no dual protobuf for mock |
| **VI. Security** | PASS | No secrets/certs; least-privilege actions gated by health; renderer unprivileged |
| **VII. Safe execution** | PASS | Mock terminal/run callbacks; revoked refuses dispatch affordances; no silent destructive ops |
| **VIII. Local-first** | PASS | Full mock detail offline without live polter-node |
| **IX. Performance** | PASS | No Main work; scroll in DetailPanel.Body only |
| **X. Resilience** | PASS | offline/degraded/revoked/draining/upgrading/unknown first-class |
| **XI. Observability** | PASS | Node id + source honesty; no fake APM claims |
| **XII. Testability** | PASS | Status helpers + fixture tests without Electron runtime |
| **XIII. Real code** | PASS | Extend existing `feature-resources`; no scenic package |
| **XIV. SSOT** | PASS | One NodeDetail composition; no dual page+panel |
| **XV. Big-bang** | PASS | Incremental on refactor branch |
| **XVI. Docs** | PASS | Spec + plan + contracts + data model + research + quickstart |

**Hard stops**: No security/dependency/SSOT violations.  
**Post-Phase 1 re-check**: PASS — design keeps mock ports, secret hygiene (N/A beyond no secrets), single detail host, health honesty.

---

## Project Structure

### Documentation (this feature)

```text
specs/014-nodes-detail/
├── plan.md                 # This file
├── research.md             # Phase 0
├── data-model.md           # Phase 1
├── quickstart.md           # Phase 1
├── contracts/
│   └── node-detail-ui.md
├── checklists/requirements.md
└── tasks.md                # /speckit-tasks — not this command
```

### Source Code (touch list)

```text
packages/features/resources/src/
├── node-types.ts           # TargetNode + NODE_FIXTURES + related DTOs
├── node-status.ts          # labels, tones, gates, capability normalization
├── node-detail.tsx         # DetailPanel composition
├── node-detail.test.ts     # helpers + composition + fixture contract
├── nodes-index.tsx         # list StatusChip via node-status helpers
├── index.tsx               # ResourcesView wire + exports
└── vitest.config.ts        # include node-detail.test.ts

apps/desktop/src/renderer/
└── detail-fixture-coverage.test.ts   # rich node fixture assertions

# Out of scope this plan
backend/ polter-node protocol
protobuf / Connect-ES generation
live heartbeat / dispatch adapters
certificate / credential material UI
```

**Structure Decision**:
- **Runtime owner**: Renderer (`packages/features/resources`)
- **Data**: `NODE_FIXTURES` in feature package as temporary adapter/port for node registry queries
- **Contracts**: UI contract only; no new shared protocol package until live node API
- **No new packages** — Constitution XIII

---

## Complexity Tracking

> No constitution violations requiring justification.

| Item | Notes |
| --- | --- |
| Mock vs live dual path | Temporary fixture SSOT only; live registry queries must inject same DTO shapes without a second detail UX (XIV exit criteria when core/node lands) |
| Health vocabulary expansion | List + detail share `nodeStatusLabel` / `nodeStatusTone`; shell Terminal uses compatible gates (`online`\|`degraded` usable via `isTerminalTargetUsable`) without cross-feature import |
