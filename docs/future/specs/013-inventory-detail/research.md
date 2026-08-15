# Research: Inventory Resource Detail

**Feature**: `013-inventory-detail`  
**Date**: 2026-08-12  
**Status**: Complete — no open NEEDS CLARIFICATION

---

## R1 — Detail host: global DetailPanel vs dedicated page

**Decision**: Inventory resource detail opens **only** in the global bottom `DetailPanel` hosted by MainSurface. Features contribute `DetailPanel.Content`. No inventory detail route/page, second inspector, modal-as-detail, or drawer-as-detail.

**Rationale**: AGENTS.md entity-detail LAW; DESIGN.md DetailPanel is the single operator detail surface. Spec 004 “Resource detail (page)” language is superseded by hierarchy/detail panel product decisions.

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| Full-page detail route | Violates DetailPanel LAW; splits collection context |
| Side inspector | Forbidden second panel |
| Modal detail | Wrong for multi-section operational read |

---

## R2 — Local Tabs rail vs single scroll sections

**Decision**: **No** local Tabs rail for Inventory detail. Use single scroll `layout="sections"` grid (metrics → overview/identity → lifecycle → capabilities → quick actions → linked architecture → activity → annotations), matching Run/Member/Node operational details.

**Rationale**: Inventory depth is lower than Integration’s seven topical domains; AGENTS/DESIGN allow Integration Tabs as a documented exception — Inventory does not need that exception. Single scroll keeps parity with other resource-like details.

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| Overview · Links · Activity · Actions tabs | Extra chrome without Integration-scale section count |
| Nested routes per section | Over-engineered for mock; loses panel context |

---

## R3 — Mock data SSOT location

**Decision**: Inventory fixtures remain in `@polterware/feature-resources` (`fixtures.ts` + `types.ts`), not shell workspace-mocks. Desktop detail-fixture-coverage imports `RESOURCE_FIXTURES` from feature-resources.

**Rationale**: Resources package already owns inventory list DTOs and mapping (`map.ts`); moving to shell would split inventory SSOT without benefit. Integration mocks live in shell because Connected list is shell-composition shared; Inventory is feature-owned.

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| Move all Resource fixtures to shell workspace-mocks | Unnecessary migration; breaks existing map/filter tests ownership |
| Live core API now | Blocks mock production-ready delivery; out of scope |
| Parallel thin and rich models | Dual types violate SSOT |

---

## R4 — Component elevation (not rewrite of host)

**Decision**: Elevate existing `ResourceDetail` module rather than inventing `InventoryDetail` dual composition. Keep activation via existing `ResourcesView` + `scopeId: resources:inventory` / `entityKind: resource`.

**Rationale**: Constitution XIV — one detail composition; list already wires ResourceDetail. Rename would churn App and tests without product value.

**Alternatives considered**: New `InventoryDetail` alongside `ResourceDetail` — permanent dual path, rejected.

---

## R5 — Lifecycle status model

**Decision**: Extend `ResourceStatus` with `stale` and `missing` alongside existing `healthy | degraded | down | unknown | syncing`. Map via `resourceStatusTone` / `resourceStatusLabel` helpers to StatusChip tones:

| Status | Tone | Label |
| --- | --- | --- |
| healthy | healthy | Healthy |
| degraded | degraded | Degraded |
| stale | degraded | Stale |
| down | down | Down |
| missing | down | Missing |
| syncing | syncing | Syncing |
| unknown | unknown | Unknown |

Optional `lifecycleNote` string explains stale/missing/degraded without claiming permanent delete.

**Rationale**: `docs/spec.md` §111: temporary disappearance → stale/missing before delete; StatusChip tones must stay in design-system union (no inventing new chip tones for every lifecycle word).

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| Only show stale as notes while keeping status healthy | Lies about health |
| Jump missing → deleted copy | Violates §111 |
| Add StatusChip tones `stale`/`missing` to design-system | Unnecessary DS surface area for one feature |

---

## R6 — Provenance and annotations separation

**Decision**: Add optional `provenance: discovered | imported | manual`. Present notes/tags as **Annotations** with explicit copy that operator annotations are separate from provider-owned fields. Provider ids (`externalKey`, `providerResourceId`) live under Identity, not annotations.

**Rationale**: §111 user annotations must not be overwritten by provider refresh conceptually; UI must teach that separation even in mock mode.

**Alternatives considered**: Merge notes into Overview only — weaker teaching of annotation boundary.

---

## R7 — Capabilities vs quick actions

**Decision**: Keep both:

- **Capabilities** — tabular availability (available / unavailable + reason) for least-privilege visibility
- **Quick actions** — interactive buttons for enabled external/navigate/operation actions with disabled reasons

Mock-only operations toast honestly; never simulate provider success without toast labeling mock.

**Rationale**: Spec FR-011 and §111 actions depend on capabilities; UI patterns from Integrations (capabilities table + actions) map cleanly without Tabs.

**Alternatives considered**: Collapse to quick actions only — loses structured capability review. Capability table with row-click execute — conflates review and act; keep buttons explicit.

---

## R8 — External hand-off and architecture link

**Decision**:

- Open provider: `externalUrl` or first enabled external quick action `href` via `window.open` (or toast if absent)
- Linked architecture row activate: optional `onOpenArchitecture(nodeId)`; else mock toast (honest unwired hand-off)

**Rationale**: Deep links preferred over console clone; product loop Inventory → Architecture without inventing nested detail.

**Alternatives considered**: Always open Architecture section hard-coded in App inside feature — couples feature to shell routes; prefer callback prop.

---

## R9 — Charts / metrics

**Decision**: Optional `metrics[]` rendered as Metadata items when present. **No** decorative segmented bars for inventory health unless a real categorical distribution exists with counts (omit for this delivery).

**Rationale**: DESIGN detail chart rules — no invented history/capacity charts.

---

## R10 — Live core swap path (future, not this delivery)

**Decision**: When polter-core inventory queries exist, inject data via the same `Resource` shape (or thin mappers) into `ResourceDetail` props/catalog. Do **not** create a second detail component. Fixtures remain for demo/offline and tests. `mapCoreResource` already produces sparse core rows — detail empty states must remain honest.

**Rationale**: Constitution XIV single UX; contracts stay UI-shaped until protocol contracts land.

---

## R11 — Testing strategy

**Decision**:

1. Pure TS unit tests: `resource-status.test.ts` (tones, labels, fixture richness, disabled-reason invariants)
2. Desktop `detail-fixture-coverage.test.ts` asserts primary rich fixture + non-healthy lifecycle presence
3. Icon/Solid component tests stay on Desktop UI vitest when needed (node env excludes Solid icon suite)

**Rationale**: Constitution XII risk-proportional; fixture contract is the main mock-delivery gate.

---

## Open questions

None. All Technical Context items resolved from constitution, DESIGN/AGENTS laws, `docs/spec.md` §21/§111, and patterns from `012-*` detail features.
