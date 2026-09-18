import { createEffect, createSignal } from "solid-js";
import { A11yDialog, Button, Field, Input, Select } from "@polterware/frontend-design-system";

export type HostNodeDraft = {
  name: string;
  platform: string;
  shell: string;
};

export function emptyHostNodeDraft(): HostNodeDraft {
  return {
    name: "",
    platform: "local",
    shell: "bash",
  };
}

const PLATFORM_OPTIONS = [
  { value: "local", label: "local" },
  { value: "darwin", label: "darwin" },
  { value: "linux", label: "linux" },
  { value: "windows", label: "windows" },
];

const SHELL_OPTIONS = [
  { value: "bash", label: "bash" },
  { value: "zsh", label: "zsh" },
  { value: "sh", label: "sh" },
  { value: "powershell", label: "powershell" },
];

export function CreateHostNodeDialog(props: {
  open: boolean;
  onClose: () => void;
  onSubmit?: (draft: HostNodeDraft) => void;
}) {
  const [name, setName] = createSignal("");
  const [platform, setPlatform] = createSignal("local");
  const [shell, setShell] = createSignal("bash");

  createEffect(() => {
    if (props.open) return;
    setName("");
    setPlatform("local");
    setShell("bash");
  });

  return (
    <A11yDialog
      open={props.open}
      title="Add node"
      size="xl"
      onClose={() => props.onClose()}
      footer={(
        <>
          <Button variant="ghost" size="sm" onClick={() => props.onClose()}>Cancel</Button>
          <Button
            variant="primary"
            size="sm"
            disabled={!name().trim()}
            onClick={() =>
              props.onSubmit?.({
                name: name().trim(),
                platform: platform(),
                shell: shell(),
              })
            }
          >
            Save
          </Button>
        </>
      )}
    >
      <div class="grid gap-3">
        <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
          This host node is stored in the local SQLite database and becomes a Terminal target.
        </p>
        <Field label="Name">
          <Input size="sm" value={name()} onInput={(event) => setName(event.currentTarget.value)} />
        </Field>
        <Field label="Platform">
          <Select size="sm" value={platform()} options={PLATFORM_OPTIONS} onChange={setPlatform} />
        </Field>
        <Field label="Shell">
          <Select size="sm" value={shell()} options={SHELL_OPTIONS} onChange={setShell} />
        </Field>
        <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
          Created in the currently selected environment from the header switcher.
        </p>
      </div>
    </A11yDialog>
  );
}
