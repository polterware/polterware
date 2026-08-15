# Quickstart: Integration Detail Panel

**Feature**: `012-integration-detail`  
**Purpose**: Validate production-ready **mock** Integration detail against [spec.md](./spec.md) and [contracts/integration-detail-ui.md](./contracts/integration-detail-ui.md) without claiming live provider connectivity.

---

## Prerequisites

- Repo checkout with Integrations + shell workspace-mocks  
- Desktop vitest UI config available (`apps/desktop/vitest.ui.config.ts`)  
- Fixture mode (default demo data) — no live OAuth required  

Agents: prefer static/unit checks; do **not** start `pnpm dev` unless the operator asks for visual validation.

---

## Automated checks (CI / agents)

From monorepo root:

```bash
# Fixture richness + secret non-display invariants
cd apps/desktop && pnpm exec vitest run --config vitest.ui.config.ts \
  --dir ../../packages/features/integrations

# Canonical detail fixture coverage (includes integration richness)
cd apps/desktop && pnpm exec vitest run --config vitest.ui.config.ts \
  src/renderer/detail-fixture-coverage.test.ts
```

**Expect**:

| Focus | Pass criteria |
| --- | --- |
| GitHub fixture | scopes, lastSuccessfulSync, console URL, resources, credentials, capabilities, sync runs, webhooks |
| Grafana fixture | `error` status, error copy, stale freshness, failed sync run, expired credential |
| Credential fixtures | no `secret` / `token` / `password` keys |
| Webhook fixtures | `signingSecretConfigured` boolean only; no raw secret |
| Sync runs | at least one non-success (`partial` or `failed`) present in set |
| detail-fixture-coverage | first integration has scopes + related mock collections |

Optional typecheck (package may report missing local `vitest` types — non-blocking if Desktop UI vitest green):

```bash
pnpm exec tsc --noEmit -p packages/features/integrations/tsconfig.json
```

---

## Manual scenario A — Healthy connection Overview (SC-001, SC-002, SC-004)

1. Open project → **Integrations → Connected**.  
2. Select **GitHub**.  
3. Confirm DetailPanel opens (not a new page).  
4. Overview shows Connected StatusChip, account, environments, last successful sync, scopes badges.  
5. Switch local Tabs: Resources · Credentials · Sync · Permissions · Webhooks · Advanced — page toolbar stays **Connected | Catalog**.

**Expect**: All seven sections render without blank panel; provider icon matches table row.

---

## Manual scenario B — Auth failure honesty (SC-004, FR-011)

1. Select **Grafana**.  
2. Overview: Error StatusChip + error reason (token expired).  
3. Last-known resource count/summary still visible (not wiped).  
4. Sync: failed run with danger chip; last successful may differ from last attempt.  
5. Credentials: expired ref status; no secret material.  
6. Header offers **Reconnect** / reconfigure path.

**Expect**: Never reads as full healthy connection.

---

## Manual scenario C — Credentials write-only (SC-003)

1. On any connected integration → **Credentials → Add credential**.  
2. Enter name + secret → Save.  
3. Confirm toast acknowledges mock save and secret not retained.  
4. Re-open dialog or inspect list — secret value never appears.  
5. Cancel path: type secret → Cancel → secret cleared from UI state.

**Expect**: Zero secret re-display.

---

## Manual scenario D — Sync / rate limit (FR-007)

1. Select **Vercel**.  
2. Sync section shows backoff / rate-limit message when fixture present.  
3. Recent runs include **partial** result with warn tone (not healthy).  
4. **Sync now** → mock toast only.

---

## Manual scenario E — Resources empty vs populated

1. **AWS** → Resources: RDS/SQS rows with StatusChip; optional row hand-off if Resources navigation wired.  
2. **Datadog** (disconnected) → Resources empty honest state; Overview disconnected.

---

## Manual scenario F — Disconnect non-destructive (SC-005, FR-012)

1. Advanced → read Disconnect copy.  
2. Disconnect (mock).  
3. Confirm toast/copy does **not** claim deletion of external repos/projects/cloud resources.  
4. Effective status becomes disconnected; external console hand-off still conceptual only.

---

## Manual scenario G — Catalog → Connected target

1. Catalog → open a **connected** provider (e.g. GitHub).  
2. App switches to Connected and selects that integration target.  
3. Detail shows the connection (not a blank catalog detail).

---

## Out of scope for this quickstart

- Live OAuth connect success against real GitHub/AWS  
- Real webhook delivery or secret storage  
- Sync Center / Conflict Resolve full flows (secondary surfaces)  
- Provider admin console clone inside Polter  

---

## Related artifacts

- [spec.md](./spec.md) — requirements & success criteria  
- [data-model.md](./data-model.md) — DTO fields & transitions  
- [contracts/integration-detail-ui.md](./contracts/integration-detail-ui.md) — UI contract  
- [research.md](./research.md) — decisions  
