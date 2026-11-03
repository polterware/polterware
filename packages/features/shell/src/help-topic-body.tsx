import { For } from "solid-js";
import type { HelpTopic } from "./help-topics";

export function HelpTopicBody(props: { topic: HelpTopic }) {
  return (
    <div class="grid gap-5">
      <For each={props.topic.body}>
        {(row) => {
          const Icon = row.icon;
          return (
            <div class="flex items-start gap-3">
              <Icon
                size={14}
                class="mt-0.5 shrink-0 text-[var(--color-text-secondary)]"
                aria-hidden
              />
              <p class="m-0 text-[13.5px] leading-relaxed text-[var(--color-text-secondary)]">
                {row.text}
              </p>
            </div>
          );
        }}
      </For>
    </div>
  );
}
