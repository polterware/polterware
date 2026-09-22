import { For, Show, createEffect, createMemo, createSignal } from "solid-js";
import { Search } from "@polterware/frontend-design-system";
import { A11yDialog, Tabs } from "@polterware/frontend-design-system";
import {
  CATEGORY_LABELS,
  type ComponentCategory,
  type ComponentTypeDefinition,
  getComponentType,
  listCatalogTypes,
  searchCatalogTypes,
} from "../domain/registry";
import type { TypePrefsPort } from "../domain/type-prefs";
import { ArchitectureNodeIcon } from "./architecture-node-icon";
import "./add-to-architecture-dialog.css";

const CATEGORY_ORDER: ComponentCategory[] = [
  "applications",
  "data",
  "infrastructure",
  "development",
  "external",
  "structure",
  "connected",
];

const TAB_ITEMS = [
  { id: "all", label: "All" },
  { id: "applications", label: "Apps" },
  { id: "data", label: "Data" },
  { id: "infrastructure", label: "Infra" },
  { id: "development", label: "Dev" },
  { id: "external", label: "External" },
  { id: "structure", label: "Structure" },
  { id: "connected", label: "Linked" },
];

/**
 * Dense catalog dialog — flush chrome (no title block), search + tabs first,
 * sections and rows separated by spacing only (no hairline dividers).
 */
