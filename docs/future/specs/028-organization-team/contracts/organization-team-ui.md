# UI Contract: Organization Team

## Collection (OrganizationTeamPage)

- Host: Organization sidebar → Team (`org-settings` / `people` routes to this page content).
- Layout: hierarchy max-width content (`max-w-[1100px] mx-auto px-6 py-8`).
- Header: title **Team**; muted description that members are organization-level (not project Access).
- Primary action: **Invite members** → mock dialog or `pushToast` with non-delivery wording.
- DataTable:
  - PrimaryCell: User icon (identity tone) + member name
  - Columns: Role, MFA (`StatusChip`), optional Status, optional Last active
  - Sortable column headers; no Actions column
  - Row activation → `detailPanelStore.activateTarget`
  - Empty: honest no-members / no-match message
- Collection registration: `scopeId = org-team:{organizationId}`, `entityKind = org-member`

## Detail (OrgMemberDetail)

- Global `DetailPanel` only.
- `DetailPanel.Body layout="sections"` + `DetailPanel.Grid`.
- Header: User icon + name; actions Change role, Invite (optional), Remove (danger) — all mock/honest.
- Metrics: Role, MFA StatusChip, Status StatusChip when present, Teams/Projects counts when present.
- Overview Metadata: Email, Role, MFA, Joined, optional Invited by, Last active, Title.
- States: loading (`PageLoading`), error/not-found (`PageError`).

## Honesty

- Never: “email sent”, “member removed remotely”, live sync claims.
- Prefer: “mock — no email delivery”, “session mock — not applied remotely”.

## Forbidden

- Hand-rolled status pills
- Second entity detail page / modal / drawer
- Project Access People/Teams detail rewrites
- Actions column on DataTable
