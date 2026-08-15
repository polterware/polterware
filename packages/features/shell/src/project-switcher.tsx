import { createMemo, createSignal, For, Show } from "solid-js";
import { Popover, Button, MenuItem } from "@polterware/frontend-design-system";
import type { ProjectSwitcherItem } from "./types";
import { PROJECT_SWITCHER_FIXTURES } from "./fixtures";
import { PopoverSearchHeader } from "./popover-search";

export function ProjectSwitcher(props: {
  open: boolean;
  onClose: () => void;
  projects?: ProjectSwitcherItem[];
  onSelect?: (projectId: string) => void;
  onCreateProject?: () => void;
}) {
  const [query, setQuery] = createSignal("");
  const projects = () => props.projects ?? PROJECT_SWITCHER_FIXTURES;
  const filtered = createMemo(() => {
    const q = query().toLowerCase().trim();
    if (!q) return projects();
    return projects().filter((p) => p.name.toLowerCase().includes(q));
  });

  return (
    <Popover open={props.open} onClose={() => props.onClose()} title="Switch project" width={360}>
      <PopoverSearchHeader
        title="Switch project"
        placeholder="Search projects…"
        value={query()}
        onInput={setQuery}
        autofocus
      />

      <div class="max-h-72 overflow-auto p-2">
        <For each={filtered()}>
          {(p) => (
                  <MenuItem
                    selected={p.isCurrent}
                    class="justify-between text-[13px]"
                    onClick={() => {
                      props.onSelect?.(p.projectId);
                      props.onClose();
                    }}
                  >
                    <span class="text-[var(--color-text-primary)]">
                      {p.isCurrent ? "✓ " : "  "}
                      {p.name}
                    </span>
                    <span class="text-[11px] text-[var(--color-text-muted)]">{p.environmentSummary}</span>
                  </MenuItem>
          )}
        </For>
        <Show when={filtered().length === 0}>
          <p class="px-2 py-4 text-center text-[12px] text-[var(--color-text-muted)]">No projects match</p>
        </Show>
      </div>
      <div class="flex flex-col gap-1 border-t border-[var(--color-border-subtle)] p-2">
        <Button
          size="sm"
          variant="ghost"
          class="justify-start"
          onClick={() => {
            props.onCreateProject?.();
            props.onClose();
          }}
        >
          + Create project
        </Button>
      </div>
    </Popover>
  );
}
