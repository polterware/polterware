# Research: Operational Component Placement

**Feature**: `011-operational-component-place`  
**Date**: 2026-08-11  
**Status**: Complete — no open NEEDS CLARIFICATION

---

## R1 — Environment source of truth

**Decision**: Identity step binds Environment to a **project environment list** supplied by a port (`ProjectEnvironmentPort`), with fixture default list when shell/project context is mock.

**Rationale**: Spec FR-007 and `docs/spec.md` §7 require environments owned by the project. Hardcoded `PLACE_ENVIRONMENTS` alone feels “fake” and diverges from Settings → Environments.

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| Keep only hardcoded four envs | Spec violation; operator distrust |
| Free-text environment | Breaks filter/consistency |
| Require live core API now | Blocks mock-first delivery |

**Implementation note**: Document stores stable **ids** (prefer lowercase: `production`, `staging`, …). Display labels come from the port. Fallback fixture mirrors common kinds if port empty — but empty list blocks place with clear message (edge case).

---

## R2 — When to offer resource link

**Decision**: Support **both** optional link during configure **and** post-place Detail CTA “Link resource”. Configure section collapsed by default (“Link existing resource”).

**Rationale**: FR-019 requires at least one path; dual entry matches Model-first (skip) and Observe-then-link without forcing a wizard page.

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| Link-only at place (mandatory) | Breaks Model-first / offline |
| Link-only after place | Buried; stickers persist in perception |
| Link only from Resources feature | Canvas loop incomplete |

---

## R3 — Card status honesty (`linked` ≠ Healthy)

**Decision**: Change presentation mapping so:

| NodeState / link | Card label (v1) |
| --- | --- |
| `planned` + unlinked | Planned |
| `linked` | Linked |
| `observed` | Observed (or product status from evidence later) |
| `stale` / `drifted` | Stale / Drifted |
| `unreachable` | Unreachable |
| `unknown` | Unknown |

**Rationale**: Current `nodeStatusLabel` maps `linked` → “Healthy”, inventing health without evidence (Constitution I, FR-014/021, 006 FR-022).

**Alternatives considered**: Keep Healthy for linked — **rejected** (fake control plane).

---

## R4 — Document fields for link

**Decision**: On link: set `resourceId`, `linkStatus: "linked"`, `state: "linked"`. On unlinked place: `resourceId: null`, `linkStatus: "unlinked"`, `state: "planned"`. Do **not** set observed/healthy from place form.

**Rationale**: Aligns with existing `ArchNode` shape in `frontend-canvas` document types.

**Alternatives considered**: Separate join table only — deferred; document already carries `resourceId` for local mock.

---

## R5 — Resource inventory port (mock)

**Decision**: `ResourceLinkPort.listCandidates({ projectId?, environmentId, componentType, subtype? })` implemented by fixture adapter using `@polterware/feature-resources` fixtures and/or shell `RESOURCE_MOCKS`, mapped to a thin `LinkableResource` DTO (id, name, type, environmentId, provider?, dataSource).

**Rationale**: Avoid architecture → deep resources UI dependency; pure list for picker. Fixture honesty via `dataSource`.

**Alternatives considered**: Import full Resources feature UI — too heavy; couple surfaces.

**Compatibility filter (v1)**:

1. Same environment id/label (normalized), or resource env unknown → allow with note  
2. Type family: map catalog `nodeType`/category to resource `type` loosely (database↔Database, api/service↔Service, queue↔Queue, …); if no match, show all in env with “may not match type” soft filter toggle default on

---

## R6 — Placement after identity

**Decision**:

- Origin known → place immediately after configure confirm (name from configure is SSOT; no mandatory inline rename).
- Origin null → placement mode with named preview after configure; Escape discards.

**Rationale**: Spec US6 + 006 FR-007/008 evolved: identity before spatial confirm when needed; avoid double-name.

**Alternatives considered**: Always inline rename after place — rejected (double gate). Always center-drop without placement mode — acceptable interim if placement mode incomplete, but plan targets FR-015.

---

## R7 — Connected catalog actions

**Decision**:

| Action | Behavior v1 |
| --- | --- |
| Link from Resources | Open resource-first picker → suggest type/name/env → confirm → place linked |
| Import from AWS/… | Honest “Not available yet” toast; no node |

**Rationale**: FR-003/005; stubs without scenic half-flows.

---

## R8 — CreateNodeDialog / Components tab

**Decision**: Components tab `+` opens the **same** catalog → configure → place pipeline (switch to Canvas tab on place if needed). Remove freeform draft that skips identity semantics.

**Rationale**: FR-033 SSOT.

---

## Existing code leverage

| Asset | Use |
| --- | --- |
| `place-config.ts` / configure dialog | Extend, already has purpose/role/owner skeleton |
| `placeConfigured` in host | Wire resourceId/state/linkStatus |
| `RESOURCE_FIXTURES` / `RESOURCE_MOCKS` | Link candidates |
| DetailPanel + `detailNode` | CTA + mapping |
| `docAddNode` | Commit path |
| Catalog flush dialog | Discover unchanged structurally |

---

## Open items deferred (not blockers)

- Live polter-core environment RPC  
- Live inventory discovery  
- Observed evidence pipeline  
- Multi-env membership on one component  
- Cross-env link policy beyond warn/prevent
