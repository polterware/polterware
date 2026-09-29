import { Show, type JSX } from "solid-js";
import { Card } from "@polterware/frontend-design-system";
import { settingsSectionId } from "./settings-search";

/** Titled group of settings: a quiet heading above one card. */
export function SettingsSection(props: {
  title: string;
  description?: string;
  children: JSX.Element;
}) {
  return (
    <section id={settingsSectionId(props.title)} class="flex scroll-mt-2 flex-col gap-2">
      <div class="px-1">
        <h3 class="m-0 text-[12px] font-medium text-[var(--color-text-muted)]">{props.title}</h3>
        <Show when={props.description}>
          <p class="m-0 mt-0.5 text-[11.5px] text-[var(--color-text-muted)]">{props.description}</p>
        </Show>
      </div>
      <Card class="flex flex-col gap-4 !p-4">{props.children}</Card>
    </section>
  );
}

/** One setting: label and optional hint on the left, control on the right. */
export function SettingsRow(props: {
  label: JSX.Element;
  description?: JSX.Element;
  for?: string;
  children?: JSX.Element;
}) {
  return (
    <div class="flex items-center justify-between gap-4">
      <div class="min-w-0 flex-1">
        <label
          for={props.for}
          class="block text-[12.5px] text-[var(--color-text-primary)]"
        >
          {props.label}
        </label>
        <Show when={props.description}>
          <p class="m-0 mt-0.5 text-[11.5px] text-[var(--color-text-muted)]">{props.description}</p>
        </Show>
      </div>
      <Show when={props.children}>
        <div class="flex shrink-0 items-center gap-2">{props.children}</div>
      </Show>
    </div>
  );
}
