# Implementation Plan: Workflow Definition Depth

**Branch**: `refactor/new` (feature dir `027-workflow-definition-depth`) | **Date**: 2026-08-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/027-workflow-definition-depth/spec.md`

## Summary

Deepen Operations → Workflows definition authoring: enrich `WorkflowDef`/`WorkflowStep` to domain §25 (version, failure strategy, retries, approval semantics, I/O summaries), upgrade sequential `WorkflowBuilder` config panels and StatusChip/version chrome, expand fixtures, add pure helpers + unit tests. Do not rewrite Run/Schedule detail; do not add freeform canvas.

## Technical Context

**Language/Version**: TypeScript (strict), SolidJS 1.9  

**Primary Dependencies**: `@polterware/frontend-design-system`, `@polterware/frontend-state`, solid-js  

**Storage**: Session-local fixtures only (`dataSource: "fixture"`)  

**Testing**: vitest (node) in `packages/features/operations`  

**Target Platform**: Desktop renderer (Electron) — Solid feature package  

**Project Type**: monorepo feature package  

**Performance Goals**: DetailPanel-only scroll; no heavy graph layout  

**Constraints**: DESIGN.md laws; no run-detail/schedule-detail edits; sequential pipeline only; no new deps  

**Scale/Scope**: Single feature package surface (workflows tab + builder)

## Constitution Check

- **I** Operational authoring + audit honesty — pass  
- **II** Domain types in feature package; no provider SDK — pass  
- **III** Mock renderer only; no claimed live engine — pass  
- **IV** `packages/features/operations` only — pass  
- **V** UI contract in `contracts/`; no protobuf change — pass  
- **VI** No secrets in fixtures — pass  
- **VII** Version immutability, approval ≠ execution, retry respects policy — pass  
- **VIII** Offline fixture UI — pass  
- **IX** No Main/UI thread heavy work — pass  
- **X** Empty/not-found honesty — pass  
- **XI** workflow id/version visible — pass  
- **XII** Unit tests required — pass  
- **XIII** No scenic packages — pass  
- **XIV** Single builder composition — pass  

**Gate**: PASS

## Project Structure

### Documentation (this feature)

```text
specs/027-workflow-definition-depth/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/workflow-builder-ui.md
├── checklists/requirements.md
└── tasks.md
```

### Source (touch only)

```text
packages/features/operations/src/
├── types.ts                 # enrich WorkflowDef / WorkflowStep
├── fixtures.ts              # deep workflow fixtures
├── workflow-status.ts       # NEW status helpers
├── workflow-status.test.ts  # NEW
├── workflow-model.ts        # NEW pure helpers (create step, version, fields)
├── workflow-model.test.ts   # NEW
├── workflow-builder.tsx     # depth UI
├── workflow-builder.test.ts # NEW composition contract via helpers
└── index.tsx                # StatusChip helpers, recent runs, not-found wiring
```

**Forbidden**: `run-detail.*`, `schedule-detail.*`, architecture/access/shell/integrations packages

## Implementation approach

1. Types + data-model alignment  
2. `workflow-status` + `workflow-model` helpers + tests  
3. Enrich fixtures (≥2 rich)  
4. Upgrade builder UI (StatusChip, version, type configs, failure/retry/approval, recent runs, empty/not-found)  
5. Wire `OperationsView` workflows list StatusChip + pass recent runs / onOpenRun  
6. Run operations package tests  

## Complexity Tracking

No constitution violations requiring justification.
