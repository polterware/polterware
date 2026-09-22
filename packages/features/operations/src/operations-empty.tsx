import { Button, PageEmpty } from "@polterware/frontend-design-system";
import { Show } from "solid-js";
import type {
  OperationsRunTemplate,
  OperationsScheduleTemplate,
  OperationsWorkflowTemplate,
} from "./create-templates";
import {
  OperationsRunTemplateGrid,
  OperationsScheduleTemplateGrid,
  OperationsWorkflowTemplateGrid,
} from "./operations-template-grid";

export function OperationsRunsEmpty(props: {
  onRunCommand?: () => void;
  onCreateFromTemplate?: (template: OperationsRunTemplate) => void;
}) {
  return (
    <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
      <Show
        when={props.onCreateFromTemplate}
        fallback={
          <PageEmpty
            title="No runs yet."
            action={
              props.onRunCommand ? (
                <Button size="sm" variant="primary" onClick={() => props.onRunCommand?.()}>
                  Preview command
                </Button>
              ) : undefined
            }
          />
        }
      >
        <OperationsRunTemplateGrid
          showLabel={false}
          onSelect={(template) => props.onCreateFromTemplate?.(template)}
        />
      </Show>
    </div>
  );
}

export function OperationsWorkflowsEmpty(props: {
  onCreateWorkflow?: () => void;
  onCreateFromTemplate?: (template: OperationsWorkflowTemplate) => void;
}) {
  return (
    <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
      <Show
        when={props.onCreateFromTemplate}
        fallback={
          <PageEmpty
            title="No workflows yet."
            action={
              props.onCreateWorkflow ? (
                <Button size="sm" variant="primary" onClick={() => props.onCreateWorkflow?.()}>
                  Add workflow
                </Button>
              ) : undefined
            }
          />
        }
      >
        <OperationsWorkflowTemplateGrid
          showLabel={false}
          onSelect={(template) => props.onCreateFromTemplate?.(template)}
        />
      </Show>
    </div>
  );
}

export function OperationsSchedulesEmpty(props: {
  onCreateSchedule?: () => void;
  onCreateFromTemplate?: (template: OperationsScheduleTemplate) => void;
  hasWorkflows?: boolean;
}) {
  return (
    <div class="flex h-full min-h-[200px] w-full items-center justify-center p-8">
      <Show
        when={props.hasWorkflows && props.onCreateFromTemplate}
        fallback={
          <PageEmpty
            title="No schedules yet."
            description={
              props.hasWorkflows
                ? undefined
                : "Create a workflow first, then attach a schedule to it."
            }
            action={
              props.hasWorkflows && props.onCreateSchedule ? (
                <Button size="sm" variant="primary" onClick={() => props.onCreateSchedule?.()}>
                  Add schedule
                </Button>
              ) : undefined
            }
          />
        }
      >
        <OperationsScheduleTemplateGrid
          showLabel={false}
          onSelect={(template) => props.onCreateFromTemplate?.(template)}
        />
      </Show>
    </div>
  );
}

export function operationsFilterEmpty(kind: "runs" | "workflows" | "schedules") {
  if (kind === "workflows") {
    return <PageEmpty title="No workflows match" description="Try another search." />;
  }
  if (kind === "schedules") {
    return <PageEmpty title="No schedules match" description="Try another search." />;
  }
  return <PageEmpty title="No runs match" description="Try another search or status filter." />;
}
