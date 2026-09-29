import { For, Index, Show } from "solid-js";
import { Button, SwitchRow, StatusChip, pushToast } from "@polterware/frontend-design-system";
import type { AgentSetupView, HookSwitchRow } from "./agents-types";

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

function asString(value: unknown): string {
  return typeof value === "string" ? value : "";
}

export function mapHookSwitches(raw: unknown): HookSwitchRow[] {
  return (Array.isArray(raw) ? raw : []).map((item) => {
    const hook = asRecord(item);
    return {
      name: asString(hook.name),
      label: asString(hook.label),
      note: asString(hook.note),
      // proto3 JSON omits false, so a missing value means the hook is off.
      enabled: hook.enabled === true,
    };
  });
}

export function mapAgentSetup(raw: unknown): AgentSetupView {
  const setup = asRecord(asRecord(raw).setup);
  return {
    hooks: mapHookSwitches(setup.hooks),
    polterwarePath: asString(setup.polterwarePath),
    claudePath: asString(setup.claudePath),
    pluginInstalled: setup.pluginInstalled === true,
    pluginEnabled: setup.pluginEnabled === true,
    pluginVersion: asString(setup.pluginVersion),
    embeddedVersion: asString(setup.embeddedVersion),
    problem: asString(setup.problem),
    skillReady: setup.skillReady === true,
    agentsLineReady: setup.agentsLineReady === true,
  };
}

type Check = { label: string; tone: "ok" | "warn"; state: string; hint?: string };

/** The setup doctor as rows: what is ready and the command that fixes what is not. */
export function setupChecks(setup: AgentSetupView): Check[] {
  const checks: Check[] = [
    setup.polterwarePath
      ? { label: "polterware on PATH", tone: "ok", state: "Found" }
      : { label: "polterware on PATH", tone: "warn", state: "Missing", hint: "Hooks skip themselves until it is on PATH." },
  ];
  if (!setup.claudePath) {
    checks.push({ label: "Claude Code", tone: "warn", state: "Not detected" });
  } else if (setup.problem) {
    checks.push({ label: "Claude plugin", tone: "warn", state: "Unreadable", hint: setup.problem });
  } else if (!setup.pluginInstalled) {
    checks.push({ label: "Claude plugin", tone: "warn", state: "Not installed", hint: "polterware setup claude" });
  } else if (setup.pluginVersion !== setup.embeddedVersion) {
    checks.push({
      label: "Claude plugin",
      tone: "warn",
      state: `${setup.pluginVersion} behind ${setup.embeddedVersion}`,
      hint: "polterware setup claude",
    });
  } else {
    checks.push({
      label: "Claude plugin",
      tone: setup.pluginEnabled ? "ok" : "warn",
      state: setup.pluginEnabled ? `${setup.pluginVersion} enabled` : `${setup.pluginVersion} disabled`,
      hint: setup.pluginEnabled ? undefined : "polterware setup claude --enable",
    });
  }
  checks.push(
    setup.skillReady
      ? { label: "Codex skill", tone: "ok", state: "Installed" }
      : { label: "Codex skill", tone: "warn", state: "Not installed", hint: "polterware setup codex" },
  );
  return checks;
}

const heading = "m-0 text-[12px] font-medium text-[var(--color-text-primary)]";
const panel =
  "rounded-[var(--radius-card)] border border-[var(--color-border-subtle)] bg-[var(--color-surface)] px-3 py-2.5";

async function copyCommand(command: string) {
  try {
    await navigator.clipboard.writeText(command);
    pushToast("Command copied.", "success");
  } catch {
    pushToast("Could not copy command.", "danger");
  }
}

/** Agent setup: doctor state and per-hook switches. Installing runs through `polterware setup`. */
export function AgentSetupCard(props: {
  setup: AgentSetupView | undefined;
  loaded: boolean;
  onToggle: (name: string, enabled: boolean) => void;
}) {
  return (
    <section class={panel}>
      <p class={heading}>Agent setup</p>
      <Show
        when={props.setup}
        fallback={
          <p class="m-0 mt-1.5 text-[12px] text-[var(--color-text-muted)]">
            {props.loaded ? "Setup state is unavailable." : "Checking setup…"}
          </p>
        }
      >
        {(setup) => (
          <>
            <div class="mt-2 space-y-1.5">
              <For each={setupChecks(setup())}>
                {(check) => (
                  <div class="flex items-center justify-between gap-3">
                    <span class="text-[12px] text-[var(--color-text-secondary)]">{check.label}</span>
                    <span class="flex min-w-0 items-center gap-2">
                      <Show when={check.hint}>
                        {(hint) => (
                          <Button
                            size="sm"
                            variant="ghost"
                            class="max-w-[220px] truncate font-mono"
                            title="Copy"
                            onClick={() => void copyCommand(hint())}
                          >
                            {hint()}
                          </Button>
                        )}
                      </Show>
                      <StatusChip tone={check.tone} label={check.state} />
                    </span>
                  </div>
                )}
              </For>
            </div>
            <p class={`${heading} mt-3`}>Hooks</p>
            <div class="mt-1 space-y-0.5">
              {/* Index, not For: hooks are rebuilt on refresh, and For would replace each switch. */}
              <Index each={setup().hooks}>
                {(hook) => (
                  <SwitchRow
                    checked={hook().enabled}
                    onCheckedChange={(checked) => props.onToggle(hook().name, checked)}
                    label={
                      <span class="flex flex-col">
                        <span class="text-[12px] text-[var(--color-text-primary)]">{hook().label}</span>
                        <span class="text-[11px] text-[var(--color-text-muted)]">{hook().note}</span>
                      </span>
                    }
                  />
                )}
              </Index>
            </div>
          </>
        )}
      </Show>
    </section>
  );
}
