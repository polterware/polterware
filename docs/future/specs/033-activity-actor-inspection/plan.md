# Implementation Plan: Activity Actor Inspection

**Branch**: `033-activity-actor-inspection` | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/033-activity-actor-inspection/spec.md`

## Summary

Deepen the Desktop **Activity** operational feed so operators can distinguish person vs agent vs system authorship (§4.11 / §8.8), filter the feed professionally, and inspect events in the global DetailPanel via a dedicated `ActivityEventDetail` composition. Keep Access Audit strictly separate.

## Technical Context

**Language/Version**: TypeScript (strict) + SolidJS  
**Primary Dependencies**: `@polterware/frontend-design-system`, `@polterware/frontend-state`, SolidJS  
**Storage**: N/A (fixture mocks only)  
**Testing**: Vitest (node) — pure helpers + source composition contracts  
**Target Platform**: Electron Desktop renderer (`apps/desktop`)  
**Project Type**: Monorepo feature package `@polterware/feature-shell`  
**Performance Goals**: Modest fixture list; DetailPanel body sole vertical scroller  
**Constraints**: DESIGN.md + DetailPanel LAW; no Actions column; no Audit fusion; mock honesty  
**Scale/Scope**: One page + detail module + helpers + fixtures + tests within shell

## Constitution Check

- **I Operational product**: Activity feed observability for human+agent co-management — pass
- **II Domain before framework**: Pure filter helpers independent of Solid — pass
- **III Runtime separation**: Renderer/shell only; fixtures not claimed as live store — pass
- **IV Directed dependencies**: shell → design-system + state; no access package import for detail — pass
- **V Contracts-first**: UI contract `activity-event-ui.md` documents composition surface — pass
- **VI Security**: no secrets; unprivileged renderer — pass
- **VII Safe execution**: read-only inspection; optional open-target navigation only — pass
- **VIII Local-first**: mock fixtures offline — pass
- **IX Performance**: no heavy work; small list — pass
- **X Resilience**: not-found / empty honesty — pass
- **XI Observability**: actor kind explicit in feed — pass
- **XII Tests**: filter helpers + composition — pass
- **XIII Real structure**: extend shell only — pass
- **XIV SSOT**: one Activity page filter path + one ActivityEventDetail — pass
- **XV Big-bang context**: continues operator surfaces on `refactor/new` — pass

## Project Structure

### Documentation (this feature)

```text
specs/033-activity-actor-inspection/
├── spec.md
├── plan.md
├── tasks.md
├── data-model.md
├── research.md
├── quickstart.md
├── contracts/
│   └── activity-event-ui.md
└── checklists/
    └── requirements.md
```

### Source (owned paths)

```text
packages/features/shell/src/
├── activity-page.tsx                 # DataTable + filters + collection registration
├── activity-event-icon.tsx           # type → icon (existing)
├── activity-event-helpers.ts         # pure filter + label/tone helpers (new)
├── activity-event-detail.tsx         # DetailPanel composition (new)
├── activity-event-detail-helpers.ts  # detail-only pure helpers (new)
├── activity-event-helpers.test.ts    # filter helper tests (new)
├── activity-event-detail.test.ts     # composition contract tests (new)
├── workspace-mocks/types.ts          # ActivityEvent (enrich if needed)
├── workspace-mocks/data.ts           # ACTIVITY_MOCKS richness
└── index.tsx                         # exports
```

### Out of scope (do not touch)

- Overview rewrite, Knowledge, Spec Kit, audit-detail.tsx / access audit page rewrite
- Fusion of Activity with Audit routes or entity kinds

## Implementation Approach

1. **Helpers**: Extract `filterActivityEvents`, actor/type labels and StatusChip tones.
2. **Detail**: Mirror ApprovalDetail / AuditDetail pattern with sections for Overview, Target, Description; honest loading/not-found; optional Open target.
3. **Page**: Register collection `activity:index` / `activity-event`; columns Event, Actor, Actor kind, Environment, When; filters type/actor/environment (+ optional period); wire DetailPanel; differentiate empty/footer copy from Audit.
4. **Fixtures**: Ensure ≥1 person, ≥1 agent, ≥1 system with clear titles (already present; enrich description/target if thin).
5. **Tests**: Helper unit tests + composition source contracts + page wiring anchors.
6. **Export**: Export detail + helpers from shell index as needed by desktop composition root (page already mounted).

## Complexity Tracking

No constitution violations. Complexity is intentional: Activity must not share Audit detail module so copy, entityKind, and governance sections stay distinct.
