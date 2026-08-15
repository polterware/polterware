# Quickstart: Inventory Resource Detail

**Feature**: `013-inventory-detail`  
**Purpose**: Validate production-ready **mock** Inventory resource detail against [spec.md](./spec.md) and [contracts/inventory-detail-ui.md](./contracts/inventory-detail-ui.md) without claiming live provider discovery.

---

## Prerequisites

- Repo checkout with `packages/features/resources` inventory list + detail  
- Vitest available for feature-resources unit tests and Desktop UI config  
- Fixture mode (default demo data) — no live cloud credentials required  

Agents: prefer static/unit checks; do **not** start `pnpm dev` unless the operator asks for visual validation.

---

## Automated checks (CI / agents)

From monorepo root:

```bash
# Status helpers + fixture contract invariants
pnpm --filter @polterware/feature-resources test

# Canonical detail fixture coverage (includes inventory richness)
cd apps/desktop && pnpm exec vitest run --config vitest.ui.config.ts \
  src/renderer/detail-fixture-coverage.test.ts
```

**Expect**:

| Focus | Pass criteria |
| --- | --- |
| `res-pg` (or primary rich fixture) | region, owner, notes, provenance, externalKey, linkedArchitecture, activity, disabled quick action + reason, unavailable capability + reason |
| Lifecycle set | fixtures include `stale`, `missing`, and `degraded` (plus other non-healthy as designed) |
| Disabled invariants | every disabled quick action has `disabledReason`; every unavailable capability has `reason` |
| dataSource | all `RESOURCE_FIXTURES` are `fixture` |
| detail-fixture-coverage | inventory assertions pass with expanded shape |

Optional typecheck (non-blocking if tests green):

```bash
pnpm exec tsc --noEmit -p packages/features/resources/tsconfig.json
```

---

## Manual scenario A — Healthy rich inventory (SC-001, SC-002, SC-006)

1. Open project → **Resources → Inventory**.  
2. Select **prod-postgres** (`res-pg`).  
3. Confirm DetailPanel opens (not a new page).  
4. Metrics: Healthy StatusChip, AWS, Production, last sync.  
5. Overview/Identity: type Database, provenance Discovered, external key / provider resource id mono, Source Fixture (mock).  
6. Capabilities table shows available + unavailable with reasons.  
7. Linked architecture lists Payment/Auth; Activity lists sync/metrics events.  
8. Annotations show notes + tags and “survive provider refresh” helper copy.

**Expect**: Full context without blank panel; header icon matches table row.

---

## Manual scenario B — Degraded lifecycle (SC-003, FR-013)

1. Select **redis-cache**.  
2. Status: Degraded chip (not healthy).  
3. Lifecycle section explains elevated eviction / degraded health.  
4. Last-known links/activity remain; no “deleted” copy.

**Expect**: Operator can tell health is degraded without reading raw IDs.

---

## Manual scenario C — Stale vs missing honesty (SC-003, FR-013)

1. Select **worker-node-01** → Status Stale + heartbeat delayed lifecycle note.  
2. Select **staging-assets-bucket** → Status Missing + “not permanently deleted until confirmed”.  
3. Confirm neither claims permanent delete or invents healthy sync.

**Expect**: Stale/missing distinguishable from healthy and from delete messaging.

---

## Manual scenario D — Capability gating (SC-004)

1. On **prod-postgres**, open Quick actions.  
2. **Run health check** disabled with visible reason (title/aria/helper).  
3. **Open in AWS Console** enabled → external hand-off (new tab) or honest absence toast.  
4. Header **Refresh inventory** → mock toast stating not live discovery.  
5. Header **Edit** → existing resource form dialog path.

**Expect**: Zero fake success for unavailable operations.

---

## Manual scenario E — Sparse / manual / unknown

1. Select **auth-service** (sparse healthy) → optional sections empty honest; metrics may omit.  
2. Select **legacy-billing-batch** → provenance Manual, status Unknown, lifecycle note about no heartbeat.  
3. Select **payment-service** → Syncing chip + refresh-in-progress note.

**Expect**: No broken layout; loading/not-found chrome still available via host props when supplied.

---

## Manual scenario F — Collection switching (FR-002, FR-017)

1. Open detail on **prod-postgres**.  
2. Activate **polter-api** row without leaving Inventory.  
3. Confirm content switches; panel host remains global DetailPanel.  
4. Collapse / maximize / Escape behave per existing DetailPanel laws (no feature reimplementation).

**Expect**: Single inspector, no second panel.

---

## Honesty checklist (always)

- [ ] Source shows Fixture (mock) for fixture rows  
- [ ] Status uses StatusChip only  
- [ ] No Actions column on linked architecture / activity tables  
- [ ] Disabled actions show reasons  
- [ ] No claim of live provider discovery or live core inventory authority  
- [ ] Stale/missing copy never equals “deleted”  

---

## Out of scope for this quickstart

- Live AWS/Vercel/GitHub discovery  
- Nodes / Processes detail redesign  
- Provisioning wizard  
- Architecture navigation wiring (optional; toast is valid until App callback exists)  
