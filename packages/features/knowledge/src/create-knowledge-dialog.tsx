import { createSignal } from "solid-js";
import { A11yDialog, Button, Field, Input } from "@polterware/frontend-design-system";

export function CreateKnowledgeDialog(props: {
  open: boolean;
  kind: "doc" | "runbook";
  onClose: () => void;
  onSubmit: (title: string) => void;
}) {
  const [title, setTitle] = createSignal("");
  const isRunbook = () => props.kind === "runbook";
  const canSubmit = () => title().trim().length > 0;

  function close() {
    setTitle("");
    props.onClose();
  }

  function submit() {
    const next = title().trim();
    if (!next) return;
    props.onSubmit(next);
    setTitle("");
  }

  return (
    <A11yDialog
      open={props.open}
      title={isRunbook() ? "New runbook" : "New document"}
      description={
        isRunbook()
          ? "The runbook is stored in the local SQLite database."
          : "The document is stored in the local SQLite database."
      }
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
      <Field label="Title">
        <Input
          size="sm"
          value={title()}
          placeholder={isRunbook() ? "Runbook title" : "Document title"}
          onInput={(event) => setTitle(event.currentTarget.value)}
        />
      </Field>
    </A11yDialog>
  );
}
