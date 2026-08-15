# Implementation Plan: Access Policies Detail

**Branch**: `018-policies-detail` (feature dir; git branch may remain `refactor/new`) | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/018-policies-detail/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `004-operator-product-surfaces` (Access IA + DetailPanel host), DESIGN.md + global `DetailPanel` LAW, shell workspace mocks, production detail patterns from `012-people-detail` / `015-teams-detail`

**Risk control**: Mock stage only — no live policy engine, real enable/delete, or frontend authz. Single `PolicyDetail` composition; no second inspector. **Hard file boundary**: do not implement or refactor Agents detail, Audit detail, or Agent session detail (parallel agents). Only read `AUDIT_EVENT_MOCKS` for related links.

**Implementation status (as of `/speckit-plan` re-run)**: Core mock UI is **landed** on the working tree (`policy-detail.tsx`, helpers/tests, enriched `WorkspacePolicy` / `POLICY_MOCKS`, Access wiring, desktop fixture coverage). This plan freezes design SSOT for review, residual polish, and task traceability. Spec status: **Implemented**.

---

## Summary

Deliver a **production-quality Access → Policies DetailPanel** (mocked): operators inspect policy posture — status, scope, enforcement, approval, description, actors, environments, allowed/blocked actions, risk/approval requirements, update history, optional activity trail, and related audit events — and can trigger honest mock governance actions (Edit, Duplicate, Enable/Disable, Delete) without leaving Access or inventing a second detail surface.

**Technical approach** (see research):

1. **Extend `WorkspacePolicy` fixtures** with detail fields (`actorEntries`, risk/approval, activity, optional notes) while keeping list string arrays.
2. **Extract `PolicyDetail`** composition matching MemberDetail / TeamDetail quality.
3. **Wire from `AccessView`** only for `scopeId: access:policies` + `entityKind: policy`; pass related audit filtered from `AUDIT_EVENT_MOCKS` and mock action handlers.
4. **Keep authz honesty**: UI never claims frontend enforcement; toasts are mock-labeled.
5. **Tests**: pure helpers + composition source contract + desktop detail-fixture-coverage for primary policy richness.

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9 (renderer features)

**Primary Dependencies**:
- `@polterware/feature-access` (owner)
- `@polterware/feature-shell` workspace mocks (`WorkspacePolicy`, `POLICY_MOCKS`, read-only `AUDIT_EVENT_MOCKS`)
- `@polterware/frontend-design-system` (`DetailPanel`, `DataTable`, `StatusChip`, icons, `PageLoading` / `PageError`, toasts)
- `@polterware/frontend-charts` (optional `DitherSegmentedBar` for allowed/blocked balance)
- `@polterware/frontend-state` (detail target activation already used by Access)

**Storage**: Fixture-only in shell workspace mocks. No SQLite / ConnectRPC / polter-core policy tables in this slice.

**Testing**:
- Vitest node: `packages/features/access` (`policy-detail-helpers`, composition source contract, fixture field presence)
- Vitest ui (desktop): `detail-fixture-coverage.test.ts` asserts primary policy completeness + related audit linkage
- Access `detail-targets` remains green for policy activation

**Target Platform**: Desktop renderer (Electron) Access → Policies; browser-safe feature package only

**Project Type**: Desktop product feature (Solid feature package)

**Performance Goals**:
- Policy detail open feels immediate (fixture resolve < 1 frame after activation)
- DetailPanel body is sole vertical scroller; fixture tables stay small (< ~20 rows/section)
- No Main-thread or IPC work for this stage

**Constraints**:
- Renderer: unprivileged; no secrets in policy DTOs
- DESIGN / AGENTS LAW: global DetailPanel only; StatusChip for status; DataTable for collections; no PrimaryCell descriptions; no Actions columns; English UI strings
- Mutations mock-only with toast honesty
- Parallel agents own sibling details — zero intentional file conflict

