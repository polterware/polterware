# Contract: Operations Run Detail UI

**Owner**: `packages/features/operations`  
**Extends**: 004 Operations runs list; aligns with `docs/spec.md` §24 Operations, §113 Operations invariants  
**Host**: Global `DetailPanel` via `DetailPanel.Content` (MainSurface)

## View model

```ts
type RunStatus =
  | "pending"
  | "waiting_approval"
  | "queued"
  | "running"
  | "success"
  | "failed"
  | "cancelled"
  | "denied"
  | "lost";

type RunActorKind = "human" | "agent" | "system";

type RunExecutionLocation = "local_core" | "node" | "provider";

type RunActionKind =
  | "command"
  | "deploy"
  | "provision"
  | "workflow"
  | "sync"
  | "agent"
  | "health_check"
  | "other";

type RunStepStatus = "waiting" | "running" | "success" | "failed" | "skipped";

type RunStep = {
  id: string;
  name: string;
  status: RunStepStatus | string;
  duration?: string;
};

type RunResult = {
  outcome: "succeeded" | "failed" | "cancelled" | "denied" | "lost" | "partial" | "pending";
  message: string;
  rollbackSupported?: boolean;
  rollbackAvailable?: boolean;
};

type RunTimelineEvent = {
  id: string;
  title: string;
  at: string;
};

type OperationRun = {
  id: string;
  title: string;
  target: string;
  actor: string;
  actorKind?: RunActorKind;
  status: RunStatus;
  startedAt: string;
  finishedAt?: string;
  duration?: string;
  environment?: string;
  actionKind?: RunActionKind;
  correlationId?: string;
  requestId?: string;
  executionLocation?: RunExecutionLocation;
  executionLocationLabel?: string;
  approvalId?: string;
  approvalStatus?: string;
  risk?: "Low" | "Medium" | "High";
  policyChecks?: Array<{ label: string; ok: boolean }>;
  workflowId?: string;
  workflowName?: string;
  workflowVersion?: number;
  steps?: RunStep[];
  outputLines?: string[];
  outputTruncated?: boolean;
  result?: RunResult;
  timeline?: RunTimelineEvent[];
  notes?: string;
  dataSource: "core" | "fixture";
};
```

## Composition API

```ts
type RunDetailProps = {
  run: OperationRun | null | undefined;
  loading?: boolean;
  error?: string | null;
  notFound?: boolean;
  onCancel?: (runId: string) => void;
  onRetry?: (runId: string) => void;
};
```

## Layout (mandatory)

1. `DetailPanel.Content` → `Header` (icon + title + actions) → `Body layout="sections"` → `Grid`.
2. Metrics row: Status · Duration · Steps (with progress visualization when steps exist) · Result outcome when terminal.
3. Sections:
   - **Summary** — actor, actor kind, started, finished, duration
   - **Identity** — correlation ID (mono), request/operation id (mono), data source
   - **Context** — action kind, target, environment, execution location
   - **Governance** — approval, risk, policy checks (when any present)
   - **Workflow** — name/version when linked
   - **Steps** — `DataTable` (Step · Status · Duration); no Actions column
   - **Output** — mono stream; explicit truncation note; empty honest copy
   - **Result** — outcome message; rollback only when supported/available
   - **Timeline** — optional event table when present
4. Header actions:
   - **Cancel run** when status is non-terminal (`pending` | `waiting_approval` | `queued` | `running`)
   - **Retry** when `failed` (fixture-honest toast until core)
5. Status everywhere uses `StatusChip` filled tones.
6. List PrimaryCell icon for runs remains the execution domain icon; header reuses the same family.

## Honesty

- `dataSource: "fixture"` never presented as live stream.
- Cancel/retry feedback must say fixture/session until core exists.
- `lost` / partial outcomes must not be painted as simple failed success paths.
- `APPROVED` approval status is not “already executed”.

## Non-goals

- Full approval decision workspace (Inbox)
- Live SSE/log tail from polter-core
- Separate full-page run editor
- Actions column on steps table
