# Contract: Overview UI (Mock)

**Feature**: `032-overview-operational`  
**Surface**: Project Overview command center (shell feature)  
**Authority**: Fixture / empty / offline honesty — not live telemetry

## Props (OverviewPage)

| Prop | Role |
| --- | --- |
| `summary?` | OverviewSummary snapshot; default `buildOverviewSummary()` |
| `activity?` | ActivityEvent[] sample; default ACTIVITY_MOCKS |
| `approvals?` | WorkspaceApproval[]; pending filter client-side |
| `runs?` | WorkspaceRun[]; in-progress filter client-side |
| `integrations?` | WorkspaceIntegration[]; health aggregate client-side |
| `dataSource?` | `fixture` \| `empty` \| `offline` (default `fixture`) |
| `onQuickAction?` | `(id) => void` — architecture, resources, operations, inbox, terminal, components, nodes, activity, … |
| `onOpenApproval?` | `(id) => void` |
| `onOpenRun?` | `(id) => void` |
| `onOpenActivity?` | `(id?) => void` |

## Navigation ids (must remain stable)

| Control | Quick action / callback |
| --- | --- |
| Shortcut Architecture | `architecture` |
| Shortcut Resources | `resources` |
| Shortcut Operations | `operations` |
| Shortcut Inbox | `inbox` (App routes to inbox:pending) |
| Shortcut Terminal | `terminal` |
| Metric Components | `components` |
| Metric Nodes | `nodes` |
| Metric Attention | `inbox` |
| Metric Resources | `resources` |
| View inbox | `inbox` |
| View activity | `onOpenActivity()` / `activity` |
| Open approval row | `onOpenApproval(id)` |
| Open run row | `onOpenRun(id)` |

## Status grammar

- System health: StatusChip from `healthTone` / `healthLabel`
- Integration aggregate status: StatusChip
- Nodes health surface: StatusChip for overall availability tone
- Run labeled status in operations list: StatusChip
- Approval risk in attention list: StatusChip (existing)
- Health event severity: StatusChip preferred over StatusDot-only for labeled status

## Honesty

| dataSource | Operator cue |
| --- | --- |
| `fixture` | Fixture/demo banner or label — not live telemetry |
| `empty` | Empty snapshot honesty; zero/empty sections |
| `offline` | Offline honesty; no live host claim |

## Non-claims

- No live metrics stream
- No mandatory Summary/Activity tabs on Overview
- No Activity page rewrite
