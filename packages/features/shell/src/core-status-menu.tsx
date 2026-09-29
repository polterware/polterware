import { For, Show, createSignal } from "solid-js";
import {
  Button,
  Copy,
  FileText,
  Play,
  Popover,
  RefreshCw,
  Server,
  Square,
  StatusChip,
  type StatusTone,
} from "@polterware/frontend-design-system";
import "./core-status-menu.css";

export type LocalCoreState = "running" | "stopped" | "starting" | "stopping" | "error";

export type LocalCoreStatus = {
  state: LocalCoreState;
  pid: number | null;
  endpoint: string | null;
  dbPath: string;
  logPath: string;
  error: string | null;
};

export type CoreStatusMenuProps = {
  status: LocalCoreStatus | null;
  onStart: () => void | Promise<unknown>;
  onStop: () => void | Promise<unknown>;
  onRestart: () => void | Promise<unknown>;
  onRevealLogs: () => void | Promise<unknown>;
  onCopy?: (text: string) => void | Promise<unknown>;
};

const headerIconControlClass =
  "control-field control-field-md relative inline-flex w-[var(--control-height-md)] shrink-0 cursor-pointer items-center justify-center !rounded-full p-0 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] focus-visible:outline-none";

const STATE_LABEL: Record<LocalCoreState, string> = {
  running: "Running",
  stopped: "Stopped",
  starting: "Starting",
  stopping: "Stopping",
  error: "Error",
};

const STATE_TONE: Record<LocalCoreState, StatusTone> = {
  running: "ok",
  stopped: "muted",
  starting: "warn",
  stopping: "warn",
  error: "danger",
};

export function CoreStatusMenu(props: CoreStatusMenuProps) {
  const [open, setOpen] = createSignal(false);
  const [actionError, setActionError] = createSignal<string | null>(null);
  let triggerRef: HTMLButtonElement | undefined;

  const state = (): LocalCoreState => props.status?.state ?? "stopped";
  const transitioning = () => state() === "starting" || state() === "stopping";

  async function run(action: () => void | Promise<unknown>) {
    setActionError(null);
    try {
      await action();
    } catch (error) {
      setActionError(error instanceof Error ? error.message : "Action failed.");
    }
  }

  const details = () => {
    const status = props.status;
    if (!status) return [];
    return [
      { label: "PID", value: status.pid === null ? "—" : String(status.pid) },
      { label: "Endpoint", value: status.endpoint ?? "—" },
      { label: "Database", value: status.dbPath },
    ];
  };

  return (
    <>
      <button
        ref={(el) => {
          triggerRef = el;
        }}
        type="button"
        aria-label={`Local core: ${STATE_LABEL[state()].toLowerCase()}`}
        aria-expanded={open()}
        title={`Local core (${STATE_LABEL[state()].toLowerCase()})`}
        onClick={() => setOpen((value) => !value)}
        class={headerIconControlClass}
      >
        <Server size={14} />
        <span class="pw-core-dot" data-state={state()} aria-hidden />
      </button>

      <Popover
        open={open()}
        onClose={() => setOpen(false)}
        title="Local core"
        width={300}
        placement="bottom-end"
        anchorRef={() => triggerRef}
      >
        <div class="flex flex-col gap-3 p-3 text-[13px]">
          <div class="flex items-center justify-between gap-2">
            <span class="font-medium text-[var(--color-text-primary)]">Local core</span>
            <StatusChip tone={STATE_TONE[state()]} label={STATE_LABEL[state()]} />
          </div>

          <dl class="m-0 flex flex-col gap-1.5">
            <For each={details()}>
              {(row) => (
                <div class="flex items-baseline justify-between gap-3">
                  <dt class="shrink-0 text-[var(--color-text-secondary)]">{row.label}</dt>
                  <dd class="m-0 min-w-0 truncate text-right text-[var(--color-text-primary)]" title={row.value}>
                    {row.value}
                  </dd>
                </div>
              )}
            </For>
          </dl>

          <Show when={actionError() ?? props.status?.error}>
            {(message) => (
              <p role="alert" class="m-0 text-[12px] text-[var(--color-danger)]">
                {message()}
              </p>
            )}
          </Show>

          <div class="flex flex-wrap gap-2">
            <Show
              when={state() === "running"}
              fallback={
                <Button size="sm" disabled={transitioning()} onClick={() => void run(props.onStart)}>
                  <Play size={12} />
                  Start
                </Button>
              }
            >
              <Button size="sm" disabled={transitioning()} onClick={() => void run(props.onRestart)}>
                <RefreshCw size={12} />
                Restart
              </Button>
              <Button size="sm" disabled={transitioning()} onClick={() => void run(props.onStop)}>
                <Square size={12} />
                Stop
              </Button>
            </Show>
            <Button size="sm" variant="ghost" onClick={() => void run(props.onRevealLogs)}>
              <FileText size={12} />
              Show logs
            </Button>
            <Show when={props.status?.endpoint && props.onCopy}>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => {
                  const endpoint = props.status?.endpoint;
                  if (endpoint) void run(() => props.onCopy?.(endpoint));
                }}
              >
                <Copy size={12} />
                Copy endpoint
              </Button>
            </Show>
          </div>

          <Show when={state() === "running"}>
            <p class="m-0 text-[12px] text-[var(--color-text-secondary)]">
              Stopping the core pauses the CLI, TUI, and MCP until it starts again.
            </p>
          </Show>
        </div>
      </Popover>
    </>
  );
}
