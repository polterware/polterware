# UI Contract: Agent Session Detail

**Feature**: `020-agent-session-detail`  
**Surface**: Access → Agents → Recent sessions → global DetailPanel  
**Stage**: Mock fixtures only

## Activation

| Field | Value |
| --- | --- |
| `scopeId` | `access:agents` |
| `entityKind` | `agent-session` |
| `entityId` | `WorkspaceAgentSession.id` |
| List host | Recent sessions `DataTable` on Access Agents tab |
| Target registration | Existing `buildAccessDetailTargets` includes agent sessions when `tab === "agents"` |

## Composition

- Component: `AgentSessionDetail` in `@polterware/feature-access`
- Host: `AccessView` mounts when active target is `access:agents` + `agent-session`
- Layout: `DetailPanel.Body layout="sections"` + `DetailPanel.Grid`
- Forbidden: second inspector page, drawer-as-detail, modal-as-detail, Actions column in steps table

## Props (view-model ports)

| Prop | Type | Required | Notes |
| --- | --- | --- | --- |
| `session` | `WorkspaceAgentSession \| null \| undefined` | yes | Entity under inspection |
| `agentName` | `string \| null \| undefined` | no | Resolved display name; fallback if missing |
| `loading` | `boolean` | no | Shows PageLoading |
| `error` | `string \| null` | no | Shows PageError |
| `notFound` | `boolean` | no | Missing entity honesty |
| `onTerminate` | `(sessionId: string) => void` | no | Mock terminate |
| `onOpenAgent` | `(agentId: string) => void` | no | Hand-off to agent identity detail host |
| `onOpenOperation` | `(operationId: string) => void` | no | Optional related operation hand-off |

## Header

| Element | Rule |
| --- | --- |
| Icon | Activity or Bot, 16px, decorative |
| Title | `session.title` single line |
| Terminate | Danger; only if status is `Running` or `Waiting` |
| Open agent | Always when session known; calls `onOpenAgent(session.agentId)` |
| Open related operation | Optional; only when `relatedOperationIds[0]` exists |

## Body sections

1. **Overview metrics** — Status (`StatusChip`), Environment, Started, Duration/Ended (when present)
2. **Objective** — prose paragraph of `objective`
3. **Agent identity** — name (+ id mono), Open agent is header; section may show id/type context
4. **Step progress metric** — done/total + `DitherSegmentedBar` (Done/Running/Waiting/Failed[/Skipped])
5. **Steps** — `DataTable`: label (PrimaryCell), status chip, tool/action, started/finished or duration; empty → “No steps for this session.”
6. **Waiting / approval** — when Waiting or waitingReason/approval present; else honest empty or omit fabricated approval
7. **Related references** — operation ids / audit ids mono metadata when present
8. **Context / identity honesty** — environment, actor label/kind, session id, correlation id, data source

## Status mapping (session)

| Status | Chip tone | Terminable |
| --- | --- | --- |
| Running | warn | yes |
| Waiting | warn | yes |
| Completed | healthy | no |
| Failed | danger/down | no |

## Status mapping (step)

| Status | Chip tone |
| --- | --- |
| Done | healthy |
| Running | warn |
| Waiting | warn / muted-adjacent warn |
| Failed | danger/down |
| Skipped | muted |

## Honesty

- `dataSource: "fixture"` → display “Fixture (mock)” (or equivalent)
- Terminate toast MUST include mock wording
- MUST NOT claim live agent runtime kill
- MUST NOT re-render agent identity detail composition

## Out of contract

- Agent identity detail (017)
- Policy detail (018)
- Audit event detail (019) beyond mono id display
- Live WebSocket/session streams
