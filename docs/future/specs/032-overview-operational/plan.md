# Implementation Plan: Overview Operational Command Center

**Branch**: `032-overview-operational` | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/032-overview-operational/spec.md`

## Summary

Elevate Project Overview in `@polterware/feature-shell` to a dense §8.1 command center: project/environment context header, StatusChip health (≥2 surfaces), nodes + integrations health, pending approvals → Inbox, recent changes/operations samples with hand-off, high-frequency shortcuts (Architecture, Resources, Operations, Inbox, Terminal), fixture/empty/offline honesty, and expanded pure-helper tests. No live telemetry, no Activity page rewrite, no mandatory Summary/Activity tabs.

## Technical Context

**Language/Version**: TypeScript 5.x · SolidJS 1.9  

**Primary Dependencies**: `@polterware/frontend-design-system` (StatusChip, Button, PageContent, FixtureBanner, DitherIconTile, …), `@polterware/frontend-charts` (existing snapshot proportion charts only)  

**Storage**: N/A (fixture props)  

**Testing**: Vitest (node) for pure helpers + source-contract checks  

**Target Platform**: Desktop Solid renderer (fixture/mock only)  

**Project Type**: Feature package operator UI  

**Performance Goals**: Instant local fixture render  

**Constraints**: StatusChip for labeled health; no fake history charts; shell overview files + fixtures/tests; App.tsx may map `terminal` quick-action if missing  

**Scale/Scope**: overview-page, overview-cards, overview-model, types/fixtures as needed, tests; optional App.tsx one-line terminal mapping  

## Constitution Check

- **I. Operational product**: Command-center snapshot for architecture/execution awareness — pass.
- **II. Domain before framework**: Aggregates/tone maps in `overview-model` — pass.
- **III. Runtime separation**: Fixture UI; no live telemetry claim — pass.
- **IV. Monorepo deps**: shell → design-system/charts — pass.
- **V. Contracts-first**: Local UI contract note — pass.
- **VI. Security**: Read-only overview chrome — pass.
- **VII. Auditable mutations**: Navigation hand-offs only — pass.
- **VIII. Local-first**: Fixture/empty/offline honesty — pass.
- **IX–XI**: Small lists; no Main work — pass.
- **XII. Tests**: Expand overview-model tests + cards contract — pass.
- **XIII–XIV**: Extend existing overview; single mock model — pass.

## Project Structure

### Documentation (this feature)

```text
specs/032-overview-operational/
├── spec.md
├── plan.md
├── tasks.md
├── checklists/requirements.md
└── contracts/overview-ui.md
```

### Source (owned)

```text
packages/features/shell/src/
├── overview-model.ts              # expand helpers
├── overview-model.test.ts         # expand coverage
├── overview-page.tsx              # context, shortcuts, dataSource, wire cards
├── overview-cards.tsx             # StatusChip health, integrations, shortcuts chrome
├── workspace-mocks/types.ts       # optional OverviewSummary fields
├── workspace-mocks/data.ts        # buildOverviewSummary integration counts
└── (tests) overview cards contract asserts in overview-model.test or dedicated test

apps/desktop/src/renderer/App.tsx  # only if terminal quick-action mapping missing
```

### Out of bounds

activity-page rewrite · knowledge · spec-kit · audit-detail · P0/P1 unrelated features · live metrics backends

## Implementation Approach

1. **Model first** — helpers:
   - `mapMockToneToStatusChip` / `getSystemHealthPresentation`
   - `calculateIntegrationHealth`
   - `overviewDataSourceLabel` + honesty message
   - `OVERVIEW_SHORTCUTS` contract list
   - keep existing filters/segments
2. **Summary fixtures** — optional integration counts + dataSource on OverviewSummary or page props; `buildOverviewSummary` remains consistent with COMPONENT/NODE/INTEGRATION mocks.
3. **Cards** — SystemHealthCard uses StatusChip for primary health; health events use StatusChip; nodes/integrations health chips; operations list StatusChip; NeedsAttention count + View inbox; OverviewShortcuts strip; no new history charts.
4. **Page** — context header (project, env, health chip, dataSource cue), shortcuts, existing grids, empty/offline/fixture modes.
5. **App** — map `terminal` → Terminal section if not present.
6. **Tests** — model unit tests + source contract for StatusChip / shortcuts / View inbox / dataSource.

## Rejection Rubric (definition of done)

- [ ] StatusChip on ≥2 health surfaces of Overview
- [ ] Pending approvals clickable → Inbox
- [ ] Empty/offline/fixture state honest
- [ ] Tests of overview-model/cards contract pass
- [ ] MUST NOT rewrite Activity page
- [ ] FORBIDDEN: claim “already implemented” because overview-cards is large

## Complexity Tracking

No constitution violations. Existing waffle/segmented bars stay as snapshot proportions (FR-011 allows). Integrations health is a new first-class §8.1 surface derived from INTEGRATION_MOCKS.
