import { For, Show, createSignal } from "solid-js";
import {
  HelpCircle,
  MenuItem,
  MenuList,
  Popover,
  SquareTerminal,
} from "@polterware/frontend-design-system";
import { HELP_TOPICS, type HelpTopicId } from "./help-topics";

const headerIconControlClass =
  "control-field control-field-md inline-flex w-[var(--control-height-md)] shrink-0 cursor-pointer items-center justify-center !rounded-full p-0 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] focus-visible:outline-none";

export function HelpMenu(props: {
  onSelectTopic: (topicId: HelpTopicId) => void;
  onOpenCliReference?: () => void;
}) {
  const [open, setOpen] = createSignal(false);
  let triggerRef: HTMLButtonElement | undefined;

  function close() {
    setOpen(false);
  }

  return (
    <>
      <button
        ref={(el) => {
          triggerRef = el;
        }}
        type="button"
        aria-label="Help and about"
        aria-expanded={open()}
        title="Help"
        onClick={() => setOpen((value) => !value)}
        class={headerIconControlClass}
      >
        <HelpCircle size={14} />
      </button>

      <Popover
        open={open()}
        onClose={close}
        title="Help"
        width={260}
        placement="bottom-end"
        anchorRef={() => triggerRef}
      >
        <MenuList class="min-h-0 flex-1 overflow-auto p-2">
          <For each={HELP_TOPICS}>
            {(topic) => {
              const Icon = topic.icon;
              return (
                <MenuItem
                  class="gap-2 py-2 text-[13px]"
                  onClick={() => {
                    close();
                    props.onSelectTopic(topic.id);
                  }}
                >
                  <Icon size={14} class="shrink-0 text-[var(--color-text-secondary)]" />
                  <span class="truncate text-[var(--color-text-primary)]">
                    {topic.label}
                  </span>
                </MenuItem>
              );
            }}
          </For>
          <Show when={props.onOpenCliReference}>
            {(openCliReference) => (
              <MenuItem
                class="gap-2 py-2 text-[13px]"
                onClick={() => {
                  close();
                  openCliReference()();
                }}
              >
                <SquareTerminal size={14} class="shrink-0 text-[var(--color-text-secondary)]" />
                <span class="truncate text-[var(--color-text-primary)]">CLI reference</span>
              </MenuItem>
            )}
          </Show>
        </MenuList>
      </Popover>
    </>
  );
}
