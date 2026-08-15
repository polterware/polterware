# Research: Access Invitation Detail

**Feature**: `024-invitation-detail`  
**Date**: 2026-08-12  
**Status**: Complete — no open NEEDS CLARIFICATION

---

## R1 — Detail host and composition pattern

**Decision**: Use the single global `DetailPanel` with `Body layout="sections"` + `Grid` / `Section` / `Metric` / `Metadata`, matching MemberDetail / TeamDetail. Extract a dedicated `InvitationDetail` component; replace the thin inline invitation block in `AccessView`.

**Rationale**:
- AGENTS.md / DESIGN.md LAW forbid second inspectors for operational entities.
- MemberDetail / TeamDetail define the production Access detail chrome.
- Extraction makes composition testable under parallel sibling work (022/021/023/025).

**Alternatives considered**:

| Alternative | Why rejected |
| --- | --- |
| Full-page invitation route | Violates DetailPanel LAW |
| Keep inline thin detail | Fails FR-011 / SC-004 and rejection rubric |
| Document layout | Reserved for Knowledge / Spec Kit |

---

## R2 — Data source for mock stage

**Decision**: Enrich `WorkspaceInvitation` in `@polterware/feature-shell` workspace mocks (`INVITATION_MOCKS`). Access consumes mocks; no SMTP, IdentityService, or org directory API.

**Rationale**:
- Invitations table already uses `INVITATION_MOCKS`; detail must not invent a second type.
- Shell mocks are the SSOT for operator surface demos.
- Future live DTO can replace fixtures without changing section structure.

**Alternatives considered**:

| Alternative | Why rejected |
| --- | --- |
| Feature-local fixtures only in access | Dual invitation SSOT |
| Fake live email API | Dishonest delivery claim |

---

## R3 — Lifecycle actions honesty

**Decision**: Header actions by status — Pending: Resend + Cancel (danger); Expired: Re-invite; Cancelled: none. All fire mock toasts with explicit `(mock)` labeling. Never claim SMTP success or mailbox delivery.

**Rationale**:
- Spec §8.10 / invitation invariants require expiry governance; UI shows intent only.
- Constitution VII: mock stage only; no plan→approve→apply email pipeline here.

---

## R4 — Team hand-off without member profile invention

**Decision**: Optional `teamId` on invitation enables `onOpenTeam` when resolvable. Never open or fabricate a member detail for unaccepted invites. Invitee is email-only until acceptance.

**Rationale**:
- Invitations sit between anonymous email and membership.
- Team detail is sibling-owned (`015-teams-detail`); invitation only activates team target.

---

## R5 — Parallel file boundaries

**Decision**: This feature may touch only:

- `invitation-detail*`
- Access `index.tsx` invitation mount wiring only
- `WorkspaceInvitation` / `INVITATION_MOCKS`
- desktop `detail-fixture-coverage` invitation asserts
- `specs/024-invitation-detail/**`

**MUST NOT touch**: approval, process, architecture view/components, agent/policy/audit/session/member/team detail modules.

---

## R6 — Fixture set

**Decision**: Keep Pending (`i1`, `i2`) and Expired (`i3`) as primary table-visible fixtures with complete assignment + provenance. Add Cancelled (`i4`) for direct-activation honesty tests; People table continues to filter Pending | Expired only.
