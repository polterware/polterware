# Contract: Invitation Detail UI

**Feature**: `024-invitation-detail`  
**Surface**: Access → People (Invitations table) → global DetailPanel  
**Stage**: Mock / fixture (production-ready composition)

## Host

- Collection: Access People invitations DataTable
- Detail host: MainSurface global DetailPanel only
- Scope id pattern: `access:people`
- Entity kind: `invitation` only
- Mount when `entityKind === "invitation"` even if fixture id is missing (honest not-found)

## Composition

- Dedicated `InvitationDetail` module (`invitation-detail.tsx` + helpers) — **MUST NOT** remain only an inline thin block in Access index
- Header: person/invite domain icon (same family as invitations PrimaryCell: `User` 16px) + single-line email title + status-appropriate `DetailPanel.Action` only
- Body: `layout="sections"` + Grid / Section / Metric / Metadata
- Status: `StatusChip` only (filled semantic chip; no hand-rolled pill)

## Header actions by status

| Status | Actions | Toast honesty |
| --- | --- | --- |
| Pending | Resend, Cancel (danger) | labels include mock intent; no SMTP success claim |
| Expired | Re-invite | mock re-invite intent only |
| Cancelled | none | — |

Optional when `teamId` is present and host provides `onOpenTeam`: **Open team** action (does not rewrite team-detail).

## Required sections

1. **Metrics** — Status (`StatusChip`), Expires, Role, Team
2. **Overview** — status, expires, role, team
3. **Assignment** — role, team, optional environments list
4. **Provenance** — invited-by, optional invited-at
5. **Message / Notes** — only when fixture-backed (`message`)

## Honesty

- loading → `PageLoading`
- error / not-found → `PageError` with “Invitation not found” / failed copy
- no fabricated member profile for unaccepted invites
- no live email delivery claims
- do not rewrite member/team/agent/policy/audit/session details
- People table may filter Pending | Expired; Cancelled fixtures remain valid for direct activation

## Non-goals

- SMTP / mailbox integration
- SSO / SCIM directory invites
- Org-level directory product surface
- Real membership mutation (accept still creates member only in future backend)
