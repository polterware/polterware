# Quickstart: Operational Component Placement

**Feature**: `011-operational-component-place`  
**Purpose**: Manual + unit validation that the authoring flow meets [spec.md](./spec.md) success criteria without claiming live cloud wiring.

---

## Prerequisites

- Repo checkout with Architecture feature packages  
- Desktop app runnable when visual check is requested by the operator (agents: prefer static/unit checks per Agents.md)  
- Fixture/mock mode available (default local demo data)

---

## Automated checks (agents / CI-friendly)

From monorepo root (adjust vitest invocation to repo conventions):

```bash
# Domain pure tests (expected after implementation)
pnpm --filter @polterware/feature-architecture exec vitest run \
  src/domain/place-config.test.ts \
  src/domain/place-link.test.ts \
  src/domain/place-hints.test.ts

# Card honesty
pnpm --filter @polterware/feature-architecture exec vitest run \
  src/canvas/architecture-node-card.test.tsx
```

**Expect**:

| Test focus | Pass criteria |
| --- | --- |
| normalizePlaceConfig | Rejects empty name / unknown env / empty env list |
| normalize with project envs | Accepts project env ids |
| link validate | Blocks cross-env; accepts match |
| nodeStatusLabel | `planned` → Planned; `linked` → Linked (not Healthy) |
| place payload helper | unlinked → planned+unlinked; with resourceId → linked |

---

## Manual scenario A — Model-first (SC-001, SC-002, SC-003)

1. Open **Architecture → Canvas** on a project with environments.  
2. Press **N** (or **+**).  
3. Search **postgres** / select **PostgreSQL**.  
4. Configure:
   - Name: `payments-db`  
   - Environment: **Staging** (from project list)  
   - Purpose optional  
5. **Place on canvas** (or click placement if required).  

**Expect**:

- Card name `payments-db`  
- Status/lifecycle **Planned** (not Healthy)  
- Detail: environment Staging, unlinked / planned copy  
- No resource association  
- Timer happy path &lt; 20s for familiar operator  

---

## Manual scenario B — Place + link (SC-004)

1. Repeat place for a type that has a fixture resource (e.g. Database / Production postgres).  
2. Either:
   - In configure, open **Link existing resource**, pick `prod-postgres` (or fixture twin), or  
   - Place unlinked → Detail → **Link resource** → pick same.  

**Expect**:

- Component **Linked**  
- Detail shows resource name/ref  
- Still **no** fake “Observed” unless evidence exists  
- Completes &lt; 45s in demo  

---

## Manual scenario C — Cancel safety (SC-006)

1. Open catalog → Escape → no node.  
2. Open catalog → pick type → configure → Cancel → no node.  
3. If placement mode: configure → Escape on canvas → no node.  

**Expect**: document node count unchanged.

---

## Manual scenario D — Connected stubs (SC-007)

1. Catalog → **Connected** / Linked tab.  
2. Trigger unavailable import action.  

**Expect**: honest unavailable message; zero new nodes.

3. If **Link from Resources** implemented: pick resource → confirm identity → linked/planned component as designed.

---

## Manual scenario E — Environment binding

1. Place component in **Development**.  
2. Set canvas environment filter to **Development**.  

**Expect**: component not dimmed (or visible per filter rules); Production-only nodes dimmed/hidden consistently.

---

## Honesty regression checklist

- [ ] Linked card does **not** say Healthy solely due to link  
- [ ] Unlinked place does **not** say live/synced  
- [ ] Configure primary button is place/commit, not Deploy/Provision  
- [ ] Environment options match project fixtures, not a random orphan list  
- [ ] Fixture inventory not labeled as live cloud when shell is in fixture mode  

---

## Mapping to contracts

| Scenario | Contract |
| --- | --- |
| A–C | [authoring-flow.md](./contracts/authoring-flow.md) |
| Env validate | [place-config.md](./contracts/place-config.md) |
| B, D | [resource-link-port.md](./contracts/resource-link-port.md) |
| Fields/states | [data-model.md](./data-model.md) |

---

## Out of scope for this quickstart

- Real AWS/Vercel import  
- Target/node install token flow  
- Provisioning plan execution  
- Full desktop e2e harness (add only if already present and operator requests)
