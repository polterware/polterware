# Research: Access Policies Detail

**Feature**: `018-policies-detail`  
**Date**: 2026-08-12  
**Status**: Complete — no open NEEDS CLARIFICATION  
**Revalidated**: 2026-08-12 (`/speckit-plan`) against landed `PolicyDetail` + fixtures

---

## R1 — Detail host and composition pattern

**Decision**: Use the single global `DetailPanel` with `Body layout="sections"` + `Grid` / `Section` / `Metric` / `Metadata`, matching People member and Teams operational details. Extract a dedicated `PolicyDetail` component; replace the thin inline policy block in `AccessView`.

**Rationale**:
- AGENTS.md / DESIGN.md LAW forbid second inspectors, drawers, or entity detail pages for operational entities.
- MemberDetail / TeamDetail already define the production Access detail chrome operators expect.
- Extraction keeps Access index maintainable under parallel agent work and makes the composition testable.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Full-page policy route | Violates DetailPanel LAW and Access IA |
| Document layout (`layout="document"`) | Reserved for Knowledge / Spec Kit writing surfaces |
| Keep inline thin detail | Fails FR-015 and SC-005 quality bar |

---

## R2 — Data source for mock stage

**Decision**: Enrich `WorkspacePolicy` in `@polterware/feature-shell` workspace mocks (`POLICY_MOCKS`). Access consumes mocks; no ConnectRPC / polter-core policy API in this slice. Related audit is **read-only filtered** from existing `AUDIT_EVENT_MOCKS` by `policyId` / `policyName` without mutating audit fixtures.

**Rationale**:
- Policies list already uses `POLICY_MOCKS`; detail must not invent a second policy type.
- Shell mocks are the established SSOT for operator surface demos.
- Parallel agent owns Audit detail; this feature only lists related rows.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Feature-local fixtures only in access | Dual policy SSOT with list vs detail drift |
| Fake live RPC with empty backend | Dishonest “live” claim (Constitution III) |
| Embed full audit events into policy fixtures | Dual audit SSOT; conflicts with parallel audit agent |

---

## R3 — Actors and actions structure

**Decision**: Keep list-friendly `actors: string[]`, `allowedActions: string[]`, `blockedActions: string[]` for the Policies table. Add structured detail fields: `actorEntries[]` (kind person | agent | role), optional `activity[]`, `riskPosture`, `approvalRequirement`, `dualApproval`, optional `notes`. Detail prefers `actorEntries` when present; falls back to mapping `actors` strings as role-kind entries for honesty.

**Rationale**:
- Existing table columns and consumers use the string arrays.
- Structured actors enable people/agents/roles PrimaryCell + kind columns without inventing a second list type.
- Fallback mapping avoids empty Actors when only legacy strings exist.

**Alternatives considered**:
| Alternative | Why rejected |
| --- | --- |
| Replace string arrays entirely | Risk of breaking Policies DataTable and other consumers mid-parallel work |
| Only string join in detail | Fails FR-007 structured collection requirement |

---

## R4 — Parallel agent file boundary

**Decision**: Touch only Policy detail composition, helpers/tests, `WorkspacePolicy` + `POLICY_MOCKS`, Access index policy Show swap, desktop fixture coverage policy asserts, and this specs folder. Do not create or edit `agent-detail*`, `audit-detail*`, `agent-session-detail*`, or rewrite Agents/Audit/Sessions tables. Do not mutate `AGENT_MOCKS`, `AGENT_SESSION_MOCKS`, `AUDIT_EVENT_MOCKS`.

**Rationale**: Three sibling agents implement those surfaces concurrently; intentional zero-conflict boundary is a hard product constraint for this delivery.

---

## R5 — Related audit navigation

**Decision**: Pass filtered related audit events into `PolicyDetail` as a prop. Row activate MAY call `onOpenAudit(eventId)`. This feature does not implement Audit detail content. Access wiring filters with `matchesPolicyAudit(policy, event)` on `AUDIT_EVENT_MOCKS` (read-only) and activates `entityKind: "audit-event"` when no external `onOpenAudit` is provided.

**Rationale**: Keeps policy composition pure and testable; Access index owns fixture filtering and activation for audit targets if/when Audit detail exists.

**Landed confirmation**: `packages/features/access/src/policy-detail.tsx` + `index.tsx` PolicyDetail mount; types in `workspace-mocks/types.ts`; primary fixture `p1` complete; package + desktop fixture tests green.
