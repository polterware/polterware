# Implementation Plan: Resources Process Detail

**Branch**: `021-process-detail` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/021-process-detail/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `004-operator-product-surfaces` (Resources Processes list + thin process detail), design-system `DetailPanel` / `StatusChip`, sibling quality of `013-inventory-detail` / `014-nodes-detail` / `012-operations-run-detail`

**Risk control**: No live process kill/restart. No invented CPU time series. No second process inspector. Mock-first honesty only. Parallel P0 siblings (approval, arch view/components, invitation) are out of scope.

---

## Summary

Elevate **Process entity detail** from a ~100-line stub to production operator quality inside the single global bottom `DetailPanel`: status via StatusChip, host node context with optional open-node hand-off, runtime command/user/port, optional related refs when fixture-backed, status-gated control actions (View logs, Open terminal, Restart, Stop), and honest loading / not-found / error / sparse states.

**Technical approach**:

1. Keep Resources tabs `inventory` | `nodes` | `processes` only — process depth lives in DetailPanel.
2. Harden dedicated `ProcessDetail` composition (`DetailPanel.Content` + sections grid).
3. Expand `ProcessRow` + `PROCESS_FIXTURES` with full + sparse + Stopped/Unknown shapes and optional related refs.
4. Add `process-status.ts` helpers for labels, tones, and control gates.
5. Wire Processes table StatusChip; fix not-found mount when scope is processes but target missing.
6. Tests: composition contract, helper gates, fixture richness, desktop detail-fixture-coverage.

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9 (renderer features)

**Primary Dependencies**:
- `@polterware/feature-resources` (owner)
- `@polterware/frontend-design-system` (DetailPanel, StatusChip, PageLoading, PageError, icons)
- `@polterware/frontend-charts` (optional `DitherSegmentedBar` for known CPU percent only)
- `@polterware/frontend-state` (detail panel collection registration — already wired)

**Storage**: Fixture/mock only. No new SQLite tables or protobuf for this delivery.

**Testing**: Vitest `packages/features/resources` includes `process-detail.test.ts`; desktop `detail-fixture-coverage.test.ts` covers full + sparse process fixtures.

**Target Platform**: Desktop Electron renderer — Resources → Processes

**Constraints**:
- Renderer unprivileged; process control UI is not live privilege elevation
- StatusChip only for status; no hand-rolled pills
- CPU visualization only when numeric percent present — never invent history
- Stop/Restart hidden or disabled for Stopped/Unknown
- Do not modify approval, architecture view/components, invitation, or rewrite node/resource detail beyond open-node hand-off

**Scale/Scope**: One entity detail surface for Resources Processes collection

---

## Constitution Check

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Process inspect + safe mock control intents |
| **II. Domain Before Framework** | PASS | ProcessRow DTOs; layout shape-driven |
| **III. Runtime Separation** | PASS | Renderer feature only; no Main process API |
| **IV. Monorepo deps** | PASS | resources → design-system + charts + state |
| **V. Contracts-First** | PASS | `contracts/process-detail-ui.md` |
| **VI. Security** | PASS | Mock control; no secrets |
| **VII. Safe execution** | PASS | Status-gated Stop/Restart; mock toast honesty |
| **VIII. Local-first** | PASS | Full mock detail offline |
| **IX. Performance** | PASS | DetailPanel.Body sole vertical scroller |
| **X. Resilience** | PASS | loading/error/not-found/sparse |
| **XII. Testability** | PASS | Helpers + composition + fixture coverage |
| **XIII. Real code** | PASS | Elevate existing process-detail module |
| **XIV. SSOT** | PASS | One ProcessDetail composition |

---

## Project Structure

### Documentation

```text
specs/021-process-detail/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/process-detail-ui.md
├── checklists/requirements.md
├── tasks.md
└── spec.md
```

### Source Code (touch list)

```text
packages/features/resources/src/
├── node-types.ts           # ProcessRow + PROCESS_FIXTURES
├── process-status.ts       # NEW: labels, tones, action gates
├── process-detail.tsx      # production DetailPanel composition
├── process-detail.test.ts  # NEW: helpers + composition + fixtures
├── processes-table.tsx     # StatusChip in status column
├── index.tsx               # mount loading/error/notFound + open-node
└── vitest.config.ts        # include process-detail.test.ts

apps/desktop/src/renderer/
└── detail-fixture-coverage.test.ts  # full + sparse process coverage
```

---

## Acceptance Rubric (implementation gate)

- [x] process-detail.tsx ≥ ~250 lines **or** helpers + dense body equivalent (~340 lines + process-status helpers)
- [x] StatusChip for process status
- [x] PageLoading + PageError/notFound
- [x] process-detail.test.ts composition contract
- [x] Fixtures full + sparse in detail-fixture-coverage
- [x] Actions respect status (Stop not offered as live for Stopped)
- [x] No invented CPU/time-series history
- [x] Never claim “already implemented” solely because the stub file exists

---

## Complexity Tracking

None — follows established Nodes/Runs detail pattern; no new packages or architectural exceptions.
