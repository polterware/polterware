# Quickstart: Knowledge Document Detail Modules

## Verify manually (Desktop)

1. Open a project → **Knowledge**.
2. **Docs**: select a document → bottom DetailPanel shows properties + Markdown body.
3. Collapse / open half: format toolbar absent. Maximize: format toolbar present.
4. Confirm StatusChip on status; owner, environment, components visible on rich fixtures.
5. **Decisions**: open ADR → status chip; alternatives/consequences reflected; Accept on proposed.
6. **Runbooks**: open critical runbook → severity chip; steps/commands summary props; Run remains mock toast.
7. Force not-found (e.g. activate unknown id in store if testing hooks) → honest not-found.

## Automated

```bash
pnpm --filter @polterware/feature-knowledge test
```

## Rubric

See [spec.md](./spec.md) Acceptance Rubric — all boxes green before done.
