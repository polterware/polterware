# Quickstart: Process Detail

## Verify locally

```bash
# Feature unit tests (process helpers + composition + fixtures)
pnpm --filter @polterware/feature-resources test

# Desktop fixture coverage (includes process full + sparse)
pnpm --filter @polterware/desktop exec vitest run --config vitest.ui.config.ts src/renderer/detail-fixture-coverage.test.ts
```

## Manual operator path

1. Open Desktop → project → Resources → Processes.
2. Activate `polter-core` (full) — expect StatusChip Running, metrics, runtime, host, related when present.
3. Activate a Stopped fixture — expect Stop/Restart hidden; logs/terminal still available as mock.
4. Activate a sparse fixture — expect em dashes; no fake CPU bar.
5. Force unknown target id in detail store — expect not-found PageError.

## Out of scope

Live agent process APIs, real kill/restart, log streaming product, Architecture Components/View, Approvals, Invitations.
