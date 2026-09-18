import { Show, createEffect, createMemo, createSignal } from "solid-js";
import {
  Button,
  PageContent,
  PageHeader,
  StatusChip,
  pushToast,
} from "@polterware/frontend-design-system";
import {
  TerminalTranscript,
  type CliConnectionState,
  type WorkspaceCatalogProvider,
} from "@polterware/feature-shell";
import { cliStateLabel, cliStateTone, providerByName } from "./setup-wizard-helpers";

export function SetupWizard(props: {
  providerName?: string;
  providers?: WorkspaceCatalogProvider[];
  onBack?: () => void;
  onFinish?: (provider: WorkspaceCatalogProvider) => void;
}) {
  const provider = createMemo(() =>
    providerByName(props.providers ?? [], props.providerName) ?? (props.providers ?? [])[0],
  );
  const [state, setState] = createSignal<CliConnectionState>("ready");
  const [lines, setLines] = createSignal<string[]>([]);

  createEffect(() => {
    const current = provider();
    if (!current) return;
    setState(current.state);
    setLines([
      `Visual authentication preview for ${current.name}`,
      `CLI: ${current.binary}`,
      "No command, browser, PTY, or authentication request will be started.",
      ...current.authCommands.map((command) => `$ ${command}`),
    ]);
  });

  function startAuthentication() {
    const current = provider();
    if (!current) return;
    setState("authenticating");
    setLines([
      ...current.transcript.slice(0, -1),
      "Authentication is waiting in this visual preview. Nothing is running.",
    ]);
  }

  function completeAuthentication() {
    const current = provider();
    if (!current) return;
    setState("connected");
    setLines([...current.transcript, "Connection stored locally for this project."]);
    pushToast(`${current.name} connection saved.`, "success");
    props.onFinish?.(current);
  }

  function failAuthentication() {
    const current = provider();
    if (!current) return;
    setState("failed");
    setLines([...current.transcript.slice(0, -1), "Authentication failed (visual only)."]);
  }

  function markUnavailable() {
    const current = provider();
    if (!current) return;
    setState("cli-unavailable");
    setLines([
      `$ ${current.binary} --version`,
      `${current.binary}: command not found (visual only)`,
      "Install the CLI outside Polterware before connecting.",
    ]);
  }

  return (
    <Show when={provider()}>
      {(current) => (
    <div class="flex h-full min-h-0 flex-col">
      <PageHeader
        title={`Connect ${current().name}`}
        subtitle="Visual CLI authentication"
        actions={<Button size="sm" variant="ghost" onClick={() => props.onBack?.()}>Cancel</Button>}
      />
      <PageContent class="mx-auto flex w-full max-w-[900px] flex-1 flex-col gap-4 p-6">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p class="m-0 text-[13px] font-medium text-[var(--color-text-primary)]">{current().binary}</p>
            <p class="m-0 mt-1 text-[12px] text-[var(--color-text-muted)]">
              {current().authCommands.join(" · ")}
            </p>
          </div>
          <StatusChip tone={cliStateTone(state())} label={cliStateLabel(state())} />
        </div>

        <TerminalTranscript
          lines={lines()}
          label={`${current().name} authentication transcript`}
          class="min-h-[320px] flex-1 rounded-[var(--radius-card)]"
        />

        <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
          Demonstration only. Polterware never asks for or stores tokens in this flow.
        </p>

        <div class="flex flex-wrap justify-between gap-2">
          <div class="flex flex-wrap gap-2">
            <Button size="sm" variant="ghost" onClick={markUnavailable}>Preview unavailable CLI</Button>
            <Button size="sm" variant="ghost" onClick={failAuthentication}>Preview failure</Button>
          </div>
          <div class="flex flex-wrap gap-2">
            <Show when={state() !== "authenticating"}>
              <Button size="sm" variant="secondary" onClick={startAuthentication}>Start visual flow</Button>
            </Show>
            <Show when={state() === "authenticating"}>
              <Button size="sm" variant="primary" onClick={completeAuthentication}>Complete visual flow</Button>
            </Show>
          </div>
        </div>
      </PageContent>
    </div>
      )}
    </Show>
  );
}