**Scale/Scope**:
- One entity kind detail: **policy**
- Four fixtures (`p1`–`p4`); `p1` is the completeness gate
- Related audit listing only; no Audit detail

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Strengthens governance inspection of policies; not a generic ACL product clone |
| **II. Domain Before Framework** | PASS | Policy semantics presented from fixtures; no authz engine inside UI; pure helpers for tone/label mapping |
| **III. Runtime Separation** | PASS | Renderer feature only; mock does not claim live core |
| **IV. Monorepo deps** | PASS | access → shell mocks, design-system, state, charts; no reverse domain imports |
| **V. Contracts-First** | PASS | UI contract `contracts/policies-detail-ui.md` |
| **VI. Security** | PASS | Display-only policy; no secrets; mock delete is not real mutation |
| **VII. Safe execution** | PASS | No destructive apply path; mock toasts only |
| **VIII. Local-first** | PASS | Fixtures work offline |
| **IX. Performance** | PASS | No Main work; small tables; body scroller only |
| **X. Resilience** | PASS | loading / error / not-found / empty collections explicit |
| **XI. Observability** | PASS | No fake correlation IDs |
| **XII. Testability** | PASS | Helpers pure; composition contract tests without Electron |
| **XIII. Real code** | PASS | Extend `feature-access` + shell mocks; no scenic package |
| **XIV. SSOT** | PASS | One `PolicyDetail`; replace thin inline block |
| **XV. Big-bang** | PASS | Incremental on operator surfaces / refactor branch |
| **XVI. Docs** | PASS | Spec + plan + contract + data model + quickstart |

**Hard stops**: No security/dependency/SSOT violations.  
**Post-Phase 1 re-check (2026-08-12)**: PASS — design + landed code keep mock ports, honest states, single `PolicyDetail` composition, display-only policy, related audit read-only filter, parallel-agent file boundary. No VI / IV / XIV violations.

---

## Project Structure

### Documentation (this feature)

```text
specs/018-policies-detail/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── policies-detail-ui.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (touch list)

```text
packages/features/access/src/
├── policy-detail.tsx             # PolicyDetail composition
├── policy-detail-helpers.ts      # status/risk/result/kind tone+label pure helpers
├── policy-detail.test.ts         # helpers + composition contract + wiring checks
└── index.tsx                     # ONLY replace policy Show block with <PolicyDetail />

packages/features/shell/src/workspace-mocks/
├── types.ts                      # WorkspacePolicy + nested actor/activity types ONLY
├── data.ts                       # POLICY_MOCKS enriched ONLY
└── index.ts                      # re-exports if new types need export

apps/desktop/src/renderer/
└── detail-fixture-coverage.test.ts  # policy asserts only
```

**Structure Decision**:
- **Runtime owner**: Desktop renderer feature `@polterware/feature-access`.
- **Domain vs adapter**: Presentation over governance policy concepts; fixtures stand in for future core policy DTO.
- **Contracts**: Feature UI contract only; no protobuf for mock stage.
- **No new packages**: Constitution XIII — extend existing access + shell mocks.

---

## Complexity Tracking

> No constitution violations requiring justification.

| Item | Note |
| --- | --- |
| Related audit prop vs embed | Filtered prop avoids dual audit SSOT and parallel-agent conflict |
| Keep string arrays + actorEntries | List compatibility while meeting structured Actors requirement |

---

## Phase 0 / Phase 1 outputs

| Artifact | Path | Status |
| --- | --- | --- |
| Research | [research.md](./research.md) | Complete — zero NEEDS CLARIFICATION |
| Data model | [data-model.md](./data-model.md) | Matches landed `WorkspacePolicy*` types |
| UI contract | [contracts/policies-detail-ui.md](./contracts/policies-detail-ui.md) | Matches `PolicyDetail` props + sections |
| Quickstart | [quickstart.md](./quickstart.md) | Manual S1–S5 + automated gates |
| Tasks | [tasks.md](./tasks.md) | Regenerated by `/speckit-tasks` 2026-08-12 — 50 tasks, all `[x]`, MVP = US1 |

## Next command

- **If re-planning only**: design frozen — no further plan work required.
- **If tasks need regeneration**: `/speckit-tasks` (current tasks already mark delivery complete).
- **If residual work**: residual polish only; core implement is landed per spec status Implemented.
