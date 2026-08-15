import { createMemo, createSignal, For, Show } from "solid-js";
import { HardDrive } from "@polterware/frontend-design-system";
import { MenuItem, Popover, StatusChip } from "@polterware/frontend-design-system";
import type { TerminalTargetItem } from "./types";
import {
  isTerminalTargetUsable,
  terminalTargetStatusLabel,
  terminalTargetStatusTone,
} from "./types";
import { TERMINAL_TARGET_FIXTURES } from "./fixtures";
import { PopoverSearchField } from "./popover-search";

export function TargetPicker(props: {
  open: boolean;
  onClose: () => void;
  targets?: TerminalTargetItem[];
  environmentId?: string;
  onSelect: (targetId: string) => void;
  /** Called when operator activates an unusable target (honest feedback path). */
  onUnusableSelect?: (target: TerminalTargetItem) => void;
  /** Terminal header button — required for Ark placement after portal. */
  anchorRef?: () => HTMLElement | undefined | null;
}) {
  const [query, setQuery] = createSignal("");
  const targets = () => props.targets ?? TERMINAL_TARGET_FIXTURES;

  const filtered = createMemo(() => {
    let list = targets();
    if (props.environmentId) {
      const envTargets = list.filter((t) => !t.environmentId || t.environmentId === props.environmentId);
      if (envTargets.length > 0) list = envTargets;
    }
    const q = query().toLowerCase().trim();
    if (!q) return list;
    return list.filter((t) => t.name.toLowerCase().includes(q) || t.shell.toLowerCase().includes(q));
  });

  function choose(target: TerminalTargetItem) {
    if (!isTerminalTargetUsable(target.status)) {
      props.onUnusableSelect?.(target);
      // Keep picker open so the operator can choose another target after feedback.
      return;
    }
    props.onSelect(target.id);
    setQuery("");
    props.onClose();
  }

  return (
    <Popover
      open={props.open}
      onClose={() => {
        setQuery("");
        props.onClose();
      }}
      title="Open terminal"
      width={300}
      align="end"
      placement="bottom-end"
      anchorRef={props.anchorRef}
    >
      <PopoverSearchField
        placeholder="Find target…"
        value={query()}
        onInput={setQuery}
        autofocus
      />

      <div class="max-h-72 overflow-auto p-1.5">
        <Show
          when={targets().length > 0}
          fallback={
            <p class="px-2 py-4 text-center text-[12px] text-[var(--color-text-muted)]">
              No targets available
            </p>
          }
        >
          <For each={filtered()}>
            {(t) => {
              const usable = () => isTerminalTargetUsable(t.status);
              return (
                <MenuItem
                  class="py-2"
                  aria-disabled={!usable() || undefined}
                  data-unusable={!usable() ? "true" : undefined}
                  title={
                    usable()
                      ? `Open session on ${t.name}`
                      : `Unavailable: ${terminalTargetStatusLabel(t.status)} — will not open a session`
                  }
                  onClick={() => choose(t)}
                >
                  <HardDrive
                    size={14}
                    class="shrink-0"
                    style={{
                      color: usable()
                        ? "var(--color-text-muted)"
                        : "var(--color-text-disabled)",
                    }}
                  />
                  <div class="min-w-0 flex-1">
                    <p
                      class={`m-0 truncate text-[13px] ${
                        usable()
                          ? "text-[var(--color-text-primary)]"
                          : "text-[var(--color-text-disabled)]"
                      }`}
                    >
                      {t.name}
                    </p>
                    <p class="m-0 truncate text-[11px] text-[var(--color-text-muted)]">
                      {usable() ? t.shell : `Cannot open · ${terminalTargetStatusLabel(t.status)}`}
                    </p>
                  </div>
                  <StatusChip
                    tone={terminalTargetStatusTone(t.status)}
                    label={terminalTargetStatusLabel(t.status)}
                  />
                </MenuItem>
              );
            }}
          </For>
          <Show when={filtered().length === 0 && targets().length > 0}>
            <p class="px-2 py-4 text-center text-[12px] text-[var(--color-text-muted)]">
              No targets match
            </p>
          </Show>
        </Show>
      </div>
    </Popover>
  );
}
