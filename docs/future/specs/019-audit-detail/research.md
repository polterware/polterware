# Research: Access Audit Event Detail

**Feature**: `019-audit-detail`  
**Date**: 2026-08-12

## Decision 1: Dedicated `AuditDetail` composition

**Decision**: Extract audit detail from the thin inline `Show when={audit()}` block in `AccessView` into `packages/features/access/src/audit-detail.tsx` (+ helpers + tests), matching MemberDetail / TeamDetail / RunDetail quality.

**Rationale**: AGENTS.md LAW requires a single global DetailPanel with feature-owned `DetailPanel.Content`. Inline shallow blocks drift from production detail quality (honest loading/not-found, StatusChip metrics, structured sections). Extraction is the established Access pattern.

**Alternatives considered**:
- Keep enriching inline block — rejected (SSOT + maintainability; sibling details already extracted).
- Separate audit detail page route — rejected (forbidden by DetailPanel LAW).

## Decision 2: Read-only surface with optional non-mutating actions

**Decision**: No Approve / Reject / Delete / Edit. Optional header actions only: **Copy event id** (always available for found events) and **Open policy** when `policyId` is present. Approval open is callback/toast only when `approvalId` present — never builds Approval detail here.

**Rationale**: Audit is append-only inspection (product + constitution VII). Mutation chrome would contradict append-only honesty. Copy/open are operator convenience, not trail mutation.

**Alternatives considered**:
- Zero header actions — acceptable but Copy event id is high-value for correlation handoff.
- Open entity deep-links (run, arch-node, integration) — deferred; entity kinds vary and sibling features own those details; out of scope for this slice.

## Decision 3: Enrich `WorkspaceAuditEvent` modestly

**Decision**: Add optional fields to the existing shell type (do not invent a second DTO):

| Field | Purpose |
| --- | --- |
| `actorId?` | Stable actor identity when known |
| `requestId?` | Operation/request correlation |
| `correlationId?` | Cross-surface correlation |
| `runId?` | Link to operation run when action is run-scoped |
| `contextNotes?` | Human-readable notes |
| `contextPayload?` | Short mono payload snippet (fixture string, not live JSON dump) |

Keep existing: id, actor, actorKind, action, entity, entityKind, environment, result, origin, at, approvalId, policyId, policyName.

**Rationale**: Spec FR-008–FR-010 require correlation and context. Fixtures already partially cover policy/approval. Modest enrichment matches RunDetail’s correlation pattern without SIEM product scope.

**Alternatives considered**:
- Nested `context: { notes, payload }` — unnecessary indirection for mock stage.
- Full structured JSON editor — wrong surface (audit is inspection, not document layout).

## Decision 4: Omit timeline mini

**Decision**: Do not render a created → decided → applied timeline.

**Rationale**: Current fixtures are point-in-time events. Spec explicitly allows omission when multi-step timestamps are unsupported. Inventing timeline steps would violate honesty (constitution XI / mock honesty).

**Alternatives considered**:
- Synthetic timeline from result state — rejected (fabricates history).

## Decision 5: Mount condition for honest not-found

**Decision**: Mount `AuditDetail` when `activeTarget()?.scopeId === "access:audit" && entityKind === "audit-event"`, pass `event={audit()}` and `notFound={!audit()}` — same pattern as MemberDetail / TeamDetail — instead of `Show when={audit()}` which hides not-found entirely.

**Rationale**: FR-011 requires not-found honesty.

## Decision 6: File boundary discipline under parallel agents

**Decision**: Touch only:

- `packages/features/access/src/audit-detail*.tsx|ts`
- `packages/features/access/src/index.tsx` (audit Show extraction only)
- shell `WorkspaceAuditEvent` + `AUDIT_EVENT_MOCKS`
- desktop `detail-fixture-coverage.test.ts` audit asserts only
- `specs/019-audit-detail/**`

Do not touch agent-detail*, policy-detail*, agent-session-detail*, AGENT_MOCKS / POLICY_MOCKS / AGENT_SESSION_MOCKS (except read for related labels if needed — prefer fixture-local policyName).

**Rationale**: Three sibling agents implement adjacent Access details concurrently.

## Decision 7: Result StatusChip tones

**Decision**: Reuse the existing result tone mapping used in Access index / member helpers:

| Result | Tone |
| --- | --- |
| Success | healthy |
| Pending | warn |
| Failed | danger |
| Denied | danger |

Centralize in `audit-detail-helpers.ts` for testability (do not depend on member-detail-helpers for domain separation).

## Decision 8: Title strategy

**Decision**: Prefer short **action** string as header title (operator-scannable). Event id appears in metadata (and Copy action). Fallback title `Audit / {id}` only if action missing (should not happen for fixtures).

**Rationale**: Spec allows action or `Audit / {id}`; action matches how operators filter/search the trail.

## Decision 9: Testing strategy

**Decision**:
1. Pure helpers unit tests (result tone/label, title, actor icon kind key).
2. Composition source contract (DetailPanel sections, StatusChip, no mutation labels, loading/not-found, wiring from index).
3. Fixture field presence via source reads + desktop detail-fixture-coverage enrichment asserts.

No Electron render tests required for this mock stage (matches People/Teams detail approach).

## Resolved unknowns

All technical unknowns resolved from existing MemberDetail / TeamDetail / RunDetail patterns and current Audit collection code. No remaining NEEDS CLARIFICATION.
