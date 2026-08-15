# Data Model: Access Agents Detail

**Feature**: `017-agents-detail`  
**Stage**: Mock fixtures (shell workspace mocks)  
**Live persistence**: out of scope

---

## Overview

```text
Agent
  ├── Status (Active | Paused | Revoked | Disabled)
  ├── Autonomy (supervised | semi-autonomous | autonomous)
  ├── Owner / Scope / Type
  ├── Environments[]
  ├── Capability[]
  ├── Tool[]
  ├── recentOperationIds[] → Operation (hand-off)
  ├── Sessions[] (by agentId) → AgentSession (hand-off)
  └── ActivityEvent[] / notes (optional)
```

Domain roots (`docs/spec.md`): **Access** governance for AI agents and application identities as first-class collaborators. UI entity is the project **Agent** under Access → Agents.

---

## Entities

### Agent — `WorkspaceAgent`

Governed collaborator shown in Access Agents list and detail.

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Stable fixture id (`a1`…) |
| `name` | string | yes | Header title; table PrimaryCell title |
| `type` | string | yes | e.g. AI Agent, Application |
| `status` | `"Active" \| "Paused" \| "Revoked" \| "Disabled"` | yes | StatusChip |
| `owner` | string | yes | Display owner |
| `capabilities` | AgentCapability[] | yes | May be empty → empty table |
| `tools` | AgentTool[] | yes | May be empty |
| `environments` | string[] | yes | Environment scope labels |
| `autonomy` | `"supervised" \| "semi-autonomous" \| "autonomous"` | yes | Overview + list column |
| `lastActivity` | string | yes | Humanized recent activity summary |
| `recentOperationIds` | string[] | yes | Operation/run ids for hand-off |
| `scope` | string | yes | Scope label (Production, Read-only, …) |
| `last` | string | yes | List-friendly relative time |
| `description` | string | no | Overview description |
| `createdAt` | string | no | Humanized create date |
| `activity` | AgentActivityEvent[] | no | Recent activity collection |
| `notes` | string | no | Provenance / operator notes |

**Validation (mock stage)**:
- `id`, `name` non-empty for list rows
- Deployment primary fixture (`a1`) MUST have non-empty `capabilities`, `tools`, `environments`, at least one `recentOperationIds` entry, and at least one related session in `AGENT_SESSION_MOCKS`
- Status/autonomy MUST be restricted enums above

**State (agent status)**:

| status | UI label | StatusChip tone |
| --- | --- | --- |
| `Active` | Active | healthy |
| `Paused` | Paused | warn |
| `Revoked` | Revoked | danger |
| `Disabled` | Disabled | muted |

**Autonomy labels**:

| autonomy | UI label |
| --- | --- |
| `supervised` | Supervised |
| `semi-autonomous` | Semi-autonomous |
| `autonomous` | Autonomous |

---

### AgentCapability

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Stable row id |
| `name` | string | yes | Capability key/display (e.g. deploy) |
| `effect` | `"allow" \| "request-only" \| "deny"` | yes | StatusChip; display only |

---

### AgentTool

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Stable row id |
| `name` | string | yes | Tool surface name (e.g. operations.run) |
| `kind` | `"mcp" \| "builtin" \| "integration"` | yes | Kind chip/label |

---

### AgentActivityEvent

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Stable row id |
| `title` | string | yes | Event title |
| `at` | string | yes | Humanized when |
| `result` | `"Success" \| "Pending" \| "Failed" \| "Denied"` | no | Optional StatusChip |

---

### Agent Session (reference only)

Owned by shell `WorkspaceAgentSession` / `AGENT_SESSION_MOCKS`. Agent detail **reads** by `agentId` for hand-off; does not redefine session schema.

| Field used by agent detail | Notes |
| --- | --- |
| `id` | Activation target |
| `agentId` | Filter key |
| `title` | PrimaryCell title |
| `status` | StatusChip |
| `environment` | Column |
| `startedAt` | Column |

---

## Relationships

```text
AGENT_MOCKS[a1]
  ├── recentOperationIds → ["run-1"]  (Operations hand-off)
  └── sessions (filter AGENT_SESSION_MOCKS by agentId=a1) → sess-184 …

Access activation:
  scopeId: access:agents
  entityKind: agent | agent-session
```

---

## Fixture completeness gate (primary)

Deployment Agent `a1`:

- capabilities.length > 0
- tools.length > 0
- environments.length > 0
- recentOperationIds includes a non-empty id
- AGENT_SESSION_MOCKS has ≥1 session with agentId `a1`
- optional but preferred: description, activity[], notes
