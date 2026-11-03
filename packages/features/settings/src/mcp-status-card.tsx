import { For, Show } from "solid-js";
import { Bot, Button, Cpu, Database, Plug, RefreshCw, SourceCodeIcon, StatusChip, pushToast } from "@polterware/frontend-design-system";
import type { McpClientView, McpStatusView } from "./agents-types";
import { SettingsRow, SettingsSection } from "./settings-section";

async function copyText(label: string, text: string) {
  try {
    await navigator.clipboard.writeText(text);
    pushToast(`${label} copied.`, "success");
  } catch {
    pushToast(`Could not copy ${label.toLowerCase()}.`, "danger");
  }
}

function truncatePath(path: string, max = 56): string {
  if (!path || path.length <= max) return path || "—";
  return `…${path.slice(-(max - 1))}`;
}

const CLIENT_STATE: Record<McpClientView["status"], { tone: "ok" | "warn" | "muted"; label: string }> = {
  configured: { tone: "ok", label: "Set up" },
  configured_other: { tone: "warn", label: "Points elsewhere" },
  not_configured: { tone: "muted", label: "Not set up" },
  not_installed: { tone: "muted", label: "Not installed" },
  unknown: { tone: "muted", label: "Unknown" },
};

/** Local MCP helper: lifecycle, process metadata, the MCP clients this machine has, and
 * client config to copy. */
export function McpStatusCard(props: {
  status: McpStatusView | undefined;
  clients: McpClientView[] | undefined;
  busy: boolean;
  onStart: () => void;
  onStop: () => void;
  onRefresh: () => void;
  onSetClient: (id: string, enabled: boolean) => void;
}) {
  const status = () => props.status;
  const running = () => Boolean(status()?.running);

  return (
    <SettingsSection
      title="MCP server"
      description="A local helper that lets MCP clients read and write this project through the core."
      action={
        <Button size="icon" variant="ghost" aria-label="Refresh" disabled={props.busy} onClick={() => props.onRefresh()}>
          <RefreshCw size={14} aria-hidden />
        </Button>
      }
    >
      <SettingsRow icon={Plug} tone="integration" seed="mcp-helper" label="Local MCP helper">
        <Show when={status()} fallback={<StatusChip tone="muted" label="Checking" />}>
          <StatusChip tone={running() ? "ok" : "muted"} label={running() ? "Running" : "Stopped"} />
        </Show>
        <Show
          when={running()}
          fallback={
            <Button size="sm" variant="primary" disabled={props.busy || !status()} onClick={() => props.onStart()}>
              Start
            </Button>
          }
        >
          <Button size="sm" disabled={props.busy} onClick={() => props.onStop()}>
            Stop
          </Button>
        </Show>
      </SettingsRow>

      <SettingsRow icon={Cpu} seed="mcp-process" label="Process">
        <span class="font-mono pw-selectable text-[12px] text-[var(--color-text-secondary)]">
          {status()?.pid ? `PID ${status()?.pid}` : "—"}
        </span>
      </SettingsRow>

      <SettingsRow icon={Database} tone="data" seed="mcp-database" label="Database">
        <span
          class="max-w-[20rem] truncate font-mono pw-selectable text-[12px] text-[var(--color-text-secondary)]"
          title={status()?.dbPath}
        >
          {truncatePath(status()?.dbPath ?? "")}
        </span>
      </SettingsRow>

      <For each={props.clients ?? []}>
        {(client) => (
          <SettingsRow
            icon={Bot}
            tone="execution"
            seed={`mcp-client-${client.id}`}
            label={client.name}
            description={client.error || undefined}
          >
            <StatusChip tone={CLIENT_STATE[client.status].tone} label={CLIENT_STATE[client.status].label} />
            <Show when={client.status !== "not_installed"}>
              <Show
                when={client.status === "configured"}
                fallback={
                  <Button size="sm" disabled={props.busy} onClick={() => props.onSetClient(client.id, true)}>
                    Set up
                  </Button>
                }
              >
                <Button size="sm" variant="ghost" disabled={props.busy} onClick={() => props.onSetClient(client.id, false)}>
                  Remove
                </Button>
              </Show>
            </Show>
          </SettingsRow>
        )}
      </For>

      <SettingsRow icon={SourceCodeIcon} seed="mcp-client" label="Connect a client" description="Paste into your agent's MCP settings.">
        <Button
          size="sm"
          variant="ghost"
          disabled={!status()?.command}
          onClick={() => void copyText("Command", status()?.command ?? "")}
        >
          Copy command
        </Button>
        <Button
          size="sm"
          variant="ghost"
          disabled={!status()?.cursorConfig}
          onClick={() => void copyText("Cursor config", status()?.cursorConfig ?? "")}
        >
          Copy Cursor config
        </Button>
      </SettingsRow>
    </SettingsSection>
  );
}
