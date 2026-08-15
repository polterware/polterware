import { createSignal, For, Show } from "solid-js";
import { Search } from "@polterware/frontend-design-system";
import { A11yDialog, Button } from "@polterware/frontend-design-system";
import "./commander.css";

export type PaletteItem = {
  id: string;
  label: string;
  group: "Navigation" | "Actions" | "Recent";
  shortcut?: string;
};

const DEFAULT_ITEMS: PaletteItem[] = [
  { id: "nav.architecture", label: "Architecture", group: "Navigation", shortcut: "⌘ 1" },
  { id: "nav.resources", label: "Resources", group: "Navigation" },
  { id: "nav.knowledge", label: "Knowledge", group: "Navigation" },
  { id: "nav.operations", label: "Operations", group: "Navigation", shortcut: "⌘ 2" },
  { id: "nav.terminal", label: "Terminal", group: "Navigation" },
  { id: "nav.integrations", label: "Integrations", group: "Navigation" },
  { id: "nav.settings", label: "Project Settings", group: "Navigation", shortcut: "⌘ ," },
  { id: "action.open-sync", label: "Sync diagnostics", group: "Actions" },
  { id: "action.create-node", label: "Create architecture node", group: "Actions", shortcut: "N" },
  { id: "tool.connect", label: "Connect architecture components", group: "Actions", shortcut: "C" },
  { id: "view.fit_document", label: "Fit architecture to view", group: "Actions", shortcut: "⇧1" },
  { id: "action.run-command", label: "Run command", group: "Actions", shortcut: "⌘ Enter" },
  { id: "action.connect-integration", label: "Connect integration", group: "Actions" },
  { id: "recent.payment", label: "Payment Service", group: "Recent" },
  { id: "recent.postgres", label: "prod-postgres", group: "Recent" },
];

export { SearchResultsPage, SEARCH_HITS, type SearchHit } from "./search-results";

/** Command palette — design-system Dialog chrome (Ark-backed); content only here. */
export function CommandPalette(props: {
  open: boolean;
  onClose: () => void;
  onCommand?: (id: string) => void;
  items?: PaletteItem[];
}) {
  const [q, setQ] = createSignal("");
  const items = () => props.items ?? DEFAULT_ITEMS;
  const filtered = () => {
    const query = q().toLowerCase().trim();
    if (!query) return items();
    return items().filter((c) => c.label.toLowerCase().includes(query));
  };
  const groups = () => {
    const map = new Map<string, PaletteItem[]>();
    for (const it of filtered()) {
      if (!map.has(it.group)) map.set(it.group, []);
      map.get(it.group)!.push(it);
    }
    return [...map.entries()];
  };

  return (
    <A11yDialog
      open={props.open}
      onClose={() => {
        setQ("");
        props.onClose();
      }}
      title="Command palette"
      chrome="flush"
      dividers={false}
    >
      <div class="flex min-h-0 flex-1 flex-col">
        {/* Full-bleed divider: cancel A11yDialog body px-5; keep content inset.
            pr-10: room for the absolute flush close button. Escape still closes via Ark. */}
        <div class="-mx-5 flex shrink-0 items-center gap-2.5 border-b border-[var(--color-border-subtle)] px-5 pb-3 pr-10">
          <Search size={16} class="shrink-0 text-[var(--color-text-muted)]" aria-hidden />
          <input
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
        </div>
        <div class="commander-results-scroll min-h-0 flex-1 overflow-y-auto pt-3 pb-1">
          <For each={groups()}>
            {([group, list]) => (
              <div class="mb-4 last:mb-0">
                <p class="m-0 px-2 py-1 text-[10px] font-medium tracking-wide text-[var(--color-text-disabled)] uppercase">
                  {group}
                </p>
                <For each={list}>
                  {(c) => (
                    <button
                      type="button"
                      class="flex w-full items-center justify-between rounded-[var(--radius-item)] px-2 py-2 text-left text-[13px] text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-primary)]"
                      onClick={() => {
                        props.onCommand?.(c.id);
                        setQ("");
                        props.onClose();
                      }}
                    >
                      <span>{c.label}</span>
                      {c.shortcut ? (
                        <span class="text-[11px] text-[var(--color-text-disabled)]">{c.shortcut}</span>
                      ) : null}
                    </button>
                  )}
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
              onClick={() => {
                props.onCommand?.(`action.search-all:${q()}`);
                setQ("");
                props.onClose();
              }}
            >
              Show all results for “{q()}”
            </button>
          </Show>
        </div>
      </div>
    </A11yDialog>
  );
}

/** @deprecated use CommandPalette */
export function CommanderView(props: { onCommand?: (id: string) => void }) {
  return (
    <section class="p-4">
      <p class="text-[13px] text-[var(--color-text-muted)]">
        Commander is available via ⌘K / Ctrl+K as a command palette.
      </p>
      <Button size="sm" class="mt-2" onClick={() => props.onCommand?.("nav.architecture")}>
        Open Architecture
      </Button>
    </section>
  );
}
