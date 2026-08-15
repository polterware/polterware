# Quickstart: Organization Team

## Verify in Desktop (when dev server allowed)

1. Open an organization workspace (e.g. Polterware).
2. Sidebar → **Team**.
3. Confirm table: identity PrimaryCell, role, MFA StatusChips (enabled + disabled).
4. Activate a row → DetailPanel opens with email/role/MFA/joined.
5. Trigger Invite members → toast/dialog says mock, not real email.
6. Trigger Change role / Remove → honest mock toast.

## Focused tests

```bash
pnpm --filter @polterware/feature-shell test
```

Expect green: `organization-team.test.ts`, `org-member-detail.test.ts`, existing shell tests unchanged in intent.

## Rubric

- [x] Page is more than a thin raw table without detail
- [x] MFA always StatusChip
- [x] Row activation opens DetailPanel
- [x] `ORG_TEAM_MOCKS` exported and reusable
- [x] Composition/page tests exist
- [x] Project Access details untouched by this feature’s files
- [x] Not “already implemented” via old 3-member local array alone
