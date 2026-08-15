# Implementation Plan: Access Agent Session Detail

**Branch**: `020-agent-session-detail` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/020-agent-session-detail/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `004-operator-product-surfaces` (Access IA + DetailPanel host), existing Access Agents Recent sessions table, DESIGN.md + global `DetailPanel` LAW, shell workspace mocks, patterns from `012-operations-run-detail` (steps + progress bar) and `012-people-detail` / `015-teams-detail` (dedicated composition + honest states)

**Risk control**: Mock stage only — no live agent runtime, no real terminate/kill, no agent identity detail rewrite. Single `AgentSessionDetail` composition; no second inspector. Parallel agents own 017/018/019 — do not touch those files.

**Parallelism boundary**: MAY call `onOpenAgent(agentId)` for hand-off. MUST NOT implement or refactor `agent-detail.tsx`, `policy-detail*`, or `audit-detail*`.

---

## Summary

Deliver a **production-quality Access → Agents → Agent Session DetailPanel** (mocked): operators inspect operational agent sessions — status, environment, timing, objective, agent identity link, step progress (including DitherSegmentedBar), ordered steps table, waiting/approval linkage when applicable, related ops/audit ids, and actor context — and can trigger honest mock Terminate (Running/Waiting only) plus Open agent / optional Open related operation without leaving Access or inventing a second detail surface.

**Technical approach** (see research):

1. **Enrich `WorkspaceAgentSession` / step fixtures** with production detail fields (timing, tools, waiting, approval, related ids, dataSource, Failed step status).
2. **Extract `AgentSessionDetail`** composition matching Run/Member/Team quality (`layout="sections"`, metrics, StatusChip, DataTable steps, progress bar).
3. **Wire from `AccessView`** only for `scopeId: access:agents` + `entityKind: agent-session`; replace inline `Show when={agentSession()}` block; pass mock handlers + agent lookup.
4. **Keep runtime honesty**: Terminate is toast-only; source labeled fixture.
5. **Tests**: pure helpers + composition contract + desktop detail-fixture-coverage expansion.

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9 (renderer features)

**Primary Dependencies**:
- `@polterware/feature-access` (owner)
- `@polterware/feature-shell` workspace mocks (`WorkspaceAgentSession`, `AGENT_SESSION_MOCKS`, `AGENT_MOCKS` lookup only)
- `@polterware/frontend-design-system` (`DetailPanel`, `DataTable`, `StatusChip`, icons, `PageLoading` / `PageError`, toasts)
- `@polterware/frontend-charts` (`DitherSegmentedBar`)
- `@polterware/frontend-state` (detail target activation already used by Access)

**Storage**: Fixture-only in shell workspace mocks. No SQLite / ConnectRPC / live agent session API in this slice.

**Testing**:
- Vitest node: `packages/features/access` (helpers, composition source contract, wiring source checks)
- Vitest ui (desktop): `detail-fixture-coverage.test.ts` asserts enriched session completeness + multi-status coverage
- Access `detail-targets` remains green for `agent-session` activation

**Target Platform**: Desktop renderer Access → Agents → Recent sessions; browser-safe feature package only

**Project Type**: Desktop product feature (Solid feature package)

**Performance Goals**:
- Session detail open feels immediate (fixture resolve < 1 frame after activation)
- DetailPanel body is sole vertical scroller; fixture step tables stay small (< ~20 rows)
- No Main-thread or IPC work for this stage

**Constraints**:
- Renderer: unprivileged; no secrets/tool credentials in session DTOs
- DESIGN / AGENTS LAW: global DetailPanel only; StatusChip for status; DataTable for collections; no PrimaryCell descriptions; no Actions columns; English UI strings
- Mutations mock-only with toast honesty
- Agent identity detail owned by parallel agent — link only
- Live agent runtime deferred

**Scale/Scope**:
- One entity kind detail: **agent-session**
- Four+ fixtures covering Running / Completed / Waiting / Failed
- File boundary limited to access session modules + shell session types/data + desktop fixture coverage + this specs folder

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Strengthens observation of agent operational sessions + waiting/approval honesty |
| **II. Domain Before Framework** | PASS | Session/step semantics from fixtures; pure helpers for tones/labels; no orchestration engine in UI |
| **III. Runtime Separation** | PASS | Renderer feature only; no Main/Node privilege; mock does not claim live core |
| **IV. Monorepo deps** | PASS | access → shell mocks, design-system, charts, state; no reverse domain imports |
| **V. Contracts-First** | PASS | UI contract `contracts/agent-session-detail-ui.md` |
| **VI. Security** | PASS | No secrets; terminate is mock; UI not authz enforcement |
| **VII. Safe execution** | PASS | No real kill path; mock toasts only |
| **VIII. Local-first** | PASS | Fixtures work offline |
| **IX. Performance** | PASS | No Main work; small tables; body scroller only |
| **X. Resilience** | PASS | loading / error / not-found / empty steps explicit |
| **XI. Observability** | PASS | Correlation/session ids when present; no fake live streams |
| **XII. Testability** | PASS | Helpers pure; composition contract tests without Electron |
| **XIII. Real code** | PASS | Extend `feature-access` + shell mocks; no scenic package |
| **XIV. SSOT** | PASS | One `AgentSessionDetail`; replace thin inline block |
| **XV. Big-bang** | PASS | Incremental on operator surfaces / refactor branch |
| **XVI. Docs** | PASS | Spec + plan + contract + data model + quickstart |

**Hard stops**: No security/dependency/SSOT violations; no invasion of 017/018/019 files.  
**Post-Phase 1 re-check**: PASS — design keeps mock ports, honest states, single composition, agent hand-off only.

---

## Project Structure

### Documentation (this feature)

```text
specs/020-agent-session-detail/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── agent-session-detail-ui.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

### Source (implementation boundary)

```text
packages/features/shell/src/workspace-mocks/
├── types.ts          # WorkspaceAgentSession + Step enrich
├── data.ts           # AGENT_SESSION_MOCKS enrich + Failed fixture
└── index.ts          # re-exports (if needed)

packages/features/access/src/
├── agent-session-detail.tsx
├── agent-session-detail-helpers.ts
├── agent-session-detail.test.ts
└── index.tsx         # ONLY replace agentSession Show with <AgentSessionDetail />

apps/desktop/src/renderer/
└── detail-fixture-coverage.test.ts   # session completeness asserts
```

**Structure decision**: Stay inside existing `feature-access` + shell mocks. Mirror `run-detail` for steps/progress and `member-detail` for loading/not-found/header actions. No new package.

---

## Complexity Tracking

> No constitution violations requiring justification.

| Item | Why needed | Simpler alternative rejected because |
| --- | --- | --- |
| — | — | — |
