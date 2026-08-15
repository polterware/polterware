# Quickstart: Architecture Node Detail Polish

## Verify as operator (Desktop)

1. Open project → Architecture → Components (or Canvas).
2. Select a component row / node → global DetailPanel opens.
3. Confirm header domain icon + Open on canvas (and Link resource if unlinked).
4. Confirm Overview StatusChips for lifecycle and link.
5. Confirm Connected Evidence column uses chips (Proven/Unproven).
6. Switch to Views → open a view detail → still works (separate composition).

## Verify as developer

```bash
pnpm --filter @polterware/feature-architecture test
```

Expect `node-detail.test.ts` (helpers + composition contract) green.

## Files

- `packages/features/architecture/src/node-detail.tsx`
- `packages/features/architecture/src/node-detail-helpers.ts`
- `packages/features/architecture/src/node-detail.test.ts`
- `packages/features/architecture/src/fixtures.ts` (enriched fixtures)
