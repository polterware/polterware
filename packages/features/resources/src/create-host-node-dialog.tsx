import { createEffect, createSignal } from "solid-js";
import { A11yDialog, Button, Field, Input } from "@polterware/frontend-design-system";
import type { ResourceNodeTemplate } from "./create-templates";
import { ResourceNodeTemplateGrid } from "./resources-template-grid";

export type HostNodeDraft = {
  name: string;
  connectionKind: "ssh";
  address: string;
  sshUser: string;
  sshPort: number;
  sshIdentityFile: string;
  platform?: string;
  shell?: string;
};

export function emptyHostNodeDraft(): HostNodeDraft {
  return {
    name: "",
    connectionKind: "ssh",
    address: "",
    sshUser: "",
    sshPort: 22,
    sshIdentityFile: "",
    platform: "linux",
    shell: "bash",
  };
}

function applyDraft(
  draft: HostNodeDraft,
  setters: {
    setName: (value: string) => void;
    setAddress: (value: string) => void;
    setSshUser: (value: string) => void;
    setSshPort: (value: string) => void;
    setSshIdentityFile: (value: string) => void;
  },
) {
  setters.setName(draft.name);
  setters.setAddress(draft.address);
  setters.setSshUser(draft.sshUser);
  setters.setSshPort(String(draft.sshPort || 22));
  setters.setSshIdentityFile(draft.sshIdentityFile);
}

export function CreateHostNodeDialog(props: {
  open: boolean;
  onClose: () => void;
  onSubmit?: (draft: HostNodeDraft) => void;
  onCreateFromTemplate?: (template: ResourceNodeTemplate) => void;
  initialDraft?: HostNodeDraft | null;
}) {
  const [name, setName] = createSignal("");
  const [address, setAddress] = createSignal("");
  const [sshUser, setSshUser] = createSignal("");
  const [sshPort, setSshPort] = createSignal("22");
  const [sshIdentityFile, setSshIdentityFile] = createSignal("");

  createEffect(() => {
    if (!props.open) {
      setName("");
      setAddress("");
      setSshUser("");
      setSshPort("22");
      setSshIdentityFile("");
      return;
    }
    if (props.initialDraft) {
      applyDraft(props.initialDraft, { setName, setAddress, setSshUser, setSshPort, setSshIdentityFile });
    }
  });

  const portNumber = () => {
    const parsed = Number.parseInt(sshPort().trim(), 10);
    return Number.isFinite(parsed) ? parsed : 0;
  };

  const canSave = () =>
    Boolean(name().trim() && address().trim() && sshUser().trim() && portNumber() >= 1 && portNumber() <= 65535);

  return (
    <A11yDialog
      open={props.open}
      title="Add SSH host"
      size="xl"
      onClose={() => props.onClose()}
      footer={(
        <>
          <Button variant="ghost" size="sm" onClick={() => props.onClose()}>Cancel</Button>
          <Button
            variant="primary"
            size="sm"
            disabled={!canSave()}
            onClick={() =>
              props.onSubmit?.({
                name: name().trim(),
                connectionKind: "ssh",
                address: address().trim(),
                sshUser: sshUser().trim(),
                sshPort: portNumber(),
                sshIdentityFile: sshIdentityFile().trim(),
                platform: "linux",
                shell: "bash",
              })
            }
          >
            Save host
          </Button>
        </>
      )}
    >
      <div class="grid gap-4">
        <ResourceNodeTemplateGrid
          onSelect={(template) => {
            applyDraft(template.draft, { setName, setAddress, setSshUser, setSshPort, setSshIdentityFile });
            props.onCreateFromTemplate?.(template);
          }}
        />
        <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
          Saves an SSH connection profile for Terminal. Private keys stay in the OS or ssh-agent; only an optional key path is stored.
        </p>
        <Field label="Alias">
          <Input size="sm" value={name()} onInput={(event) => setName(event.currentTarget.value)} />
        </Field>
        <Field label="User">
          <Input size="sm" value={sshUser()} onInput={(event) => setSshUser(event.currentTarget.value)} />
        </Field>
        <Field label="Host">
          <Input
            size="sm"
            value={address()}
            placeholder="hostname or IP"
            onInput={(event) => setAddress(event.currentTarget.value)}
          />
        </Field>
        <Field label="Port">
          <Input size="sm" value={sshPort()} onInput={(event) => setSshPort(event.currentTarget.value)} />
        </Field>
        <Field label="Identity file path (optional)">
          <Input
            size="sm"
            value={sshIdentityFile()}
            placeholder="~/.ssh/id_ed25519"
            onInput={(event) => setSshIdentityFile(event.currentTarget.value)}
          />
        </Field>
        <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
          Created in the currently selected environment from the header switcher. Becomes a Terminal target.
        </p>
      </div>
    </A11yDialog>
  );
}
