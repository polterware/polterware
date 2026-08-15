# Implementation Plan: Architecture Node Detail Polish

**Branch**: `026-architecture-node-detail` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/026-architecture-node-detail/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: Existing `ArchitectureNodeDetail`, `domain/node-lifecycle`, Components tab + View detail hosts, design-system `DetailPanel` / `StatusChip` / `DataTable`, Architecture index wiring for `arch-node`.

**Risk control**: Polish detail composition only. Do not rewrite canvas engine, place flow, view-detail, or components-table. Do not touch parallel P1 surfaces (027–031).

**Parallelism boundary**: MAY touch `packages/features/architecture/src/node-detail.tsx`, helpers, tests, `types`/`fixtures` for `ArchitectureNodeView` if needed, this specs folder. MUST NOT implement workflow/org-team/integrations/settings/terminal work.

---

## Summary

Elevate **Architecture → node entity detail** to production operator density: StatusChips for lifecycle, resource-link posture, and connection evidence; denser Overview metrics; domain icon header; preserved Open on canvas / Link resource; honest empty/loading/error; helpers + composition tests.

**Technical approach**:

1. **Helpers** (`node-detail-helpers.ts`): normalize lifecycle label/tone (reuse `node-lifecycle`), link label/tone (+ optional resource name line), evidence Proven/Unproven label/tone.
2. **Fixtures**: enrich `NODE_DETAIL_FIXTURES` / sparse fallback with lifecycle, linkStatus, canLinkResource where missing so chips and actions are exerciseable.
3. **Composition** (`node-detail.tsx`): header `NodeIcon` + actions; Overview metrics/metadata with StatusChips; Connected table evidence chips; keep docs/activity/loading/error.
4. **Tests** (`node-detail.test.ts`): pure helpers + source composition contract (StatusChip ≥2 axes, evidence chips, Open on canvas, Link resource, no raw-only Proven/Unproven).
5. **Do not break** Components/Views `Show` wiring in `index.tsx` (consume existing `ArchitectureNodeDetail` only; no structural host rewrite required unless mapping lacks fields for chips — then minimal view mapping enrichment).

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9 (renderer features)

**Primary Dependencies**:
- `@polterware/feature-architecture` (owner)
- `@polterware/frontend-design-system` (`DetailPanel`, `DataTable`, `StatusChip`, `PageLoading`/`PageError`)
- Existing `domain/node-lifecycle`, `domain/node-icon`

**Storage**: Fixture/document projection only. No new Core API.

**Testing**: Vitest node in `packages/features/architecture` — `node-detail.test.ts` (and helpers coverage).

**Target Platform**: Desktop Architecture canvas + Components detail panel

**Constraints**:
- DESIGN/AGENTS LAW: global DetailPanel; StatusChip for status axes; DataTable no Actions column
- Lifecycle chips MUST NOT invent service health from Planned/Linked alone
- English UI strings

**Scale/Scope**: One entity detail polish; small helper module; fixture enrichment; tests.

---

## Constitution Check

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Node detail makes canvas operational (§4.1) |
| **II. Domain Before Framework** | PASS | Lifecycle/link/evidence pure helpers |
| **III. Runtime Separation** | PASS | Renderer feature only |
| **IV. Monorepo deps** | PASS | architecture → design-system only |
| **V. Contracts-First** | PASS | UI contract in `contracts/` |
| **VI. Security** | PASS | No secrets; link action remains host-gated |
| **VII. Safe execution** | PASS | No dual persistence |
| **VIII. Local-first** | PASS | Fixtures offline |
| **IX. Performance** | PASS | Composition-only; body scroller |
| **X. Resilience** | PASS | loading / error / not-found kept |
| **XI. Observability** | PASS | Evidence + lifecycle honesty |
| **XII. Testability** | PASS | Helpers + composition contract |
| **XIII. Real code** | PASS | Extend existing package |
| **XIV. SSOT** | PASS | Reuse node-lifecycle tones |
| **XV. Big-bang** | PASS | Incremental polish |
| **XVI. Docs** | PASS | This feature folder |

**Hard stops**: No canvas engine rewrite; no parallel P1 features; no “already done” claim on thin detail.

---

## Project Structure

### Documentation (this feature)

```text
specs/026-architecture-node-detail/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
├── contracts/
│   └── architecture-node-detail-ui.md
└── checklists/
    └── requirements.md
```

### Source (implementation)

```text
packages/features/architecture/src/
├── node-detail.tsx                 # polish composition
├── node-detail-helpers.ts          # NEW labels/tones
├── node-detail.test.ts             # NEW helpers + composition contract
├── fixtures.ts                     # enrich NODE_DETAIL_FIXTURES
├── types.ts                        # only if view needs icon fields
├── domain/node-lifecycle.ts        # REUSE (no rewrite)
└── index.tsx                       # consume only; minimal mapping if needed
```

---

## Implementation Phases

### Phase 0 — Research

Confirm reuse of `nodeLifecycleTone` / `linkStatusTone`; evidence mapping; composition test style from Resources `node-detail.test.ts`.

### Phase 1 — Design artifacts

data-model, contract, quickstart written with this plan.

### Phase 2 — Implementation

Helpers → fixtures → node-detail composition → tests → vitest green → rubric check.

---

## Risks

| Risk | Mitigation |
| --- | --- |
| Break Components/Views wiring | Only change detail body; keep export + props |
| Lifecycle painted as health | Reuse lifecycle tones (planned/linked not healthy) |
| Fixture too thin for chips | Enrich primary fixture + sparse empty path |
| Parallel agent file collisions | Stay inside node-detail/helpers/fixtures/tests |

---

## Done when

- Rubric green
- `pnpm --filter @polterware/feature-architecture test` covers node-detail tests green
