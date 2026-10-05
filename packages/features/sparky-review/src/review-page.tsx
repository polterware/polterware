import { Show, onCleanup, onMount } from "solid-js";
import {
  PageContent, PageEmpty, PageError, PageHeader, PageLoading, PageOffline, PageToolbar,
  SegmentedControl, StatusChip,
} from "@polterware/frontend-design-system";
import { ContributionCalendar } from "./calendar";
import { FocusMinutesChart, WeeklyActivityChart } from "./charts";
import { honestyTone } from "./model";
import { reviewStore, type ReviewStore } from "./store";
import { SummaryCard } from "./summary";
import "./review.css";

const RANGE_OPTIONS = [
  { value: "week", label: "Week" },
  { value: "month", label: "Month" },
  { value: "year", label: "Year" },
];

export function ReviewPage(props: { store?: ReviewStore }) {
  const store = props.store ?? reviewStore;
  onMount(() => onCleanup(store.connect()));

  return (
    <div data-review-page class="sparky-review">
      <PageHeader
        title="Review"
        subtitle={<StatusChip tone={honestyTone(store.loadState())}>{store.statusLabel()}</StatusChip>}
      />
      <PageToolbar.Root>
        <PageToolbar.Leading>
          <SegmentedControl
            aria-label="Review range"
            value={store.range()}
            options={RANGE_OPTIONS}
            onValueChange={details => {
              const next = details.value;
              if (next === "week" || next === "month" || next === "year") store.setRange(next);
            }}
          />
        </PageToolbar.Leading>
      </PageToolbar.Root>
      <PageContent>
        <Show when={store.loadState() === "queued"}><PageLoading label="Loading review" /></Show>
        <Show when={store.loadState() === "unavailable"}>
          <PageOffline title="Review unavailable" message={store.loadError() || "The local core is unreachable."} onRetry={() => void store.reload()} />
        </Show>
        <Show when={store.loadState() === "failed"}>
          <PageError title="Review failed" message={store.loadError()} onRetry={() => void store.reload()} />
        </Show>
        <Show when={store.loadState() === "live" && store.report()}>
          <div class="sparky-review-grid">
            <SummaryCard report={store.report()!} />
            <Show when={store.empty()}>
              <PageEmpty title="No completions yet" description="Complete a memory to start a streak. Focus minutes appear here after a session is recorded." />
            </Show>
            <Show when={!store.empty()}>
              <WeeklyActivityChart days={store.report()!.days ?? []} />
              <ContributionCalendar days={store.report()!.contribution ?? []} weekStart={store.weekStart()} />
              <FocusMinutesChart
                days={store.report()!.days ?? []}
                range={store.range()}
                weekStart={store.weekStart()}
                total={store.report()!.focusMinutes ?? 0}
              />
            </Show>
          </div>
        </Show>
      </PageContent>
    </div>
  );
}
