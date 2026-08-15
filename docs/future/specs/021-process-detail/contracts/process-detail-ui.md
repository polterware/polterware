# Contract: Process Detail UI

**Feature**: `021-process-detail`  
**Surface**: Resources → Processes → global DetailPanel  
**Stage**: Mock / fixture (production-ready composition)

## Host

- Collection: Resources Processes DataTable
- Detail host: MainSurface global DetailPanel only
- Scope id pattern: `resources:processes` (or current Resources processes scope)
- Entity kind: `process`

## Composition

- `DetailPanel.Content`
- Header: domain icon + process name + `DetailPanel.Action`s
- Body: `layout="sections"` + `Grid` / `Section` / `Metric` / `Metadata`
- Status: `StatusChip` only
- Metrics visualization: optional `DitherSegmentedBar` when CPU/memory percent known — never invent history

## Header actions (mock or hand-off)

| Action | Tone | Mock-stage behavior |
| --- | --- | --- |
| View logs | default | toast or existing logs hand-off |
| Open terminal | default | toast or terminal hand-off for host node |
| Restart | default | mock toast |
| Stop | danger | mock toast |

## Required sections

1. Overview metrics — status, PID, node, CPU, memory, started
2. Runtime metadata — command, user, port, data source
3. Host context — node name/id; optional open node
4. Optional related — component / resource / operation when fixture-backed

## Honesty

- dataSource fixture | future core
- not-found / loading / sparse fields required
- no claim of live process control or log stream authority
