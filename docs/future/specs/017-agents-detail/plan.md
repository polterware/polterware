# Implementation Plan: Access Agents Detail

**Branch**: `017-agents-detail` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/017-agents-detail/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `004-operator-product-surfaces` (Access IA + DetailPanel host), `007-remaining-operator-surfaces` (Agents table), `012-people-detail` / `015-teams-detail` patterns, DESIGN.md + global `DetailPanel` LAW, shell workspace mocks

**Risk control**: Mock stage only — no live IdentityService, real pause/revoke, or MCP invocation. Capability/tool tables are display-only. Single `AgentDetail` composition; no second inspector. **Parallel agents own** session/policy/audit details — this plan MUST NOT rewrite those files.

---

## Summary

Deliver a **production-quality Access → Agents DetailPanel** (mocked): operators inspect agent posture — identity, status, autonomy, owner, scope, structured capabilities/tools, environments, linked operations, recent sessions (hand-off only), and activity/notes — and can trigger honest mock governance actions (pause/resume, edit permissions, revoke) without leaving Access or inventing a second detail surface. Session and operation deep-links use callbacks so sibling features can own their UIs.

**Technical approach** (see research):

1. **Extend `WorkspaceAgent` fixtures** with structured capabilities/tools and optional description/activity/notes.
2. **Extract `AgentDetail`** composition matching MemberDetail / TeamDetail quality (`layout="sections"`, metrics, metadata, DataTables, StatusChip).
3. **Wire from `AccessView`** only for `scopeId: access:agents` + `entityKind: agent`; pass mock action handlers, filtered sessions, and operation/session callbacks.
4. **Keep authz honesty**: UI never claims frontend authorization or live agent runtime; toasts are mock-labeled.
5. **Tests**: pure helpers + source/composition contract + desktop detail-fixture-coverage for Deployment Agent richness.

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9 (renderer features)

**Primary Dependencies**:
- `@polterware/feature-access` (owner)
- `@polterware/feature-shell` workspace mocks (`WorkspaceAgent`, `AGENT_MOCKS`, read `AGENT_SESSION_MOCKS`)
- `@polterware/frontend-design-system` (`DetailPanel`, `DataTable`, `StatusChip`, icons, `PageLoading` / `PageError`, toasts)
- `@polterware/frontend-state` (detail target activation already used by Access)

**Storage**: Fixture-only in shell workspace mocks. No SQLite / ConnectRPC / polter-core agent control tables in this slice.

**Testing**:
- Vitest node: `packages/features/access` (`agent-detail-helpers`, composition source contract, fixture field presence via source/fixture reads)
- Vitest ui (desktop): `detail-fixture-coverage.test.ts` asserts Deployment Agent (`a1`) structured fields + session link
- Access `detail-targets` remains green for agent activation

**Target Platform**: Desktop renderer (Electron) Access → Agents; browser-safe feature package only

**Project Type**: Desktop product feature (Solid feature package)

**Performance Goals**:
- Agent detail open feels immediate (fixture resolve < 1 frame after activation)
- DetailPanel body is sole vertical scroller; fixture tables stay small (< ~20 rows/section)
- No Main-thread or IPC work for this stage

**Constraints**:
- Renderer: unprivileged; no secrets, tokens, or credential material in agent DTOs
- DESIGN / AGENTS LAW: global DetailPanel only; StatusChip for status; DataTable for collections; no PrimaryCell descriptions; no Actions columns; English UI strings
- Mutations mock-only with toast honesty (no plan→approve→apply claim)
- Parallel file boundaries: do not touch sibling detail files
- Live agent runtime control deferred

**Scale/Scope**:
- One entity kind detail: **agent** (not agent-session)
- Four fixtures (Deployment / Architecture / GitHub Sync / Linear Bridge); Deployment is the completeness gate
- Project Access Agents only

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Strengthens governance inspection of agents as first-class collaborators |
| **II. Domain Before Framework** | PASS | Agent/capability/tool semantics from fixtures; pure helpers for tone/label mapping |
| **III. Runtime Separation** | PASS | Renderer feature only; mock does not claim live core / IdentityService |
| **IV. Monorepo deps** | PASS | access → shell mocks, design-system, state; no reverse domain imports |
| **V. Contracts-First** | PASS | UI contract `contracts/agents-detail-ui.md` |
| **VI. Security** | PASS | Display-only capabilities/tools; no secrets; mock revoke is not real mutation |
| **VII. Safe execution** | PASS | No destructive apply path; mock toasts only |
| **VIII. Local-first** | PASS | Fixtures work offline; no cloud dependency |
| **IX. Performance** | PASS | No Main work; small tables; body scroller only |
| **X. Resilience** | PASS | loading / error / not-found / empty collections explicit |
| **XI. Observability** | PASS | No fake correlation IDs; activity is presentation of fixture events |
| **XII. Testability** | PASS | Helpers pure; composition contract tests without Electron |
| **XIII. Real code** | PASS | Extend `feature-access` + shell mocks; no scenic package |
| **XIV. SSOT** | PASS | One `AgentDetail`; replace thin inline block |
| **XV. Big-bang** | PASS | Incremental on operator surfaces / refactor branch |
| **XVI. Docs** | PASS | Spec + plan + contract + data model + quickstart |

**Hard stops**: No security/dependency/SSOT violations; no sibling file rewrites.  
**Post-Phase 1 re-check**: PASS — design keeps mock ports, honest states, single composition, display-only capability picture, session hand-off via callbacks only.

---

## Project Structure

```text
specs/017-agents-detail/
  spec.md
  plan.md
  research.md
  data-model.md
  quickstart.md
  tasks.md
  checklists/requirements.md
  contracts/agents-detail-ui.md

packages/features/access/src/
  agent-detail.tsx              # NEW — DetailPanel composition
  agent-detail-helpers.ts       # NEW — pure tone/label helpers
  agent-detail.test.ts          # NEW — helpers + composition contract
  index.tsx                     # MINIMAL — replace agent Show with AgentDetail

packages/features/shell/src/workspace-mocks/
  types.ts                      # WorkspaceAgent + nested types only
  data.ts                       # AGENT_MOCKS only

apps/desktop/src/renderer/
  detail-fixture-coverage.test.ts  # agent asserts only
```

---

## Implementation Phases (summary)

1. **Foundational**: Extend `WorkspaceAgent` + enrich `AGENT_MOCKS`; pure helpers + tests skeleton.
2. **US1 Inspect**: `AgentDetail` sections + wire Access agent mount.
3. **US2 Govern**: Header actions + mock toast wiring.
4. **US3 Navigate**: Session/operation callbacks + filtered sessions prop.
5. **US4 Honesty**: loading/not-found/empty states + entity kind isolation.
6. **Validation**: package tests green; fixture coverage agent asserts.

---

## Complexity Tracking

| Item | Why needed | Mitigation |
| --- | --- | --- |
| Structured capability/tool rows | Spec requires non-join collections | Same DataTable pattern as member permissions |
| Session hand-off without session UI | Parallel sibling ownership | Callbacks only; read session mocks |
| Parallel file boundaries | Three other Access details concurrent | Strict allowlist paths |

No unjustified new packages or design-system primitives.
