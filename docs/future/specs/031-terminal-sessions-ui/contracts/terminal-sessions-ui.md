# UI Contract: Terminal Sessions (Mock)

**Feature**: `031-terminal-sessions-ui`  
**Surfaces**: Terminal page, Terminal side panel, Target picker  
**Honesty**: Fixture/mock only — never claim live SSH/PTY host connection

## Session model

| Field | Required | Notes |
| --- | --- | --- |
| `id` | yes | Session id; display mono |
| `correlationId` | yes | Audit correlation; display mono |
| `status` | yes | `active` \| `idle` \| `closed` → StatusChip |
| `nodeId` / `nodeName` | yes | Target reference + display name |
| `shell` | yes | e.g. bash, zsh |
| `startedAt` | yes | ISO or display timestamp |
| `lines` | yes | Mock output lines |
| `dataSource` | recommended | `fixture` \| `mock` |

### Session StatusChip mapping

| status | label | tone |
| --- | --- | --- |
| active | Active | healthy |
| idle | Idle | warn |
| closed | Closed | muted |

## Target usability

| status | usable for new session |
| --- | --- |
| online, degraded | yes |
| offline, revoked, registering, draining, upgrading, unknown | no — feedback required |

Target picker rows use **StatusChip** with `terminalTargetStatusTone` / `terminalTargetStatusLabel`.

## Empty states

- **No sessions**: title + guidance to open a target / add session
- **No targets**: picker and page copy state that no targets are available
- **No match (search)**: “No targets match”

## Forbidden claims

- “Connected to real host”
- Live SSH/PTY success without real runtime feature
- Silent open of unusable targets

## Allowed actions (mock)

- Create mock session (usable target only)
- Switch session
- Close mock session
- Expand side panel to full Terminal page (navigation only)
