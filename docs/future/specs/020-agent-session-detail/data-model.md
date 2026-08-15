# Data Model: Access Agent Session Detail

**Feature**: `020-agent-session-detail`  
**Date**: 2026-08-12  
**Runtime source (mock stage)**: `packages/features/shell/src/workspace-mocks/{types,data}.ts`

## Entities

### AgentSession (`WorkspaceAgentSession`)

Operational execution trace for one agent identity pursuing an objective.

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Stable fixture id (`sess-184`, …) |
| `agentId` | string | yes | FK to `WorkspaceAgent.id` |
| `title` | string | yes | Header / list title |
| `objective` | string | yes | Operator-facing goal text |
| `status` | `"Running" \| "Completed" \| "Waiting" \| "Failed"` | yes | Session lifecycle for this surface |
| `environment` | string | yes | Environment label (Production, Staging, …) |
| `startedAt` | string | yes | Relative or absolute display time |
| `endedAt` | string | no | Present on terminal fixtures |
| `duration` | string | no | Display duration |
| `steps` | `WorkspaceAgentSessionStep[]` | yes | Ordered; may be empty |
| `waitingReason` | string | no | Required for rich Waiting fixtures |
| `approvalId` | string | no | Linkage only; not approval workspace |
| `approvalStatus` | string | no | e.g. pending / approved |
| `relatedOperationIds` | string[] | no | Operations hand-off |
| `relatedAuditIds` | string[] | no | Audit hand-off (metadata) |
| `correlationId` | string | no | Observability hand-off |
| `actorLabel` | string | no | Who requested / owns the session context |
| `actorKind` | `"agent" \| "human" \| "system"` | no | Actor kind for context section |
| `dataSource` | `"fixture" \| "core"` | no | Honesty; mock fixtures use `"fixture"` |

**Validation (mock)**:
- `id` unique across `AGENT_SESSION_MOCKS`
- `agentId` SHOULD resolve in `AGENT_MOCKS` for primary fixtures; missing name falls back in UI
- Terminal statuses (Completed, Failed) SHOULD set `duration` and/or `endedAt`
- Waiting fixtures SHOULD set `waitingReason` and preferably `approvalId`

### AgentSessionStep (`WorkspaceAgentSessionStep`)

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Unique within session |
| `label` | string | yes | Step title |
| `status` | `"Done" \| "Running" \| "Waiting" \| "Failed" \| "Skipped"` | yes | Step progress |
| `tool` | string | no | Tool / capability name |
| `action` | string | no | Concrete action label |
| `startedAt` | string | no | Display start |
| `finishedAt` | string | no | Display finish |
| `duration` | string | no | Display duration |

### Agent identity (read-only link)

Resolved from existing `WorkspaceAgent` / `AGENT_MOCKS`:

| Field used | Purpose |
| --- | --- |
| `id` | Match `session.agentId` |
| `name` | Display label for identity section / Open agent |

Full agent posture is **out of model ownership** for this feature.

## Relationships

```text
WorkspaceAgent (1) ──< owns ── (N) WorkspaceAgentSession
WorkspaceAgentSession (1) ──< contains ── (N) WorkspaceAgentSessionStep
WorkspaceAgentSession ──? links ── Approval (id only)
WorkspaceAgentSession ──? links ── Operation run ids
WorkspaceAgentSession ──? links ── Audit event ids
```

## State transitions (UI-facing, mock)

```text
Running  → (Terminate mock) intent only; no forced model mutate required
Waiting  → (Terminate mock) intent only
Completed / Failed = terminal (no Terminate)
```

Step statuses are independent presentation of progress; session status is the aggregate operator-facing state.

## Fixture set (minimum)

| Id | Status | Role |
| --- | --- | --- |
| `sess-184` | Running | Primary rich deploy session; mixed step statuses; approval context |
| `sess-183` | Completed | Successful architecture review session |
| `sess-182` | Completed | Sync-style completed session |
| `sess-181` | Waiting | Waiting reason + approval linkage |
| `sess-180` (new) | Failed | Failed step coverage |

## Derived view helpers (UI pure)

- `sessionStatusLabel` / `sessionStatusTone`
- `stepStatusLabel` / `stepStatusTone`
- `isSessionTerminable(status)` → Running | Waiting
- `countSessionSteps(steps)` → Done/Running/Waiting/Failed/Skipped totals
- `resolveAgentName(agentId, agents)` → name or fallback
