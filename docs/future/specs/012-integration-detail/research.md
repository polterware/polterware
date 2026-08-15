# Research: Integration Detail Panel

**Feature**: `012-integration-detail`  
**Date**: 2026-08-11  
**Status**: Complete — no open NEEDS CLARIFICATION

---

## R1 — Detail host: global DetailPanel vs dedicated page

**Decision**: Integration entity detail opens **only** in the global bottom `DetailPanel` hosted by MainSurface. Features contribute `DetailPanel.Content`. No integration detail route/page, second inspector, modal-as-detail, or drawer-as-detail.

**Rationale**: AGENTS.md entity-detail LAW; DESIGN.md DetailPanel is the single operator detail surface. Spec 004 originally mentioned “detail page” language — superseded by hierarchy/detail panel product decisions.

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| Full-page detail route | Violates DetailPanel LAW; splits collection context |
| Side inspector | Forbidden second panel |
| Modal detail | Wrong for multi-section operational read |

---

## R2 — Page tabs vs local detail sections

**Decision**: Integrations **page** tabs remain **Connected · Catalog** only. **Overview · Resources · Credentials · Sync · Permissions · Webhooks · Advanced** are **local detail sections** via design-system `Tabs` rail under the detail header (before `DetailPanel.Grid`).

**Rationale**: AGENTS hierarchy nav LAW; DESIGN explicitly allows Integration detail Tabs rail under header. Older 004 contract listed Credentials/Sync as page tabs — product IA moved those into detail.

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| Credentials/Sync as page tabs | Clutters destination; breaks “detail owns entity depth” |
| All sections stacked without tabs | Scroll death; hard scan for 7 areas |
| Nested routes per section | Over-engineered for mock; loses panel context |

---

## R3 — Mock data SSOT location

**Decision**: All integration detail fixtures live in `@polterware/feature-shell` workspace-mocks (`types.ts` + `data.ts`), exported through shell package public API. Feature package consumes them; does not hardcode parallel secret lists in UI modules (except pure UI-only constants if any remain temporary).

**Rationale**: Matches Resources/Knowledge/Access fixture pattern; Desktop composition already imports shell mocks; detail-fixture-coverage can assert richness in one place.

**Alternatives considered**:

| Option | Rejected because |
| --- | --- |
| Fixtures only inside feature-integrations | Breaks App-level fixture coverage patterns; harder shell reuse |
| Live core API now | Blocks mock production-ready delivery; out of scope |
| Duplicate local WEBHOOKS arrays in UI | Desync risk (already observed as gap) |

---

## R4 — Component extraction

**Decision**: Extract `IntegrationDetail` as a dedicated composition module (like `ResourceDetail` / `ArchitectureNodeDetail`), called from `IntegrationsView` when Connected target is active. Props: integration DTO, effective status, mock action callbacks, loading/error/notFound.

**Rationale**: Index was monolithic (~700 lines) with inline detail; extraction improves testability and matches product detail pattern.

**Alternatives considered**: Keep inline only — rejected for maintainability. Full controller store for section state — unnecessary; local `createSignal` for section id is enough.

---

## R5 — Credential secret hygiene

**Decision**: Credential fixtures expose only **references** (name, scope, status, rotated). Create dialog uses password `Input`, write-only; clear secret on submit **and** cancel. Never store secret on fixture objects. Webhook signing shows Configured | Missing only.

**Rationale**: `docs/spec.md` §112 + Constitution VI; security is non-negotiable even in mock mode.

**Alternatives considered**: Masked partial secret display — rejected (still secret material). Persist mock secret in memory for “show again” — rejected.

---

## R6 — Health / status model for UI

**Decision**: Keep integration status union `connected | error | disconnected | disabled`. Map to StatusChip tones: connected→healthy, error→danger, disabled→warn, disconnected→muted. Overlay local mock **disabled** and **disconnected** sets without erasing fixture error metadata. Surface **freshness** (`fresh | stale | unknown`) and **rateLimitState** as additional chips/copy, not by greening failed sync.

**Rationale**: Matches existing Connected table; §112 requires auth failure degraded without erasing last-known; partial/failed sync must not read as full health.

**Alternatives considered**: Introduce separate `degraded` status string now — deferred; `error` + error message + stale covers operator needs for mock.

---

## R7 — Disconnect / revoke semantics (UI)

**Decision**: Disconnect is a mock local state transition to disconnected + toast. Copy MUST state external resources are **not** destroyed. Disable is reversible enable without claiming provider-side revoke APIs.

**Rationale**: §112 item 8; operators fear “Disconnect = delete my AWS account resources”.

**Alternatives considered**: Immediate row removal from Connected list — rejected for mock clarity (keep row, show disconnected). Call fake provider delete — forbidden.

---

## R8 — Resource hand-off from integration

**Decision**: Integration resource rows may carry optional `inventoryResourceId` for Resources hand-off. Activate row calls `onOpenResource` when provided; otherwise no fake navigation.

**Rationale**: Integrations feed inventory; deep link closes the product loop without inventing inventory rows.

**Alternatives considered**: Always open external provider console for resources — wrong for Polter inventory. Nested detail-in-detail — forbidden.

---

## R9 — Charts / coverage bars

**Decision**: Use `DitherSegmentedBar` only for real categorical distributions (permission allowed/denied; webhook active/failed/paused) with visible count text; omit when total is zero.

**Rationale**: DESIGN detail chart rules — no decorative empty bars.

---

## R10 — Live core swap path (future, not this delivery)

**Decision**: When polter-core integrations queries exist, inject data via the same DTO shapes (or thin mappers) into `IntegrationDetail` props/store selectors. Do **not** create a second detail component. Mock fixtures remain for demo/offline and tests.

**Rationale**: Constitution XIV single UX; contracts stay UI-shaped until protocol contracts land.

**Alternatives considered**: Dual “live detail” component — rejected dual SSOT.

---

## Resolved clarifications

No NEEDS CLARIFICATION remained in the feature spec. Defaults fixed by product docs (DetailPanel, local sections, mock-first, secret hygiene).
