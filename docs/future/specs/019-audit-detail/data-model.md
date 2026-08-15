# Data Model: Access Audit Event Detail

**Feature**: `019-audit-detail`  
**Date**: 2026-08-12  
**Owner type location**: `packages/features/shell/src/workspace-mocks/types.ts`  
**Fixtures**: `packages/features/shell/src/workspace-mocks/data.ts` → `AUDIT_EVENT_MOCKS`

## Entity: WorkspaceAuditEvent

Append-only project audit trail record for Access → Audit inspection.

```ts
type WorkspaceAuditEvent = {
  id: string;
  actor: string;
  actorKind: "person" | "agent";
  actorId?: string;
  action: string;
  entity: string;
  entityKind?: string;
  environment: string;
  result: "Success" | "Pending" | "Failed" | "Denied";
  origin: string;
  at: string;
  approvalId?: string;
  policyId?: string;
  policyName?: string;
  requestId?: string;
  correlationId?: string;
  runId?: string;
  contextNotes?: string;
  contextPayload?: string;
};
```

### Field rules

| Field | Required | Validation / notes |
| --- | --- | --- |
| `id` | yes | Stable fixture key (`au1`…); activation target entityId |
| `actor` | yes | Display name of person or agent |
| `actorKind` | yes | `person` \| `agent` only |
| `actorId` | no | When known (e.g. `m1`, `ag1`) |
| `action` | yes | Dotted capability-style string; mono in UI |
| `entity` | yes | Human-readable subject of the action |
| `entityKind` | no | e.g. `arch-node`, `run`, `policy`, `approval`, `integration`, `invitation` |
| `environment` | yes | Named env or `All` |
| `result` | yes | Closed set above |
| `origin` | yes | Client/channel string (Desktop, Agent session, Integration webhook, …) |
| `at` | yes | Display timestamp (fixture short clock form is OK in mock stage) |
| `approvalId` | no | Related approval id when gated |
| `policyId` / `policyName` | no | Related policy reference |
| `requestId` | no | Operation/request id |
| `correlationId` | no | Cross-surface correlation |
| `runId` | no | Related operation run when applicable |
| `contextNotes` | no | Short human notes |
| `contextPayload` | no | Short mono snippet; not live secret material |

### Invariants

1. Audit events are **immutable** in product semantics — UI never mutates the trail.
2. Related policy/approval ids are **references only** — not embedded detail models.
3. Correlation/request/run ids MUST NOT be invented when absent.
4. `actorKind` drives icon only; not authorization.
5. No secrets, tokens, or credential material in audit DTOs.

## Relationships

```text
WorkspaceAuditEvent
  ├─ (optional) policyId ──► Policy fixture id (label via policyName; open via callback)
  ├─ (optional) approvalId ─► Approval / Inbox reference (open via callback/toast only)
  ├─ (optional) runId ──────► Operation run reference (display only this slice)
  └─ (optional) actorId ────► Person or Agent id (display only; open not required)
```

## Fixture coverage plan

| Id | Role in detail QA |
| --- | --- |
| `au1` | Person success, arch-node, no related controls; context optional |
| `au2` | Agent pending ops request; **policy + approval**; request/correlation/run ids |
| `au3` | Person approve; approval + policy; success |
| `au4` | Agent integration sync success; no policy |
| `au5` | Person policy update; policy related |
| `au6` | Person invite success; no related controls |
| `au7` | Agent **Denied** ops apply; policy only |

Minimum completeness gates:

- ≥1 event with both `policyId` and `approvalId`
- ≥1 `Denied` result
- ≥1 with `requestId` + `correlationId`
- ≥1 with neither policy nor approval
- ≥1 with `contextNotes` or `contextPayload`
- All events have required identity/result/action fields

## State transitions

None. Audit events are point-in-time records. Result is a stored outcome label, not a UI-driven state machine.

## Out of model

- Multi-step lifecycle timeline timestamps
- Full JSON structured logging product
- Live polter-core audit store schema / migrations
