# Contract: Workflow Builder UI (Definition Depth)

**Owner**: `packages/features/operations`  
**Aligns**: `docs/spec.md` §25 workflows, §114 Workflows  
**Host**: Global `DetailPanel` via `DetailPanel.Content`

## View model (excerpt)

```ts
type WorkflowStepType =
  | "validate"
  | "approval"
  | "build"
  | "deploy"
  | "health_check"
  | "command"
  | "notify";

type WorkflowFailureStrategy = "stop" | "continue" | "fail_pipeline";

type WorkflowRetryPolicy = {
  maxAttempts: number;
  backoff: "none" | "fixed" | "exponential";
  delaySeconds?: number;
  respectsPolicy: true;
};

type WorkflowApprovalSemantics = {
  condition: string;
  reviewers: string;
  expiration: string;
  decisionIsNotExecution: true;
  separationOfDuties?: boolean;
};

type WorkflowStep = {
  id: string;
  label: string;
  type: WorkflowStepType;
  config?: Record<string, string>;
  condition?: string;
  retryPolicy?: WorkflowRetryPolicy;
  failureStrategy?: WorkflowFailureStrategy;
  approval?: WorkflowApprovalSemantics;
};

type WorkflowDef = {
  id: string;
  name: string;
  description?: string;
  trigger: string;
  lastRun: string;
  status: string;
  version: number;
  steps?: WorkflowStep[];
  failureStrategy?: WorkflowFailureStrategy;
  inputSummary?: string[];
  outputSummary?: string[];
  artifactsSummary?: string[];
  notes?: string;
  dataSource: "core" | "fixture";
};

type WorkflowRecentRunRef = {
  id: string;
  title: string;
  status: string;
  startedAt: string;
};
```

## Composition API

```ts
type WorkflowBuilderProps = {
  workflow?: WorkflowDef | null;
  mode?: "create" | "edit";
  /** When true or workflow is null/undefined with notFound, show not-found chrome */
  notFound?: boolean;
  recentRuns?: WorkflowRecentRunRef[];
  onSave?: (workflow: WorkflowDef) => void;
  onOpenRun?: (runId: string) => void;
};
```

## UI contract

1. **Header**: domain Workflow icon, title (Create/Edit + name), actions Run test + Save via `DetailPanel.Action`.
2. **Status**: `StatusChip` for definition status (workflow status helpers).
3. **Version**: visible (“Version N”); Save increments for edit mode.
4. **Pipeline column**: Trigger → insert controls → ordered steps → End; empty state when no steps.
5. **Config column**:
   - Trigger: name, description, trigger select, failure strategy, input/output summaries (when present/editable).
   - Step: type-specific fields (≥3 distinct type panels); optional retry block for executable types; approval semantics for approval.
6. **Recent runs**: list or buttons calling `onOpenRun` — no embedded RunDetail.
7. **Not found**: honest message when `notFound` or missing workflow in edit context.
8. **Honesty**: fixture toasts; no live engine claims.

## Non-goals

- Freeform canvas
- Rewrite of RunDetail / ScheduleDetail
- Actions column on workflows DataTable
