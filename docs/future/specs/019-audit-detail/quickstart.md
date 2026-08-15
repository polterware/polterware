# Quickstart: Access Audit Event Detail

**Feature**: `019-audit-detail`

## Prerequisites

- Monorepo deps installed (`pnpm install` already done on machine)
- Desktop not required for unit contracts; optional for visual smoke

## Validation scenarios

### 1. Unit helpers + composition contract

Prefer the audit-owned file while sibling Access details land in parallel:

```bash
pnpm --filter @polterware/feature-access exec vitest run --config vitest.config.ts src/audit-detail.test.ts
```

Full package suite (may include incomplete sibling detail wires):

```bash
pnpm --filter @polterware/feature-access test
```

Expect:

- `audit-detail-helpers` maps result tones/labels
- composition source includes `layout="sections"`, StatusChip, PageLoading/PageError, Copy event id
- composition source excludes Approve/Delete mutation labels
- Access index imports and mounts `<AuditDetail`
- fixtures expose enriched fields (requestId/correlationId/policy+approval/Denied)

### 2. Detail fixture coverage (desktop)

```bash
pnpm --filter @polterware/desktop exec vitest run --config vitest.ui.config.ts src/renderer/detail-fixture-coverage.test.ts
```

Expect audit fixtures: policy+approval pair, denied result, correlation identifiers, required identity fields.

### 3. Manual smoke (optional; only if user runs desktop)

1. Open Access → Audit.
2. Confirm filters still narrow the virtualized table.
3. Activate `au2` (pending deploy request) → detail shows StatusChip Pending, policy + approval related, correlation ids.
4. Activate `au7` (denied) → StatusChip Denied, policy related, no mutation actions.
5. Activate unknown id via store (if forced) → not-found.
6. Copy event id → mock toast; trail unchanged.

## Expected outcomes

| Check | Pass criteria |
| --- | --- |
| Dedicated composition | `audit-detail.tsx` exists; inline thin block removed |
| Read-only | No Approve/Delete/Edit header actions |
| Related controls | Present when fixture supplies policy/approval |
| Collection intact | Filters + virtualization still in Access index |
| Tests green | access package tests + audit fixture coverage |

## References

- [spec.md](./spec.md)
- [plan.md](./plan.md)
- [data-model.md](./data-model.md)
- [contracts/audit-detail-ui.md](./contracts/audit-detail-ui.md)
