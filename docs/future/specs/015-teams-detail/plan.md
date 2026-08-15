# Implementation Plan: Access Teams Detail

**Branch**: `015-teams-detail` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/015-teams-detail/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `004-operator-product-surfaces` (Access IA + DetailPanel host), `007-remaining-operator-surfaces` (teams table on People), `012-people-detail` (MemberDetail + bidirectional team link), DESIGN.md + global `DetailPanel` LAW, shell workspace mocks

**Risk control**: Mock stage only — no live membership API, org directory product, or real archive/mutation. Policy/membership tables are display-only. Single `TeamDetail` composition; no second inspector. No Agents / Invitation / Roles matrix rewrite.

**Implementation status (as of plan date)**: Core mock UI is already landed on the working branch (`team-detail.tsx`, enriched `WorkspaceTeam` fixtures, Access wiring, contract tests, fixture coverage). This plan freezes the design SSOT for review, residual polish, and `/speckit-tasks` if any gaps remain.

---

## Summary

Deliver a **production-quality Access → People (Teams) DetailPanel** (mocked): operators inspect team posture — identity, status, lead, on-call/escalation, environments, member roster, owned resources, linked policies, and recent activity — and can trigger honest mock governance actions (edit team, add member, archive team) without leaving Access or inventing a second detail surface. Bidirectional navigation with People member detail completes the ownership inspection loop.

**Technical approach** (see research):

1. **Extend `WorkspaceTeam` fixtures** with posture + collections required by detail (roster, owned resources, policies, activity, ops metadata).
2. **Extract `TeamDetail`** composition matching MemberDetail / ResourceDetail quality (`layout="sections"`, metrics, metadata, DataTables, StatusChip).
3. **Wire from `AccessView`** only for `scopeId: access:people` + `entityKind: team`; pass mock action handlers and `onOpenMember` for roster activation.
4. **Keep authz honesty**: UI never claims frontend authorization; toasts are mock-labeled.
5. **Tests**: pure helpers + source/composition contract + desktop detail-fixture-coverage for Platform Team richness.

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9 (renderer features)

**Primary Dependencies**:
- `@polterware/feature-access` (owner)
- `@polterware/feature-shell` workspace mocks (`WorkspaceTeam`, `TEAM_MOCKS`, related person ids)
- `@polterware/frontend-design-system` (`DetailPanel`, `DataTable`, `StatusChip`, icons, `PageLoading` / `PageError`, toasts)
- `@polterware/frontend-state` (detail target activation already used by Access)

**Storage**: Fixture-only in shell workspace mocks. No SQLite / ConnectRPC / polter-core team membership tables in this slice.

**Testing**:
- Vitest node: `packages/features/access` (`team-detail-helpers`, composition source contract, fixture field presence via source/fixture reads)
- Vitest ui (desktop): `detail-fixture-coverage.test.ts` asserts Platform Team (`t1`) has roster/resources/policies/activity and count consistency
- Access `detail-targets` remains green for team activation

**Target Platform**: Desktop renderer (Electron) Access → People → Teams table; browser-safe feature package only

**Project Type**: Desktop product feature (Solid feature package)

**Performance Goals**:
- Team detail open feels immediate (fixture resolve < 1 frame after activation)
- DetailPanel body is sole vertical scroller; fixture tables stay small (< ~20 rows/section)
- No Main-thread or IPC work for this stage

**Constraints**:
- Renderer: unprivileged; no secrets, tokens, or credential material in team DTOs
- DESIGN / AGENTS LAW: global DetailPanel only; StatusChip for status; DataTable for collections; no PrimaryCell descriptions; no Actions columns; English UI strings
- Mutations mock-only with toast honesty (no plan→approve→apply claim)
- Members / invitations keep separate detail branches on the same People scope
- Live org directory and roles matrix deferred

