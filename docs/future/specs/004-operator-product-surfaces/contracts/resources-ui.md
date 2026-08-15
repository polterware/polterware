# Contract: Resources UI

**Owner**: `packages/features/resources`  
**Consumers**: `App.tsx`  
**Extends**: `003` resources index/detail/dialog

## Tabs

`inventory` | `nodes` | `processes`

## Inventory

- Columns: name, type, environment, provider, status, last sync  
- Category filters: all | cloud | local | databases | queues | …  
- Row open → detail `entityKind: "resource"`

## Nodes

- Columns: name, location, OS, version, status, last heartbeat  
- Summary: online/offline/total  
- Row open → detail `entityKind: "node"` (capabilities, assigned resources, recent ops, terminal/run actions)

## Processes

- Columns: process, PID, node, CPU, memory, port, status  
- Live updates with pause/resume  
- Row open → **drawer only** (`drawer: { type: "process", processId }`)  
- MUST NOT navigate to full-page process detail

## Resource detail (page)

Overview, linked architecture, notes, activity, quick actions (provider hand-off preferred).

## Dialogs

- resource-create / resource-edit (003)  
- Add node may be dialog or hand-off depending on core support—honest if unavailable
