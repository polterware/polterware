# Implementation Plan: Inbox Approval Detail

**Branch**: `022-approval-detail` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/022-approval-detail/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `004-operator-product-surfaces` (Inbox IA + DetailPanel host), shell `WorkspaceApproval` / `APPROVAL_MOCKS`, production detail patterns (`member-detail`, `run-detail`, `team-detail`), DESIGN.md + global `DetailPanel` LAW

**Risk control**: Mock/session decide only — no live policy engine, no remote apply, no plan→approve→apply claim. Single `ApprovalDetail` composition; no second inspector. **Parallel agents own** process / architecture-view / invitation / architecture-components — this plan MUST NOT touch those files.

---

## Summary

Deliver a **production-quality Inbox Approval DetailPanel** (mocked): operators inspect approval requests from Pending / Approved / Rejected tabs — risk, status, environment, requester (person|agent), related system context, policy, justification, and decision trail — and can Approve/Reject pending items via session-local state with honest mock feedback. Extract the thin inline DetailPanel block from `inbox-page.tsx` into a dedicated module matching MemberDetail / RunDetail quality.

**Technical approach**:

1. **Keep / lightly enrich `WorkspaceApproval` fixtures** (pending + approved + rejected; ≥1 high-risk; ≥1 agent-requested — already present in APPROVAL_MOCKS).
2. **Extract `ApprovalDetail`** composition (`layout="sections"`, metrics with StatusChip, metadata, loading/error/not-found).
3. **Wire from `InboxPage`** for `scopeId: inbox:{tab}` + `entityKind: approval`; pass session decide handlers and optional open-run callback.
4. **Keep honesty**: decide mutates session/local status (+ decision trail fields when deciding); never claims live policy or remote apply.
5. **Tests**: pure helpers + composition source contract + desktop detail-fixture-coverage for approval richness.

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9 (renderer features)

**Primary Dependencies**:
- `@polterware/feature-shell` (owner — Inbox + workspace approval mocks)
- `@polterware/frontend-design-system` (`DetailPanel`, `StatusChip`, icons, `PageLoading` / `PageError`)
- `@polterware/frontend-state` (detail target activation already used by Inbox)

**Storage**: Fixture-only in shell workspace mocks + session-local status map in InboxPage. No SQLite / ConnectRPC approval API in this slice.

**Testing**:
- Vitest node: `packages/features/shell` (`approval-detail-helpers`, composition source contract, fixture field presence)
- Vitest ui (desktop): `detail-fixture-coverage.test.ts` asserts high-risk + agent-requested + decision trail

**Target Platform**: Desktop renderer Inbox → Pending/Approved/Rejected; browser-safe feature package only

**Project Type**: Desktop product feature (Solid feature package)

**Performance Goals**:
- Approval detail open feels immediate (fixture resolve after activation)
- DetailPanel body is sole vertical scroller
- No Main-thread or IPC work for this stage

**Constraints**:
- Renderer: unprivileged; mock decide is not privilege escalation
- DESIGN / AGENTS LAW: global DetailPanel only; StatusChip for risk and status; English UI strings
- Mutations mock/session only
- Parallel file boundaries: do not touch process/architecture/invitation detail files or run-detail implementation
- Live policy engine / apply pipeline deferred

**Scale/Scope**:
- One entity kind detail: **approval**
- Five fixtures (appr-1…appr-5) covering pending/approved/rejected, high-risk, agent-requested
- Project Inbox only

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Governance gate for privileged operations in plan→approve→apply loop |
| **II. Domain Before Framework** | PASS | Approval/risk/requester semantics from fixtures; pure helpers for tones |
| **III. Runtime Separation** | PASS | Renderer feature only; mock does not claim live core policy engine |
| **IV. Monorepo deps** | PASS | shell → design-system, state; no reverse domain imports |
| **V. Contracts-First** | PASS | UI contract `contracts/approval-detail-ui.md` |
| **VI. Security** | PASS | UI is not privilege escalation; mock approve is not real mutation |
| **VII. Safe execution** | PASS | No destructive apply path; session-local status only |
| **VIII. Local-first** | PASS | Fixtures work offline; no cloud dependency |
| **IX. Performance** | PASS | No Main work; body scroller only |
| **X. Resilience** | PASS | loading / error / not-found / sparse related fields explicit |
| **XI. Observability** | PASS | Related run id/result when fixture-backed; no fake correlation |
| **XII. Testability** | PASS | Helpers pure; composition contract tests without Electron |
| **XIII. Real code** | PASS | Extend `feature-shell`; no scenic package |
| **XIV. SSOT** | PASS | One `ApprovalDetail`; replace thin inline block |
| **XV. Big-bang** | PASS | Incremental on operator surfaces / refactor branch |
| **XVI. Docs** | PASS | Spec + plan + contract + data model + quickstart |

**Hard stops**: No security/dependency/SSOT violations; no sibling feature rewrites.  
**Post-Phase 1 re-check**: PASS — design keeps mock ports, honest states, single composition.

---

## Project Structure

```text
specs/022-approval-detail/
  spec.md
  plan.md
  research.md
  data-model.md
  quickstart.md
  tasks.md
  checklists/requirements.md
  contracts/approval-detail-ui.md

packages/features/shell/src/
  approval-detail.tsx              # NEW — DetailPanel composition
  approval-detail-helpers.ts       # NEW — pure tone/label helpers
  approval-detail.test.ts          # NEW — helpers + composition contract
  inbox-page.tsx                   # MINIMAL — replace inline with ApprovalDetail

packages/features/shell/src/workspace-mocks/
  types.ts                         # WorkspaceApproval (existing; enrich only if needed)
  data.ts                          # APPROVAL_MOCKS (ensure coverage gates)

apps/desktop/src/renderer/
  detail-fixture-coverage.test.ts  # Expand approval asserts only
```

---

## Complexity Tracking

| Item | Why needed | Simpler alternative rejected |
| --- | --- | --- |
| Session-local status + decision fields on decide | US2 requires decided trail after Approve/Reject without live API | Toast-only without status move fails SC-002 / collection tabs |
| Optional `onOpenRun` callback | Hand-off to existing run detail without reimplementing | Inline run mini-view would dual-implement run-detail |
