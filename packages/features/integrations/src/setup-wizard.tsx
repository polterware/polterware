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

function bridge() {
  return (globalThis as unknown as {
    polterware: {
      core: {
        integrations: {
          probeProvider: (payload: Record<string, unknown>) => Promise<Record<string, unknown>>;
          beginAuth: (payload: Record<string, unknown>) => Promise<Record<string, unknown>>;
        };
      };
      app: {
        openExternal: (url: string) => Promise<unknown>;
      };
    };
  }).polterware;
}

function core() {
  return bridge().core;
}

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
  const [busy, setBusy] = createSignal(false);
  const [accountLabel, setAccountLabel] = createSignal("");

  async function probeCurrent() {
    const current = provider();
    if (!current) return;
    setBusy(true);
    try {
      const result = await core().integrations.probeProvider({ providerId: current.id });
      const status = String(result.status ?? "ready") as CliConnectionState;
      const message = String(result.message ?? "");
      const path = String(result.binaryPath ?? "");
      const label = String(result.accountLabel ?? "");
      setAccountLabel(label);
      setState(status === "cli-unavailable" || status === "connected" || status === "failed" || status === "ready" || status === "authenticating"
        ? status
        : "ready");
      setLines([
        `Probing ${current.name} CLI…`,
        path ? `Binary: ${path}` : `Binary: ${current.binary}`,
        message,
        label ? `Account: ${label}` : "No authenticated account label detected.",
        "Polterware never requests or stores provider tokens.",
      ]);
    } catch (error: unknown) {
      setState("failed");
      setLines([
        `Could not probe ${current.name}.`,
        error instanceof Error ? error.message : "Probe failed.",
      ]);
    } finally {
      setBusy(false);
    }
  }

  createEffect(() => {
    const current = provider();
    if (!current) return;
    void probeCurrent();
  });

  async function startAuthentication() {
    const current = provider();
    if (!current) return;
    setBusy(true);
    setState("authenticating");
    try {
      const plan = await core().integrations.beginAuth({ providerId: current.id });
      const command = String(plan.authCommand ?? current.authCommands[0] ?? current.binary);
      const docsUrl = String(plan.docsUrl ?? "");
      const message = String(plan.message ?? "");
      setLines([
        `Starting local authentication for ${current.name}`,
        `$ ${command}`,
        message,
        docsUrl ? `Docs: ${docsUrl}` : "Complete the CLI login outside Polterware, then refresh status.",
      ]);
      if (docsUrl) {
        await bridge().app.openExternal(docsUrl);
      }
      pushToast(`Complete ${current.name} login in the CLI, then refresh status.`, "default");
    } catch (error: unknown) {
      setState("failed");
      setLines([
        `Could not start authentication for ${current.name}.`,
        error instanceof Error ? error.message : "Begin auth failed.",
      ]);
    } finally {
      setBusy(false);
    }
  }

  async function refreshAndConnect() {
    const current = provider();
    if (!current) return;
    setBusy(true);
    try {
      const result = await core().integrations.probeProvider({ providerId: current.id });
      const status = String(result.status ?? "");
      const label = String(result.accountLabel ?? current.name);
      setAccountLabel(label);
      setLines([
        `Refreshed ${current.name} status`,
        String(result.message ?? ""),
        label ? `Account: ${label}` : "No account label yet.",
      ]);
      if (status === "cli-unavailable") {
        setState("cli-unavailable");
        pushToast(`${current.binary} is not available on PATH.`, "danger");
        return;
      }
      if (status !== "connected") {
        setState("ready");
        pushToast("CLI is available, but no authenticated session was detected yet.", "default");
        return;
      }
      setState("connected");
      pushToast(`${current.name} connection ready.`, "success");
      props.onFinish?.({
        ...current,
        state: "connected",
        name: label || current.name,
      });
    } catch (error: unknown) {
      setState("failed");
      pushToast(error instanceof Error ? error.message : "Could not refresh connection status.", "danger");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Show when={provider()}>
      {(current) => (
    <div class="flex h-full min-h-0 flex-col">
      <PageHeader
        title={`Connect ${current().name}`}
        subtitle="Local CLI authentication"
        actions={<Button size="sm" variant="ghost" onClick={() => props.onBack?.()}>Cancel</Button>}
      />
      <PageContent class="mx-auto flex w-full max-w-[900px] flex-1 flex-col gap-4 p-6">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p class="m-0 text-[13px] font-medium text-[var(--color-text-primary)]">{current().binary}</p>
            <p class="m-0 mt-1 text-[12px] text-[var(--color-text-muted)]">
              {current().authCommands.join(" · ")}
            </p>
            <Show when={accountLabel()}>
              <p class="m-0 mt-1 text-[12px] text-[var(--color-text-secondary)]">{accountLabel()}</p>
            </Show>
          </div>
          <StatusChip tone={cliStateTone(state())} label={cliStateLabel(state())} />
        </div>

        <TerminalTranscript
          lines={lines()}
          label={`${current().name} authentication transcript`}
          class="min-h-[320px] flex-1 rounded-[var(--radius-card)]"
        />

        <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
          Credentials stay in the OS/CLI credential store. Polterware never asks for or stores provider tokens.
        </p>

        <div class="flex flex-wrap justify-end gap-2">
          <Button size="sm" variant="ghost" disabled={busy()} onClick={() => void probeCurrent()}>
            Refresh status
          </Button>
          <Show when={state() !== "connected"}>
            <Button size="sm" variant="secondary" disabled={busy() || state() === "cli-unavailable"} onClick={() => void startAuthentication()}>
              Start CLI login
            </Button>
          </Show>
          <Button size="sm" variant="primary" disabled={busy()} onClick={() => void refreshAndConnect()}>
            {state() === "connected" ? "Save connection" : "Check and connect"}
          </Button>
        </div>
      </PageContent>
    </div>
      )}
    </Show>
  );
}
