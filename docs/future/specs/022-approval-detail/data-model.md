# Data Model: Inbox Approval Detail

**Feature**: `022-approval-detail`  
**Source of truth**: `packages/features/shell/src/workspace-mocks/types.ts` → `WorkspaceApproval`  
**Fixtures**: `APPROVAL_MOCKS` in `workspace-mocks/data.ts`

## Entity: Approval Request (`WorkspaceApproval`)

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Stable fixture id (`appr-*`) |
| `title` | string | yes | Header title / list primary |
| `description` | string | yes | Short list context (not PrimaryCell description) |
| `requestedBy` | string | yes | Display name of person or agent |
| `requestedByKind` | `"person" \| "agent"` | yes | First-class requester kind |
| `componentId` / `componentName` | string | optional | Related architecture component |
| `resourceId` / `resourceName` | string | optional | Related inventory resource |
| `nodeId` / `nodeName` | string | optional | Related machine/node |
| `environment` | string | yes | Environment label |
| `risk` | `"Low" \| "Medium" \| "High"` | yes | Risk posture → StatusChip |
| `status` | `"pending" \| "approved" \| "rejected"` | yes | Decision status → StatusChip |
| `at` | string | yes | Relative time requested |
| `justification` | string | yes | Operator-readable body |
| `policy` | string | yes | Governing policy label |
| `decidedBy` | string | optional | Required for decided fixtures |
| `decidedAt` | string | optional | Required for decided fixtures |
| `decisionNote` | string | optional | Approver rationale |
| `runId` | string | optional | Related operation run id |
| `runResult` | string | optional | Fixture-backed operation outcome only |

## State transitions (mock/session)

```text
pending  --approve-->  approved  (+ decidedBy, decidedAt, decisionNote)
pending  --reject-->   rejected  (+ decidedBy, decidedAt, decisionNote)
approved / rejected  →  no live re-decide actions in header
```

Session overlay in InboxPage may override `status` (and decision fields) without mutating the frozen fixture array source of truth until parent remounts.

## Fixture coverage gates

| Gate | Requirement |
| --- | --- |
| Status breadth | ≥1 pending, ≥1 approved, ≥1 rejected |
| Risk | ≥1 High |
| Requester | ≥1 `requestedByKind: "agent"` and ≥1 `"person"` |
| Decision trail | approved/rejected include decidedBy + decidedAt + decisionNote |
| Sparse | ≥1 optional related field absent on some fixture (honest em dash) |

## Non-entities (out of scope)

- Live Policy engine evaluation
- Remote apply / mutation plan objects
- Full Audit event rewrite
- Run step timeline (belongs to run-detail)
