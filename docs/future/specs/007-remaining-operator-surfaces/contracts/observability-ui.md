# Contract: Observability UI

**Owner**: `packages/features/observability`  
**Consumers**: `apps/desktop` App Match when `section=observability`

## Section shell

```ts
type ObservabilityAppProps = {
  route: ClientRoute; // section observability
  onNavigate: (next: ClientRoute) => void;
  environmentId: string;
  timeRange: string; // e.g. 1h
  onTimeRange: (r: string) => void;
  dataSource: DataSource;
  externalDashboards?: { id: string; label: string; href: string }[];
};
```

Tabs: `overview` | `logs` | `metrics` | `traces` | `monitors` (in-page only).

## Overview

- Health cards: services summary, error rate, active alerts.  
- Service health table/list.  
- External dashboard buttons when integrations exist.  
- Empty: connect observability integration CTA.

## Logs

```ts
type LogsTabProps = {
  events: LogEventView[];
  live: boolean;
  onLive: (v: boolean) => void;
  query: string;
  serviceFilter?: string;
  levelFilter?: string;
  onOpenEvent: (id: string) => void;
  onExport?: () => void;
  onOpenProvider?: () => void;
};
```

Rules:

1. Virtualize long streams.  
2. Pause control required for live mode.  
3. Event detail = focused dialog (payload + links), not permanent inspector.  
4. Link to trace/resource when ids present.

## Metrics

- Service selector + time range.  
- Key charts (request rate, latency, error rate, CPU or equivalent).  
- Hand-off: “Open complete dashboard in {provider}”.

## Traces

- Table: trace id, entry service, duration, spans, status, started.  
- Filters: all | errors | slow | successful.  
- Detail page: waterfall + selected span panel + open resource / view logs.

## Monitors

- Table: name, target, status, last check, alerts.  
- Create monitor when permitted.  
- Detail: condition, current value + sparkline, related context links, mute/edit/resolve.

## Honesty & non-goals

- Never claim live metrics/traces when `dataSource` is fixture/unavailable.  
- Not a full APM product; advanced work is hand-off.
