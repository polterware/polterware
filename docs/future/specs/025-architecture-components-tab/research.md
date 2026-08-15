# Research: Architecture Components Tab

**Date**: 2026-08-12 | **Feature**: 025-architecture-components-tab

## Decisions

### D1 — Defect location

**Decision**: Primary fix is `@polterware/feature-architecture` (`ARCH_TABS` missing Components; `tab()` maps `components` → `canvas`). Shell already has Components in `SECTION_TABS` and `ARCH_TABS` set; `setTab(..., "components")` tests pass.

**Rationale**: Avoid unnecessary shell churn; peer routes already correct.

**Alternatives**: Rewrite shell only — insufficient because feature still redirects.

### D2 — Detail scope

**Decision**: Components collection uses `scopeId: "architecture:components"` with `entityKind: "arch-node"`. Detail composition reuses `ArchitectureNodeDetail`. Canvas keeps `architecture:canvas`.

**Rationale**: Matches collection-per-tab law; same entity kind allows shared detail projection.

**Alternatives**: Force all activations onto `architecture:canvas` scope — blurs which collection owns targets when tab switches.

### D3 — SSOT

**Decision**: Rows project from live `doc()` signal (same document Canvas edits). No second fixture array of components.

**Rationale**: docs/spec.md §8.2 / §20.

### D4 — Status vocabulary

**Decision**: Primary StatusChip column = **lifecycle** from `NodeState` via `nodeStatusLabel` + honest tones (observed ≠ invent health from planned alone). Secondary column or cell for **link status**.

**Rationale**: Existing node-lifecycle domain; avoids fake health for planned nodes.

### D5 — Out of scope peers

**Decision**: Do not implement view detail, process, approval, invitation; do not rewrite place/configure.

## Risks

| Risk | Mitigation |
| --- | --- |
| Tab switch loses selection | Session snapshot already used for canvas; components activation independent |
| Fixture environmentIds display labels | Reuse `environmentLabel` + DEFAULT_PROJECT_ENVIRONMENTS |
| Icon variance | Reuse canvas `resolveNodeIconName` + NodeIcon |
