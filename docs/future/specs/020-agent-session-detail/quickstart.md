# Quickstart: Access Agent Session Detail

**Feature**: `020-agent-session-detail`

## Prerequisites

- Monorepo install (`pnpm install` at repo root) already done
- Desktop fixtures load from `@polterware/feature-shell` workspace mocks
- Access Agents tab registers Recent sessions detail targets (`entityKind: agent-session`)

## What to verify (manual / fixture UI)

1. Open Desktop → project workspace → **Access → Agents**.
2. In **Recent sessions**, activate **Deployment Agent / Session #184** (Running).
3. Confirm DetailPanel:
   - Title matches session
   - Status chip Running
   - Objective, environment, started
   - Agent identity resolves to **Deployment Agent**
   - Step progress bar shows Done/Running/Waiting mix
   - Steps table has status chips (and tool/timing on enriched fixtures)
   - Terminate + Open agent header actions present
4. Activate a **Waiting** session → waiting reason / approval metadata visible; Terminate still available.
5. Activate a **Completed** session → Terminate hidden; Open agent still available.
6. Activate a **Failed** session → Failed status chip + failed step honesty.
7. Click **Terminate** on Running/Waiting → mock toast only (no crash).
8. Click **Open agent** → host navigates/activates agent identity path (parallel 017 may own that panel).

## Automated checks (agent / developer)

From repo root (or package cwd):

```bash
# Access package unit tests (helpers + composition contract)
pnpm --filter @polterware/feature-access test

# Desktop fixture completeness gate (if session asserts expanded)
pnpm --filter @polterware/desktop exec vitest run --config vitest.ui.config.ts src/renderer/detail-fixture-coverage.test.ts
```

Do **not** run `pnpm dev` / full product build unless Erick asks.

## Expected outcomes

- All access tests green for agent-session modules
- `detail-fixture-coverage` asserts multi-status sessions + rich primary steps
- No edits under `agent-detail.tsx`, `policy-detail*`, `audit-detail*` for this feature

## Contract / model references

- [agent-session-detail-ui.md](./contracts/agent-session-detail-ui.md)
- [data-model.md](./data-model.md)
- [spec.md](./spec.md)
