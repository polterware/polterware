# Quickstart: Project Settings Environments

## Verify implementation

```bash
# From monorepo root
pnpm --filter @polterware/feature-settings test
```

## Manual smoke (when Desktop is already running)

1. Open a project → Settings (footer).
2. **General**: edit name → Save changes → toast says mock.
3. **Environments**: confirm kind + protection StatusChips; activate row → DetailPanel rules + promote toast.
4. **Advanced**: Archive project → Dialog → Cancel / Confirm (mock only).
5. Confirm tab strip is only General · Environments · Advanced.

## Rubric

- [ ] Environments DataTable + StatusChip (kind and protection)
- [ ] Environment DetailPanel
- [ ] General/Advanced Field/Input/Dialog — no invented inputClass
- [ ] Tests pass
- [ ] No Access/Integrations tabs
- [ ] Not “already implemented” shallow forms
