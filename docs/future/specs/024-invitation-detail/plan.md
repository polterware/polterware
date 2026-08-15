# Implementation Plan: Access Invitation Detail

**Branch**: `024-invitation-detail` (feature dir; git may remain `refactor/new`) | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/024-invitation-detail/spec.md`

**Related**: [research.md](./research.md) · [data-model.md](./data-model.md) · [quickstart.md](./quickstart.md) · [contracts/](./contracts/)

**Depends on**: `004-operator-product-surfaces` (Access IA + DetailPanel host), `007-remaining-operator-surfaces` (People invitations table), `012-people-detail` / `015-teams-detail` patterns, DESIGN.md + global `DetailPanel` LAW, shell workspace mocks

**Risk control**: Mock stage only — no SMTP, IdentityService, or real membership mutation. Single `InvitationDetail` composition; no second inspector. **Parallel P0 agents own** approval (022), process (021), architecture view (023), architecture components (025) — this plan MUST NOT rewrite those surfaces or sibling Access details (member/team/agent/policy/audit/session).

---

## Summary

Deliver a **production-quality Access → People (Invitations) DetailPanel** (mocked): operators inspect invitation posture — invitee email, status, expiry, role/team assignment, optional environments, provenance, optional message — and trigger honest mock lifecycle actions (Resend / Cancel for Pending; Re-invite for Expired) without inventing a member profile or claiming email delivery.

**Technical approach**:

1. **Extend `WorkspaceInvitation`** with optional `teamId`, `invitedAt`, `environments`, `message`.
2. **Enrich `INVITATION_MOCKS`** — Pending + Expired complete; optional Cancelled for honesty.
3. **Extract `InvitationDetail`** (+ helpers + composition tests) matching MemberDetail quality (`layout="sections"`, StatusChip, DetailPanel.Action).
4. **Wire from `AccessView`** only for `scopeId: access:people` + `entityKind: invitation` (mount even when not found).
5. **Tests**: helpers + source contract + desktop detail-fixture-coverage invitation asserts.

---

## Technical Context

**Language/Version**: TypeScript 5.x, SolidJS 1.9 (renderer features)

**Primary Dependencies**:
- `@polterware/feature-access` (owner)
- `@polterware/feature-shell` workspace mocks (`WorkspaceInvitation`, `INVITATION_MOCKS`, `TEAM_MOCKS` for resolve)
- `@polterware/frontend-design-system` (`DetailPanel`, `StatusChip`, icons, `PageLoading` / `PageError`, toasts)
- `@polterware/frontend-state` (detail target activation already used by Access)

**Storage**: Fixture-only in shell workspace mocks. No SQLite / ConnectRPC / SMTP in this slice.

**Testing**:
- Vitest node: `packages/features/access` (`invitation-detail-helpers`, composition source contract, fixture field presence)
- Vitest ui (desktop): `detail-fixture-coverage.test.ts` asserts Pending + Expired richness
- Access `detail-targets` remains green for invitation activation

**Target Platform**: Desktop renderer Access → People → Invitations; browser-safe feature package only

**Project Type**: Desktop product feature (Solid feature package)

**Performance Goals**:
- Invitation detail open feels immediate (fixture resolve < 1 frame)
- DetailPanel body is sole vertical scroller
- No Main-thread or IPC work for this stage

**Constraints**:
- Renderer: unprivileged; no secrets
- DESIGN / AGENTS LAW: global DetailPanel only; StatusChip for status; no Actions columns
- Mutations mock-only with toast honesty (`(mock)` labels)
- Parallel file boundaries strict
- No fabricated member profile for unaccepted invites

**Scale/Scope**:
- One entity kind detail: **invitation**
- ~3–4 fixtures (Pending ×2, Expired ×1, optional Cancelled)
- Project Access People only

---

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --- | --- | --- |
| **I. Operational Architecture** | PASS | Strengthens governed membership onboarding inspection |
| **II. Domain Before Framework** | PASS | Invitation/assignment/provenance semantics from fixtures; pure helpers |
| **III. Runtime Separation** | PASS | Renderer feature only; mock does not claim live core / SMTP |
| **IV. Monorepo deps** | PASS | access → shell mocks, design-system, state |
| **V. Contracts-First** | PASS | UI contract `contracts/invitation-detail-ui.md` |
| **VI. Security** | PASS | No secrets; mock cancel/resend is not real mutation |
| **VII. Mutation safety** | PASS | Mock toasts only |
| **XII. Test plan** | PASS | helpers + composition + fixture coverage |
| **XIV. Single source of truth** | PASS | one InvitationDetail composition |

**Post-design re-check**: PASS — extraction + enriched fixtures + honest states satisfy FR/SC without live delivery claims.

---

## Project Structure

### Documentation (this feature)

```text
specs/024-invitation-detail/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/invitation-detail-ui.md
└── checklists/requirements.md
```

### Source (touched)

```text
packages/features/shell/src/workspace-mocks/
  types.ts          # WorkspaceInvitation fields
  data.ts           # INVITATION_MOCKS enrichment
packages/features/access/src/
  invitation-detail.tsx
  invitation-detail-helpers.ts
  invitation-detail.test.ts
  index.tsx         # invitation Show wiring only
apps/desktop/src/renderer/
  detail-fixture-coverage.test.ts  # invitation asserts only
```

### Forbidden (parallel P0 / sibling Access)

- approval-detail*, process-detail*, architecture view/components
- agent-detail*, policy-detail*, audit-detail*, agent-session-detail*
- member-detail*, team-detail* (except reading TEAM_MOCKS for resolve)

---

## Implementation Phases

1. **Foundational**: extend type + fixtures + helpers + tests for helpers
2. **US1**: InvitationDetail composition (sections, StatusChip, Overview/Assignment/Provenance)
3. **US2**: status-appropriate actions + mock toasts from Access wiring
4. **US3**: optional onOpenTeam + message section
5. **US4**: loading / not-found / Cancelled fixture honesty
6. **Polish**: fixture coverage gate + access tests green

---

## Complexity Tracking

No constitution violations requiring justification. Scope is intentionally narrow (one entity kind, mock stage).
