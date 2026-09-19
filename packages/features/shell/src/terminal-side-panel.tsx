import { Show, createEffect, createMemo, createSignal } from "solid-js";
import { ExternalLink } from "@polterware/frontend-design-system";
import { Button, Drawer, StatusChip, pushToast } from "@polterware/frontend-design-system";
import type { TerminalTargetItem } from "./types";
import {
  terminalTargetStatusLabel,
  terminalTargetStatusTone,
} from "./types";
import type { LiveTerminalSession } from "./live-terminal-session";
import { TerminalEmulator } from "./terminal-emulator";
import {
  terminalSessionStatusLabel,
  terminalSessionStatusTone,
} from "./terminal-session-model";
import {
  closeDrawerTerminalSession,
  hasLiveTerminalBridge,
  openDrawerTerminalSession,
  resolveDrawerTarget,
  type DrawerTerminalCallbacks,
} from "./terminal-side-panel-session";

export function TerminalSidePanel(
  props: DrawerTerminalCallbacks & {
    open: boolean;
    targetId: string | null;
    targets?: TerminalTargetItem[];
    onClose: () => void;
    onExpand: (payload: { targetId: string; session: LiveTerminalSession }) => void;
  },
) {
  const targets = () => props.targets ?? [];
  const [session, setSession] = createSignal<LiveTerminalSession | null>(null);
  const [opening, setOpening] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  let handoffSessionId: string | null = null;
  let activeSessionId: string | null = null;

  const callbacks = (): DrawerTerminalCallbacks => ({
    projectId: props.projectId,
    onCreateSession: props.onCreateSession,
    onWriteSession: props.onWriteSession,
    onResizeSession: props.onResizeSession,
    onCloseSession: props.onCloseSession,
    onReadSession: props.onReadSession,
  });

  const target = createMemo(() => resolveDrawerTarget(targets(), props.targetId));
  const live = () => hasLiveTerminalBridge(callbacks());
  const visible = () => props.open && Boolean(props.targetId);

  async function disposeSession(options?: { handoff?: boolean }) {
    const id = activeSessionId;
    if (!id) {
      setSession(null);
      return;
    }
    if (options?.handoff) {
      handoffSessionId = id;
      activeSessionId = null;
      setSession(null);
      return;
    }
    if (handoffSessionId === id) {
      activeSessionId = null;
      setSession(null);
      return;
    }
    // Unmount the emulator before closing the PTY so in-flight polls stop first.
    activeSessionId = null;
    setSession(null);
    await new Promise<void>((resolve) => {
      window.setTimeout(resolve, 0);
    });
    await closeDrawerTerminalSession(callbacks(), id);
  }

  createEffect((previousTargetId?: string | null) => {
    const open = props.open;
    const targetId = props.targetId;
    if (!open || !targetId) {
      if (previousTargetId) {
        void disposeSession();
      }
      return null;
    }
    if (targetId === previousTargetId) return previousTargetId;

    const nextTarget = resolveDrawerTarget(targets(), targetId);
    if (!nextTarget) {
      setError("Select a target to open a terminal.");
      setSession(null);
      return targetId;
    }

    if (!live()) {
      setError("Local terminal is unavailable.");
      setSession(null);
      return targetId;
    }

    setOpening(true);
    setError(null);
    void (async () => {
      await disposeSession();
      try {
        const created = await openDrawerTerminalSession({
          callbacks: callbacks(),
          target: nextTarget,
        });
        if (props.targetId !== targetId || !props.open) {
          await closeDrawerTerminalSession(callbacks(), created.id);
          return;
        }
        activeSessionId = created.id;
        handoffSessionId = null;
        setSession(created);
      } catch (cause: unknown) {
        const message = cause instanceof Error ? cause.message : "Could not open terminal.";
        setError(message);
        pushToast(message, "danger");
      } finally {
        setOpening(false);
      }
    })();

    return targetId;
  });

  async function handleClose() {
    await disposeSession();
    setError(null);
    props.onClose();
  }

  async function handleExpand() {
    const current = session();
    const targetId = props.targetId;
    if (!current || !targetId) return;
    await disposeSession({ handoff: true });
    props.onExpand({ targetId, session: current });
  }

  const title = () => target()?.name ?? "Terminal";
  const subtitle = () => target()?.shell ?? "No target selected";

  return (
    <Drawer
      open={visible()}
      onClose={() => void handleClose()}
      title={title()}
      subtitle={subtitle()}
      width={560}
      closeOnInteractOutside={false}
      flush
      footer={
        <div class="flex w-full flex-wrap items-center justify-between gap-2">
          <div class="flex min-w-0 flex-wrap items-center gap-2">
            <Show
              when={session()}
              fallback={
                <StatusChip
                  tone={target() ? terminalTargetStatusTone(target()!.status) : "unknown"}
                  label={target() ? terminalTargetStatusLabel(target()!.status) : "Unknown"}
                />
              }
            >
              {(current) => (
                <StatusChip
                  tone={terminalSessionStatusTone(current().status)}
                  label={terminalSessionStatusLabel(current().status)}
                />
              )}
            </Show>
            <span class="text-[11px] text-[var(--color-text-muted)]">{target()?.shell ?? "—"}</span>
          </div>
          <Button
            size="sm"
            variant="secondary"
            disabled={!session()}
            onClick={() => void handleExpand()}
          >
            <span class="inline-flex items-center gap-1.5">
              <ExternalLink size={13} style={{ color: "var(--color-text-secondary)" }} />
              Expand
            </span>
          </Button>
        </div>
      }
    >
      <div class="relative flex min-h-0 flex-1 flex-col">
        <Show when={opening()}>
          <p class="m-0 p-4 text-[12px] text-[var(--color-text-muted)]">Opening local terminal…</p>
        </Show>
        <Show when={!opening() && error()}>
          <p class="m-0 p-4 text-[12px] text-[var(--color-text-muted)]">{error()}</p>
        </Show>
        <Show when={!opening() && !error() && live() ? session() : null}>
          {(current) => (
            <div
              class="relative min-h-0 flex-1"
              aria-label={`Terminal for ${target()?.name ?? "target"}`}
            >
              <div class="absolute inset-0">
                <TerminalEmulator
                  sessionId={current().id}
                  onWrite={(data) => {
                    void props.onWriteSession?.(current().id, data);
                  }}
                  onResize={(cols, rows) => {
                    void props.onResizeSession?.(current().id, cols, rows);
                  }}
                  onRead={(offset) => {
                    if (!props.onReadSession) {
                      return Promise.resolve({ data: "", nextOffset: offset, closed: true });
                    }
                    return props.onReadSession(current().id, offset);
                  }}
                />
              </div>
            </div>
          )}
        </Show>
      </div>
    </Drawer>
  );
}
