# Contract: Operations

## Operation message

Includes FR-027 fields plus: `actor_type`, `requested_capability`, `workspace_id`, full policy decision object.

## OperationType (completion)

| Type | Risk | Provider |
| --- | --- | --- |
| PROCESS_START / STOP | normal/privileged | process |
| GIT_* read ops | read-only | git |
| PKG_SCRIPT | normal | pkg |
| PIPELINE_RUN | inherits | workflows |
| DECLARATIVE_PLAN / APPLY | plan read / apply privileged | declarative |
| HANDOFF_OPEN | read-only | handoff |

## RPCs

**RequestOperation** → authenticate → validate → Authorize(role, caps, capability, ctx) → policy → approval? → execute  
**GetOperation** / **ListOperations** (paginated)  
**CancelOperation**  
**SubscribeOperationEvents** stream  

## Idempotency

`idempotency_key` replay returns original op; no double side effects.

## Approval

ResolveApproval: human Owner (or capability-approved role) only; Agent self-approve → permission_denied + FR-027b audit.

## Audit

Executed → FR-027a field set; Denied → FR-027b. Contract/integration tests assert field presence (SC-006).
