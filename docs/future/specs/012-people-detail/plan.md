# Implementation Plan: Access People Detail

**Branch**: `012-people-detail` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-11 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/012-people-detail/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `004-operator-product-surfaces` (Access IA + DetailPanel host), `007-remaining-operator-surfaces` (access expansions / teams / invitations), DESIGN.md + global `DetailPanel` LAW, shell workspace mocks

**Risk control**: Mock stage only — no live IdP, membership API, or session kill. Permission table is display-only. Single `MemberDetail` composition; no second inspector. No Agents / Invitation redesign.

**Implementation status (as of plan date)**: Core mock UI is already landed on the working branch (`member-detail.tsx`, enriched `WorkspacePerson` fixtures, Access wiring, contract tests). This plan freezes the design SSOT for review, residual polish, and `/speckit-tasks` if any gaps remain.

---

## Summary

Deliver a **production-quality Access → People member DetailPanel** (mocked): operators inspect human membership posture — identity, role, environments, teams, permissions picture, auth/MFA, sessions, and recent activity — and can trigger honest mock governance actions (change role, add to team, revoke access, revoke non-current session) without leaving Access or inventing a second detail surface.

**Technical approach** (see research):

1. **Extend `WorkspacePerson` fixtures** with posture + collections required by detail (permissions, sessions, activity, auth, MFA, joinedAt).
2. **Extract `MemberDetail`** composition matching ResourceDetail / ArchitectureNodeDetail quality (`layout="sections"`, metrics, metadata, DataTables, StatusChip).
3. **Wire from `AccessView`** only for `scopeId: access:people` + `entityKind: member`; pass filtered teams and mock action handlers.
4. **Keep authz honesty**: UI never claims frontend authorization; toasts are mock-labeled.
5. **Tests**: pure helpers + source/composition contract + desktop detail-fixture-coverage for people richness.

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9 (renderer features)

**Primary Dependencies**:
- `@polterware/feature-access` (owner)
- `@polterware/feature-shell` workspace mocks (`WorkspacePerson`, `WorkspaceTeam`, `PEOPLE_MOCKS`, `TEAM_MOCKS`)
- `@polterware/frontend-design-system` (`DetailPanel`, `DataTable`, `StatusChip`, icons, `PageLoading` / `PageError`, toasts)
- `@polterware/frontend-state` (detail target activation already used by Access)

**Storage**: Fixture-only in shell workspace mocks. No SQLite / ConnectRPC / polter-core membership tables in this slice.

**Testing**:
- Vitest node: `packages/features/access` (`member-detail-helpers`, composition source contract, fixture field presence via source/fixture reads)
- Vitest ui (desktop): `detail-fixture-coverage.test.ts` asserts Admin person has permissions/sessions/activity/auth
- Access `detail-targets` remains green for People activation

**Target Platform**: Desktop renderer (Electron) Access → People; browser-safe feature package only

**Project Type**: Desktop product feature (Solid feature package)

**Performance Goals**:
- Member detail open feels immediate (fixture resolve &lt; 1 frame after activation)
- DetailPanel body is sole vertical scroller; fixture tables stay small (&lt; ~20 rows/section)
- No Main-thread or IPC work for this stage

**Constraints**:
- Renderer: unprivileged; no secrets, tokens, or credential material in person DTOs
- DESIGN / AGENTS LAW: global DetailPanel only; StatusChip for status; DataTable for collections; no PrimaryCell descriptions; no Actions columns; English UI strings
- Mutations mock-only with toast honesty (no plan→approve→apply claim)
- Agents, invitations, teams keep separate detail branches
- Role matrix editor deferred (007)

**Scale/Scope**:
- One entity kind detail: human **member**
- Three primary fixtures (Admin / Operator / Viewer) complete
- No org-wide directory product; project Access People only

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Strengthens collaboration + governance inspection of human co-managers; not a static directory clone |
| **II. Domain Before Framework** | PASS | Membership semantics presented from fixtures; no authz engine inside UI; pure helpers for tone/label mapping |
| **III. Runtime Separation** | PASS | Renderer feature only; no Main/Node privilege; mock does not claim live core |
| **IV. Monorepo deps** | PASS | access → shell mocks, design-system, state; no reverse domain imports |
| **V. Contracts-First** | PASS | UI contract `contracts/people-detail-ui.md`; extends 004 access-ui member detail |
| **VI. Security** | PASS | Display-only permissions; no secrets; mock revoke is not real session kill |
| **VII. Safe execution** | PASS | No destructive apply path; mock toasts only |
| **VIII. Local-first** | PASS | Fixtures work offline; no cloud dependency |
| **IX. Performance** | PASS | No Main work; small tables; body scroller only |
| **X. Resilience** | PASS | loading / error / not-found / empty collections explicit |
| **XI. Observability** | PASS | No fake correlation IDs; activity is presentation of fixture events |
| **XII. Testability** | PASS | Helpers pure; composition contract tests without Electron |
| **XIII. Real code** | PASS | Extend `feature-access` + shell mocks; no scenic package |
| **XIV. SSOT** | PASS | One `MemberDetail`; replace thin inline block |
| **XV. Big-bang** | PASS | Incremental on operator surfaces / refactor branch |
| **XVI. Docs** | PASS | Spec + plan + contract + data model + quickstart |

**Hard stops**: No security/dependency/SSOT violations.  
**Post-Phase 1 re-check**: PASS — design keeps mock ports, honest states, single composition, display-only authz picture.

---

## Project Structure

### Documentation (this feature)

```text
specs/012-people-detail/
├── plan.md                 # This file
├── research.md             # Phase 0
├── data-model.md           # Phase 1
├── quickstart.md           # Phase 1
├── contracts/
│   └── people-detail-ui.md # Phase 1 (UI contract)
├── checklists/
│   └── requirements.md
└── tasks.md                # /speckit-tasks — not this command
```

### Source Code (touch list)

```text
packages/features/access/src/
├── member-detail.tsx           # MemberDetail composition (DetailPanel sections)
├── member-detail-helpers.ts    # presence/effect/result tone+label pure helpers
├── member-detail.test.ts       # helpers + composition contract + fixture field checks
├── index.tsx                   # AccessView: mount MemberDetail for member targets
├── detail-targets.ts           # unchanged activation model (member entityKind)
└── detail-targets.test.ts

packages/features/shell/src/workspace-mocks/
├── types.ts                    # WorkspacePerson detail fields
├── data.ts                     # PEOPLE_MOCKS enriched
└── index.ts                    # re-exports (unchanged surface for consumers)

apps/desktop/src/renderer/
└── detail-fixture-coverage.test.ts  # Admin people completeness
```

**Structure Decision**:
- **Runtime owner**: Desktop renderer feature `@polterware/feature-access`.
- **Domain vs adapter**: Presentation over identity/organizations membership concepts; fixtures stand in for future core membership DTO. No new domain package.
- **Contracts**: Feature UI contract only; no protobuf for mock stage.
- **No new packages**: Constitution XIII — extend existing access + shell mocks.

---

## Complexity Tracking

> No constitution violations requiring justification.

| Item | Note |
| --- | --- |
| Feature dir number collision | Repo has multiple `012-*` dirs; this plan owns **`specs/012-people-detail` only**. Active `.specify/feature.json` should point here when working this feature. |
