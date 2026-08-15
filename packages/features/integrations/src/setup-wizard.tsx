import { Show, createMemo, createSignal } from "solid-js";
import {
  Button,
  PageContent,
  PageHeader,
  StatusChip,
  pushToast,
} from "@polterware/frontend-design-system";
import {
  CATALOG_PROVIDER_MOCKS,
  TerminalTranscript,
  type CliConnectionState,
  type WorkspaceCatalogProvider,
} from "@polterware/feature-shell";
import { cliStateLabel, cliStateTone, providerByName } from "./setup-wizard-helpers";

export function SetupWizard(props: {
  providerName?: string;
  onBack?: () => void;
  onFinish?: (provider: WorkspaceCatalogProvider) => void;
}) {
  const provider = createMemo(() =>
    providerByName(CATALOG_PROVIDER_MOCKS, props.providerName) ?? CATALOG_PROVIDER_MOCKS[0]!,
  );
  const [state, setState] = createSignal<CliConnectionState>(provider().state);
  const [lines, setLines] = createSignal<string[]>([
    `Visual authentication preview for ${provider().name}`,
    `CLI: ${provider().binary}`,
    "No command, browser, PTY, or authentication request will be started.",
    ...provider().authCommands.map((command) => `$ ${command}`),
  ]);

  function startAuthentication() {
    setState("authenticating");
    setLines([
      ...provider().transcript.slice(0, -1),
      "Authentication is waiting in this visual preview. Nothing is running.",
    ]);
  }

  function completeAuthentication() {
    setState("connected");
    setLines([...provider().transcript, "Connection stored in memory for this session only."]);
    pushToast(`${provider().name} connected for this preview session.`, "success");
    props.onFinish?.(provider());
  }

  function failAuthentication() {
    setState("failed");
    setLines([...provider().transcript.slice(0, -1), "Authentication failed (mock)."]);
  }

  function markUnavailable() {
    setState("cli-unavailable");
    setLines([
      `$ ${provider().binary} --version`,
      `${provider().binary}: command not found (mock)`,
      "Install the CLI outside Polterware before connecting.",
    ]);
  }

  return (
    <div class="flex h-full min-h-0 flex-col">
      <PageHeader
        title={`Connect ${provider().name}`}
        subtitle="Visual CLI authentication"
        actions={<Button size="sm" variant="ghost" onClick={() => props.onBack?.()}>Cancel</Button>}
      />
      <PageContent class="mx-auto flex w-full max-w-[900px] flex-1 flex-col gap-4 p-6">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p class="m-0 text-[13px] font-medium text-[var(--color-text-primary)]">{provider().binary}</p>
            <p class="m-0 mt-1 text-[12px] text-[var(--color-text-muted)]">
              {provider().authCommands.join(" · ")}
            </p>
          </div>
          <StatusChip tone={cliStateTone(state())} label={cliStateLabel(state())} />
        </div>

        <TerminalTranscript
          lines={lines()}
          label={`${provider().name} authentication transcript`}
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
  );
}
