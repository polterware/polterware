# Contract: Operations UI

**Owner**: `packages/features/operations` (may compose workflows/approvals/terminal packages)  
**Consumers**: `App.tsx`

## Tabs

`runs` | `workflows` | `approvals` | `terminal`

## Runs

- Columns: operation, target, actor, status, started  
- Filters: all | running | pending | failed | completed  
- Entry: Run command → dialog `run-command`  
- Detail: summary, context, steps, output, cancel/retry when allowed

## Workflows

- Columns: workflow, trigger, last run, status, actions  
- Filters: all | manual | scheduled | event-driven  
- Detail: step graph, recent runs, Run workflow action

## Approvals

- Subfilters: pending | approved | rejected | expired  
- Detail: impact, policy checks, comments, Approve/Reject  
- MUST block self-approve when policy requires separation of duties  
- MUST NOT apply operation without resolve path when approval required

## Terminal

- Multi-session tabs (node + shell)  
- Connection status, clear, close  
- Execution via core/process bridge only—not raw Node in renderer

## Mutation safety

All request paths that mutate production MUST surface approval requirement when policy demands it. UI labels demo/fixture runs when not live.
