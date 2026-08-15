# Research: Architecture View Detail

**Feature**: `023-architecture-view-detail`  
**Date**: 2026-08-12

## Decision 1: Dedicated composition, not inline index block

**Decision**: Extract `ArchitectureViewDetail` into `view-detail.tsx` (+ helpers + tests). Architecture index only resolves the active arch-view target and mounts the composition.

**Rationale**: Spec FR-010 and rejection rubric forbid remaining as a thin inline Summary + nodeCount metric. Matches Team/Member/Run detail extraction pattern used across operator surfaces.

**Alternatives considered**:
- Keep expanding inline in `index.tsx` — fails SSOT and rubric.
- Separate route/page for views — violates DetailPanel LAW.

## Decision 2: Open reuses canvas view activation

**Decision**: Open sets existing `activeViewId` and switches Architecture tab to `canvas` via `onTabChange("canvas")`. No second canvas host, no fork of ArchitectureCanvas.

**Rationale**: Canvas already has saved-view FilterPopover + `activeViewId` state. Spec FR-011 and product docs require reusing the existing activation path.

**Alternatives considered**:
- Mount a second canvas inside DetailPanel — forbidden.
- Persist view open to core layout API — out of mock stage scope.

## Decision 3: Edit reuses ViewEditor deep flow

**Decision**: Edit sets `editViewName` and `deepMode("view-edit")`, mounting existing `ViewEditor` from `deep-flows.tsx`.

**Rationale**: View editor mock already exists; inventing a second editor would dual-implement.

## Decision 4: Membership honesty

**Decision**:
- When `members` array is present (including empty array), render Membership as DataTable or honest empty (“No members in this view.”).
- When `members` is **undefined** and only `nodeCount` exists, show unavailable copy: “Membership list not available in mock.” — never invent names from count.

**Rationale**: Spec US3/US4 and rejection rubric. Operators must trust that listed names are real fixture data.

**Alternatives considered**:
- Derive placeholder “Node 1…N” from nodeCount — **forbidden**.
- Hide Membership section when unavailable — fails rubric requiring Membership section existence.

## Decision 5: Fixture enrichment targets

**Decision**: Enrich at least **System view** (`v-system`) and **Payment flow** (`v-pay`) with owner, environments, tags/purpose, explicit members aligned to canvas fixture node ids/labels, optional notes/filters. Leave at least one secondary view sparse (nodeCount only) to exercise unavailable membership.

**Rationale**: SC-002 + desktop fixture coverage gate; prove both rich and sparse paths.

## Decision 6: Not-found wiring

**Decision**: Mount `ArchitectureViewDetail` when active target is arch-view on views scope, even if fixture missing — pass `notFound` / undefined view so PageError path renders.

**Rationale**: Current `Show when={detailView()}` silently hides the panel for unknown ids; production details use explicit not-found.

## Open questions resolved

None remaining — all NEEDS CLARIFICATION avoided via product defaults above.
