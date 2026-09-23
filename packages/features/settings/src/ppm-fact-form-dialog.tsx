import { createEffect, createSignal } from "solid-js";
import { A11yDialog, Button, Field, Input, Select, Textarea } from "@polterware/frontend-design-system";
import {
  PPM_TRUTH_CLASS_OPTIONS,
  type PpmFactDraft,
  type PpmFactTruthClass,
} from "./ppm-facts-types";

export function PpmFactFormDialog(props: {
  open: boolean;
  mode: "create" | "edit";
  initial?: PpmFactDraft;
  onClose: () => void;
  onSubmit: (draft: PpmFactDraft) => void;
}) {
  const [statement, setStatement] = createSignal("");
  const [truthClass, setTruthClass] = createSignal<PpmFactTruthClass>("declared");
  const [entityId, setEntityId] = createSignal("");

  createEffect(() => {
    if (!props.open) return;
    const initial = props.initial;
    setStatement(initial?.statement ?? "");
    setTruthClass(initial?.truthClass ?? "declared");
    setEntityId(initial?.entityId ?? "");
  });

  const canSubmit = () => statement().trim().length > 0;

  function submit() {
    const next = statement().trim();
    if (!next) return;
    props.onSubmit({
      id: props.initial?.id,
      statement: next,
      truthClass: truthClass(),
      entityId: entityId().trim(),
    });
  }

  return (
    <A11yDialog
      open={props.open}
      title={props.mode === "edit" ? "Edit fact" : "Add fact"}
      description="PPM facts are declared, observed, inferred, or verified statements about the project."
      size="md"
      onClose={() => props.onClose()}
      footer={
        <>
          <Button size="sm" variant="ghost" onClick={() => props.onClose()}>
            Cancel
          </Button>
          <Button size="sm" variant="primary" disabled={!canSubmit()} onClick={submit}>
            {props.mode === "edit" ? "Save" : "Create"}
          </Button>
        </>
      }
    >
      <div class="grid gap-3">
        <Field label="Statement">
          <Textarea
            rows={4}
            value={statement()}
            placeholder="The API depends on PostgreSQL"
            onInput={(event) => setStatement(event.currentTarget.value)}
          />
        </Field>
        <Field label="Truth class">
          <Select
            options={[...PPM_TRUTH_CLASS_OPTIONS]}
            value={truthClass()}
            onChange={(value) => setTruthClass(value as PpmFactTruthClass)}
          />
        </Field>
        <Field label="Entity id">
          <Input
            value={entityId()}
            placeholder="optional PPM entity id"
            onInput={(event) => setEntityId(event.currentTarget.value)}
          />
        </Field>
      </div>
    </A11yDialog>
  );
}
