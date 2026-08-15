# Quickstart: Spec Kit Document Modules

## Verify modules

```bash
# From monorepo root
pnpm --filter @polterware/feature-spec-kit test
```

## Manual smoke (Desktop)

1. Open a project → **Spec Kit**.
2. **Constitution**: row → detail shows Kind, Status chip, Path `.specify/memory/constitution.md`, no Feature column in table.
3. **Specs**: multi-feature rows; filter one feature; activate row → Feature + mono Path in properties.
4. Maximize detail → Markdown format toolbar appears; restore → toolbar hides.
5. Copy markdown → success toast mock.
6. Empty: filter nonsense → kind-specific “No … match” empty state.
7. Confirm UI never says Speckit CLI ran.

## Rubric

- [x] Dedicated detail module file exists (`spec-artifact-detail.tsx` + helpers)
- [x] logicalPath + featureKey in properties (`buildSpecArtifactProperties`)
- [x] StatusChip for status
- [x] `layout="document"` preserved
- [x] Package tests green (`pnpm --filter @polterware/feature-spec-kit test` → 10/10)
- [x] Not merged with Knowledge
- [x] Not “already implemented” as monolith-only
