# Quickstart: Access People Detail

**Feature**: `012-people-detail`  
**Goal**: Validate the mock People member DetailPanel end-to-end without claiming live authz.

Related: [spec.md](./spec.md) · [contracts/people-detail-ui.md](./contracts/people-detail-ui.md) · [data-model.md](./data-model.md)

---

## Prerequisites

- Repo on branch with Access People list + global DetailPanel host working (operator product surfaces baseline).
- Workspace mocks include enriched `PEOPLE_MOCKS` (`joinedAt`, `authMethod`, `mfaEnabled`, `permissions`, `sessions`, `activity`).
- Desktop can open a project workspace (fixture org/project).

---

## Setup

```bash
# From monorepo root — only when the operator wants a live UI check
# (Agents.md: do not start dev servers unless asked)
pnpm --filter @polterware/desktop dev
```

Navigate: **Project → Access → People**.

---

## Automated validation (preferred for agents)

```bash
# Access package contract + helpers
cd packages/features/access
pnpm test

# Desktop fixture completeness for Admin person
cd apps/desktop
./node_modules/.bin/vitest run --config vitest.ui.config.ts src/renderer/detail-fixture-coverage.test.ts
```

**Expected**:
- access: `detail-targets` + `member-detail` tests pass
- fixture coverage: `PEOPLE_MOCKS[0]` has non-empty `permissions`, `sessions`, `activity`, plus `authMethod` / `joinedAt` / `mfaEnabled`

---

## Manual scenarios

### S1 — Inspect Admin member (P1)

1. Open Access → People.
2. Activate **Erick Barcelos** (Admin).
3. Confirm DetailPanel header shows name + User icon.
4. Confirm sections: Overview (email, SSO, MFA Enabled), Environments, Permissions (Allow / Request only), Teams (Platform), Sessions (Current + secondary), Activity, optional Notes.
5. Confirm no second page/modal inspector.

**Pass**: All sections visible; StatusChips for presence/MFA/effects; no crash.

### S2 — Mock governance actions (P1)

1. With Admin detail open, click **Change role** → toast “Role updated (mock)”.
2. Click **Add to team** → toast “Team picker opened (mock)”.
3. Click **Revoke access** → toast “Access revoked (mock)”.
4. Activate a **non-current** session row → toast session revoked (mock).
5. Current session row does not revoke.

**Pass**: Toasts honest; panel stays on member; no fake persistence claim beyond toast.

### S3 — Viewer least-privilege posture (P2)

1. Activate **João Souza** (Viewer).
2. Permissions show **Deny** / **Request only** for write-style capabilities.
3. MFA shows **Disabled**.
4. If no matching teams filtered, Teams empty state shows.

**Pass**: Deny posture visible; no unrestricted write allows for Viewer fixture.

### S4 — Not found / empty honesty (P2)

1. Force activation of unknown member id (dev hook or temporary target) → **Member not found**.
2. Or pass empty `teams` / empty collections → explicit empty table copy (No team memberships / No active sessions / No recent activity).

**Pass**: Honest empty/not-found; no invented rows.

### S5 — Entity kind isolation

1. On People tab, open an **Invitation** or **Team** detail.
2. Confirm `MemberDetail` is not mounted for those kinds.
3. Return to a member row → member detail returns.

**Pass**: No dual content clash.

---

## Definition of done (plan gate)

| Check | Evidence |
| --- | --- |
| FR-001–FR-015 covered by UI contract + composition | [people-detail-ui.md](./contracts/people-detail-ui.md) |
| Admin fixture complete | detail-fixture-coverage |
| Helpers + composition contract | `packages/features/access` tests |
| SSOT single MemberDetail | `index.tsx` imports `./member-detail` only |
| No frontend authz claim | contract Security honesty + mock toasts |

---

## Out of scope for this quickstart

- Live IdP / SSO configuration
- Real session termination API
- Roles matrix editor
- Agents detail
- ConnectRPC membership CRUD
