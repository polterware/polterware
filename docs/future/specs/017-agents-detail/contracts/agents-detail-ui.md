# Contract: Agents Detail UI

**Owner**: `packages/features/access`  
**Consumers**: `AccessView` (Access → Agents tab)  
**Host**: global `DetailPanel` via `MainSurface`  
**Data (mock stage)**: `@polterware/feature-shell` workspace mocks (`WorkspaceAgent`, read-only `WorkspaceAgentSession`)

## Scope

Production-ready **agent** detail under Access → Agents. Mocked only.

Out of scope: live IdentityService / agent control APIs, real pause/revoke mutation, agent **session** detail UI (sibling), policy detail, audit detail, MCP invocation from this panel.

## Activation

```ts
scopeId: "access:agents"
entityKind: "agent"
entityId: agent.id
```

## Props

```ts
type AgentDetailProps = {
  agent: WorkspaceAgent | null | undefined;
  sessions?: readonly WorkspaceAgentSession[];
  loading?: boolean;
  error?: string | null;
  notFound?: boolean;
  onPauseResume?: () => void;
  onEditPermissions?: () => void;
  onRevoke?: () => void;
  onOpenSession?: (sessionId: string) => void;
  onOpenOperation?: (operationId: string) => void;
};
```

## Layout (LAW)

- `DetailPanel.Content`
- `DetailPanel.Header` + identity `Bot` icon (16px) + `DetailPanel.Actions` / `DetailPanel.Action`
- `DetailPanel.Body layout="sections"`
- `DetailPanel.Grid` + `Section` / `Metric` / `Metadata` — no `Card`, no second scroller
- Status / capability effect / tool kind / session status / activity result → `StatusChip` (filled)
- Collections → compound `DataTable` (no Actions column; no PrimaryCell `description=`)

## Sections

| Section | Content |
| --- | --- |
| Metrics | Status StatusChip, Autonomy, Scope, Environments count (or Type) |
| Overview | Type, status, autonomy, owner, scope, last activity, optional description/created |
| Capabilities | Structured capability name + effect StatusChip |
| Tools | Tool name + kind StatusChip/label |
| Environments | Environment scope list or empty copy |
| Linked operations | Operation ids; activate → `onOpenOperation` |
| Recent sessions | Filtered sessions; activate → `onOpenSession` only (no embedded session UI) |
| Activity | Optional events when present |
| Notes | Optional free-text when present |

## Header actions

| Action | Tone | Mock behavior |
| --- | --- | --- |
| Pause / Resume | default | label depends on status; toast success `(mock)` |
| Edit permissions | default | toast default `(mock)` |
| Revoke | danger | toast success `(mock)` |
| Open recent session | default | only when sessions exist; callback `onOpenSession` |

## States

| State | UI |
| --- | --- |
| loading | Header “Loading agent” + `PageLoading` |
| error | Header “Failed to load” + `PageError` |
| notFound / null | Header “Agent not found” + `PageError` |
| ready | Full sections |

## Fixture requirements

`WorkspaceAgent` MUST include (mock stage):

- identity/list: `id`, `name`, `type`, `status`, `autonomy`, `scope`, `last`, `owner`
- posture: `lastActivity`, `environments[]`, optional `description`, `createdAt`, `notes`
- collections: structured `capabilities[]`, `tools[]`, `recentOperationIds[]`, optional `activity[]`

Primary Deployment Agent fixture (`a1`) MUST have non-empty structured `capabilities` and `tools`, non-empty `environments`, at least one `recentOperationIds` entry, and at least one `AGENT_SESSION_MOCKS` row with `agentId === "a1"`.

## Security honesty

- Capability and tool tables are **display only** — not authorization.
- Mock mutations MUST NOT claim live core / IdentityService enforcement.
- No secrets, tokens, or credential material in detail DTOs.
- `dataSource` remains fixture; never claim live agent runtime.

## Cross-navigation

- Session row / Open recent session → `onOpenSession(sessionId)` → Access activates `entityKind: "agent-session"` (session detail owned by sibling).
- Linked operation → `onOpenOperation(operationId)` when host provides Operations hand-off.
- MUST NOT mount or redesign policy / audit / session detail compositions from this feature.

## Parallel file ownership

| May change | Must not change |
| --- | --- |
| `agent-detail.tsx`, helpers, tests | `policy-detail*`, `audit-detail*`, `agent-session-detail*` |
| Access index agent mount only | `member-detail*`, `team-detail*` (except required imports) |
| `WorkspaceAgent` + `AGENT_MOCKS` only | `POLICY_MOCKS`, `AUDIT_EVENT_MOCKS`, rewrite of `AGENT_SESSION_MOCKS` |
| agent asserts in desktop fixture coverage | design-system primitives (unless blocking bug) |
