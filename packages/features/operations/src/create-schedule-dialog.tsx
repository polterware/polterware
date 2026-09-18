import { Show, createEffect, createMemo, createSignal } from "solid-js";
import { A11yDialog, Button, Field, Input, Select } from "@polterware/frontend-design-system";

export type CreateScheduleDraft = {
  name: string;
  workflowId: string;
  expression: string;
  timezone: string;
};

export type CreateScheduleWorkflowOption = {
  id: string;
  name: string;
};

const TIMEZONE_OPTIONS = [
  { value: "UTC", label: "UTC" },
  { value: "America/Sao_Paulo", label: "America/Sao_Paulo" },
  { value: "America/New_York", label: "America/New_York" },
  { value: "Europe/London", label: "Europe/London" },
] as const;

export function CreateScheduleDialog(props: {
  open: boolean;
  workflows: CreateScheduleWorkflowOption[];
  onClose: () => void;
  onSubmit: (draft: CreateScheduleDraft) => void;
}) {
  const [name, setName] = createSignal("");
  const [workflowId, setWorkflowId] = createSignal("");
  const [expression, setExpression] = createSignal("0 * * * *");
  const [timezone, setTimezone] = createSignal("UTC");

  const workflowOptions = createMemo(() =>
    props.workflows.map((workflow) => ({ value: workflow.id, label: workflow.name })),
  );
  const hasWorkflows = () => props.workflows.length > 0;
  const canSubmit = () =>
    hasWorkflows()
    && name().trim().length > 0
    && workflowId().trim().length > 0
    && expression().trim().length > 0;

  createEffect(() => {
    if (props.open) return;
    setName("");
    setWorkflowId("");
    setExpression("0 * * * *");
    setTimezone("UTC");
  });

  createEffect(() => {
    if (!props.open) return;
    const options = props.workflows;
    if (options.length === 0) {
      setWorkflowId("");
      return;
    }
    if (!options.some((workflow) => workflow.id === workflowId())) {
      setWorkflowId(options[0]!.id);
    }
  });

  function close() {
    props.onClose();
  }

  function submit() {
    if (!canSubmit()) return;
    props.onSubmit({
      name: name().trim(),
      workflowId: workflowId(),
      expression: expression().trim(),
      timezone: timezone(),
    });
  }

  return (
    <A11yDialog
      open={props.open}
      title="Add schedule"
      description="This schedule is stored locally and runs its workflow when due while the Desktop core is open. Missed times while offline are skipped."
      onClose={close}
      footer={
        <>
          <Button size="sm" variant="ghost" onClick={close}>
            Cancel
          </Button>
          <Button size="sm" variant="primary" disabled={!canSubmit()} onClick={submit}>
            Create
          </Button>
        </>
      }
    >
      <div class="grid gap-3">
        <Show
          when={hasWorkflows()}
          fallback={
            <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
              Create a workflow first, then attach a schedule to it.
            </p>
          }
        >
          <Field label="Name">
            <Input
              size="sm"
              value={name()}
              placeholder="Schedule name"
              onInput={(event) => setName(event.currentTarget.value)}
            />
          </Field>
          <Field label="Workflow">
            <Select
              size="sm"
              value={workflowId()}
              options={workflowOptions()}
              onChange={setWorkflowId}
            />
          </Field>
          <Field label="Expression">
            <Input
              size="sm"
              class="font-mono"
              value={expression()}
              placeholder="0 * * * *"
              onInput={(event) => setExpression(event.currentTarget.value)}
            />
          </Field>
          <Field label="Timezone">
            <Select
              size="sm"
              value={timezone()}
              options={[...TIMEZONE_OPTIONS]}
              onChange={setTimezone}
            />
          </Field>
          <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
            Created in the currently selected environment from the header switcher.
          </p>
        </Show>
      </div>
    </A11yDialog>
  );
}
