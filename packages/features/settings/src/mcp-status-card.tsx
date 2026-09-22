import { Button, Card, StatusChip, pushToast } from "@polterware/frontend-design-system";
import type { McpStatusView } from "./agents-types";

async function copyText(label: string, text: string) {
  try {
    await navigator.clipboard.writeText(text);
    pushToast(`${label} copied.`, "success");
  } catch {
    pushToast(`Could not copy ${label.toLowerCase()}.`, "danger");
  }
}

/** Local MCP helper status, start/stop, and Cursor config snippet. */
export function McpStatusCard(props: {
  status: McpStatusView | undefined;
  busy: boolean;
  onStart: () => void;
  onStop: () => void;
  onRefresh: () => void;
}) {
  const status = () => props.status;
  const running = () => Boolean(status()?.running);

  return (
    <Card>
      <div class="flex items-start justify-between gap-3">
        <div>
          <p class="m-0 text-[12px] font-medium text-[var(--color-text-primary)]">
            Local MCP
          </p>
          <p class="m-0 mt-1 text-[12px] text-[var(--color-text-muted)]">
            Experimental stdio server on the same SQLite store. Cursor or Claude spawn their own
            process; Start parks a Desktop helper so you can verify the binary.
          </p>
        </div>
        <StatusChip tone={running() ? "ok" : "warn"} label={running() ? "Running" : "Stopped"} />
      </div>

      <dl class="m-0 mt-3 grid gap-2 text-[12px]">
        <div>
          <dt class="m-0 text-[var(--color-text-muted)]">PID</dt>
          <dd class="m-0 font-mono text-[var(--color-text-primary)]">
            {status()?.pid ?? "—"}
          </dd>
        </div>
        <div>
          <dt class="m-0 text-[var(--color-text-muted)]">Database</dt>
          <dd class="m-0 break-all font-mono text-[11px] text-[var(--color-text-primary)]">
            {status()?.dbPath ?? "—"}
          </dd>
        </div>
      </dl>

      <div class="mt-3 flex flex-wrap gap-2">
        <Button
          size="sm"
          variant="primary"
          disabled={props.busy || running()}
          onClick={() => props.onStart()}
        >
          Start
        </Button>
        <Button
          size="sm"
          variant="ghost"
          disabled={props.busy || !running()}
          onClick={() => props.onStop()}
        >
          Stop
        </Button>
        <Button size="sm" variant="ghost" disabled={props.busy} onClick={() => props.onRefresh()}>
          Refresh
        </Button>
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
      </div>

      <pre class="m-0 mt-3 max-h-40 overflow-auto rounded-[var(--radius-card)] border border-[var(--color-border-subtle)] bg-[var(--color-surface-raised)] p-2 font-mono text-[10.5px] text-[var(--color-text-secondary)]">
        {status()?.cursorConfig?.trim() || "MCP status unavailable."}
      </pre>
    </Card>
  );
}
