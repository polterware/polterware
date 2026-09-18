import { For, Show, createEffect, createMemo, createSignal } from "solid-js";
import { Plus } from "@polterware/frontend-design-system";
import {
  Button,
  PageToolbar,
  StatusChip,
  Tabs,
  pushToast,
} from "@polterware/frontend-design-system";
import type { TerminalSessionMock } from "./workspace-mocks";
import type { TerminalTargetItem } from "./types";
import type { LiveTerminalSession } from "./live-terminal-session";
import { TERMINAL_TARGET_FIXTURES } from "./fixtures";
import { TargetPicker } from "./target-picker";
import { TerminalEmulator } from "./terminal-emulator";
import {
  canOpenTerminalSessionOnTarget,
  formatTerminalStartedAt,
  terminalSessionStatusLabel,
  terminalSessionStatusTone,
  unusableTerminalTargetFeedback,
} from "./terminal-session-model";

export type { LiveTerminalSession } from "./live-terminal-session";

export function TerminalPage(props: {
  sessionId?: string;
  projectId?: string;
  targets?: TerminalTargetItem[];
  initialSession?: LiveTerminalSession | null;
  onSelectSession?: (id: string) => void;
  onCreateSession?: (input: {
    projectId: string;
    targetId: string;
    cols: number;
    rows: number;
  }) => Promise<LiveTerminalSession>;
  onWriteSession?: (id: string, data: string) => Promise<void>;
  onResizeSession?: (id: string, cols: number, rows: number) => Promise<void>;
  onCloseSession?: (id: string) => Promise<void>;
  onReadSession?: (id: string, offset: number) => Promise<{
    data: string;
    nextOffset: number;
    closed: boolean;
  }>;
}) {
  const [sessions, setSessions] = createSignal<LiveTerminalSession[]>([]);
  const [targetPickerOpen, setTargetPickerOpen] = createSignal(false);
  const [activeId, setActiveId] = createSignal(props.sessionId ?? "");
  const [opening, setOpening] = createSignal(false);
  let addTargetButtonEl: HTMLButtonElement | undefined;
  let seededSessionId: string | null = null;

  createEffect(() => {
    const seed = props.initialSession;
    if (!seed || seed.id === seededSessionId) return;
    seededSessionId = seed.id;
    setSessions((list) => (list.some((session) => session.id === seed.id) ? list : [...list, seed]));
    setActiveId(seed.id);
    props.onSelectSession?.(seed.id);
  });

  createEffect(() => {
    const routeSessionId = props.sessionId;
    if (!routeSessionId) return;
    if (sessions().some((session) => session.id === routeSessionId)) {
      setActiveId(routeSessionId);
    }
  });

  const active = createMemo(() => sessions().find((s) => s.id === activeId()) ?? null);
  const targets = () => props.targets ?? TERMINAL_TARGET_FIXTURES;
  const hasTargets = createMemo(() => targets().length > 0);
  const openSessions = createMemo(() => sessions().filter((s) => s.status !== "closed"));
  const availableTargets = createMemo(() =>
    targets().filter(
      (target) => !sessions().some((session) => session.targetId === target.id && session.status !== "closed"),
    ),
  );

  function selectSession(id: string) {
    setActiveId(id);
    props.onSelectSession?.(id);
  }

  async function closeSession(id: string) {
    try {
      await props.onCloseSession?.(id);
    } catch {
      // Still remove from UI if close RPC fails.
    }
    setSessions((list) => {
      const next = list.filter((session) => session.id !== id);
      if (activeId() === id) {
        setActiveId(next[0]?.id ?? "");
      }
      return next;
    });
  }

  async function openTargetSession(targetId: string) {
    const existing = sessions().find((session) => session.targetId === targetId && session.status !== "closed");
    if (existing) {
      selectSession(existing.id);
      return;
    }

    const target = targets().find((item) => item.id === targetId);
    if (!target) {
      pushToast("Target not found.", "danger");
      return;
    }

    if (!canOpenTerminalSessionOnTarget(target.status)) {
      pushToast(unusableTerminalTargetFeedback(target), "danger");
      return;
    }

    if (!props.onCreateSession || !props.projectId) {
      pushToast("Local terminal is unavailable.", "danger");
      return;
    }

    setOpening(true);
    try {
      const session = await props.onCreateSession({
        projectId: props.projectId,
        targetId: target.id,
        cols: 120,
        rows: 32,
      });
      setSessions((list) => [...list, session]);
      selectSession(session.id);
      pushToast(`Opened local session on ${target.name}.`, "success");
    } catch (error: unknown) {
      pushToast(error instanceof Error ? error.message : "Could not open terminal session.", "danger");
    } finally {
      setOpening(false);
    }
  }

  return (
    <div class="flex h-full min-w-0 flex-1 flex-col">
      <PageToolbar.Root>
        <PageToolbar.Tabs>
          <Show
            when={sessions().length > 0}
            fallback={
              <span class="px-2 text-[12px] text-[var(--color-text-muted)]">No sessions</span>
            }
          >
            <Tabs
              items={sessions().map((session) => ({
                id: session.id,
                label: session.name,
                closable: true,
              }))}
              value={activeId()}
              onChange={selectSession}
              onClose={(id) => {
                void closeSession(id);
              }}
              class="pw-tabs--flush flex-nowrap overflow-x-auto"
              aria-label="Terminal sessions"
            />
          </Show>
        </PageToolbar.Tabs>
        <PageToolbar.Actions>
          <Button
            ref={(el) => {
              addTargetButtonEl = el;
            }}
            size="icon"
            variant="primary"
            class="!rounded-full"
            aria-label="New terminal session"
            aria-expanded={targetPickerOpen()}
            title={hasTargets() ? "New session" : "No targets available"}
            disabled={!hasTargets() || opening()}
            onClick={() => {
              if (!hasTargets()) {
                pushToast("No targets available in this project.", "danger");
                return;
              }
              setTargetPickerOpen((open) => !open);
            }}
          >
            <Plus size={14} />
          </Button>
          <TargetPicker
            open={targetPickerOpen()}
            onClose={() => setTargetPickerOpen(false)}
            targets={availableTargets()}
            onSelect={(targetId) => {
              void openTargetSession(targetId);
            }}
            onUnusableSelect={(target) => {
              pushToast(unusableTerminalTargetFeedback(target), "danger");
            }}
            anchorRef={() => addTargetButtonEl}
          />
        </PageToolbar.Actions>
      </PageToolbar.Root>

      <Show
        when={active()}
        fallback={
          <div class="flex flex-1 items-center justify-center p-8">
            <div class="max-w-sm text-center">
              <Show
                when={hasTargets()}
                fallback={
                  <>
                    <p class="m-0 text-[14px] text-[var(--color-text-primary)]">No targets available</p>
                    <p class="m-0 mt-1 text-[12px] text-[var(--color-text-muted)]">
                      Register or select a project target before opening a terminal session.
                    </p>
                  </>
                }
              >
                <p class="m-0 text-[14px] text-[var(--color-text-primary)]">No terminal sessions</p>
                <p class="m-0 mt-1 text-[12px] text-[var(--color-text-muted)]">
                  Open a new interactive local session from a usable target.
                </p>
                <Show when={openSessions().length === 0 && sessions().length > 0}>
                  <p class="m-0 mt-2 text-[11px] text-[var(--color-text-disabled)]">
                    Closed sessions were removed from the switcher.
                  </p>
                </Show>
              </Show>
            </div>
          </div>
        }
      >
        {(s) => (
          <div class="flex min-h-0 flex-1 flex-col">
            <header
              class="flex shrink-0 flex-wrap items-center gap-x-3 gap-y-2 border-b border-[var(--color-border-subtle)] px-3 py-2"
              aria-label="Active session details"
            >
              <StatusChip
                tone={terminalSessionStatusTone(s().status)}
                label={terminalSessionStatusLabel(s().status)}
              />
              <span class="text-[12px] text-[var(--color-text-primary)]">{s().nodeName}</span>
              <span class="text-[12px] text-[var(--color-text-muted)]">{s().shell}</span>
              <span class="text-[12px] text-[var(--color-text-muted)]">
                Started {formatTerminalStartedAt(s().startedAt)}
              </span>
            </header>
            <Show
              when={props.onReadSession && props.onWriteSession}
              fallback={
                <p class="m-0 p-4 text-[12px] text-[var(--color-text-muted)]">
                  Terminal bridge is not available.
                </p>
              }
            >
              <div class="min-h-0 flex-1">
                <TerminalEmulator
                  sessionId={s().id}
                  onWrite={(data) => {
                    void props.onWriteSession?.(s().id, data);
                  }}
                  onResize={(cols, rows) => {
                    void props.onResizeSession?.(s().id, cols, rows);
                  }}
                  onRead={(offset) => props.onReadSession!(s().id, offset)}
                />
              </div>
            </Show>
          </div>
        )}
      </Show>
    </div>
  );
}

/** Session list rows for tests / alternate switcher compositions. */
export function TerminalSessionMetaRow(props: { session: TerminalSessionMock | LiveTerminalSession }) {
  return (
    <div class="flex items-center gap-2 text-[12px]">
      <StatusChip
        tone={terminalSessionStatusTone(props.session.status)}
        label={terminalSessionStatusLabel(props.session.status)}
      />
      <span>{props.session.nodeName}</span>
      <span class="text-[var(--color-text-muted)]">{props.session.shell}</span>
      <span class="text-[var(--color-text-muted)]">
        {formatTerminalStartedAt(props.session.startedAt)}
      </span>
    </div>
  );
}

/** Debug helper: list all session chips (kept for contract tests via source). */
export function TerminalSessionStatusList(props: { sessions: TerminalSessionMock[] }) {
  return (
    <ul class="m-0 list-none p-0">
      <For each={props.sessions}>
        {(session) => (
          <li class="py-1">
            <TerminalSessionMetaRow session={session} />
          </li>
        )}
      </For>
    </ul>
  );
}
