# Quickstart: Access Invitation Detail

**Feature**: `024-invitation-detail`  
**Goal**: Validate the mock Invitation DetailPanel without claiming live email delivery.

Related: [spec.md](./spec.md) · [contracts/invitation-detail-ui.md](./contracts/invitation-detail-ui.md) · [data-model.md](./data-model.md)

---

## Prerequisites

- Access → People invitations table + global DetailPanel host working.
- Enriched `INVITATION_MOCKS` (Pending + Expired with assignment/provenance).
- Desktop can open a project workspace (fixture org/project).

---

## Setup

```bash
# Only when the operator wants a live UI check
# (Agents.md: do not start dev servers unless asked)
pnpm --filter @polterware/desktop dev
```

Navigate: **Project → Access → People → Invitations**.

---

## Automated validation (preferred)

```bash
cd packages/features/access
pnpm test

# Invitation-related asserts in desktop fixture coverage (from monorepo root / desktop)
cd apps/desktop
./node_modules/.bin/vitest run --config vitest.ui.config.ts src/renderer/detail-fixture-coverage.test.ts
```

**Expected**:
- access: `invitation-detail` tests pass (helpers + composition + Access wiring)
- fixture coverage: Pending and Expired invitations have assignment + provenance fields

---

## Manual scenarios

1. Activate Pending invite (`ana@example.com`) → Overview StatusChip Pending, Resend + Cancel actions.
2. Activate Expired invite (`old@example.com`) → Re-invite only; no Cancel.
3. Trigger Resend/Cancel/Re-invite → toast includes `(mock)`; no SMTP claim.
4. When team is resolvable → Open team hand-off activates team detail.
5. Unknown invitation id → honest not-found (entityKind invitation mount).
