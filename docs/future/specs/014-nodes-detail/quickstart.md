# Quickstart: Resources Nodes Detail

**Feature**: `014-nodes-detail`  
**Purpose**: Validate production-ready **mock** Node detail against [spec.md](./spec.md) and [contracts/node-detail-ui.md](./contracts/node-detail-ui.md) without claiming live polter-node connectivity.

---

## Prerequisites

- Repo checkout with Resources feature package  
- Package vitest config includes `src/node-detail.test.ts`  
- Desktop vitest UI config available (`apps/desktop/vitest.ui.config.ts`)  
- Fixture mode (default demo data) — no live node protocol required  

Agents: prefer static/unit checks; do **not** start `pnpm dev` unless the operator asks for visual validation.

---

## Automated checks (CI / agents)

From monorepo root:

```bash
# Status helpers + composition contract + fixture richness
pnpm --filter @polterware/feature-resources test

# Canonical detail fixture coverage (includes expanded node shape)
cd apps/desktop && pnpm exec vitest run --config vitest.ui.config.ts \
  src/renderer/detail-fixture-coverage.test.ts
```

**Expect**:

| Focus | Pass criteria |
| --- | --- |
| `node-nitro` | nodeId ≠ name, capabilities, assigned resources, recent ops, cpu/memory, connection.state, dataSource fixture |
| Lifecycle set | offline + at least one of degraded/draining/upgrading/revoked present |
| Status helpers | online→healthy, offline→down, revoked→danger; dispatch denied for offline/revoked |
| Composition | DetailPanel sections, StatusChip, Open terminal / Run command labels, honest empties |
| detail-fixture-coverage | rich node assertions green |

Optional typecheck:

```bash
pnpm exec tsc --noEmit -p packages/features/resources/tsconfig.json
```

---

## Manual scenario A — Rich online inspect (SC-001, SC-002, SC-005)

1. Open project → **Resources → Nodes**.  
2. Select **nitro**.  
3. Confirm DetailPanel opens (not a new page).  
4. Metrics: Online StatusChip, last heartbeat, version, capability count.  
5. Identity: Node ID mono (`nd_nitro_homelab`) distinct from display name; Source = Fixture (mock).  
6. Machine + Connection & health populated; outbound model note visible.  
7. Tables: Capabilities, Assigned resources, Recent operations with StatusChips.  
8. Header: **Open terminal** + **Run command** available.

**Expect**: No blank structural chrome; HardDrive icon matches table row family.

---

## Manual scenario B — Offline honesty (SC-003, FR-013)

1. Select **legacy-node**.  
2. Status Offline (down tone).  
3. Health note mentions heartbeat loss does **not** prove processes exited.  
4. Header does **not** offer Run command / Open terminal as live success controls.  
5. Recent operations may show failed health check.

**Expect**: Never reads as Online healthy; no fake dispatch path.

---

## Manual scenario C — Revoked / draining / upgrading gates (SC-004)

1. Select **retired-edge** (revoked) → danger chip; no dispatch actions.  
2. Select **staging-vps** (draining) → warn chip; drain note; no dispatch.  
3. Select **build-runner-03** (upgrading) → warn chip; upgrade available; no dispatch.

**Expect**: 100% of denied-health fixtures hide terminal/run primary live paths.

---

## Manual scenario D — Degraded still operable (FR-012)

1. Select **prod-vps-01** (degraded).  
2. Status warn; connection reconnecting or delayed ACK copy.  
3. Terminal / Run command still available for investigate (mock).  
4. Upgrade available label present when fixture supplies it.

**Expect**: Degraded ≠ offline; actions allowed with honest health chip.

---

## Manual scenario E — Sparse unknown (SC-002)

1. Select **edge-unknown**.  
2. Metrics show Unknown / Never / — as needed.  
3. Empty capabilities/resources/operations use honest empty copy.  
4. No broken table layout; no invented IP/hostname.

---

## Manual scenario F — Row switch keeps host (FR-001, FR-002)

1. With detail open on nitro, activate **worker-node-01**.  
2. Content swaps; panel host remains; scope stays `resources:nodes`.  
3. Collection under panel remains Nodes list.

---

## Out of scope for this quickstart

- Live polter-node registration / heartbeat stream  
- Real terminal SSH or dispatch ACK against a machine  
- Certificate rotation UI  
- Process kill / remote desktop  
- Inventory resource detail (013) and Processes detail  

---

## Related artifacts

- [spec.md](./spec.md) — requirements & success criteria  
- [data-model.md](./data-model.md) — DTO fields & transitions  
- [contracts/node-detail-ui.md](./contracts/node-detail-ui.md) — UI contract  
- [research.md](./research.md) — decisions  
- [plan.md](./plan.md) — implementation plan  
