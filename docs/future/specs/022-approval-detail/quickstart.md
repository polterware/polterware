# Quickstart validation: Inbox Approval Detail

**Feature**: `022-approval-detail`

## Prerequisites

- Repo root monorepo install already available (`pnpm` workspace)
- Feature artifacts under `specs/022-approval-detail/`

## Automated checks

```bash
# Shell package helpers + composition contract
pnpm --filter @polterware/feature-shell exec vitest run --config vitest.config.ts src/approval-detail.test.ts

# Full shell package tests
pnpm --filter @polterware/feature-shell exec vitest run --config vitest.config.ts

# Desktop fixture coverage (approval asserts)
pnpm --filter @polterware/desktop exec vitest run --config vitest.ui.config.ts src/renderer/detail-fixture-coverage.test.ts
```

## Manual operator walkthrough (mock stage)

1. Open Desktop project → **Inbox → Pending**.
2. Activate **Protected production deploy** (agent, high risk).
3. Confirm DetailPanel: Inbox/security icon, title, StatusChip risk + status, environment, requester kind Agent, related entities, policy, justification, Approve + Reject actions.
4. Click **Approve** — item leaves Pending; open **Approved** and confirm Decision section (decided-by / at / note). No claim of remote apply.
5. Open a **Rejected** fixture (e.g. Open inbound port 5432) — no Approve/Reject; Decision trail visible.
6. Force unknown id (if test harness supports) — not-found branch without crash.

## Honesty checklist

- [ ] No “policy engine enforced” or “apply completed remotely” copy unless `runResult` is fixture-backed
- [ ] StatusChip used for risk and status in detail body
- [ ] ApprovalDetail is imported module, not only inline inbox markup

## Contracts

- UI: [contracts/approval-detail-ui.md](./contracts/approval-detail-ui.md)
- Data: [data-model.md](./data-model.md)
