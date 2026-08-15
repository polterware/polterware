# Contract: Access Audit Event Detail UI

**Owner**: `packages/features/access`  
**Extends**: 004 Access Audit collection; aligns with `docs/spec.md` audit / governance inspection  
**Host**: Global `DetailPanel` via `DetailPanel.Content` (MainSurface)

## View model

```ts
type AuditActorKind = "person" | "agent";

type AuditResult = "Success" | "Pending" | "Failed" | "Denied";

type WorkspaceAuditEvent = {
  id: string;
  actor: string;
  actorKind: AuditActorKind;
  actorId?: string;
  action: string;
  entity: string;
  entityKind?: string;
  environment: string;
  result: AuditResult;
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

## Component API

```ts
function AuditDetail(props: {
  event: WorkspaceAuditEvent | null | undefined;
  loading?: boolean;
  error?: string | null;
  notFound?: boolean;
  onCopyEventId?: (eventId: string) => void;
  onOpenPolicy?: (policyId: string) => void;
  onOpenApproval?: (approvalId: string) => void;
}): JSX.Element;
```

## Layout contract

1. Root is `DetailPanel.Content` only (no second page/drawer/modal).
2. Body uses `layout="sections"` + `DetailPanel.Grid`.
3. **Header**
   - Icon: User (person) or Bot (agent) at 16px — same family as Audit table PrimaryCell.
   - Title: event `action` (fallback `Audit / {id}`).
   - Actions (optional, non-mutating only):
     - `Copy event id` → `onCopyEventId`
     - `Open policy` when `policyId` present → `onOpenPolicy`
     - `Open approval` when `approvalId` present → `onOpenApproval`
   - **Forbidden**: Approve, Reject, Delete, Edit, Revoke, Terminate, any mutation tone as primary governance change.
4. **Metrics row** (found event)
   - Result → `StatusChip` with semantic tone
   - Environment
   - Origin
   - Time (`at`, mono preferred in metadata; metric OK as value)
5. **Sections**
   - **Actor**: name, kind label, optional actorId (mono)
   - **Action**: action (mono), entity, optional entityKind
   - **Correlation** (only if any of requestId / correlationId / runId): mono ids
   - **Related controls** (only if policy or approval present): policy name/id, approval id
   - **Context**: notes and/or payload mono; honest empty when both absent
6. **Read-only note**: muted text inside Context empty or a small footer line inside body is allowed; no competing banner chrome.
7. **States**
   - `loading` → PageLoading
   - `error` / `notFound` / missing event → PageError
   - Found → sections above

## StatusChip tones

| Result | Tone |
| --- | --- |
| Success | healthy |
| Pending | warn |
| Failed | danger |
| Denied | danger |

## Host wiring (AccessView)

```ts
// Mount when audit tab target is active (not only when fixture found):
activeTarget()?.scopeId === "access:audit"
  && activeTarget()?.entityKind === "audit-event"

// Pass:
event={audit()}
notFound={!audit()}
onCopyEventId={(id) => /* clipboard + mock toast */}
onOpenPolicy={(id) => activate(id, "policy")} // or parent callback
onOpenApproval={(id) => pushToast(`Open approval ${id} (mock)`, "default")}
```

## Collection non-regression

- Audit DataTable remains virtualized.
- Filter chips (actor kind, action prefix, environment, result) unchanged.
- Collection footer “Audit is read-only in this mock stage.” may remain.
- `entityKind: "audit-event"` activation targets unchanged.

## Out of contract

- Agents / Policies / Agent session detail compositions
- Building PolicyDetail or Approval detail inside AuditDetail
- Live audit API / ConnectRPC
- Timeline mini without multi-step fixture timestamps
- PrimaryCell descriptions, Actions columns, hand-rolled status pills
