import { For, Show } from "solid-js";
import { Button, Field, Input, StatusChip } from "@polterware/frontend-design-system";
import type { DriftPreviewResult } from "./agents-types";
import { driftFindingSeverityTone } from "./drift-step-payload";

/** Narrow Drift Checker preview for the current project (Desktop inspection only). */
export function DriftPreviewCard(props: {
  entityId: string;
  busy: boolean;
  result: DriftPreviewResult | undefined;
  error: string | undefined;
  onEntityIdChange: (value: string) => void;
  onCheck: () => void;
}) {
  return (
    <div class="space-y-4">
      <div class="grid gap-3 sm:grid-cols-[1fr_auto] sm:items-end">
        <Field label="Entity id">
          <Input
            value={props.entityId}
            placeholder="optional PPM entity id"
            onInput={(event) => props.onEntityIdChange(event.currentTarget.value)}
          />
        </Field>
        <Button
          size="sm"
          variant="primary"
          disabled={props.busy}
          onClick={() => props.onCheck()}
        >
          Check drift
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
              <StatusChip tone="ok" label={`checker: ${result().checker || "narrow"}`} />
              <StatusChip
                tone={result().findings.length > 0 ? "warn" : "ok"}
                label={
                  result().findings.length === 0
                    ? "No findings"
                    : `${result().findings.length} finding(s)`
                }
              />
              <Show when={result().status}>
                <StatusChip tone="muted" label={result().status} />
              </Show>
            </div>
            <Show
              when={result().findings.length > 0}
              fallback={
                <p class="m-0 text-[12px] text-[var(--color-text-muted)]">
                  No findings under the narrow rules — not “no checker”.
                </p>
              }
            >
              <ul class="m-0 list-none divide-y divide-[var(--color-border-subtle)] overflow-hidden rounded-[var(--radius-card)] border border-[var(--color-border-subtle)] bg-[var(--color-surface)] p-0">
                <For each={result().findings}>
                  {(finding) => (
                    <li class="px-3 py-2">
                      <div class="flex flex-wrap items-center gap-1.5">
                        <StatusChip
                          tone={driftFindingSeverityTone(finding.severity)}
                          label={finding.severity || "finding"}
                        />
                        <Show when={finding.type}>
                          <span class="font-mono text-[11px] text-[var(--color-text-muted)]">
                            {finding.type}
                          </span>
                        </Show>
                      </div>
                      <p class="m-0 mt-1 text-[12px] text-[var(--color-text-primary)]">
                        {finding.summary || "—"}
                      </p>
                      <Show when={finding.subjectIds.length > 0}>
                        <p class="m-0 mt-1 font-mono text-[11px] text-[var(--color-text-muted)] pw-selectable">
                          subjects: {finding.subjectIds.join(", ")}
                        </p>
                      </Show>
                      <Show when={finding.evidenceIds.length > 0}>
                        <p class="m-0 mt-0.5 font-mono text-[11px] text-[var(--color-text-muted)] pw-selectable">
                          evidence: {finding.evidenceIds.join(", ")}
                        </p>
                      </Show>
                    </li>
                  )}
                </For>
              </ul>
            </Show>
          </div>
        )}
      </Show>
    </div>
  );
}
