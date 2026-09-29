import {
  Button,
  SwitchRow,
  Select,
  StatusChip,
  type SelectOption,
  type StatusTone,
} from "@polterware/frontend-design-system";
import { Show, createSignal, onCleanup, onMount } from "solid-js";
import { SettingsRow, SettingsSection } from "./settings-section";

type CoreState = "running" | "stopped" | "starting" | "stopping" | "error";
type CoreStatus = { state: CoreState; pid: number | null; endpoint: string | null; error: string | null };
type IdleOption = "never" | "10m" | "30m" | "1h";
type CorePreferences = { idle: IdleOption; autoStart: boolean };
type QuitPreferences = { confirmOnQuit: boolean; quitAction: "keep" | "stop" };
type QuitMode = "ask" | "keep" | "stop";

type CoreBridge = {
  app: {
    getCoreStatus: () => Promise<CoreStatus>;
    startCore: () => Promise<CoreStatus>;
    stopCore: () => Promise<CoreStatus>;
    restartCore: () => Promise<CoreStatus>;
    revealCoreLogs: () => Promise<unknown>;
    getCorePreferences: () => Promise<CorePreferences>;
    setCorePreferences: (patch: Partial<CorePreferences>) => Promise<CorePreferences>;
    getQuitPreferences: () => Promise<QuitPreferences>;
    setQuitPreferences: (patch: Partial<QuitPreferences>) => Promise<QuitPreferences>;
  };
  core: { onStatusChange: (callback: (status: CoreStatus) => void) => () => void };
};

function coreBridge(): CoreBridge | undefined {
  const root = globalThis as { polterware?: Partial<CoreBridge> };
  const app = root.polterware?.app;
  if (!app?.getCorePreferences || !app?.getCoreStatus || !root.polterware?.core?.onStatusChange) {
    return undefined;
  }
  return root.polterware as CoreBridge;
}

const IDLE_OPTIONS: readonly SelectOption[] = [
  { value: "never", label: "Never (keep running)" },
  { value: "10m", label: "After 10 minutes" },
  { value: "30m", label: "After 30 minutes" },
  { value: "1h", label: "After 1 hour" },
];

const QUIT_OPTIONS: readonly SelectOption[] = [
  { value: "ask", label: "Ask me" },
  { value: "keep", label: "Keep the core running" },
  { value: "stop", label: "Stop the core" },
];

const STATE_LABEL: Record<CoreState, string> = {
  running: "Running",
  stopped: "Stopped",
  starting: "Starting",
  stopping: "Stopping",
  error: "Error",
};

const STATE_TONE: Record<CoreState, StatusTone> = {
  running: "ok",
  stopped: "muted",
  starting: "warn",
  stopping: "warn",
  error: "danger",
};

function quitMode(preferences: QuitPreferences): QuitMode {
  return preferences.confirmOnQuit ? "ask" : preferences.quitAction;
}

/** Local core: status, lifecycle actions, idle timeout, auto-start, and quit behavior. */
export function CoreCard() {
  const bridge = coreBridge();
  const [status, setStatus] = createSignal<CoreStatus | null>(null);
  const [idle, setIdle] = createSignal<IdleOption>("never");
  const [appliedIdle, setAppliedIdle] = createSignal<IdleOption>("never");
  const [autoStart, setAutoStart] = createSignal(true);
  const [mode, setMode] = createSignal<QuitMode>("ask");
  const [ready, setReady] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const state = () => status()?.state ?? "stopped";
  const busy = () => state() === "starting" || state() === "stopping";

  onMount(() => {
    if (!bridge) return;
    const unsubscribe = bridge.core.onStatusChange(setStatus);
    onCleanup(unsubscribe);
    void Promise.all([
      bridge.app.getCoreStatus(),
      bridge.app.getCorePreferences(),
      bridge.app.getQuitPreferences(),
    ]).then(([nextStatus, core, quit]) => {
      setStatus(nextStatus);
      setIdle(core.idle);
      setAppliedIdle(core.idle);
      setAutoStart(core.autoStart);
      setMode(quitMode(quit));
      setReady(true);
    });
  });

  async function run(action: () => Promise<CoreStatus | unknown>) {
    setError(null);
    try {
      const result = await action();
      if (result && typeof result === "object" && "state" in result) setStatus(result as CoreStatus);
      // A (re)started core picks up the saved idle setting.
      if (state() === "running") setAppliedIdle(idle());
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "Action failed.");
    }
  }

  async function saveMode(next: QuitMode) {
    setMode(next);
    const saved = await bridge?.app.setQuitPreferences(
      next === "ask" ? { confirmOnQuit: true } : { confirmOnQuit: false, quitAction: next },
    );
    if (saved) setMode(quitMode(saved));
  }

  return (
    <SettingsSection
      title="Local core"
      description="The local core serves the Desktop, CLI, TUI, and MCP clients. It keeps running after you quit until you stop it."
    >
      <SettingsRow label="Status">
        <StatusChip tone={STATE_TONE[state()]} label={STATE_LABEL[state()]} />
        <Show
          when={state() === "running"}
          fallback={
            <Button size="sm" disabled={!ready() || busy()} onClick={() => void run(() => bridge!.app.startCore())}>
              Start
            </Button>
          }
        >
          <Button size="sm" disabled={busy()} onClick={() => void run(() => bridge!.app.restartCore())}>
            Restart
          </Button>
          <Button size="sm" disabled={busy()} onClick={() => void run(() => bridge!.app.stopCore())}>
            Stop
          </Button>
        </Show>
        <Button size="sm" variant="ghost" disabled={!ready()} onClick={() => void run(() => bridge!.app.revealCoreLogs())}>
          Show logs
        </Button>
      </SettingsRow>

      <Show when={error() ?? status()?.error}>
        {(message) => (
          <p role="alert" class="m-0 text-[12px] text-[var(--color-danger)]">
            {message()}
          </p>
        )}
      </Show>

      <SettingsRow
        label="Stop automatically when idle"
        description={
          state() === "running" && idle() !== appliedIdle()
            ? "Applies the next time the core starts. Restart it to apply now."
            : undefined
        }
      >
        <Select
          aria-label="Core idle timeout"
          options={IDLE_OPTIONS}
          value={idle()}
          disabled={!ready()}
          onChange={(value) => {
            setIdle(value as IdleOption);
            void bridge?.app.setCorePreferences({ idle: value as IdleOption }).then((saved) => setIdle(saved.idle));
          }}
        />
      </SettingsRow>

      <SettingsRow label="Start the core when Polterware opens">
        <SwitchRow
          checked={autoStart()}
          disabled={!ready()}
          label={<span class="sr-only">Start the core when Polterware opens</span>}
          onCheckedChange={(checked) => {
            setAutoStart(checked);
            void bridge?.app.setCorePreferences({ autoStart: checked }).then((saved) => setAutoStart(saved.autoStart));
          }}
        />
      </SettingsRow>

      <SettingsRow label="When quitting Polterware">
        <Select
          aria-label="Core behavior on quit"
          options={QUIT_OPTIONS}
          value={mode()}
          disabled={!ready()}
          onChange={(value) => void saveMode(value as QuitMode)}
        />
      </SettingsRow>
    </SettingsSection>
  );
}
