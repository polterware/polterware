import { createEffect, createSignal } from "solid-js";
import { A11yDialog, Button, Field, Input, Textarea } from "@polterware/frontend-design-system";
import type { OperationsWorkflowTemplate } from "./create-templates";
import { OperationsWorkflowTemplateGrid } from "./operations-template-grid";

export type CreateWorkflowDraft = {
  name: string;
  description: string;
};

export function CreateWorkflowDialog(props: {
  open: boolean;
  onClose: () => void;
  onSubmit: (draft: CreateWorkflowDraft) => void;
  onCreateFromTemplate?: (template: OperationsWorkflowTemplate) => void;
}) {
  const [name, setName] = createSignal("");
  const [description, setDescription] = createSignal("");
  const canSubmit = () => name().trim().length > 0;

  createEffect(() => {
    if (props.open) return;
    setName("");
    setDescription("");
  });

  function close() {
    props.onClose();
  }

  function submit() {
    const next = name().trim();
    if (!next) return;
    props.onSubmit({
      name: next,
      description: description().trim(),
    });
  }

  return (
    <A11yDialog
      open={props.open}
      title="Add workflow"
      description="Pick a template or create a blank workflow stored in the local SQLite database."
      size="xl"
      onClose={close}
      footer={
        <>
          <Button size="sm" variant="ghost" onClick={close}>
            Cancel
          </Button>
          <Button size="sm" variant="primary" disabled={!canSubmit()} onClick={submit}>
            Create blank
          </Button>
        </>
      }
    >
      <div class="grid gap-4">
        {props.onCreateFromTemplate ? (
          <OperationsWorkflowTemplateGrid
            onSelect={(template) => props.onCreateFromTemplate?.(template)}
          />
        ) : null}
        <Field label="Name">
          <Input
            size="sm"
            value={name()}
            placeholder="Workflow name"
            onInput={(event) => setName(event.currentTarget.value)}
          />
        </Field>
        <Field label="Description">
          <Textarea
            value={description()}
            placeholder="Optional description"
            onInput={(event) => setDescription(event.currentTarget.value)}
          />
        </Field>
      </div>
    </A11yDialog>
  );
}
