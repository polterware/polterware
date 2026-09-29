import { For, Show, createMemo } from "solid-js";
import { Card, StatusChip } from "@polterware/frontend-design-system";
import type { WorkStage, WorkspaceWorkCard } from "@polterware/feature-shell";
import { WORK_STAGES, cardOpenQuestions, cardTaskProgress } from "./work-stage";
import "./work-board.css";

export function WorkBoard(props: {
  cards: readonly WorkspaceWorkCard[];
  activeSlug?: string | null;
  onActivate: (slug: string) => void;
}) {
  const columns = createMemo(() => {
    const byStage = new Map<WorkStage, WorkspaceWorkCard[]>(
      WORK_STAGES.map((stage) => [stage.id, []]),
    );
    for (const card of props.cards) {
      byStage.get(card.stage)?.push(card);
    }
    return WORK_STAGES.map((stage) => ({ ...stage, cards: byStage.get(stage.id) ?? [] }));
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
                {column.cards.length}
              </span>
            </header>
            <div class="pw-work-column__cards">
              <Show
                when={column.cards.length > 0}
                fallback={
                  <p class="m-0 px-1 py-2 text-[12px] text-[var(--color-text-muted)]">
                    No cards
                  </p>
                }
              >
                <For each={column.cards}>
                  {(card) => (
                    <WorkCard
                      card={card}
                      active={props.activeSlug === card.slug}
                      onActivate={() => props.onActivate(card.slug)}
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
  card: WorkspaceWorkCard;
  active: boolean;
  onActivate: () => void;
}) {
  const progress = createMemo(() => cardTaskProgress(props.card));
  const openQuestions = createMemo(() => cardOpenQuestions(props.card));
  const tasks = createMemo(() => props.card.tasks.filter((task) => task.state !== "dropped"));

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
        <span class="pw-work-card__title">{props.card.title}</span>
        <span class="truncate font-mono text-[11px] text-[var(--color-text-muted)]">
          {props.card.slug}
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
          {props.card.owner ? `${props.card.owner} · ${props.card.updatedAt}` : props.card.updatedAt}
        </span>
      </button>
      <Show when={tasks().length > 0}>
        <ul class="pw-work-card__tasks" aria-label={`${props.card.title} tasks`}>
          <For each={tasks()}>
            {(task) => (
              <li class="pw-work-card__task" data-state={task.state}>
                <span class="pw-work-card__task-number font-mono">T{task.number}</span>
                <span class="min-w-0 flex-1 truncate">
                  {task.state === "done"
                    ? "Done"
                    : task.claimHolder
                      ? task.claimHolder
                      : "Open"}
                </span>
                <Show when={task.state === "open" && task.claimHolder && task.scope}>
                  <span class="pw-work-card__task-scope font-mono" title={task.scope}>
                    {task.scope}
                  </span>
                </Show>
              </li>
            )}
          </For>
        </ul>
      </Show>
    </Card>
  );
}
