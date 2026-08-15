# Implementation Plan: Architecture View Detail

**Branch**: `023-architecture-view-detail` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/023-architecture-view-detail/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `004-operator-product-surfaces` / Architecture Views table + DetailPanel host, existing canvas view activation (`activeViewId` + tab canvas), existing `ViewEditor` deep flow, DESIGN.md DetailPanel LAW

**Risk control**: Mock/fixture stage only. No second canvas host. No Components tab work (parallel agent 025). No canvas engine rewrite. No inventing membership names from `nodeCount` alone.

**Parallelism boundary**: MAY touch `view-detail*`, Architecture index Show of `arch-view`, `VIEW_FIXTURES` / `ArchViewCard`, session open-view existing, this specs folder. MUST NOT rewrite `node-detail.tsx` canvas path, Components tab, process/approval/invitation surfaces.

---

## Summary

Deliver a **production-quality Architecture → Views → saved view DetailPanel** (mocked): operators inspect overview (description, node count, updated, owner, environments, tags/purpose), membership (fixture members or honest unavailable), optional notes/filters, and use **Open** (canvas view activation) + **Edit** (existing view-edit deep flow) from the global DetailPanel.

**Technical approach**:

1. **Enrich `ArchViewCard` / `VIEW_FIXTURES`** with optional owner, environments, tags, purpose, members, notes, filterSummary, dataSource — at least two primary views fully enriched with explicit members.
2. **Extract `ArchitectureViewDetail`** (`view-detail.tsx` + helpers + tests) matching Team/Member production pattern (`layout="sections"`, metrics, membership DataTable, honest states).
3. **Wire from `ArchitectureView`** for `scopeId: architecture:views` + `entityKind: arch-view`; replace thin inline summary+metric block; pass Open/Edit handlers to existing `setActiveViewId` + `onTabChange("canvas")` and `setDeepMode("view-edit")`.
4. **Honesty**: membership never synthesized from `nodeCount`; not-found / loading / empty description explicit.
5. **Tests**: pure helpers + composition source contract + desktop detail-fixture-coverage expansion.

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9 (renderer features)

**Primary Dependencies**:
- `@polterware/feature-architecture` (owner)
- `@polterware/frontend-design-system` (`DetailPanel`, `DataTable`, `PageLoading` / `PageError`, icons)
- `@polterware/frontend-state` (detail target activation already used by Architecture)

**Storage**: Fixture-only in `extra-fixtures.ts`. No SQLite / ConnectRPC / live view library in this slice.

**Testing**:
- Vitest node: `packages/features/architecture` (`view-detail.test.ts`)
- Vitest ui (desktop): `detail-fixture-coverage.test.ts` asserts enriched views (≥2 with fields beyond name/description/nodeCount/updated)

**Target Platform**: Desktop renderer Architecture → Views; browser-safe feature package only

**Project Type**: Desktop product feature (Solid feature package)

**Performance Goals**:
- View detail open feels immediate (fixture resolve < 1 frame after activation)
- DetailPanel body is sole vertical scroller; membership tables stay small

**Constraints**:
- DESIGN / AGENTS LAW: global DetailPanel only; DataTable for membership; no Actions columns; English UI strings
- Open reuses existing canvas activation — no second canvas host
- Components tab owned by parallel agent — out of scope
- Never invent member names from nodeCount alone

**Scale/Scope**:
- One entity kind detail: **arch-view**
- Four fixture views; ≥2 primary enriched with members + owner/environments/tags
- File boundary: architecture view-detail modules + extra-fixtures + architecture index Show + desktop fixture coverage + this specs folder

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Named lenses on system map — core operator understanding |
| **II. Domain Before Framework** | PASS | View membership/scope from fixtures; pure helpers for membership availability |
| **III. Runtime Separation** | PASS | Renderer feature only; mock does not claim live view library |
| **IV. Monorepo deps** | PASS | architecture → design-system, state, canvas; no reverse domain imports |
| **V. Contracts-First** | PASS | UI contract `contracts/architecture-view-detail-ui.md` |
| **VI. Security** | PASS | No secrets; edit/open mock/session only |
| **VII. Safe execution** | PASS | Open/Edit reuse existing mock paths; no dual persistence story |
| **VIII. Local-first** | PASS | Fixtures work offline |
| **IX. Performance** | PASS | No Main work; small membership tables; body scroller only |
| **X. Resilience** | PASS | loading / error / not-found / empty description / unavailable membership |
| **XI. Observability** | PASS | View id + dataSource honesty when present |
| **XII. Testability** | PASS | Helpers pure; composition contract tests without Electron |
| **XIII. Real code** | PASS | Extend `feature-architecture`; no scenic package |
| **XIV. SSOT** | PASS | One `ArchitectureViewDetail`; replace thin inline block |
| **XV. Big-bang** | PASS | Incremental on operator surfaces / refactor branch |
| **XVI. Docs** | PASS | Spec + plan + contract + data model + quickstart |

**Hard stops**: No inventing members; no second canvas host; no Components tab; no node-detail rewrite.  
**Post-Phase 1 re-check**: PASS — design keeps mock ports, honest states, single composition, existing open/edit reuse.

---

## Project Structure

### Documentation (this feature)

```text
specs/023-architecture-view-detail/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/architecture-view-detail-ui.md
├── checklists/requirements.md
└── tasks.md
```

### Source (implementation)

```text
packages/features/architecture/src/
├── extra-fixtures.ts          # ArchViewCard + VIEW_FIXTURES enrichment
├── view-detail.tsx            # ArchitectureViewDetail composition
├── view-detail-helpers.ts     # membership availability, empty copy
├── view-detail.test.ts        # helpers + composition + wiring contract
├── views-table.tsx            # unchanged collection (consumes ArchViewCard)
└── index.tsx                  # Show arch-view → ArchitectureViewDetail

apps/desktop/src/renderer/
└── detail-fixture-coverage.test.ts  # enriched view assertions
```

---

## Complexity Tracking

No constitution violations requiring justification. Scope is a single entity detail extraction + fixture enrichment.

---

## Phase 0 / Phase 1

See [research.md](./research.md), [data-model.md](./data-model.md), [contracts/architecture-view-detail-ui.md](./contracts/architecture-view-detail-ui.md), [quickstart.md](./quickstart.md).
