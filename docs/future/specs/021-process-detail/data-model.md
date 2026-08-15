# Data Model: Process Detail

**Feature**: `021-process-detail`  
**Stage**: Mock fixtures in `@polterware/feature-resources`

## ProcessRow

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `id` | string | yes | Collection entity id |
| `name` | string | yes | Header title / PrimaryCell |
| `pid` | string \| number | yes | Display as mono when numeric/string |
| `nodeId` | string | yes | Host node registry id for hand-off |
| `nodeName` | string | yes | Host display name |
| `status` | `Running` \| `Stopped` \| `Unknown` | yes | StatusChip source |
| `dataSource` | `core` \| `fixture` | yes | Honesty label |
| `cpuPercent` | number | no | Bar only when present |
| `memoryMb` | number | no | Display as `N MB` |
| `port` | string \| number | no | Runtime mono |
| `command` | string | no | Runtime mono |
| `user` | string | no | Runtime |
| `startedAt` | string | no | Relative or absolute fixture string |
| `uptime` | string | no | Optional distinct uptime label |
| `relatedComponent` | `{ id, name }` | no | Optional related |
| `relatedResource` | `{ id, name }` | no | Optional related |
| `relatedOperation` | `{ id, name }` | no | Optional related |

## Fixture coverage requirements

- **Full**: ≥1 process with command, user, port, CPU, memory, startedAt, host node, optional related ref.
- **Sparse**: ≥1 process missing most optional fields (no command/port/CPU/memory invent).
- **Stopped**: ≥1 process with status Stopped (control actions gated).
- **Unknown**: ≥1 process with status Unknown (control actions gated).
- All fixtures `dataSource: "fixture"`.

## Status gates

| Status | Stop | Restart | Logs / Terminal |
| --- | --- | --- | --- |
| Running | show | show | show |
| Stopped | hide | hide | show (mock/hand-off) |
| Unknown | hide | hide | show (mock/hand-off) |
