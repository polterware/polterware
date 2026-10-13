import {
  A11yDialog,
  Button,
  ControlButton,
  PageEmpty,
  PageError,
  PageToolbar,
  Tabs,
  ToastRegion,
  X,
  pushToast,
} from "@polterware/frontend-design-system";
import { MainFrame, MainSurfaceFrame, SidebarFrame } from "@polterware/frontend-shell";
import { ErrorBoundary, For, Show, createMemo, createSignal, onCleanup, onMount } from "solid-js";
import {
  type ConnectPayload,
  type ConnectionStateEvent,
  type HostKeyPrompt,
  type MetricsSnapshot,
  type NetworkInterface,
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
  mode?: ConnectionStateEvent["mode"];
  snapshot: MetricsSnapshot | null;
  history: SessionHistory;
}

const emptyHistory = (): SessionHistory => ({ cpu: [], interfaces: {} });

function appendSample(values: number[], next: number): number[] {
  return [...values, next].slice(-120);
}

function appendInterfaceHistory(
  previous: SessionHistory["interfaces"],
  network: NetworkInterface[],
): SessionHistory["interfaces"] {
  const next: SessionHistory["interfaces"] = {};
  for (const iface of network) {
    const prior = previous[iface.name];
    next[iface.name] = {
      rx: appendSample(prior?.rx ?? [], iface.rxBytesPerSec),
      tx: appendSample(prior?.tx ?? [], iface.txBytesPerSec),
    };
  }
  return next;
}

export function App() {
  const [saved, setSaved] = createSignal<SavedServerPublic[]>([]);
  const [tabs, setTabs] = createSignal<TabState[]>([]);
  const [activeId, setActiveId] = createSignal<string | null>(null);
  const [hostKey, setHostKey] = createSignal<HostKeyPrompt | null>(null);
  const [view, setView] = createSignal<"metrics" | "terminal">("metrics");
  const [terminalSessions, setTerminalSessions] = createSignal<string[]>([]);
  const [killTarget, setKillTarget] = createSignal<{ sessionId: string; pid: number } | null>(null);
  const active = createMemo(() => tabs().find((tab) => tab.sessionId === activeId()) ?? null);
  const closedSessions = new Set<string>();

  const refreshSaved = () => {
    if (!window.vitals) return;
    void window.vitals.listServers().then(setSaved);
  };

  onMount(() => {
    if (!window.vitals) return;
    refreshSaved();
    const seenStatus = new Map<string, ConnectionStateEvent["status"]>();

    const offState = window.vitals.onConnectionState((event) => {
      if (closedSessions.has(event.sessionId)) return;
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
              mode: event.mode,
              snapshot: null,
              history: emptyHistory(),
            },
          ];
        }
        const next = current.slice();
        const tab = current[index];
        if (!tab) return current;
        next[index] = {
          ...tab,
          status: event.status,
          message: event.message,
          label: event.label,
          host: event.host,
          mode: event.mode ?? tab.mode,
        };
        return next;
      });
      setActiveId((id) => id ?? event.sessionId);
    });

    const offMetrics = window.vitals.onMetrics((event) => {
      setTabs((current) =>
        current.map((tab) => {
          if (tab.sessionId !== event.sessionId) return tab;
          return {
            ...tab,
            snapshot: event.snapshot,
            history: {
              cpu: appendSample(tab.history.cpu, event.snapshot.cpuTotalPercent),
              interfaces: appendInterfaceHistory(tab.history.interfaces, event.snapshot.network ?? []),
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

  const closeTab = (sessionId: string) => {
    closedSessions.add(sessionId);
    const current = tabs();
    const index = current.findIndex((tab) => tab.sessionId === sessionId);
    const remaining = current.filter((tab) => tab.sessionId !== sessionId);
    const neighbor = remaining[index] ?? remaining[index - 1] ?? null;
    setTabs(remaining);
    setTerminalSessions((ids) => ids.filter((id) => id !== sessionId));
    setActiveId((id) => (id === sessionId ? neighbor?.sessionId ?? null : id));
    void window.vitals.disconnect(sessionId);
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
        <div class="vitals-layout">
          <SidebarFrame class="vitals-sidebar-frame" aria-label="Connection">
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
          </SidebarFrame>
          <MainFrame class="vitals-main-frame">
            <div class="vitals-tabs">
              <For each={tabs()}>
                {(tab) => (
                  <div class="vitals-tab-row" data-active={tab.sessionId === activeId() ? "true" : "false"}>
                    <Button
                      variant="ghost"
                      class="vitals-tab"
                      data-active={tab.sessionId === activeId() ? "true" : "false"}
                      onClick={() => setActiveId(tab.sessionId)}
                    >
                      {tab.label}
                    </Button>
                    <button type="button" class="vitals-tab-close" aria-label={`Close ${tab.label}`} onClick={() => closeTab(tab.sessionId)}>
                      <X size={12} />
                    </button>
                  </div>
                )}
              </For>
            </div>
          <MainSurfaceFrame class="vitals-main">
            <div class="vitals-surface-head">
              <Show when={active()}>
              {(session) => (
                <PageToolbar.Root>
                  <PageToolbar.Tabs>
                    <Tabs
                      aria-label="Session view"
                      items={[
                        { id: "metrics", label: "Metrics" },
                        { id: "terminal", label: "Terminal" },
                      ]}
                      value={view()}
                      onChange={(id) => {
                        if (id === "terminal") {
                          setView("terminal");
                          const sessionId = session().sessionId;
                          setTerminalSessions((ids) => (ids.includes(sessionId) ? ids : [...ids, sessionId]));
                          return;
                        }
                        setView("metrics");
                      }}
                    />
                  </PageToolbar.Tabs>
                </PageToolbar.Root>
              )}
            </Show>
            </div>
            {/* Keyed by sessionId string: tab objects are replaced on every metrics event,
                and a For over objects would remount the terminal (closing its shell) each time. */}
            <For each={terminalSessions()}>
              {(sessionId) => {
                const tab = () => tabs().find((candidate) => candidate.sessionId === sessionId);
                return (
                  <Show when={tab()}>
                    <div
                      class="vitals-terminal-pane"
                      hidden={!(view() === "terminal" && sessionId === activeId())}
                    >
                      <TerminalView
                        sessionId={sessionId}
                        connected={tab()?.status === "connected"}
                        visible={view() === "terminal" && sessionId === activeId()}
                      />
                    </div>
                  </Show>
                );
              }}
            </For>
            <Show
              when={active()}
              fallback={<PageEmpty title="No host connected" description="Connect to a Linux host to see live metrics." />}
            >
              <Show when={view() === "metrics"}>
              <MetricsDashboard
                snapshot={active()!.snapshot}
                history={active()!.history}
                status={active()!.status}
                mode={active()!.mode}
                message={active()!.message}
                onKill={(pid) => setKillTarget({ sessionId: active()!.sessionId, pid })}
              />
              </Show>
            </Show>
          </MainSurfaceFrame>
          </MainFrame>
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
              <ControlButton tone="secondary" onClick={() => setKillTarget(null)}>
                Cancel
              </ControlButton>
              <ControlButton tone="danger" onClick={() => void confirmKill()}>
                Kill
              </ControlButton>
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
