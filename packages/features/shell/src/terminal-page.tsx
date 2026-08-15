import { For, Show, createMemo, createSignal } from "solid-js";
import { Plus } from "@polterware/frontend-design-system";
import {
  Button,
  PageToolbar,
  StatusChip,
  Tabs,
  pushToast,
} from "@polterware/frontend-design-system";
import type { TerminalSessionMock } from "./workspace-mocks";
import { TERMINAL_SESSION_MOCKS } from "./workspace-mocks";
import type { TerminalTargetItem } from "./types";
import { TERMINAL_TARGET_FIXTURES } from "./fixtures";
import { TargetPicker } from "./target-picker";
import { TerminalTranscript } from "./terminal-transcript";
import {
  canOpenTerminalSessionOnTarget,
  createMockTerminalSession,
  findSessionForTarget,
  formatTerminalStartedAt,
  removeTerminalSession,
  terminalSessionStatusLabel,
  terminalSessionStatusTone,
  unusableTerminalTargetFeedback,
} from "./terminal-session-model";

export function TerminalPage(props: {
  sessionId?: string;
  sessions?: TerminalSessionMock[];
  targets?: TerminalTargetItem[];
  onSelectSession?: (id: string) => void;
}) {
  const [sessions, setSessions] = createSignal(props.sessions ?? TERMINAL_SESSION_MOCKS);
  const [targetPickerOpen, setTargetPickerOpen] = createSignal(false);
  const [activeId, setActiveId] = createSignal(
    props.sessionId ??
      sessions().find((s) => s.status !== "closed")?.id ??
      sessions()[0]?.id ??
      "",
  );
  let addTargetButtonEl: HTMLButtonElement | undefined;

  const active = createMemo(() => sessions().find((s) => s.id === activeId()) ?? null);
  const targets = () => props.targets ?? TERMINAL_TARGET_FIXTURES;
  const hasTargets = createMemo(() => targets().length > 0);
  const openSessions = createMemo(() => sessions().filter((s) => s.status !== "closed"));
  /** Targets not already bound to an open (non-closed) session. */
  const availableTargets = createMemo(() =>
    targets().filter((target) => !findSessionForTarget(sessions(), target.id)),
  );

  function selectSession(id: string) {
    setActiveId(id);
    props.onSelectSession?.(id);
  }

  function closeSession(id: string) {
    const result = removeTerminalSession(sessions(), id);
    setSessions(result.sessions);
    if (activeId() === id) {
      setActiveId(result.nextActiveId);
    }
  }

  function openTargetSession(targetId: string) {
    const existing = findSessionForTarget(sessions(), targetId);
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

    const next = createMockTerminalSession({ target, dataSource: "mock" });
    setSessions((list) => [...list, next]);
    selectSession(next.id);
    pushToast(`Opened mock session on ${target.name}.`, "success");
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
              onClose={closeSession}
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
            disabled={!hasTargets()}
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
            onSelect={openTargetSession}
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
            <div class="text-center max-w-sm">
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
                  Open a new session from a usable target.
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
            <TerminalTranscript
              lines={s().lines}
              label={`Terminal: ${s().name}`}
              class="flex-1"
            />
          </div>
        )}
      </Show>
    </div>
  );
}

/** Session list rows for tests / alternate switcher compositions. */
export function TerminalSessionMetaRow(props: { session: TerminalSessionMock }) {
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
