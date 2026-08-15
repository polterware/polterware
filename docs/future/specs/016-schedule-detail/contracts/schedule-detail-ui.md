# Contract: Operations Schedule Detail UI

**Owner**: `packages/features/operations`  
**Extends**: 007 Operations Schedules list; aligns with `docs/spec.md` Operations (`operation_schedules`), Workflows (scheduled trigger), scheduled service identity invariants  
**Host**: Global `DetailPanel` via `DetailPanel.Content` (MainSurface)

## View model

```ts
type ScheduleStatus = "active" | "paused" | "disabled" | "error";

type ScheduleMissedRunPolicy = "skip" | "catch_up" | "alert" | "unknown";

type ScheduleServiceIdentity = {
  id: string;
  name: string;
  kind: "service" | "system" | "agent";
  capabilities?: string[];
};

type ScheduleRecentRun = {
  id: string;
  title: string;
  status: string;
  startedAt: string;
  finishedAt?: string;
  duration?: string;
  correlationId?: string;
};

type ScheduleTimelineEvent = {
  id: string;
  title: string;
  at: string;
};

type OperationSchedule = {
  id: string;
  name: string;
  description?: string;
  status: ScheduleStatus;
  /** Cron or product recurrence expression (e.g. "0 3 * * *"). */
  cron?: string;
  /** Human-readable cadence (e.g. "Daily at 03:00"). */
  recurrenceLabel?: string;
  timezone: string;
  nextRunAt?: string;
  /** Honest copy when next fire is not scheduled (paused/disabled). */
  nextRunLabel?: string;
  lastRunAt?: string;
  lastRunStatus?: string;
  lastRunOutcomeLabel?: string;
  /** Optional recent success ratio label, e.g. "4/5". */
  recentSuccessRate?: string;
  workflowId?: string;
  workflowName?: string;
  workflowVersion?: number;
  environment?: string;
  target?: string;
  executionLocation?: "local_core" | "node" | "provider";
  executionLocationLabel?: string;
  serviceIdentity?: ScheduleServiceIdentity;
  approvalRequiredOnFire?: boolean;
  risk?: "Low" | "Medium" | "High";
  policyNotes?: string[];
  missedRunPolicy?: ScheduleMissedRunPolicy;
  createdAt?: string;
  updatedAt?: string;
  recentRuns?: ScheduleRecentRun[];
  timeline?: ScheduleTimelineEvent[];
  notes?: string;
  dataSource: "core" | "fixture";
};
```

## Composition API

```ts
type ScheduleDetailProps = {
  schedule: OperationSchedule | null | undefined;
  loading?: boolean;
  error?: string | null;
  notFound?: boolean;
  onPause?: (scheduleId: string) => void;
  onResume?: (scheduleId: string) => void;
  onRunNow?: (scheduleId: string) => void;
};
```

## Layout (mandatory)

1. `DetailPanel.Content` → `Header` (icon + title + actions) → `Body layout="sections"` → `Grid`.
2. Metrics row: Status · Next run · Last outcome · Success rate (or Timezone when rate absent).
3. Sections:
   - **Cadence** — recurrence label, cron (mono), timezone, missed-run policy
   - **Identity** — schedule id (mono), created, updated, data source
   - **Context** — workflow, version, environment, target, execution location
   - **Service identity** — name, kind, capability chips/list (honest empty)
   - **Governance** — approval on fire, risk, policy notes (honest empty)
   - **Recent runs** — `DataTable` (Run · Status · Started); no Actions column
   - **Timeline** — optional lifecycle events when present
   - **Notes** — optional operator notes
4. Header actions:
   - **Pause schedule** when `status === "active"`
   - **Resume schedule** when `status === "paused"`
   - **Run now** when `status` is `active` or `paused` (fixture-honest toast)
5. Status everywhere uses `StatusChip` filled tones.
6. List PrimaryCell icon for schedules remains `CalendarClock` / execution domain; header reuses the same family.

## List wiring

- `scopeId`: `operations:schedules`
- `entityKind`: `schedule`
- Collection registration includes all visible schedule fixtures when Schedules tab is active.
- Status column uses `StatusChip` (not raw string).

## Honesty

- `dataSource: "fixture"` never presented as live scheduler.
- Pause/resume/run-now feedback must say fixture/session until core exists.
- Paused/disabled next-run copy must not imply an imminent automatic fire.
- Active status + last failed fire: both shown honestly.

## Non-goals

- Full schedule authoring form / cron builder
- Live tick engine or durable pause persistence
- Calendar heatmap product
- Separate full-page schedule editor
- Actions column on recent runs table
