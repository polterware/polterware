import { createSignal, For, Show, type Accessor, type JSX } from "solid-js";
import { Search } from "@polterware/frontend-design-system";
import {
  capabilityEnabled,
  type PlatformContext,
} from "@polterware/frontend-platform";
import { formatShortcut } from "@polterware/frontend-state";
import { DEFAULT_PALETTE_ITEMS, type PaletteItem } from "./palette-items";
import "./commander.css";

export type PaletteBodyProps = {
  items?: PaletteItem[];
  platform?: PlatformContext;
  onCommand?: (id: string) => void;
  onConsumed?: () => void;
  /** Extra class on the search row (e.g. dialog flush cancel). */
  searchRowClass?: string;
  searchInputRef?: (el: HTMLInputElement) => void;
  /** Fires whenever the search query changes (including clear). */
  onQueryChange?: (query: string) => void;
  /** Content below search, outside the results scroll (e.g. drill-down). */
  belowSearch?: JSX.Element;
  /** Content at the top of the results scroll (e.g. overview cards). */
  scrollLeading?: JSX.Element;
  /** Optional actions after the search input (Open Desktop, Close). */
  trailing?: JSX.Element;
  /** When true, hide the command results list (detail view). */
  hideResults?: boolean;
};

export function usePaletteFilter(
  items: Accessor<PaletteItem[]>,
  onQueryChange?: (query: string) => void,
): {
  q: Accessor<string>;
  setQ: (value: string) => void;
  filtered: Accessor<PaletteItem[]>;
  groups: Accessor<[string, PaletteItem[]][]>;
} {
  const [q, setQState] = createSignal("");
  const setQ = (value: string) => {
    setQState(value);
    onQueryChange?.(value);
  };
  const filtered = () => {
    const query = q().toLowerCase().trim();
    if (!query) return items();
    return items().filter((c) => c.label.toLowerCase().includes(query));
  };
  const groups = () => {
    const map = new Map<string, PaletteItem[]>();
    for (const it of filtered()) {
      if (!map.has(it.group)) map.set(it.group, []);
      const arr = map.get(it.group);
      if (arr) arr.push(it);
    }
    return [...map.entries()];
  };
  return { q, setQ, filtered, groups };
}

/** Shared search + results list for in-app palette and global Commander window. */
export function PaletteBody(props: PaletteBodyProps) {
  const items = () =>
    (props.items ?? DEFAULT_PALETTE_ITEMS).filter((item) =>
      capabilityEnabled(props.platform, item.requiredCapability),
    );
  const { q, setQ, filtered, groups } = usePaletteFilter(items, props.onQueryChange);

  function select(id: string) {
    props.onCommand?.(id);
    setQ("");
    props.onConsumed?.();
  }

  return (
    <div class="flex min-h-0 flex-1 flex-col">
      <div
        class={[
          "commander-search-row flex shrink-0 items-center gap-2 border-b border-[var(--color-border-subtle)] px-5",
          props.searchRowClass ?? "",
        ].join(" ")}
      >
        <Search size={16} class="commander-search-row__icon shrink-0 text-[var(--color-text-muted)]" aria-hidden />
        <input
          ref={props.searchInputRef}
          class="commander-search-input min-w-0 flex-1 border-0 bg-transparent text-[13px] text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)]"
          type="text"
          role="searchbox"
          placeholder="Search projects, resources, commands…"
          value={q()}
          onInput={(e) => setQ(e.currentTarget.value)}
          autofocus
          autocomplete="off"
          spellcheck={false}
        />
        <Show when={props.trailing}>
          <div class="commander-search-row__trailing flex shrink-0 items-center gap-1">
            {props.trailing}
          </div>
        </Show>
      </div>
      {props.belowSearch}
      <Show when={!props.hideResults}>
        <div class="commander-results-scroll min-h-0 flex-1 overflow-y-auto pt-3 pb-1">
          {props.scrollLeading}
          <For each={groups()}>
            {([group, list]) => (
              <div class="mb-4 last:mb-0">
                <p class="m-0 px-2 py-1 text-[10px] font-medium tracking-wide text-[var(--color-text-disabled)] uppercase">
                  {group}
                </p>
                <For each={list}>
                  {(c) => {
                    const hint = c.shortcutId ? formatShortcut(c.shortcutId) : "";
                    return (
                      <button
                        type="button"
                        class="flex w-full items-center justify-between rounded-[var(--radius-item)] px-2 py-2 text-left text-[13px] text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-primary)]"
                        onMouseDown={(e) => {
                          e.preventDefault();
                          select(c.id);
                        }}
                      >
                        <span>{c.label}</span>
                        {hint ? (
                          <span class="text-[11px] text-[var(--color-text-disabled)]">{hint}</span>
                        ) : null}
                      </button>
                    );
                  }}
                </For>
              </div>
            )}
          </For>
          <Show when={filtered().length === 0}>
            <p class="px-2 py-4 text-center text-[12px] text-[var(--color-text-muted)]">No matches</p>
          </Show>
          <Show when={q().trim().length > 0}>
            <button
              type="button"
              class="mt-2 w-full rounded-[var(--radius-item)] px-2 py-2 text-left text-[12px] text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-primary)]"
              onMouseDown={(e) => {
                e.preventDefault();
                select(`action.search-all:${q()}`);
              }}
            >
              Show all results for “{q()}”
            </button>
          </Show>
        </div>
      </Show>
    </div>
  );
}
