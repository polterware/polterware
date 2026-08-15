# Implementation Plan: Access Audit Event Detail

**Branch**: feature dir `019-audit-detail` · git branch `refactor/new` | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/019-audit-detail/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `004-operator-product-surfaces` (Access IA + DetailPanel host), existing Access → Audit virtualized collection, DESIGN.md + global `DetailPanel` LAW, shell workspace mocks (`WorkspaceAuditEvent`, `AUDIT_EVENT_MOCKS`), production detail patterns from MemberDetail / TeamDetail / RunDetail

**Risk control**: Mock stage only — append-only inspection, no live audit API, no SIEM product. Single `AuditDetail` composition; no second inspector. **Parallel agents own Agents / Policies / Agent session details — do not touch those files.** Collection filters/virtualization stay intact.

**Planning status (`/speckit-plan` 2026-08-12)**: Phase 0 research + Phase 1 design complete; Constitution Check PASS (pre/post). No NEEDS CLARIFICATION. Core mock UI already landed (`audit-detail.tsx`, helpers, enriched fixtures, Access wiring, composition tests, desktop fixture coverage). This plan remains the design SSOT for residual polish, review, and `/speckit-tasks` / `/speckit-implement` if gaps reappear.

---

## Summary

Deliver a **production-quality Access → Audit event DetailPanel** (mocked, read-only): operators inspect a single audit event — result, environment/origin/time, actor (person/agent), action/entity, correlation identifiers, related policy/approval references, and context notes — with optional non-mutating Copy event id / Open related callbacks. Extract the shallow inline detail from `AccessView` into a dedicated composition, enrich fixtures, and cover with helpers + composition contract + fixture coverage tests.

**Technical approach** (see research):

1. **Enrich `WorkspaceAuditEvent` + `AUDIT_EVENT_MOCKS`** with actorId, request/correlation/run ids, context notes/payload where meaningful.
2. **Extract `AuditDetail`** composition matching MemberDetail quality (`layout="sections"`, metrics, StatusChip result, honest loading/not-found).
3. **Wire from `AccessView`** only for `scopeId: access:audit` + `entityKind: audit-event`; pass mock copy/open handlers.
4. **Keep trail honesty**: no mutation actions; no invented timeline.
5. **Tests**: pure helpers + composition source contract + desktop detail-fixture-coverage audit asserts.

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9 (renderer features)

**Primary Dependencies**:
- `@polterware/feature-access` (owner)
- `@polterware/feature-shell` workspace mocks (`WorkspaceAuditEvent`, `AUDIT_EVENT_MOCKS`)
- `@polterware/frontend-design-system` (`DetailPanel`, `StatusChip`, icons, `PageLoading` / `PageError`, toasts)
- `@polterware/frontend-state` (detail target activation already used by Access)

**Storage**: Fixture-only in shell workspace mocks. No SQLite / ConnectRPC / polter-core audit tables in this slice.

**Testing**:
- Vitest node: `packages/features/access` (`audit-detail-helpers`, composition source contract, fixture field presence)
- Vitest ui (desktop): `detail-fixture-coverage.test.ts` audit enrichment asserts
- Access `detail-targets` remains green for audit-event activation

**Target Platform**: Desktop renderer (Electron) Access → Audit; browser-safe feature package only

**Project Type**: Desktop product feature (Solid feature package)

**Performance Goals**:
- Audit detail open feels immediate (fixture resolve after activation)
- DetailPanel body sole vertical scroller
- Audit collection keeps virtualization; no Main-thread/IPC work

**Constraints**:
- Renderer unprivileged; no secrets in audit DTOs
- DESIGN / AGENTS LAW: global DetailPanel only; StatusChip for result; English UI strings
- Read-only trail; optional copy/open only
- Parallel file boundaries strictly enforced
- Timeline mini omitted (point-in-time fixtures)

