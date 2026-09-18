import { createEffect, createSignal } from "solid-js";
import { A11yDialog, Button, Field, Input } from "@polterware/frontend-design-system";

export type ProcessDraft = {
  name: string;
  command: string;
  cwd: string;
};

export function emptyProcessDraft(): ProcessDraft {
  return { name: "", command: "", cwd: "" };
}

export function CreateProcessDialog(props: {
  open: boolean;
  onClose: () => void;
  onSubmit?: (draft: ProcessDraft) => void;
}) {
  const [name, setName] = createSignal("");
  const [command, setCommand] = createSignal("");
  const [cwd, setCwd] = createSignal("");

  createEffect(() => {
    if (props.open) return;
    setName("");
    setCommand("");
    setCwd("");
  });

  return (
    <A11yDialog
      open={props.open}
      title="Start process"
      size="xl"
      onClose={() => props.onClose()}
      footer={(
        <>
          <Button variant="ghost" size="sm" onClick={() => props.onClose()}>Cancel</Button>
          <Button
            variant="primary"
            size="sm"
            disabled={!command().trim()}
            onClick={() =>
              props.onSubmit?.({
                name: name().trim(),
                command: command().trim(),
                cwd: cwd().trim(),
              })
            }
          >
            Start
          </Button>
        </>
      )}
    >
      <div class="grid gap-3">
        <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
          Runs on the local machine through polterware-core. Remote nodes are not supported.
        </p>
        <Field label="Name (optional)">
          <Input size="sm" value={name()} onInput={(event) => setName(event.currentTarget.value)} placeholder="Display name" />
        </Field>
        <Field label="Command">
          <Input
            size="sm"
            class="font-mono"
            value={command()}
            onInput={(event) => setCommand(event.currentTarget.value)}
            placeholder="sleep 60"
          />
        </Field>
        <Field label="Working directory (optional)">
          <Input
            size="sm"
            class="font-mono"
            value={cwd()}
            onInput={(event) => setCwd(event.currentTarget.value)}
            placeholder="/path/to/cwd"
          />
        </Field>
      </div>
    </A11yDialog>
  );
}
