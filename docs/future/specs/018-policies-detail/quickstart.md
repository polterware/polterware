# Quickstart: Access Policies Detail

**Feature**: `018-policies-detail`  
**Stage**: Mock UI validation

## Prerequisites

- Desktop app can open Access → Policies (fixture data).
- Detail panel host works (existing MainSurface / DetailPanel).

## Scenarios

### S1 — Open primary policy

1. Navigate to a project → **Access** → **Policies**.
2. Activate **Production operations approval**.
3. Confirm header: ShieldCheck icon + policy name.
4. Confirm metrics: Active StatusChip, scope Production, enforcement Required, approval Required.
5. Confirm structured Actors, Environments, Allowed/Blocked actions, Approval & risk, Update history, Related audit (non-empty).

### S2 — Mock governance actions

1. With policy detail open, click **Edit**, **Duplicate**, **Disable**, **Delete**.
2. Each action shows a toast explicitly labeled **(mock)**.
3. Panel remains on the same policy (no second page/modal).

### S3 — Empty blocked actions

1. Open **Staging operator access** (`p4` or equivalent with empty blocked list).
2. Confirm Blocked actions shows explicit empty state, not invented rows.

### S4 — Not found

1. Activate a policy entity id not present in `POLICY_MOCKS` (dev/test hook).
2. Confirm **Policy not found** + PageError message.

### S5 — Related audit callback only

1. On primary policy, activate a Related audit row if interactive.
2. Confirm navigation/callback intent only; this feature does not own Audit detail body.

## Automated gates (preferred)

```bash
cd packages/features/access && pnpm test
cd apps/desktop && ./node_modules/.bin/vitest run --config vitest.ui.config.ts src/renderer/detail-fixture-coverage.test.ts
```

Agents.md: do not run `pnpm dev` / full build unless the operator asks.
