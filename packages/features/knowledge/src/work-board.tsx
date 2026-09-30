import { For, Index, Show, createEffect, createMemo, onCleanup, onMount } from "solid-js";
import {
  Card,
  createFluidHover,
  FluidHoverHighlight,
  spring,
  StatusChip,
  type FluidHover,
} from "@polterware/frontend-design-system";
import type { WorkStage, WorkspaceWorkCard } from "@polterware/feature-shell";
import { WORK_STAGES, cardOpenQuestions, cardTaskProgress } from "./work-stage";
import "./work-board.css";

/**
 * One hover highlight for the whole board, resolving the nearest card in two dimensions
 * (fluid-functionalism's card grid), and one active marker that springs between cards,
 * across columns too. Cards keep their surface under both, their content above.
 */
export function WorkBoard(props: {
  cards: readonly WorkspaceWorkCard[];
  activeSlug?: string | null;
  onActivate: (slug: string) => void;
}) {
  let board: HTMLDivElement | undefined;
  // A board has generous whitespace: gap clicks land only within 16px of the lit card.
  const hover = createFluidHover(() => board, { axis: "xy", gapClick: { maxDistance: 16 } });

  const columns = createMemo(() => {
    const byStage = new Map<WorkStage, WorkspaceWorkCard[]>(
      WORK_STAGES.map((stage) => [stage.id, []]),
    );
    for (const card of props.cards) {
      byStage.get(card.stage)?.push(card);
    }
    return WORK_STAGES.map((stage) => ({ ...stage, cards: byStage.get(stage.id) ?? [] }));
  });
  const columnOffsets = createMemo(() => {
    let offset = 0;
    return columns().map((column) => {
      const start = offset;
      offset += column.cards.length;
      return start;
    });
  });
  const activeRect = () => {
    let index = 0;
    for (const column of columns()) {
      for (const card of column.cards) {
        if (card.slug === props.activeSlug) return hover.rects()[index] ?? null;
        index++;
      }
    }
    return null;
  };

  return (
    <div ref={board} class="pw-work-board" role="list" aria-label="Work board" {...hover.handlers}>
      <FluidHoverHighlight hover={hover} class="pw-work-board-hover" />
      {/* A card changing stage keeps its slug: the marker follows it without a spring. */}
      <FluidHoverHighlight
        class="pw-work-board-active"
        rect={activeRect()}
        targetKey={props.activeSlug ?? null}
        transition={spring.moderate}
      />
      <Index each={columns()}>
        {(column, columnIndex) => (
          <section class="pw-work-column" role="listitem" aria-label={column().label}>
            <header class="pw-work-column__header">
              <span class="text-[12px] font-medium text-[var(--color-text-secondary)]">
                {column().label}
              </span>
              <span class="text-[12px] tabular-nums text-[var(--color-text-muted)]">
                {column().cards.length}
              </span>
            </header>
            <div class="pw-work-column__cards">
              <Show
                when={column().cards.length > 0}
                fallback={
                  <p class="m-0 px-1 py-2 text-[12px] text-[var(--color-text-muted)]">
                    No cards
                  </p>
                }
              >
                <Index each={column().cards}>
                  {(card, index) => (
                    <WorkCard
                      card={card()}
                      hover={hover}
                      index={(columnOffsets()[columnIndex] ?? 0) + index}
                      active={props.activeSlug === card().slug}
                      onActivate={() => props.onActivate(card().slug)}
                    />
                  )}
                </Index>
              </Show>
            </div>
          </section>
        )}
      </Index>
    </div>
  );
}

function WorkCard(props: {
  card: WorkspaceWorkCard;
  hover: FluidHover;
  index: number;
  active: boolean;
  onActivate: () => void;
}) {
  const progress = createMemo(() => cardTaskProgress(props.card));
  const openQuestions = createMemo(() => cardOpenQuestions(props.card));
  const tasks = createMemo(() => props.card.tasks.filter((task) => task.state !== "dropped"));
  let element: HTMLDivElement | undefined;
  onMount(() => {
    const card = element;
    if (!card) return;
    createEffect(() => onCleanup(props.hover.registerItem(props.index, card)));
  });

  return (
    <Card
      ref={element}
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
