# Research: Organization Team

**Date**: 2026-08-12 | **Feature**: [spec.md](./spec.md)

## Current state

- `organization-team.tsx` (~62 lines): local `MEMBERS` array of 3, MFA Enabled as StatusChip but Disabled as plain text, no row activation, no DetailPanel, invite is optional callback only.
- Org sidebar **Team** routes to `org-settings` / `people` via `goOrganizationTab` → `OrganizationSettingsPage` people tab, which duplicates another local 3-member table (also no detail).
- `OrganizationTeamPage` is exported but not the live path content today.
- Project Access People use `PEOPLE_MOCKS` + `MemberDetail` — different scope (project), richer permissions/sessions model. Must not reuse as if identical.

## Product source of truth

- `docs/spec.md` §8.10 Organizations: ownership, teams, memberships, invitations.
- §107 Organizations: at least one owner; remove member revokes new access (real behavior future — mock only here).
- AGENTS hierarchy: organization sidebar Team is first-class; project Access is separate.

## Decisions

| Decision | Choice | Rationale |
| --- | --- | --- |
| Entity model | New `WorkspaceOrgMember` + `ORG_TEAM_MOCKS` | Distinct from project `WorkspacePerson`; avoids rewriting Access fixtures |
| Detail host | Global DetailPanel via `detailPanelStore` | Product LAW; mirror Inbox/ApprovalDetail |
| MFA display | StatusChip for enabled (healthy) and disabled (muted) | Rejection rubric; consistency with MemberDetail MFA |
| Invite honesty | `pushToast` with explicit mock wording | User requirement: never claim email delivery |
| Live route | Render `OrganizationTeamPage` from org-settings people tab | Team sidebar already lands there; avoids route churn for parallel agents |
| Nested org teams table | Out of page MVP | Spec focuses on memberships; project Access Teams remains separate |

## Alternatives rejected

- Reusing `PEOPLE_MOCKS` as org members: conflates project Access with org membership.
- Building a second inspector page: forbidden by DetailPanel LAW.
- Claiming “Invite sent” success toast (current App.tsx onInvite): dishonest.

## References

- Pattern: `packages/features/shell/src/inbox-page.tsx` + `approval-detail.tsx`
- MFA StatusChip: `packages/features/access/src/member-detail.tsx` (reference only — do not edit)
