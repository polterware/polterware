import { For, Show } from "solid-js";
import { Bot, Skeleton, StatusChip } from "@polterware/frontend-design-system";
import type { AgentSessionsView } from "./agents-types";
import { SettingsSection } from "./settings-section";

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

function asString(value: unknown): string {
  return typeof value === "string" ? value : "";
}

function asList(value: unknown): Record<string, unknown>[] {
  return Array.isArray(value) ? value.map(asRecord) : [];
}

export function mapAgentSessions(raw: unknown): AgentSessionsView {
  const record = asRecord(raw);
  return {
    branch: asString(record.branch),
    sessions: asList(record.sessions).map((session) => ({
      runId: asString(session.runId),
      agent: asString(session.agent),
      task: asString(session.task),
      startedAt: asString(session.startedAt),
      claims: asList(session.claims).map((claim) => ({
        slug: asString(claim.slug),
        number: typeof claim.number === "number" ? claim.number : 0,
        title: asString(claim.title),
        scope: asString(claim.scope),
      })),
    })),
    handoffs: asList(record.handoffs).map((handoff) => ({
      slug: asString(handoff.slug),
      branch: asString(handoff.branch),
      goal: asString(handoff.goal),
      done: asString(handoff.done),
      inProgress: asString(handoff.inProgress),
      next: asString(handoff.next),
      agent: asString(handoff.agent),
      at: asString(handoff.at),
    })),
  };
}

/** Compact age such as "14m" or "2h"; empty when the time is unknown. */
export function sessionAge(iso: string, now = Date.now()): string {
  const started = Date.parse(iso);
  if (!Number.isFinite(started)) return "";
  const minutes = Math.max(0, Math.floor((now - started) / 60_000));
  if (minutes < 1) return "<1m";
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.floor(minutes / 60);
  if (hours < 48) return `${hours}h`;
  return `${Math.floor(hours / 24)}d`;
}

/** Live agent sessions and the newest handoffs of the current project. */
export function AgentSessionsCard(props: { view: AgentSessionsView | undefined; loaded: boolean }) {
  const view = () => props.view;
  return (
    <>
      <SettingsSection
        title="Live sessions"
        description={view()?.branch ? `Agents working on ${view()?.branch} now.` : "Agents working on this project now."}
      >
        <Show
          when={(view()?.sessions.length ?? 0) > 0}
          fallback={
            <Show
              when={props.loaded}
              fallback={
                <div class="flex flex-col gap-3" role="status" aria-label="Loading sessions…">
                  <Skeleton class="h-3 w-[55%]" />
                  <Skeleton class="h-3 w-[40%]" />
                </div>
              }
            >
              <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
                No agent is working on this project now.
              </p>
            </Show>
          }
        >
          <For each={view()?.sessions}>
            {(session) => (
              <div class="flex items-start justify-between gap-4">
                <div class="flex min-w-0 flex-1 gap-2.5">
                  <span class="mt-0.5 shrink-0 text-[var(--color-text-muted)]">
                    <Bot size={16} />
                  </span>
                  <div class="min-w-0">
                    <p class="m-0 text-[12.5px] text-[var(--color-text-primary)]">
                      {session.agent || "Unknown agent"}
                    </p>
                    <Show when={session.task}>
                      <p class="m-0 mt-0.5 text-[11.5px] text-[var(--color-text-secondary)]">{session.task}</p>
                    </Show>
                    <Show when={session.claims.length > 0}>
                      <ul class="m-0 mt-1.5 flex list-none flex-col gap-0.5 p-0">
                        <For each={session.claims}>
                          {(claim) => (
                            <li class="truncate font-mono text-[11px] text-[var(--color-text-muted)]">
                              {claim.slug}#T{claim.number}
                              {claim.title ? ` ${claim.title}` : ""}
                              {claim.scope ? ` · ${claim.scope}` : ""}
                            </li>
                          )}
                        </For>
                      </ul>
                    </Show>
                  </div>
                </div>
                <div class="flex shrink-0 items-center gap-2">
                  <Show when={sessionAge(session.startedAt)}>
                    {(age) => <span class="text-[11.5px] tabular-nums text-[var(--color-text-muted)]">{age()}</span>}
                  </Show>
                  <StatusChip tone={session.runId ? "ok" : "muted"} label={session.runId ? "Running" : "Claims only"} />
                </div>
              </div>
            )}
          </For>
        </Show>
      </SettingsSection>

      <SettingsSection title="Recent handoffs" description="What the last agents finished and left for the next one.">
        <Show
          when={(view()?.handoffs.length ?? 0) > 0}
          fallback={
            <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
              {props.loaded ? "No handoffs yet. Agents write one with polterware handoff." : "Loading handoffs…"}
            </p>
          }
        >
          <div class="flex flex-col gap-5">
            <For each={view()?.handoffs}>
              {(handoff) => (
                <article class="flex flex-col gap-1.5">
                  <div class="flex items-baseline justify-between gap-3">
                    <p class="m-0 min-w-0 truncate text-[12.5px] text-[var(--color-text-primary)]">
                      {handoff.goal || handoff.slug}
                    </p>
                    <span class="shrink-0 text-[11.5px] text-[var(--color-text-muted)]">
                      {[handoff.agent, sessionAge(handoff.at)].filter(Boolean).join(" · ")}
                    </span>
                  </div>
                  <p class="m-0 truncate font-mono text-[11px] text-[var(--color-text-muted)]">
                    {[handoff.slug, handoff.branch].filter(Boolean).join(" · ")}
                  </p>
                  <Show when={handoff.done || handoff.inProgress || handoff.next}>
                    <dl class="m-0 mt-1 grid grid-cols-[5.5rem_minmax(0,1fr)] gap-x-3 gap-y-1 text-[12px]">
                      <HandoffLine label="Done" value={handoff.done} />
                      <HandoffLine label="In progress" value={handoff.inProgress} />
                      <HandoffLine label="Next" value={handoff.next} />
                    </dl>
                  </Show>
                </article>
              )}
            </For>
          </div>
        </Show>
      </SettingsSection>
    </>
  );
}

function HandoffLine(props: { label: string; value: string }) {
  return (
    <Show when={props.value}>
      <dt class="text-[var(--color-text-muted)]">{props.label}</dt>
      <dd class="m-0 whitespace-pre-wrap text-[var(--color-text-secondary)]">{props.value}</dd>
    </Show>
  );
}
