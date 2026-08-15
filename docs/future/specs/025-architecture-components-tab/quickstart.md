# Quickstart: Architecture Components Tab

## Manual check

1. Open Desktop → project → Architecture.
2. Confirm toolbar tabs: Canvas · Components · Views.
3. Click Components — list of architecture nodes appears (same names as canvas).
4. Activate a row — global DetailPanel shows node detail (`arch-node`).
5. Click Canvas — same nodes on canvas; open one — same detail kind.
6. Deep link / set tab to `components` — stays on Components (no redirect).

## Tests

```bash
pnpm --filter @polterware/feature-architecture test
pnpm --filter @polterware/feature-shell test
```

## Rubric

See spec rejection rubric in feature request: tab visible, no redirect, DataTable+StatusChip, detail arch-node, SSOT, normalize test green.
