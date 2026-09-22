import { For, Match, Show, Switch } from "solid-js";
import { ChevronLeft, StatusChip } from "@polterware/frontend-design-system";
import {
  OVERVIEW_OPEN_COMMAND,
  OVERVIEW_TITLES,
  type OverviewDetailPayload,
  type OverviewFeatureRow,
  type OverviewId,
  type OverviewIntegrationRow,
} from "./overview-types";
import "./overview-detail.css";

function featureTone(status: string): "healthy" | "warn" | "muted" {
  if (status === "ready") return "healthy";
  if (status === "draft") return "warn";
  return "muted";
}

function integrationTone(status: string): "healthy" | "danger" | "syncing" | "muted" | "warn" {
  const value = status.toLowerCase();
  if (value === "connected") return "healthy";
  if (value === "failed") return "danger";
  if (value === "authenticating") return "syncing";
  if (value === "ready") return "warn";
  return "muted";
}

function DetailRow(props: {
  title: string;
  meta: string;
  status: string;
  tone: "healthy" | "syncing" | "danger" | "muted" | "warn";
}) {
  return (
    <div class="commander-detail-row">
      <div class="commander-detail-row__text">
        <p class="commander-detail-row__title">{props.title}</p>
        <p class="commander-detail-row__meta">{props.meta}</p>
      </div>
      <StatusChip tone={props.tone} label={props.status} />
    </div>
  );
}

/** Drill-down list for one overview card inside the global Commander. */
export function OverviewDetail(props: {
  cardId: OverviewId;
  detail: OverviewDetailPayload;
  onBack: () => void;
  onOpenDesktop: (commandId: string) => void;
}) {
  const title = () => OVERVIEW_TITLES[props.cardId];
  const openCommand = () => OVERVIEW_OPEN_COMMAND[props.cardId];

  const features = () => props.detail.features;
  const integrations = () => props.detail.integrations;

  const empty = () => {
    switch (props.cardId) {
      case "knowledge":
        return features().length === 0;
      case "integrations":
        return integrations().length === 0;
    }
  };

  return (
    <div class="commander-detail [-webkit-app-region:no-drag]">
      <div class="commander-detail__toolbar">
        <button
          type="button"
          class="commander-detail__back"
          onMouseDown={(event) => {
            event.preventDefault();
            props.onBack();
          }}
        >
          <ChevronLeft size={16} aria-hidden />
          <span>{title()}</span>
        </button>
        <button
          type="button"
          class="commander-detail__open"
          onMouseDown={(event) => {
            event.preventDefault();
            props.onOpenDesktop(openCommand());
          }}
        >
          Open in Desktop
        </button>
      </div>

      <div class="commander-detail__list">
        <Show
          when={!empty()}
          fallback={
            <p class="commander-detail__empty">Nothing to show for this project yet.</p>
          }
        >
          <Switch>
            <Match when={props.cardId === "knowledge"}>
              <For each={features()}>
                {(row: OverviewFeatureRow) => (
                  <DetailRow
                    title={row.title}
                    meta={row.kind === "constitution" ? "Constitution" : "Spec feature"}
                    status={row.status}
                    tone={featureTone(row.status)}
                  />
                )}
              </For>
            </Match>
            <Match when={props.cardId === "integrations"}>
              <For each={integrations()}>
                {(row: OverviewIntegrationRow) => (
                  <DetailRow
                    title={row.name}
                    meta={row.label}
                    status={row.status}
                    tone={integrationTone(row.status)}
                  />
                )}
              </For>
            </Match>
          </Switch>
        </Show>
      </div>
    </div>
  );
}
