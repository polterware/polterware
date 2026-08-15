# Quickstart: Activity Actor Inspection

## Dev

1. Open Desktop on a project workspace.
2. Navigate **GENERAL → Activity**.
3. Confirm table columns: Event, Actor, Actor kind, Environment, When.
4. Filter **Actor → People** / **Agents** / **System** and confirm distinct fixtures remain.
5. Activate a row → global DetailPanel shows overview + target + description.
6. Over-constrain filters → honest empty state.

## Tests

```bash
pnpm --filter @polterware/feature-shell exec vitest run --config vitest.config.ts src/activity-event-helpers.test.ts src/activity-event-detail.test.ts
```

## Boundaries

- Do not open Access → Audit expecting Activity events.
- Do not edit audit-detail or Overview rewrite in this feature.
