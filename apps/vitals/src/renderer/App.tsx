import {
  A11yDialog,
  Activity,
  Button,
  PageEmpty,
  PageError,
  StatusChip,
  Surface,
  ToastRegion,
  pushToast,
} from "@polterware/frontend-design-system";
import { ErrorBoundary, For, Show, createEffect, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import {
  connectionPresentation,
  type ConnectPayload,
  type ConnectionStateEvent,
  type HostKeyPrompt,
  type MetricsSnapshot,
  type SavedServerPublic,
  type SessionHistory,
} from "./api";
import { ConnectionForm } from "./components/ConnectionForm";
import { HostKeyModal } from "./components/HostKeyModal";
import { MetricsDashboard } from "./components/MetricsDashboard";
import { TerminalView } from "./components/TerminalView";

interface TabState {
  sessionId: string;
  label: string;
  host: string;
  status: ConnectionStateEvent["status"];
  message?: string;
  snapshot: MetricsSnapshot | null;
  history: SessionHistory;
}

const emptyHistory = (): SessionHistory => ({ rx: [], tx: [], cpu: [] });

function appendSample(values: number[], next: number): number[] {
  return [...values, next].slice(-120);
}

export function App() {
  const [theme, setTheme] = createSignal<"dark" | "light">("dark");
  const [saved, setSaved] = createSignal<SavedServerPublic[]>([]);
  const [tabs, setTabs] = createSignal<TabState[]>([]);
  const [activeId, setActiveId] = createSignal<string | null>(null);
  const [hostKey, setHostKey] = createSignal<HostKeyPrompt | null>(null);
  const [view, setView] = createSignal<"metrics" | "terminal">("metrics");
  const [terminalSessions, setTerminalSessions] = createSignal<string[]>([]);
  const [killTarget, setKillTarget] = createSignal<{ sessionId: string; pid: number } | null>(null);
  const active = createMemo(() => tabs().find((tab) => tab.sessionId === activeId()) ?? null);

  createEffect(() => {
    document.documentElement.dataset.theme = theme();
  });

  const refreshSaved = () => {
    if (!window.vitals) return;
    void window.vitals.listServers().then(setSaved);
  };

  onMount(() => {
    if (!window.vitals) return;
    void window.vitals.getTheme().then(setTheme);
    refreshSaved();
    const seenStatus = new Map<string, ConnectionStateEvent["status"]>();

    const offState = window.vitals.onConnectionState((event) => {
      const previous = seenStatus.get(event.sessionId);
      seenStatus.set(event.sessionId, event.status);
      if ((event.status === "error" || event.status === "disconnected") && previous !== event.status) {
        pushToast(event.message || `${event.label} failed`, "danger");
      }
      if (event.status === "reconnecting" && previous !== "reconnecting") {
        pushToast(event.message || `${event.label} is reconnecting`, "default");
      }
      setTabs((current) => {
        const index = current.findIndex((tab) => tab.sessionId === event.sessionId);
        if (index < 0) {
          return [
            ...current,
            {
              sessionId: event.sessionId,
              label: event.label,
              host: event.host,
              status: event.status,
              message: event.message,
              snapshot: null,
              history: emptyHistory(),
            },
          ];
        }
        const next = current.slice();
        const tab = current[index];
        if (!tab) return current;
        next[index] = { ...tab, status: event.status, message: event.message, label: event.label, host: event.host };
        return next;
      });
      setActiveId((id) => id ?? event.sessionId);
    });

    const offMetrics = window.vitals.onMetrics((event) => {
      setTabs((current) =>
        current.map((tab) => {
          if (tab.sessionId !== event.sessionId) return tab;
          const rx = event.snapshot.network[0]?.rxBytesPerSec ?? 0;
          const tx = event.snapshot.network[0]?.txBytesPerSec ?? 0;
          return {
            ...tab,
            snapshot: event.snapshot,
            history: {
              rx: appendSample(tab.history.rx, rx),
              tx: appendSample(tab.history.tx, tx),
              cpu: appendSample(tab.history.cpu, event.snapshot.cpuTotalPercent),
            },
          };
        }),
      );
    });

    const offHost = window.vitals.onHostKeyPrompt(setHostKey);
    onCleanup(() => {
      offState();
      offMetrics();
      offHost();
    });
  });

  const connect = (payload: ConnectPayload) => {
    void window.vitals.connect(payload).catch((error: unknown) => {
      pushToast(error instanceof Error ? error.message : "Connect failed", "danger");
    });
  };

  const toggleTheme = () => {
    const next = theme() === "dark" ? "light" : "dark";
    setTheme(next);
    void window.vitals.setTheme(next);
  };

  const confirmKill = async () => {
    const target = killTarget();
    if (!target) return;
    try {
      await window.vitals.killProcess({ sessionId: target.sessionId, pid: target.pid, signal: "TERM" });
      setKillTarget(null);
    } catch (error) {
      pushToast(error instanceof Error ? error.message : "Failed to stop the process", "danger");
    }
  };

  return (
    <ErrorBoundary
      fallback={(error, reset) => (
        <PageError title="Vitals failed to render" message={error.message} onRetry={reset} />
      )}
    >
      <div class="vitals-shell">
        <header class="vitals-topbar">
          <span class="vitals-brand">
            <Activity size={16} />
            Vitals
          </span>
          <div class="vitals-tabs">
            <For each={tabs()}>
              {(tab) => {
                const presentation = connectionPresentation(tab.status);
                return (
                  <Button
                    variant="ghost"
                    class="vitals-tab"
                    data-active={tab.sessionId === activeId() ? "true" : "false"}
                    onClick={() => setActiveId(tab.sessionId)}
                  >
                    <StatusChip tone={presentation.tone}>{presentation.label}</StatusChip>
                    {tab.label}
                  </Button>
                );
              }}
            </For>
          </div>
          <Button variant="secondary" onClick={toggleTheme}>
            {theme() === "dark" ? "Light" : "Dark"}
          </Button>
          <Show when={active()}>
            {(session) => (
              <Button variant="secondary" onClick={() => void window.vitals.disconnect(session().sessionId)}>
                Disconnect
              </Button>
            )}
          </Show>
        </header>

        <div class="vitals-layout">
          <Surface>
            <div class="vitals-sidebar">
            <ConnectionForm
              saved={saved()}
              onConnect={connect}
              onDeleteSaved={(id) => {
                void window.vitals.deleteServer(id).then(refreshSaved);
              }}
              onRefreshSaved={refreshSaved}
            />
            </div>
          </Surface>
          <main class="vitals-main">
            <Show when={active()}>
              {(session) => (
                <div class="vitals-view-switch">
                  <Button
                    variant={view() === "metrics" ? "secondary" : "ghost"}
                    onClick={() => setView("metrics")}
                  >
                    Metrics
                  </Button>
                  <Button
                    variant={view() === "terminal" ? "secondary" : "ghost"}
                    onClick={() => {
                      setView("terminal");
                      setTerminalSessions((ids) =>
                        ids.includes(session().sessionId) ? ids : [...ids, session().sessionId],
                      );
                    }}
                  >
                    Terminal
                  </Button>
                </div>
              )}
            </Show>
            <For each={tabs().filter((tab) => terminalSessions().includes(tab.sessionId))}>
              {(tab) => (
                <div
                  class="vitals-terminal-pane"
                  hidden={!(view() === "terminal" && tab.sessionId === activeId())}
                >
                  <TerminalView
                    sessionId={tab.sessionId}
                    connected={tab.status === "connected"}
                    visible={view() === "terminal" && tab.sessionId === activeId()}
                  />
                </div>
              )}
            </For>
            <Show
              when={active() && view() === "metrics"}
              fallback={<PageEmpty title="No host connected" description="Connect to a Linux host to see live metrics." />}
            >
              <MetricsDashboard
                snapshot={active()!.snapshot}
                history={active()!.history}
                status={active()!.status}
                message={active()!.message}
                onKill={(pid) => setKillTarget({ sessionId: active()!.sessionId, pid })}
              />
            </Show>
          </main>
        </div>

        <Show when={hostKey()}>
          {(prompt) => (
            <HostKeyModal
              host={prompt().host}
              port={prompt().port}
              fingerprint={prompt().fingerprint}
              onAccept={() => {
                void window.vitals.acceptHostKey(prompt().sessionId);
                setHostKey(null);
              }}
              onReject={() => {
                void window.vitals.rejectHostKey(prompt().sessionId);
                setHostKey(null);
              }}
            />
          )}
        </Show>

        <A11yDialog
          open={killTarget() !== null}
          size="sm"
          title={killTarget() ? `Kill process ${killTarget()?.pid}?` : "Kill process"}
          description="This sends SIGTERM on the remote host."
          onClose={() => setKillTarget(null)}
          footer={
            <div class="vitals-actions">
              <Button variant="secondary" onClick={() => setKillTarget(null)}>
                Cancel
              </Button>
              <Button variant="danger" onClick={() => void confirmKill()}>
                Kill
              </Button>
            </div>
          }
        >
          <p class="vitals-sub">The process stops only if the remote user is allowed to signal it.</p>
        </A11yDialog>
        <ToastRegion />
      </div>
    </ErrorBoundary>
  );
}