**Scale/Scope**:
- One entity kind detail: **team**
- Three fixtures (Platform / Payments / Identity); Platform is the completeness gate
- No org-wide directory product; project Access People teams only

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Strengthens collaboration + ownership/governance inspection of teams; not a static HR directory clone |
| **II. Domain Before Framework** | PASS | Team/membership semantics presented from fixtures; no authz engine inside UI; pure helpers for tone/label mapping |
| **III. Runtime Separation** | PASS | Renderer feature only; no Main/Node privilege; mock does not claim live core |
| **IV. Monorepo deps** | PASS | access → shell mocks, design-system, state; no reverse domain imports |
| **V. Contracts-First** | PASS | UI contract `contracts/teams-detail-ui.md`; extends 004 access-ui team detail |
| **VI. Security** | PASS | Display-only policies/membership; no secrets; mock archive is not real mutation |
| **VII. Safe execution** | PASS | No destructive apply path; mock toasts only |
| **VIII. Local-first** | PASS | Fixtures work offline; no cloud dependency |
| **IX. Performance** | PASS | No Main work; small tables; body scroller only |
| **X. Resilience** | PASS | loading / error / not-found / empty collections explicit |
| **XI. Observability** | PASS | No fake correlation IDs; activity is presentation of fixture events |
| **XII. Testability** | PASS | Helpers pure; composition contract tests without Electron |
| **XIII. Real code** | PASS | Extend `feature-access` + shell mocks; no scenic package |
| **XIV. SSOT** | PASS | One `TeamDetail`; replace thin inline block |
| **XV. Big-bang** | PASS | Incremental on operator surfaces / refactor branch |
| **XVI. Docs** | PASS | Spec + plan + contract + data model + quickstart |

**Hard stops**: No security/dependency/SSOT violations.  
**Post-Phase 1 re-check**: PASS — design keeps mock ports, honest states, single composition, display-only policy/membership picture, bidirectional navigation via existing activation targets.

---

## Project Structure

### Documentation (this feature)

```text
specs/015-teams-detail/
├── plan.md                 # This file
├── research.md             # Phase 0
├── data-model.md           # Phase 1
├── quickstart.md           # Phase 1
├── contracts/
│   └── teams-detail-ui.md  # Phase 1 (UI contract)
├── checklists/
│   └── requirements.md
└── tasks.md                # /speckit-tasks — not this command
```

### Source Code (touch list)

```text
packages/features/access/src/
├── team-detail.tsx             # TeamDetail composition (DetailPanel sections)
├── team-detail-helpers.ts      # status/presence/policy/result/kind tone+label pure helpers
├── team-detail.test.ts         # helpers + composition contract + fixture field checks
├── index.tsx                   # AccessView: mount TeamDetail for team targets
├── detail-targets.ts           # unchanged activation model (team entityKind)
├── detail-targets.test.ts
└── member-detail.tsx           # existing onOpenTeam path remains

packages/features/shell/src/workspace-mocks/
├── types.ts                    # WorkspaceTeam + nested roster/resource/policy/activity types
├── data.ts                     # TEAM_MOCKS enriched
└── index.ts                    # type re-exports via export type *

apps/desktop/src/renderer/
└── detail-fixture-coverage.test.ts  # Platform Team completeness + count consistency
```

**Structure Decision**:
- **Runtime owner**: Desktop renderer feature `@polterware/feature-access`.
- **Domain vs adapter**: Presentation over organizations/teams/membership concepts; fixtures stand in for future core team DTO. No new domain package.
- **Contracts**: Feature UI contract only; no protobuf for mock stage.
- **No new packages**: Constitution XIII — extend existing access + shell mocks.

---

## Complexity Tracking

> No constitution violations requiring justification.

| Item | Note |
| --- | --- |
| Feature vs git branch | Spec dir is `015-teams-detail`; git branch may remain `refactor/new`. Active `.specify/feature.json` MUST point to `specs/015-teams-detail` when planning/implementing this feature. |
| Pre-landed implementation | Plan documents as-built design; residual work is polish/validation via `/speckit-tasks` or manual QA. |
