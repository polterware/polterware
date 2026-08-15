# Contract: Approval Detail UI

**Feature**: `022-approval-detail`  
**Surface**: Inbox (Pending · Approved · Rejected) → global DetailPanel  
**Stage**: Mock / session-local decision (production-ready composition)

## Host

- Collection: Inbox DataTable per status tab
- Detail host: MainSurface global DetailPanel only
- Scope id pattern: `inbox:{pending|approved|rejected}`
- Entity kind: `approval`
- Owner package: `@polterware/feature-shell`
- Composition module: `packages/features/shell/src/approval-detail.tsx`

## Props (host → composition)

| Prop | Type | Notes |
| --- | --- | --- |
| `approval` | `WorkspaceApproval \| null \| undefined` | Resolved fixture / session overlay |
| `loading` | boolean | optional |
| `error` | string \| null | optional |
| `notFound` | boolean | optional |
| `onApprove` | `(id: string) => void` | pending only |
| `onReject` | `(id: string) => void` | pending only |
| `onOpenRun` | `(runId: string) => void` | optional hand-off; never reimplements run-detail |

## Composition

- Dedicated `ApprovalDetail` module (extract from inline Inbox block)
- Header: domain inbox/security icon + request title + `DetailPanel.Action` only
- Body: `layout="sections"` + Grid / Section / Metric / Metadata
- Risk and status: `StatusChip` only (no hand-rolled pills)
- Honest states: `PageLoading`, `PageError`, not-found copy

## Header actions

| Status | Actions |
| --- | --- |
| pending | Reject, Approve (`tone="primary"`) |
| approved / rejected | none live approve/reject; optional Open related run when `runId` present |

## Required sections

1. **Overview** — risk (`StatusChip`), status (`StatusChip`), environment
2. **Request context** — requested by, kind (Person|Agent), component/node/resource (em dash when absent), policy, when, related run id
3. **Justification** — body text
4. **Decision** (non-pending) — decided-by, decided-at, decision note, optional operation result (`runResult`)

## Session decide semantics

- Host may overlay `status` and decision fields after Approve/Reject
- UI MUST NOT claim live policy engine enforcement or remote apply completion
- Optional `runResult` is fixture- or session-authored presentation only

## Honesty

- mock/session decide only
- no claim of live policy engine or remote apply unless fixture `runResult` exists
- not-found / sparse related fields required
- collection activation independent of panel snap mode
