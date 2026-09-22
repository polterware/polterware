import { Button, pushToast } from "@polterware/frontend-design-system";
import type { McpStatusView } from "./agents-types";

async function copyText(label: string, text: string) {
  try {
    await navigator.clipboard.writeText(text);
    pushToast(`${label} copied.`, "success");
  } catch {
    pushToast(`Could not copy ${label.toLowerCase()}.`, "danger");
  }
}

function truncatePath(path: string, max = 48): string {
  if (!path || path.length <= max) return path || "—";
  return `…${path.slice(-(max - 1))}`;
}

/** Compact MCP helper metadata and copy actions (Start/Stop live in the toolbar). */
export function McpStatusCard(props: {
  status: McpStatusView | undefined;
}) {
  const status = () => props.status;

  return (
    <section class="rounded-[var(--radius-card)] border border-[var(--color-border-subtle)] bg-[var(--color-surface)] px-3 py-2.5">
      <div class="flex items-center justify-between gap-3">
        <p class="m-0 text-[12px] font-medium text-[var(--color-text-primary)]">Local MCP</p>
        <div class="flex flex-wrap gap-1.5">
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
      </div>
      <p class="m-0 mt-1.5 truncate font-mono text-[11px] text-[var(--color-text-muted)]" title={status()?.dbPath}>
        PID {status()?.pid ?? "—"} · {truncatePath(status()?.dbPath ?? "")}
      </p>
    </section>
  );
}
