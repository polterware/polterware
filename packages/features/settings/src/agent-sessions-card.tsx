import { For, Show } from "solid-js";
import { Skeleton, StatusChip } from "@polterware/frontend-design-system";
import type { AgentSessionsView } from "./agents-types";

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

const heading = "m-0 text-[12px] font-medium text-[var(--color-text-primary)]";
const panel =
  "rounded-[var(--radius-card)] border border-[var(--color-border-subtle)] bg-[var(--color-surface)] px-3 py-2.5";

/** Live agent sessions and the newest handoffs of the current project. */
export function AgentSessionsCard(props: { view: AgentSessionsView | undefined; loaded: boolean }) {
  const view = () => props.view;
  return (
    <div class="space-y-8">
      <section class="space-y-3">
        <div class="flex items-center justify-between gap-3">
          <h2 class={heading}>Live sessions</h2>
          <Show when={view()?.branch}>
            <span class="font-mono text-[11px] text-[var(--color-text-muted)]">{view()?.branch}</span>
          </Show>
        </div>
        <Show
          when={(view()?.sessions.length ?? 0) > 0}
          fallback={
            <Show
              when={props.loaded}
              fallback={
                <div class="space-y-2" role="status" aria-label="Loading sessions…">
                  <Skeleton class="h-12 w-full" />
                  <Skeleton class="h-12 w-full" />
                </div>
              }
            >
              <p class="m-0 text-[12px] text-[var(--color-text-muted)]">No agent is working on this project now.</p>
            </Show>
          }
        >
          <For each={view()?.sessions}>
            {(session) => (
              <div class={panel}>
                <div class="flex items-center justify-between gap-3">
                  <p class="m-0 flex items-center gap-2 text-[12px] font-medium text-[var(--color-text-primary)]">
                    {session.agent || "unknown agent"}
                    <StatusChip tone={session.runId ? "ok" : "warn"} label={session.runId ? "Running" : "Claims only"} />
                  </p>
                  <span class="text-[11px] text-[var(--color-text-muted)]">{sessionAge(session.startedAt)}</span>
                </div>
                <Show when={session.task}>
                  <p class="m-0 mt-1 text-[12px] text-[var(--color-text-secondary)]">{session.task}</p>
                </Show>
                <For each={session.claims}>
                  {(claim) => (
                    <p class="m-0 mt-1 font-mono text-[11px] text-[var(--color-text-muted)]">
                      {claim.slug}#T{claim.number}
                      {claim.scope ? ` · ${claim.scope}` : ""}
                    </p>
                  )}
                </For>
              </div>
            )}
          </For>
        </Show>
      </section>

      <section class="space-y-3">
        <h2 class={heading}>Recent handoffs</h2>
        <Show
          when={(view()?.handoffs.length ?? 0) > 0}
          fallback={
            <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
              {props.loaded ? "No handoffs yet. Agents write one with polterware handoff." : ""}
            </p>
          }
        >
          <For each={view()?.handoffs}>
            {(handoff) => (
              <div class={panel}>
                <div class="flex items-center justify-between gap-3">
                  <p class="m-0 font-mono text-[12px] text-[var(--color-text-primary)]">{handoff.slug}</p>
                  <span class="text-[11px] text-[var(--color-text-muted)]">
                    {[handoff.agent, handoff.branch, sessionAge(handoff.at)].filter(Boolean).join(" · ")}
                  </span>
                </div>
                <Show when={handoff.goal}>
                  <p class="m-0 mt-1 text-[12px] text-[var(--color-text-secondary)]">{handoff.goal}</p>
                </Show>
                <Show when={handoff.done}>
                  <p class="m-0 mt-1 text-[11px] text-[var(--color-text-muted)]">Done: {handoff.done}</p>
                </Show>
                <Show when={handoff.inProgress}>
                  <p class="m-0 mt-0.5 text-[11px] text-[var(--color-text-muted)]">In progress: {handoff.inProgress}</p>
                </Show>
                <Show when={handoff.next}>
                  <p class="m-0 mt-0.5 text-[11px] text-[var(--color-text-muted)]">Next: {handoff.next}</p>
                </Show>
              </div>
            )}
          </For>
        </Show>
      </section>
    </div>
  );
}