export function AddToArchitectureDialog(props: {
  open: boolean;
  prefs: TypePrefsPort;
  onClose: () => void;
  onSelect: (def: ComponentTypeDefinition) => void;
  onUnavailableAction?: (def: ComponentTypeDefinition) => void;
}) {
  const [query, setQuery] = createSignal("");
  const [activeTab, setActiveTab] = createSignal("all");
  const [focusIdx, setFocusIdx] = createSignal(0);
  let searchRef: HTMLInputElement | undefined;

  createEffect(() => {
    if (props.open) {
      setQuery("");
      setFocusIdx(0);
      setActiveTab("all");
      queueMicrotask(() => searchRef?.focus());
    }
  });

  const searching = () => query().trim().length > 0;

  const visibleList = createMemo((): ComponentTypeDefinition[] => {
    if (searching()) return searchCatalogTypes(query());
    const tab = activeTab();
    if (tab === "all") return listCatalogTypes();
    return listCatalogTypes().filter((t) => t.category === tab);
  });

  const recents = createMemo(() =>
    props.prefs
      .getRecents()
      .map((id) => getComponentType(id))
      .filter((x): x is ComponentTypeDefinition => Boolean(x)),
  );

  const favorites = createMemo(() =>
    props.prefs
      .getFavorites()
      .map((id) => getComponentType(id))
      .filter((x): x is ComponentTypeDefinition => Boolean(x)),
  );

  function pick(def: ComponentTypeDefinition) {
    if (!def.placeable || def.placementKind === "action") {
      props.onUnavailableAction?.(def);
      return;
    }
    props.prefs.pushRecent(def.id);
    props.onSelect(def);
  }

  function onKeyDown(e: KeyboardEvent) {
    const list = visibleList();
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setFocusIdx((i) => Math.min(list.length - 1, i + 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setFocusIdx((i) => Math.max(0, i - 1));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const item = list[focusIdx()];
      if (item) pick(item);
    }
  }

  return (
    <A11yDialog
      open={props.open}
      title="Add to architecture"
      chrome="flush"
      dividers={false}
      onClose={() => props.onClose()}
      size="xl"
    >
      <div class="-mb-3 flex min-h-0 w-full flex-1 flex-col" onKeyDown={onKeyDown}>
        <div class="-mx-5 flex shrink-0 items-center gap-2.5 border-b border-[var(--color-border-subtle)] px-5 pb-3 pr-10">
          <Search size={16} class="shrink-0 text-[var(--color-text-muted)]" aria-hidden />
          <input
            type="text"
            class="architecture-catalog-search min-w-0 flex-1 border-0 bg-transparent text-[13px] text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)]"
            ref={searchRef}
            role="searchbox"
            placeholder="Search types (postgres, next, cron…)"
            value={query()}
            autocomplete="off"
            spellcheck={false}
            onInput={(e) => {
              setQuery(e.currentTarget.value);
              setFocusIdx(0);
            }}
          />
        </div>

        <div class="relative -mx-5 flex min-h-0 flex-1 flex-col">
          <Show when={!searching()}>
            <div class="architecture-catalog-tabs-fade">
              <Tabs
                items={TAB_ITEMS}
                value={activeTab()}
                onChange={(id) => {
                  setActiveTab(id);
                  setFocusIdx(0);
                }}
                aria-label="Component categories"
                class="architecture-catalog-tabs pw-tabs--flush shrink-0"
              />
            </div>
          </Show>
          <Show when={searching()}>
            <p class="m-0 shrink-0 px-5 py-2 text-[11px] text-[var(--color-text-muted)]">
              Search results · {visibleList().length} match{visibleList().length === 1 ? "" : "es"}
            </p>
          </Show>

          <div
            class="flex min-h-0 flex-1 flex-col gap-6 overflow-y-auto px-5"
            classList={{ "architecture-catalog-list--under-tabs": !searching() }}
          >
          <Show when={!searching() && activeTab() === "all"}>
            <Show when={favorites().length > 0}>
              <Section title="Favorites">
                <For each={favorites()}>
                  {(t) => (
                    <TypeRow
                      def={t}
                      recent={false}
                      favorite
                      onPick={() => pick(t)}
                      onToggleFavorite={() => props.prefs.toggleFavorite(t.id)}
                    />
                  )}
                </For>
              </Section>
            </Show>
            <Show when={recents().length > 0}>
              <Section title="Recent">
                <For each={recents()}>
                  {(t) => (
                    <TypeRow
                      def={t}
                      recent
                      favorite={props.prefs.getFavorites().includes(t.id)}
                      onPick={() => pick(t)}
                      onToggleFavorite={() => props.prefs.toggleFavorite(t.id)}
                    />
                  )}
                </For>
              </Section>
            </Show>
          </Show>

          <Show when={!searching() && activeTab() !== "all"}>
            <Section title={Object.entries(CATEGORY_LABELS).find(([key]) => key === activeTab())?.[1] ?? activeTab()}>
              <For each={visibleList()}>
                {(t, i) => (
                  <TypeRow
                    def={t}
                    recent={props.prefs.getRecents().includes(t.id)}
                    favorite={props.prefs.getFavorites().includes(t.id)}
                    focused={focusIdx() === i()}
                    onPick={() => pick(t)}
                    onToggleFavorite={() => props.prefs.toggleFavorite(t.id)}
                  />
                )}
              </For>
            </Section>
          </Show>

          <Show when={!searching() && activeTab() === "all"}>
            <For each={CATEGORY_ORDER}>
              {(cat) => {
                const items = () => listCatalogTypes().filter((t) => t.category === cat);
                return (
                  <Show when={items().length > 0}>
                    <Section title={CATEGORY_LABELS[cat]}>
                      <For each={items()}>
                        {(t) => (
                          <TypeRow
                            def={t}
                            recent={props.prefs.getRecents().includes(t.id)}
                            favorite={props.prefs.getFavorites().includes(t.id)}
                            onPick={() => pick(t)}
                            onToggleFavorite={() => props.prefs.toggleFavorite(t.id)}
                          />
                        )}
                      </For>
                    </Section>
                  </Show>
                );
              }}
            </For>
          </Show>

          <Show when={searching()}>
            <div class="flex flex-col gap-0.5">
              <For each={visibleList()}>
                {(t, i) => (
                  <TypeRow
                    def={t}
                    recent={props.prefs.getRecents().includes(t.id)}
                    favorite={props.prefs.getFavorites().includes(t.id)}
                    focused={focusIdx() === i()}
                    onPick={() => pick(t)}
                    onToggleFavorite={() => props.prefs.toggleFavorite(t.id)}
                  />
                )}
              </For>
            </div>
            <Show when={visibleList().length === 0}>
              <p class="px-2 py-8 text-center text-[12px] text-[var(--color-text-muted)]">
                No types match.
              </p>
            </Show>
          </Show>
          </div>
        </div>
      </div>
    </A11yDialog>
  );
}

function Section(props: { title: string; children: import("solid-js").JSX.Element }) {
  return (
    <section class="flex flex-col gap-2">
      <h3 class="m-0 px-1 text-[10px] font-medium tracking-wide text-[var(--color-text-muted)] uppercase">
        {props.title}
      </h3>
      <div class="flex flex-col gap-0.5">{props.children}</div>
    </section>
  );
}

function TypeRow(props: {
  def: ComponentTypeDefinition;
  recent?: boolean;
  favorite?: boolean;
  focused?: boolean;
  onPick: () => void;
  onToggleFavorite: () => void;
}) {
  return (
    <div
      class="flex items-center gap-3 rounded-[var(--radius-control)] px-2 py-2 text-left data-[disabled=true]:[&_span]:text-[var(--color-text-disabled)]"
      data-disabled={!props.def.placeable ? "true" : undefined}
      classList={{
        "bg-[var(--color-surface-selected)]": props.focused,
        "hover:bg-[var(--color-surface-hover)]": !props.focused,
      }}
    >
      <button type="button" class="min-w-0 flex-1 text-left" onClick={() => props.onPick()}>
        <div class="flex items-center gap-2.5">
          <span class="inline-flex h-8 w-8 shrink-0 items-center justify-center">
            <ArchitectureNodeIcon
              name={props.def.icon}
              subtype={props.def.subtype}
              provider={props.def.provider}
              compact={false}
              dimmed={!props.def.placeable}
            />
          </span>
          <span class="min-w-0 flex-1">
            <span class="flex items-center gap-2">
              <span class="text-[13px] font-medium text-[var(--color-text-primary)]">
                {props.def.label}
              </span>
              <Show when={props.recent}>
                <span class="text-[10px] text-[var(--color-text-muted)]">Recent</span>
              </Show>
            </span>
            <span class="mt-0.5 block text-[11px] text-[var(--color-text-muted)]">
              {CATEGORY_LABELS[props.def.category]} · {props.def.description}
            </span>
          </span>
        </div>
      </button>
      <Show when={props.def.placeable}>
        <button
          type="button"
          class="shrink-0 px-1.5 text-[14px] text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]"
          title={props.favorite ? "Remove favorite" : "Add favorite"}
          onClick={(e) => {
            e.stopPropagation();
            props.onToggleFavorite();
          }}
        >
          {props.favorite ? "★" : "☆"}
        </button>
      </Show>
    </div>
  );
}
