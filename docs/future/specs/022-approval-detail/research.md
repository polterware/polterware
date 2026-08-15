# Research: Inbox Approval Detail

**Feature**: `022-approval-detail`  
**Date**: 2026-08-12

## Decision 1: Dedicated module in feature-shell

**Decision**: Own ApprovalDetail in `@polterware/feature-shell` next to `inbox-page.tsx` (not a new package, not access).

**Rationale**: Approvals live under project Inbox (GENERAL nav), not Access. Shell already owns `WorkspaceApproval` / `APPROVAL_MOCKS` and Inbox collection registration.

**Alternatives considered**:
- `feature-operations` — rejected; runs are execution history; approvals are governance inbox.
- `feature-access` — rejected; Access is People/Agents/Policies/Audit.

## Decision 2: Extract, do not deepen inline block

**Decision**: Replace the ~50-line inline DetailPanel in `inbox-page.tsx` with `<ApprovalDetail />`.

**Rationale**: Spec FR-011 / SC-006 and rejection rubric forbid treating thin inline DetailPanel as done.

**Alternatives considered**:
- Keep inline and only add StatusChip — fails density and module requirements.

## Decision 3: StatusChip for risk and status

**Decision**: Map risk (Low/Medium/High) and status (pending/approved/rejected) through pure helpers to design-system StatusChip tones; never hand-roll pills.

**Rationale**: DESIGN / AGENTS LAW for operational status everywhere.

## Decision 4: Session-local decide with decision trail fields

**Decision**: InboxPage keeps a local status map; on decide, also set decidedBy / decidedAt / decisionNote on a session overlay so Decision section renders after approve/reject without claiming remote apply.

**Rationale**: US2 acceptance requires decided trail when operator reopens under Approved/Rejected.

**Alternatives considered**:
- Toast only without status move — breaks tab counts and history inspection.
- Live Connect API — out of scope for mock stage.

## Decision 5: Related run is hand-off only

**Decision**: Show runId / runResult when present; optional `onOpenRun(runId)` callback. Do not import or reimplement run-detail.

**Rationale**: Parallel ownership and SSOT for Runs under Operations.

## Decision 6: Fixture coverage already rich enough as base

**Decision**: APPROVAL_MOCKS already includes pending/approved/rejected, high-risk, agent-requested. Enrich only if a field is missing for Decision trail or sparse-entity coverage; do not invent live policy claims.

**Rationale**: Avoid scope creep; desktop fixture coverage expands asserts only.

## Resolved unknowns

| Unknown | Resolution |
| --- | --- |
| Where to put files | shell package next to inbox-page |
| Live policy engine | Explicitly not this slice |
| Open run navigation | Optional callback; host may no-op in mock stage |
