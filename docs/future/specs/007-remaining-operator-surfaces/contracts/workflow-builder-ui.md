# Contract: Workflow Builder & Schedules UI

**Owner**: `packages/features/operations` (+ workflows helpers)  
**Extends**: 004 Operations workflows list/detail

## Builder

```ts
type WorkflowStepType =
  | "validate"
  | "approval"
  | "build"
  | "deploy"
  | "health_check"
  | "command"
  | "notify";

type WorkflowStep = {
  id: string;
  label: string;
  type: WorkflowStepType;
  config?: Record<string, string>;
};

type WorkflowBuilderProps = {
  workflow?: {
    id: string;
    name: string;
    trigger: string;
    steps?: WorkflowStep[];
    version?: number;
    status?: string;
  };
  mode?: "create" | "edit";
  onSave?: (workflow: WorkflowDef) => void;
};
```

### Layout (sequential queue diagram)

1. **Two-region layout** inside global `DetailPanel` (not free Architecture canvas):
   - **Left — pipeline queue**: fixed vertical order (trigger → steps → end), auto-laid-out cards with CSS connectors and `+` insert controls between nodes.
   - **Right — configuration**: selected trigger or step form.
2. Step positions are derived from array order only (no free pan/zoom node graph).
3. Approval steps expose name, condition, reviewers, expiration.
4. Create opens a draft in the same builder (empty step queue); Save creates/updates version visibility and does not bypass policy for execution.
5. Test run and Save are header actions; fixture/session feedback is honest (toasts) until core persist exists.

## Schedules (secondary strip on Workflows)

```ts
type WorkflowSchedulesProps = {
  schedules: WorkflowSchedule[];
  selectedId?: string;
  onSelect: (id: string) => void;
  onCreate: () => void;
  onRunNow: (id: string) => void;
  onEdit: (id: string) => void;
  onPause: (id: string) => void;
};
```

Primary ops tabs remain runs | workflows | schedules. Schedules are not a sidebar item.
