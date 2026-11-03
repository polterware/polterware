import type { JSX } from "solid-js";
import { For, Match, Switch } from "solid-js";
import "./skeleton.css";

export type PageSkeletonLayout = "page" | "table" | "canvas" | "detail";

export type SkeletonProps = {
  class?: string;
  style?: JSX.CSSProperties | string;
  /** Accessible name when the bone is the sole status indicator. */
  label?: string;
};

/** Opaque loading bone — size via class (height and width utilities). */
export function Skeleton(props: SkeletonProps) {
  return (
    <span
      class={`pw-skeleton ${props.class ?? ""}`}
      style={props.style}
      aria-hidden={props.label ? undefined : true}
      aria-label={props.label}
      role={props.label ? "status" : undefined}
    />
  );
}

export type PageSkeletonProps = {
  layout?: PageSkeletonLayout;
  label?: string;
  class?: string;
};

function ToolbarStrip() {
  return (
    <div class="pw-page-skeleton__toolbar" aria-hidden>
      <Skeleton class="h-4 w-28" />
      <span class="ml-auto" />
      <Skeleton class="h-7 w-40" />
      <Skeleton class="h-7 w-7" />
      <Skeleton class="h-7 w-7" />
    </div>
  );
}

function TableRows() {
  return (
    <div class="pw-page-skeleton__rows" aria-hidden>
      <For each={[0, 1, 2, 3, 4, 5]}>
        {() => (
          <div class="pw-page-skeleton__row">
            <Skeleton class="h-7 w-7" />
            <Skeleton class="h-3 w-[28%]" />
            <Skeleton class="h-3 w-[18%]" />
            <Skeleton class="h-3 w-[14%]" />
            <Skeleton class="ml-auto h-3 w-[12%]" />
          </div>
        )}
      </For>
    </div>
  );
}

function PageBlocks() {
  return (
    <div class="pw-page-skeleton__blocks" aria-hidden>
      <Skeleton class="h-24 w-full" />
      <Skeleton class="h-16 w-[85%]" />
      <Skeleton class="h-16 w-[70%]" />
    </div>
  );
}

function CanvasPlane() {
  return <Skeleton class="pw-page-skeleton__canvas w-full" aria-hidden />;
}

function DetailBody() {
  return (
    <div class="pw-page-skeleton__detail" aria-hidden>
      <div class="pw-page-skeleton__detail-main">
        <Skeleton class="h-5 w-48" />
        <Skeleton class="h-3 w-32" />
        <Skeleton class="mt-2 h-24 w-full" />
        <Skeleton class="h-3 w-[90%]" />
        <Skeleton class="h-3 w-[75%]" />
      </div>
      <div class="pw-page-skeleton__detail-rail">
        <Skeleton class="h-3 w-20" />
        <Skeleton class="h-8 w-full" />
        <Skeleton class="h-8 w-full" />
        <Skeleton class="h-8 w-full" />
      </div>
    </div>
  );
}

/**
 * Full-panel loading placeholder. Prefer via `PageLoading` in product surfaces.
 */
export function PageSkeleton(props: PageSkeletonProps) {
  const layout = () => props.layout ?? "page";
  const label = () => props.label ?? "Loading…";

  return (
    <div
      class={`pw-page-skeleton ${props.class ?? ""}`}
      role="status"
      aria-live="polite"
      aria-busy="true"
    >
      <span class="sr-only">{label()}</span>
      <Switch>
        <Match when={layout() === "table"}>
          <ToolbarStrip />
          <TableRows />
        </Match>
        <Match when={layout() === "canvas"}>
          <ToolbarStrip />
          <CanvasPlane />
        </Match>
        <Match when={layout() === "detail"}>
          <DetailBody />
        </Match>
        <Match when={layout() === "page"}>
          <ToolbarStrip />
          <PageBlocks />
        </Match>
      </Switch>
    </div>
  );
}
