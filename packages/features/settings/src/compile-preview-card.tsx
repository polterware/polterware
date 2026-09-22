import { For, Show } from "solid-js";
import { Button, Card, Field, Input, StatusChip } from "@polterware/frontend-design-system";
import type { CompilePreviewResult } from "./agents-types";

/** CompileContext playground for the current project. */
export function CompilePreviewCard(props: {
  query: string;
  budget: string;
  busy: boolean;
  result: CompilePreviewResult | undefined;
  error: string | undefined;
  onQueryChange: (value: string) => void;
  onBudgetChange: (value: string) => void;
  onCompile: () => void;
}) {
  return (
    <Card>
      <p class="m-0 text-[12px] font-medium text-[var(--color-text-primary)]">
        Compile context
      </p>
      <p class="m-0 mt-1 text-[12px] text-[var(--color-text-muted)]">
        Preview the bounded package agents receive. Uses the live Context Compiler over this
        project’s PPM and Knowledge.
      </p>
      <div class="mt-3 grid gap-3 sm:grid-cols-[1fr_7rem_auto] sm:items-end">
        <Field label="Query">
          <Input
            value={props.query}
            placeholder="architecture payment"
            onInput={(event) => props.onQueryChange(event.currentTarget.value)}
          />
        </Field>
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
          disabled={props.busy || !props.query.trim()}
          onClick={() => props.onCompile()}
        >
          Compile
        </Button>
      </div>

      <Show when={props.error}>
        {(message) => (
          <p class="m-0 mt-3 text-[12px] text-[var(--color-danger)]">{message()}</p>
        )}
      </Show>

      <Show when={props.result}>
        {(result) => (
          <div class="mt-3 space-y-2">
            <div class="flex flex-wrap gap-2">
              <StatusChip
                tone="ok"
                label={`${result().tokensUsed} / ${result().tokenBudget} tokens`}
              />
              <Show when={result().truncated}>
                <StatusChip tone="warn" label="Truncated" />
              </Show>
            </div>
            <ul class="m-0 list-none space-y-2 p-0">
              <For each={result().items}>
                {(item) => (
                  <li class="rounded-[var(--radius-card)] border border-[var(--color-border-subtle)] px-2 py-1.5">
                    <p class="m-0 text-[12px] font-medium text-[var(--color-text-primary)]">
                      {item.title}
                    </p>
                    <p class="m-0 mt-0.5 text-[11px] text-[var(--color-text-muted)]">
                      {item.kind} · {item.truthClass} · {item.source}
                      {item.path ? ` · ${item.path}` : ""}
                    </p>
                    <Show when={item.excerpt}>
                      <p class="m-0 mt-1 text-[11.5px] text-[var(--color-text-secondary)]">
                        {item.excerpt}
                      </p>
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
    </Card>
  );
}
