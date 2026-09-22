import { For, Show } from "solid-js";
import { Button, Field, Input, StatusChip } from "@polterware/frontend-design-system";
import type { CompilePreviewResult } from "./agents-types";

/** CompileContext playground for the current project. */
export function CompilePreviewCard(props: {
  query: string;
  entityId: string;
  budget: string;
  busy: boolean;
  result: CompilePreviewResult | undefined;
  error: string | undefined;
  onQueryChange: (value: string) => void;
  onEntityIdChange: (value: string) => void;
  onBudgetChange: (value: string) => void;
  onCompile: () => void;
}) {
  return (
    <div class="space-y-4">
      <div class="grid gap-3 sm:grid-cols-[1fr_1fr] sm:items-end">
        <Field label="Query">
          <Input
            value={props.query}
            placeholder="architecture payment (optional)"
            onInput={(event) => props.onQueryChange(event.currentTarget.value)}
          />
        </Field>
        <Field label="Entity id">
          <Input
            value={props.entityId}
            placeholder="optional PPM entity id"
            onInput={(event) => props.onEntityIdChange(event.currentTarget.value)}
          />
        </Field>
      </div>
      <div class="grid gap-3 sm:grid-cols-[7rem_auto] sm:items-end">
        <Field label="Budget">
          <Input
            value={props.budget}
            inputMode="numeric"
            onInput={(event) => props.onBudgetChange(event.currentTarget.value)}
          />
        </Field>
        <Button
          size="sm"
          variant="primary"
          disabled={props.busy}
          onClick={() => props.onCompile()}
        >
          Compile
        </Button>
      </div>

      <Show when={props.error}>
        {(message) => (
          <p class="m-0 text-[12px] text-[var(--color-danger)]">{message()}</p>
        )}
      </Show>

      <Show when={props.result}>
        {(result) => (
          <div class="space-y-2">
            <div class="flex flex-wrap gap-2">
              <StatusChip
                tone="ok"
                label={`${result().tokensUsed} / ${result().tokenBudget} tokens`}
              />
              <Show when={result().truncated}>
                <StatusChip tone="warn" label="Truncated" />
              </Show>
            </div>
            <ul class="m-0 list-none divide-y divide-[var(--color-border-subtle)] overflow-hidden rounded-[var(--radius-card)] border border-[var(--color-border-subtle)] bg-[var(--color-surface)] p-0">
              <For each={result().items}>
                {(item) => (
                  <li class="px-3 py-2">
                    <p class="m-0 text-[12px] font-medium text-[var(--color-text-primary)]">
                      {item.title}
                    </p>
                    <p class="m-0 mt-0.5 truncate text-[11px] text-[var(--color-text-muted)]">
                      {item.kind} · {item.truthClass} · {item.source}
                      {item.path ? ` · ${item.path}` : ""}
                    </p>
                    <Show when={item.reasons.length > 0}>
                      <p class="m-0 mt-1 text-[11px] text-[var(--color-text-muted)]">
                        reasons: {item.reasons.join(", ")}
                      </p>
                    </Show>
                    <Show when={item.excerpt}>
                      <p class="m-0 mt-1 line-clamp-2 text-[11.5px] text-[var(--color-text-secondary)]">
                        {item.excerpt}
                      </p>
                    </Show>
                    <Show when={item.evidence.length > 0}>
                      <ul class="m-0 mt-1.5 list-none space-y-0.5 p-0">
                        <For each={item.evidence}>
                          {(ev) => (
                            <li class="truncate text-[11px] text-[var(--color-text-muted)]">
                              evidence · {ev.kind}
                              {ev.path ? ` · ${ev.path}` : ""}
                              {ev.summary ? ` — ${ev.summary}` : ""}
                            </li>
                          )}
                        </For>
                      </ul>
                    </Show>
                  </li>
                )}
              </For>
            </ul>
            <Show when={result().items.length === 0}>
              <p class="m-0 text-[12px] text-[var(--color-text-muted)]">No items in this package.</p>
            </Show>
          </div>
        )}
      </Show>
    </div>
  );
}
