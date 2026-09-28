import { For, Show, createMemo } from "solid-js";
import { Card, StatusChip } from "@polterware/frontend-design-system";
import type { WorkStage, WorkspaceSpecFeature } from "@polterware/feature-shell";
import {
  WORK_STAGES,
  featureOpenQuestions,
  featureTaskProgress,
  resolveFeatureStage,
} from "./work-stage";
import "./work-board.css";

export function WorkBoard(props: {
  features: readonly WorkspaceSpecFeature[];
  activeFeatureKey?: string | null;
  onActivate: (featureKey: string) => void;
}) {
  const columns = createMemo(() => {
    const byStage = new Map<WorkStage, WorkspaceSpecFeature[]>(
      WORK_STAGES.map((stage) => [stage.id, []]),
    );
    for (const feature of props.features) {
      byStage.get(resolveFeatureStage(feature))?.push(feature);
    }
    return WORK_STAGES.map((stage) => ({ ...stage, features: byStage.get(stage.id) ?? [] }));
  });

  return (
    <div class="pw-work-board" role="list" aria-label="Work board">
      <For each={columns()}>
        {(column) => (
          <section class="pw-work-column" role="listitem" aria-label={column.label}>
            <header class="pw-work-column__header">
              <span class="text-[12px] font-medium text-[var(--color-text-secondary)]">
                {column.label}
              </span>
              <span class="text-[12px] tabular-nums text-[var(--color-text-muted)]">
                {column.features.length}
              </span>
            </header>
            <div class="pw-work-column__cards">
              <Show
                when={column.features.length > 0}
                fallback={
                  <p class="m-0 px-1 py-2 text-[12px] text-[var(--color-text-muted)]">
                    No features
                  </p>
                }
              >
                <For each={column.features}>
                  {(feature) => (
                    <WorkCard
                      feature={feature}
                      active={props.activeFeatureKey === feature.featureKey}
                      onActivate={() => props.onActivate(feature.featureKey)}
                    />
                  )}
                </For>
              </Show>
            </div>
          </section>
        )}
      </For>
    </div>
  );
}

function WorkCard(props: {
  feature: WorkspaceSpecFeature;
  active: boolean;
  onActivate: () => void;
}) {
  const progress = createMemo(() => featureTaskProgress(props.feature));
  const openQuestions = createMemo(() => featureOpenQuestions(props.feature));

  return (
    <Card
      class={[
        "pw-work-card !p-0",
        props.active ? "pw-work-card--active" : "",
      ].join(" ")}
    >
      <button
        type="button"
        class="pw-work-card__button"
        aria-pressed={props.active ? "true" : "false"}
        onClick={() => props.onActivate()}
      >
        <span class="pw-work-card__title">{props.feature.title}</span>
        <span class="truncate font-mono text-[11px] text-[var(--color-text-muted)]">
          {props.feature.featureKey}
        </span>
        <span class="flex flex-wrap items-center gap-1.5">
          <Show when={progress().total > 0}>
            <StatusChip
              tone={progress().done === progress().total ? "healthy" : "muted"}
              label={`Tasks ${progress().done}/${progress().total}`}
            />
          </Show>
          <Show when={openQuestions() > 0}>
            <StatusChip
              tone="warn"
              label={openQuestions() === 1 ? "1 open question" : `${openQuestions()} open questions`}
            />
          </Show>
        </span>
        <span class="text-[11px] text-[var(--color-text-muted)]">
          {props.feature.owner} · {props.feature.updatedAt}
        </span>
      </button>
    </Card>
  );
}
