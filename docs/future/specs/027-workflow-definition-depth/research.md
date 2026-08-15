# Research: Workflow Definition Depth

**Date**: 2026-08-12  
**Feature**: 027-workflow-definition-depth

## Decision 1: Sequential pipeline only

- **Decision**: Keep the existing trigger → ordered steps → end pipeline. Do not introduce freeform multi-edge canvas.
- **Rationale**: docs/spec.md and product laws position Polter workflows as governed sequential orchestration, not n8n clone. Existing builder already has sequential UX to deepen.
- **Alternatives considered**: Freeform canvas (rejected — out of scope); branching DAG editor (deferred).

## Decision 2: Enrich WorkflowDef in feature types (mock)

- **Decision**: Extend `WorkflowDef` / `WorkflowStep` in `packages/features/operations/src/types.ts` with version (required), failureStrategy, input/output/artifact summaries, per-step retryPolicy, approval semantics, condition.
- **Rationale**: Domain §25 fields; mock production parity without protobuf/core work.
- **Alternatives considered**: Flat string config only (rejected — insufficient depth); shared contracts package (overkill for fixture UI).

## Decision 3: Definition status vs run status

- **Decision**: Dedicated `workflowStatusLabel` / `workflowStatusTone` helpers. Stop mapping definition status through run-status helpers alone.
- **Rationale**: Draft/Ready are definition lifecycle; Succeeded/Failed on a definition row often mean last outcome — helpers normalize both families.
- **Alternatives considered**: Reuse run-status only (rejected — Draft/Ready map poorly).

## Decision 4: Version bump semantics

- **Decision**: Create save → version 1; edit save → version + 1; UI shows version in header/footer; copy states runs bind immutable version at start.
- **Rationale**: Domain invariant “workflow is versioned; each run references immutable version.”
- **Alternatives considered**: Dirty-flag-only versioning (weaker); semantic versioning strings (unnecessary for mock).

## Decision 5: Recent runs via callback

- **Decision**: Builder accepts optional `recentRuns` + `onOpenRun`; host resolves from RUN_FIXTURES by workflowId. Does not import or rewrite RunDetail.
- **Rationale**: Rubric forbids run-detail rewrite; still closes definition↔run loop.
- **Alternatives considered**: Embed mini run detail (rejected); navigate only with toast (weaker).

## Decision 6: Tests as pure helpers + fixture contracts

- **Decision**: Extract pure model helpers for createStep defaults, version bump, field contracts; unit-test under vitest node without Solid render if not already configured.
- **Rationale**: operations vitest is node environment; schedule/run packages already use helper + fixture contract tests.
- **Alternatives considered**: Full Solid component mount tests (heavier; optional later).
