# Research: Resources Nodes Detail

**Feature**: `014-nodes-detail`  
**Date**: 2026-08-12  
**Status**: Complete — no open NEEDS CLARIFICATION

---

## R1 — Detail host: global DetailPanel vs dedicated page

**Decision**: Node entity detail opens **only** in the global bottom `DetailPanel` hosted by MainSurface. Features contribute `DetailPanel.Content`. No node detail route/page, second inspector, modal-as-detail, or drawer-as-detail.

**Rationale**: AGENTS.md entity-detail LAW; DESIGN.md DetailPanel is the single operator detail surface. Spec 003/004 page-language for node detail is superseded by global panel product decisions.

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| Full-page node detail route | Violates DetailPanel LAW; loses collection context |
| Side inspector | Forbidden second panel |
| Modal detail | Wrong for multi-section operational read |

---

## R2 — Page tabs vs detail sections

**Decision**: Resources **page** tabs remain **Inventory · Nodes · Processes** only. Node depth uses stacked **sections** (Identity, Machine, Connection & health, Capabilities, Assigned resources, Recent operations) under `DetailPanel.Body layout="sections"`. No local Tabs rail required (unlike Integration’s seven sections).

**Rationale**: Node depth is narrower than Integration; metrics + metadata + three tables fit one scroll without section rail complexity. AGENTS allows Integration Tabs exception only.

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| Local Tabs (Overview/Capabilities/Ops) | Extra chrome without enough section density |
| Processes nested as node detail tab | Processes already a first-class Resources tab |
| Node settings as page destination | Out of scope; registry inspect first |

---

## R3 — Mock data SSOT location

**Decision**: Node fixtures and types live in `@polterware/feature-resources` (`node-types.ts` → `NODE_FIXTURES`, `TargetNode`). Desktop coverage imports from feature package public exports. Not relocated to shell workspace-mocks for this delivery (nodes are already resource-owned, unlike people/integrations).

**Rationale**: Matches existing Nodes list ownership; avoids shell bloat; detail-fixture-coverage already imports `NODE_FIXTURES` from feature-resources.

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| Move all nodes into feature-shell mocks | Larger migration; no multi-feature consumer need today |
| Live polter-node registry now | Blocks mock production-ready delivery; out of scope |
| Parallel fixture arrays in UI modules | Desync risk |

---

## R4 — Component extraction

**Decision**: Keep dedicated `NodeDetail` composition module (already existed thin); harden to production pattern matching `RunDetail` / `MemberDetail`: loading / error / not-found branches, metrics row, StatusChip everywhere, gated header actions.

**Rationale**: Spec FR-016; testability; parity with sibling operational details.

**Alternatives considered**: Inline only in `ResourcesView` — rejected. Full Zustand entity store for node payload — rejected (panel stores targets only; entity remains fixture lookup).

---

## R5 — Health status vocabulary

**Decision**: Expand UI status union beyond binary online/offline to:

`registering | online | degraded | offline | draining | upgrading | revoked | unknown`

Map via `nodeStatusLabel` / `nodeStatusTone` (see contract). Fixtures cover online (rich), degraded, draining, offline, upgrading, revoked, unknown (sparse).

**Rationale**: `docs/spec.md` §28 health states; operators need non-binary lifecycle for drain/upgrade/revoke honesty.

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| Keep online/offline only | Hides drain/upgrade/revoke; contradicts §28 |
| Full state machine with transition UI | Over-scope for mock inspect |
| Provider-style “healthy” wording only | Confuses node registry with inventory resource health |

---

## R6 — Heartbeat vs process death

**Decision**: Offline/degraded fixtures MAY include `healthNote` stating heartbeat loss does **not** prove remote processes exited. Copy is operator-facing, not a second status enum.

**Rationale**: §28 invariant: heartbeat loss ≠ process death. Prevents false “everything is dead” operator conclusions.

**Alternatives considered**: Automatic process status inference from heartbeat — rejected (false precision).

---

## R7 — Identity: node id vs display name

**Decision**: Model carries stable `id` (row/collection key) and optional `nodeId` (registry identity string). UI shows **Node ID** mono from `resolveNodeId` (nodeId ?? id) separately from **Display name** / hostname. PrimaryCell title remains display name.

**Rationale**: §28 — node ID is different from machine display name.

**Alternatives considered**: Single name field only — rejected for audit/hand-off honesty.

---

## R8 — Capability model

**Decision**: Accept `capabilities` as `string[]` or `{ id?, name, authorized? }[]`. Normalize in `capabilityRows` for table rendering. Authorization column shows Authorized/Denied chip when boolean present, otherwise “—”.

**Rationale**: §28 capability advertisement + authorized execution; string-only fixtures remain valid for sparse nodes.

**Alternatives considered**: Force object-only model — breaks sparse fixtures. Capability matrix editor — out of scope.

---

## R9 — Action gating (Terminal / Run command)

**Decision**:

| Health | Open terminal | Run command |
| --- | --- | --- |
| online, degraded | available | available |
| offline, revoked, draining, upgrading, registering, unknown | hidden | hidden |

Callbacks remain App-owned mocks; UI never claims live ACK.

**Rationale**: §28 revoked refuses new dispatch; offline/draining/upgrading must not fake success paths. Degraded still allows investigate/ops entry with honest health chip.

**Alternatives considered**: Disable with tooltip instead of hide — acceptable later; hide keeps header clean for terminal failures. Allow Run on draining — rejected for mock safety.

---

## R10 — Connection summary

**Decision**: Optional `connection` object: `state` (connected/reconnecting/disconnected/unknown), `lastConnectedAt`, `modelNote` (outbound default copy). Rendered in “Connection & health” when any field present.

**Rationale**: §10.3 / §28 outbound connection model; educates operators without protocol UI.

**Alternatives considered**: Live connection graph visualization — out of scope. Inbound control port UI — forbidden by product defaults.

---

## R11 — Security / secrets

**Decision**: Fixtures and UI never include certificates, private keys, bootstrap tokens, or full credential bodies. Upgrade/revoke described as status only.

**Rationale**: Constitution VI + §55; mock mode still security-safe.

---

## R12 — Live core / polter-node swap path (future)

**Decision**: When node registry queries exist, inject `TargetNode` (or thin mapper) into the same `NodeDetail` props. Do **not** create a second detail component. Fixtures remain for demo/offline/tests.

**Rationale**: Constitution XIV single UX.

**Alternatives considered**: Dual “live node detail” — rejected SSOT.

---

## Resolved clarifications

No NEEDS CLARIFICATION remained in the feature spec. Defaults fixed by product docs (DetailPanel, §28 health, mock-first, no secrets, capability-gated actions).