**Scale/Scope**:
- One entity kind detail: **audit-event**
- Seven existing fixtures enriched (not expanded into SIEM-scale volume)
- Collection IA already complete; detail quality is the slice

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Strengthens audit/governance inspection for humans + agents |
| **II. Domain Before Framework** | PASS | Audit semantics from fixtures; pure helpers for tone/title; no authz engine |
| **III. Runtime Separation** | PASS | Renderer feature only; mock does not claim live core |
| **IV. Monorepo deps** | PASS | access → shell mocks, design-system, state; no reverse domain imports |
| **V. Contracts-First** | PASS | UI contract `contracts/audit-detail-ui.md` |
| **VI. Security** | PASS | Display-only trail; no secrets; no silent policy bypass |
| **VII. Safe execution** | PASS | Append-only inspection; no destructive apply path |
| **VIII. Local-first** | PASS | Fixtures offline; no cloud dependency |
| **IX. Performance** | PASS | No Main work; virtualized collection preserved |
| **X. Resilience** | PASS | loading / error / not-found / empty context explicit |
| **XI. Observability** | PASS | Correlation/request ids displayed only when fixture-supplied |
| **XII. Testability** | PASS | Helpers pure; composition contract without Electron |
| **XIII. Real code** | PASS | Extend feature-access + shell mocks; no scenic package |
| **XIV. SSOT** | PASS | One `AuditDetail`; replace thin inline block |
| **XV. Big-bang** | PASS | Incremental on operator surfaces / refactor branch |
| **XVI. Docs** | PASS | Spec + plan + contract + data model + quickstart |

**Hard stops**: No security/dependency/SSOT violations; no sibling detail file collisions.  
**Post-Phase 1 re-check**: PASS — design keeps mock ports, honest states, single composition, read-only trail, optional non-mutating actions only.

---

## Project Structure

### Documentation (this feature)

```text
specs/019-audit-detail/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── audit-detail-ui.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source Code (touch list)

```text
packages/features/access/src/
├── audit-detail.tsx              # NEW AuditDetail composition
├── audit-detail-helpers.ts       # NEW pure labels/tones/title
├── audit-detail.test.ts          # NEW helpers + composition contract
└── index.tsx                     # ONLY: extract audit Show → <AuditDetail />

packages/features/shell/src/workspace-mocks/
├── types.ts                      # WorkspaceAuditEvent optional fields only
└── data.ts                       # AUDIT_EVENT_MOCKS enrichment only

apps/desktop/src/renderer/
└── detail-fixture-coverage.test.ts  # audit asserts only
```

### Forbidden touch list (parallel agents)

```text
packages/features/access/src/agent-detail*
packages/features/access/src/policy-detail*
packages/features/access/src/agent-session-detail*
AGENT_MOCKS / POLICY_MOCKS / AGENT_SESSION_MOCKS (write)
People/Teams/Agents/Policies table rewrites
```

**Structure Decision**: Runtime owner is the Desktop **renderer** feature package `@polterware/feature-access` (no Main/Node privilege). Domain presentation uses shell **workspace-mock fixtures** as stand-ins for future polter-core audit store — not a new package, not a second DTO layer. Contracts are UI-only (`contracts/audit-detail-ui.md`); no protobuf/Connect surface in this slice. Touch list is bounded to access audit detail + shell audit type/fixtures + desktop fixture-coverage audit asserts. No scenic scaffolds.

---

## Implementation phases (preview; tasks.md owns execution)

1. **Model** — extend type + enrich fixtures  
2. **Helpers** — result tone/label, title, actor kind  
3. **Composition** — AuditDetail sections + states + optional actions  
4. **Wire** — AccessView mount with not-found honesty  
5. **Tests** — package + desktop fixture coverage  
6. **Polish** — mark tasks complete; confirm collection non-regression statically

> Execution tracking lives in [tasks.md](./tasks.md). Plan command does not create or rewrite task checkboxes except by reference.

---

## Complexity Tracking

No constitution violations requiring justification. Complexity is intentionally low: one read-only detail composition + fixture enrichment.

---

## Phase 0–1 validation (this `/speckit-plan` run)

| Artifact | Status | Notes |
| --- | --- | --- |
| `research.md` | Complete | 9 decisions; no remaining NEEDS CLARIFICATION |
| `data-model.md` | Complete | Matches landed `WorkspaceAuditEvent` optional fields |
| `contracts/audit-detail-ui.md` | Complete | Props/layout/actions/read-only contract |
| `quickstart.md` | Complete | Package + desktop + optional smoke paths |
| Constitution Check | PASS (pre + post design) | Hard stops VI/IV/XIV clear |
| Implementation alignment | Landed | Code paths in touch list present; collection virtualization retained |

**Ready for**: residual review, `/speckit-tasks` only if tasks.md is regenerated, or visual smoke — not blocked on further design unknowns.
