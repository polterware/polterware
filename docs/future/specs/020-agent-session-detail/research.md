# Research: Access Agent Session Detail

**Feature**: `020-agent-session-detail`  
**Date**: 2026-08-12

## R1 — Detail host and composition pattern

**Decision**: Use the single global `DetailPanel` with a dedicated `AgentSessionDetail` composition (`layout="sections"`), matching MemberDetail / TeamDetail / RunDetail.

**Rationale**: AGENTS.md LAW forbids second inspectors; Access already activates `entityKind: "agent-session"` and hosts an inline thin detail. Extraction is required for production quality and tests.

**Alternatives considered**:
- Keep inline detail in `index.tsx` — rejected (fails FR-017 / SSoT XIV; hard to test).
- Full-page session route — rejected (forbidden detail pattern).

## R2 — Progress visualization

**Decision**: Reuse `DitherSegmentedBar` for step progress (Done / Running / Waiting / Failed / optional Skipped), same family as Operations Run detail.

**Rationale**: Operators already learn progress semantics on Runs; sessions are agent-side execution traces with analogous step state. Charts package already depended on by access.

**Alternatives considered**:
- Text-only counts without bar — weaker production parity with run-detail.
- Custom feature SVG progress — forbidden parallel chrome.

## R3 — Status vocabulary and tones

**Decision**: Keep product-facing session statuses `Running | Completed | Waiting | Failed`. Map to StatusChip tones: Completed→healthy, Running→warn (or info-equivalent warn), Waiting→warn, Failed→danger/down. Step statuses extend with `Failed` and optional `Skipped` for honesty.

**Rationale**: Existing list fixtures already use the four session statuses; enriching steps with Failed is required for SC-003 / failed fixtures without inventing new session lifecycle.

**Alternatives considered**:
- Mirror full run lifecycle (`waiting_approval`, `lost`, …) — over-scope for Access session table; keep simpler operator labels.
- Leave steps as Done/Running/Waiting only — insufficient for Failed session story.

## R4 — Agent identity hand-off

**Decision**: Resolve `agentId` → display name via `AGENT_MOCKS` lookup passed as prop or resolved by host; expose `onOpenAgent(agentId)` only. Do not import or edit `agent-detail.tsx`.

**Rationale**: Parallel agent implements 017. Session detail must not dual-own agent posture UI.

**Alternatives considered**:
- Embed mini agent card with capabilities/tools — reimplements agent detail; boundary violation.
- Deep-link only with raw id — worse UX; name lookup is cheap fixture work.

## R5 — Terminate semantics

**Decision**: Header action Terminate only when status is Running or Waiting; click → mock toast; no fixture mutation required in this slice (honest intent acknowledgment).

**Rationale**: Matches run cancel honesty; real kill is core/runtime ownership later.

**Alternatives considered**:
- Optimistic local status flip to Failed — implies persistence; avoid without store model.
- Always show Terminate disabled — clutter; hide for terminal states.

## R6 — Fixture enrichment scope

**Decision**: Extend `WorkspaceAgentSession` / `WorkspaceAgentSessionStep` in shell workspace mocks; expand `AGENT_SESSION_MOCKS` with duration/endedAt, tools, waitingReason, approvalId, relatedOperationIds, relatedAuditIds, correlationId, actor fields, dataSource, plus a Failed session fixture.

**Rationale**: Current steps are label+status only — insufficient for production steps table and Waiting/Failed stories. Shell mocks are the SSOT for Access fixtures.

**Alternatives considered**:
- Feature-local overlay map keyed by session id — dual model, drifts from list.
- Backend schema change — out of mock stage.

## R7 — Related operations / audit

**Decision**: Display mono ids in metadata; optional header Open related operation when first related operation id exists, wired to host `onOpenOperation` if available.

**Rationale**: FR-013 allows hand-off without embedding Operations/Audit detail.

**Alternatives considered**:
- Nested operation mini-detail — scope creep / dual inspector risk.
- Ignore related ids entirely — weaker operator continuity.

## R8 — Testing strategy

**Decision**: Pure helpers for status tones/labels/terminable gate; source-level composition contract tests (same style as member/team); expand desktop `detail-fixture-coverage` for multi-status + rich primary session.

**Rationale**: Constitution XII risk-proportional; no Electron mount required for composition contract.

**Alternatives considered**:
- Full Solid render tests only — heavier; composition source contracts already gate LAW effectively in this monorepo.
