# Quickstart: Architecture View Detail

**Feature**: `023-architecture-view-detail`

## Prerequisites

- Monorepo install (`pnpm install` already done in workspace)
- Architecture package tests: `packages/features/architecture`
- Desktop UI coverage test for fixtures: `apps/desktop` vitest ui config

## Validation scenarios

### 1. Unit / composition (architecture package)

```bash
pnpm --filter @polterware/feature-architecture test
```

Expect:
- Helper tests for membership availability
- Composition source contract: sections layout, Open/Edit actions, Membership section, PageLoading/PageError
- Wiring check: Architecture index mounts `ArchitectureViewDetail` for `arch-view`
- No invented member synthesis from nodeCount

### 2. Fixture coverage (desktop)

```bash
pnpm --filter @polterware/desktop exec vitest run --config vitest.ui.config.ts src/renderer/detail-fixture-coverage.test.ts
```

Expect:
- At least two views with fields beyond `name` / `description` / `nodeCount` / `updated`
- At least one view with explicit `members` array
- Sparse path still has honest nodeCount

### 3. Manual operator path (when app is running)

1. Open Architecture → **Views**
2. Activate **Payment flow** or **System view**
3. Confirm DetailPanel: Overview (owner/environments/tags), Membership table, Open + Edit
4. **Open** → Canvas tab with that view active in View filter
5. **Edit** → ViewEditor deep flow with view name
6. Activate a sparse view (if present) → Membership unavailable copy, not invented names

## Out of scope checks

- Do not require Components tab
- Do not require live core view persistence
- Do not require browser E2E in agent default (Agents.md: no `pnpm dev` unless asked)

## Contract / model

- UI contract: [contracts/architecture-view-detail-ui.md](./contracts/architecture-view-detail-ui.md)
- Data model: [data-model.md](./data-model.md)
