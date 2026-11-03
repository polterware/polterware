import {
  createEffect,
  createMemo,
  createSignal,
  createUniqueId,
  Index,
  on,
  onCleanup,
  onMount,
  Show,
  type Accessor,
  type JSX,
} from "solid-js";
import {
  Activity,
  Button,
  Terminal,
  Box,
  Briefcase06Icon,
  centerItemInView,
  ChartRelationshipIcon,
  createFluidHover,
  FluidHoverHighlight,
  History,
  IconTile,
  Inbox,
  Keyboard,
  Layers,
  Plug,
  Plus,
  Pencil,
  prefersReducedMotion,
  PropertyPills,
  Search,
  Settings,
  ShieldCheck,
  spring,
  type ComposerProperty,
  type FluidHover,
  type IconTileTone,
} from "@polterware/frontend-design-system";
import {
  capabilityEnabled,
  type PlatformContext,
} from "@polterware/frontend-platform";
import { SHORTCUT_SEQUENCES, formatSequence, formatShortcut } from "@polterware/frontend-state";
import { animate } from "motion";
import { DEFAULT_PALETTE_ITEMS, type PaletteItem } from "./palette-items";
import "./commander.css";

const PALETTE_TILES: Record<string, { tone: IconTileTone; icon: () => JSX.Element }> = {
  "nav.inbox": { tone: "knowledge", icon: () => <Inbox size={12} /> },
  "nav.canvas": { tone: "neutral", icon: () => <Pencil size={12} /> },
  "nav.structure": { tone: "architecture", icon: () => <ChartRelationshipIcon size={12} /> },
  "nav.milestones": { tone: "execution", icon: () => <Briefcase06Icon size={12} /> },
  "nav.truth": { tone: "knowledge", icon: () => <Layers size={12} /> },
  "nav.activity": { tone: "neutral", icon: () => <History size={12} /> },
  "nav.platforms": { tone: "integration", icon: () => <ShieldCheck size={12} /> },
  "nav.cli-reference": { tone: "identity", icon: () => <Terminal size={12} /> },
  "nav.settings": { tone: "neutral", icon: () => <Settings size={12} /> },
  "nav.app-settings": { tone: "neutral", icon: () => <Settings size={12} /> },
  "action.open-sync": { tone: "execution", icon: () => <Activity size={12} /> },
  "action.create-node": { tone: "architecture", icon: () => <Plus size={12} /> },
  "action.connect-integration": { tone: "integration", icon: () => <Plug size={12} /> },
  "action.show-shortcuts": { tone: "neutral", icon: () => <Keyboard size={12} /> },
};

function paletteTile(id: string, own?: PaletteItem["tile"]): { tone: IconTileTone; seed: string; icon: JSX.Element } {
  const known = own ?? PALETTE_TILES[id];
  if (known) return { tone: known.tone, seed: id, icon: known.icon() };
  return { tone: "architecture", seed: id, icon: <Box size={12} /> };
}

function PaletteItemTile(props: { id: string; tile?: PaletteItem["tile"] }) {
  const tile = () => paletteTile(props.id, props.tile);
  return (
    <IconTile size={20} tone={tile().tone} seed={tile().seed} class="shrink-0">
      {tile().icon}
    </IconTile>
  );
}

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
  /**
   * The query doubles as an agent prompt: once it matches no command it reads as a run, the
   * search icon turns into Run, the run pills open, and Run an agent is the one row, so
   * Enter runs it; Cmd+Enter always does. It selects `action.run-agent:<query>`.
   */
  runAgent?: boolean;
  /** Pills under the search that set how a run goes, such as its route and card; shown for a run only. */
  runProperties?: readonly ComposerProperty[];
  /** Off for an app with no search: a query only narrows the commands, with no Show all results row. */
  searchAll?: boolean;
  /** The search field's placeholder, for an app's own commands. */
  placeholder?: string;
};

/** The command id Run an agent selects; the prompt follows the colon. */
export const RUN_AGENT_PREFIX = "action.run-agent:";

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

/** Returns true when it handled the key (the caller prevents the default). */
type ResultsKeyHandler = (event: KeyboardEvent) => boolean;

/** Shared search + results list for in-app palette and global Commander window. */
export function PaletteBody(props: PaletteBodyProps) {
  const items = () =>
    (props.items ?? DEFAULT_PALETTE_ITEMS).filter((item) =>
      capabilityEnabled(props.platform, item.requiredCapability),
    );
  const { q, setQ, filtered, groups } = usePaletteFilter(items, props.onQueryChange);
  // Text that matches no command is a prompt, not a search.
  const runMode = createMemo(() => Boolean(props.runAgent) && q().trim().length > 0 && filtered().length === 0);
  let body: HTMLDivElement | undefined;
  let runPills: HTMLDivElement | undefined;
  let detailsInput: HTMLTextAreaElement | undefined;
  // A run's details, under its title like the Composer's description.
  const [details, setDetails] = createSignal("");
  // The panel's height the moment before a keystroke may switch search and run.
  let heightBefore = 0;
  let motions: { stop: () => void }[] = [];
  const stopMotions = () => { for (const motion of motions) motion.stop(); motions = []; };
  /** The in-app panel; the global Commander window keeps its own size. */
  const panel = () => body?.closest<HTMLElement>(".pw-dialog-content.commander-panel") ?? undefined;
  const setQuery = (value: string) => {
    heightBefore = panel()?.getBoundingClientRect().height ?? 0;
    setQ(value);
  };
  createEffect(on(runMode, (open, previous) => {
    if (runPills) {
      runPills.hidden = !open;
      runPills.inert = !open;
      runPills.dataset.state = open ? "open" : "closed";
    }
    // First paint snaps; only a switch between search and run moves.
    if (previous === undefined || !body) return;
    morphPanel(open);
  }));
  onCleanup(stopMotions);

  /**
   * Search and run swap their content at once; the panel then glides between the two heights
   * on `spring.slow` (the list's fixed frame, the run's content height), clipping what it
   * has not reached yet, while the incoming content fades in (the run's pills and Start run
   * also rise 6px). An interrupted switch starts from the height it had reached. Under reduced
   * motion the height snaps and only the fade remains.
   */
  function morphPanel(open: boolean) {
    stopMotions();
    const reduced = prefersReducedMotion();
    const incoming = open
      ? [runPills, body!.querySelector<HTMLElement>(".commander-run-footer")]
      : [body!.querySelector<HTMLElement>(".commander-results-scroll")];
    for (const element of incoming) {
      if (!element) continue;
      const rise = open && !reduced;
      motions.push(animate(element, { opacity: [0, 1], ...(rise ? { y: [6, 0] } : {}) }, spring.moderate));
    }
    const frame = panel();
    if (!frame || !heightBefore || reduced) return;
    frame.style.removeProperty("height");
    const to = frame.getBoundingClientRect().height;
    if (Math.abs(to - heightBefore) < 1) return;
    const paint = (value: number) => frame.style.setProperty("height", `${value}px`, "important");
    paint(heightBefore);
    motions.push(animate(heightBefore, to, {
      ...(to < heightBefore ? { type: "tween" as const, ease: "easeOut" as const, ...spring.slow.exit } : spring.slow),
      onUpdate: paint,
      onComplete: () => frame.style.removeProperty("height"),
    }));
  }
  const listId = createUniqueId();
  const [activeRowId, setActiveRowId] = createSignal<string>();
  let searchInput: HTMLInputElement | undefined;
  let handleResultsKey: ResultsKeyHandler | undefined;

  function select(id: string) {
    props.onCommand?.(id);
    setDetails("");
    setQuery("");
    props.onConsumed?.();
  }
  const runAgent = () => {
    const title = q().trim();
    const more = details().trim();
    if (title) select(`${RUN_AGENT_PREFIX}${more ? `${title}\n\n${more}` : title}`);
  };
  /** As in the Composer: Enter in a run's title moves to its details, and Cmd+Enter (Ctrl+Enter off macOS) always starts the run. */
  const handleRunKey = (event: KeyboardEvent) => {
    if (!props.runAgent || event.key !== "Enter" || event.isComposing || event.altKey || event.shiftKey) return false;
    if (event.metaKey || event.ctrlKey) {
      runAgent();
      return true;
    }
    if (!runMode()) return false;
    detailsInput?.focus();
    return true;
  };

  return (
    <div ref={body} class="flex min-h-0 flex-1 flex-col">
      <div
        class={[
          "commander-search-row flex shrink-0 items-center gap-2",
          props.searchRowClass ?? "",
        ].join(" ")}
      >
        <input
          ref={(element) => { searchInput = element; props.searchInputRef?.(element); }}
          class="commander-search-input min-w-0 flex-1 border-0 bg-transparent text-[var(--color-text-primary)] placeholder:text-[var(--color-text-disabled)]"
          type="text"
          role="searchbox"
          placeholder={props.placeholder ?? (props.runAgent ? "Run an agent or search…" : "Search projects, resources, commands…")}
          value={q()}
          onInput={(e) => setQuery(e.currentTarget.value)}
          onKeyDown={(event) => {
            if (handleRunKey(event) || handleResultsKey?.(event)) event.preventDefault();
          }}
          aria-controls={props.hideResults ? undefined : listId}
          aria-activedescendant={props.hideResults ? undefined : activeRowId()}
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
      {/* A run is laid out as Start run's Composer: its details, then the pills, then Start run. */}
      <Show when={props.runAgent}>
        <div ref={runPills} class="commander-run-pills-region pw-composer" hidden>
          <textarea
            ref={detailsInput}
            class="pw-composer__description"
            aria-label="Description"
            rows={3}
            placeholder="Details, files to touch, how to check it…"
            value={details()}
            spellcheck={false}
            onInput={(event) => setDetails(event.currentTarget.value)}
            onKeyDown={(event) => {
              if (handleRunKey(event)) event.preventDefault();
            }}
          />
          <Show when={(props.runProperties?.length ?? 0) > 0}>
            <PropertyPills
              class="pw-composer__properties"
              properties={props.runProperties ?? []}
              // The search field keeps the keys: a pick hands focus back to it.
              onPickerClose={() => requestAnimationFrame(() => searchInput?.focus())}
            />
          </Show>
        </div>
      </Show>
      {props.belowSearch}
      {/* A run lists nothing: like Start run's Composer, its one action is the button at the end. */}
      <Show when={!props.hideResults && !runMode()}>
        <PaletteResults
          listId={listId}
          query={q()}
          groups={groups()}
          leading={props.scrollLeading}
          runAgent={props.runAgent}
          searchAll={props.searchAll}
          onSelect={select}
          onActiveRowId={setActiveRowId}
          bindKeys={(handler) => {
            handleResultsKey = handler;
            onCleanup(() => {
              if (handleResultsKey === handler) handleResultsKey = undefined;
              setActiveRowId(undefined);
            });
          }}
        />
      </Show>
      <Show when={!props.hideResults && runMode()}>
        <footer class="commander-run-footer pw-dialog-footer flex shrink-0 items-center justify-end gap-2 px-7 pt-3 pb-6">
          <Button size="sm" variant="primary" onMouseDown={(event) => event.preventDefault()} onClick={runAgent}>
            Start run
          </Button>
        </footer>
      </Show>
    </div>
  );
}

/**
 * The results list, after fluid-functionalism's CommandMenu: the search field keeps DOM
 * focus and points at the lit row through `aria-activedescendant`. The highlight is the
 * fluid hover fill and nothing else; the pointer moves it, ↑/↓ move it (wrapping), and
 * Enter runs what it sits on. A new row set re-lights the first row, so Enter always has
 * a target, and the pointer leaving keeps the highlight where it was.
 */
function PaletteResults(props: {
  listId: string;
  query: string;
  groups: [string, PaletteItem[]][];
  leading?: JSX.Element;
  runAgent?: boolean;
  searchAll?: boolean;
  onSelect: (id: string) => void;
  onActiveRowId: (id: string | undefined) => void;
  bindKeys: (handler: ResultsKeyHandler) => void;
}) {
  let scroller: HTMLDivElement | undefined;
  const hover = createFluidHover(() => scroller, {
    keepOnLeave: true,
    // Group headings sit between rows; a click there only runs a row it nearly touches.
    gapClick: { maxDistance: 8 },
  });
  const commands = createMemo(() => props.groups.flatMap(([, list]) => list));
  const groupOffsets = createMemo(() => {
    let offset = 0;
    return props.groups.map(([, list]) => {
      const start = offset;
      offset += list.length;
      return start;
    });
  });
  const showSearchAll = () => props.searchAll !== false && props.query.trim().length > 0;
  const rowCount = () => commands().length + (showSearchAll() ? 1 : 0);
  const searchAllIndex = () => commands().length;
  const rowElementId = (index: number) => `${props.listId}-row-${index}`;

  const runRow = (index: number) => {
    const command = commands()[index];
    if (command) props.onSelect(command.id);
    else if (index === searchAllIndex() && showSearchAll()) {
      props.onSelect(`action.search-all:${props.query}`);
    }
  };

  const moveTo = (index: number) => {
    hover.releasePointer();
    hover.setActiveIndex(index);
    const rect = hover.rects()[index];
    if (scroller && rect) centerItemInView(scroller, rect, { toTop: index === 0 });
  };

  // A changed row set re-lights the first row and returns to the top.
  const rowSetKey = createMemo(() =>
    [...commands().map((command) => command.id), showSearchAll() ? "\0search-all" : ""].join("\0"),
  );
  createEffect(
    on(rowSetKey, () => {
      hover.setActiveIndex(rowCount() > 0 ? 0 : null);
      if (scroller) scroller.scrollTop = 0;
    }),
  );

  createEffect(() => {
    const index = hover.activeIndex();
    props.onActiveRowId(index === null ? undefined : rowElementId(index));
  });

  onMount(() => {
    props.bindKeys((event) => {
      // IME composition and modified arrows keep their editing meaning.
      if (event.isComposing || event.keyCode === 229) return false;
      if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return false;
      const count = rowCount();
      if (event.key === "ArrowDown" || event.key === "ArrowUp") {
        if (count === 0) return false;
        const current = hover.activeIndex();
        const step = event.key === "ArrowDown" ? 1 : -1;
        moveTo(current === null ? (step > 0 ? 0 : count - 1) : (current + step + count) % count);
        return true;
      }
      if (event.key === "Enter") {
        const current = hover.activeIndex();
        if (current === null) return false;
        runRow(current);
        return true;
      }
      return false;
    });
  });

  const rowClass =
    "relative flex w-full items-center justify-between rounded-[var(--radius-item)] px-2 py-1.5 text-left text-[var(--color-text-secondary)] transition-colors duration-[var(--duration-fast)] data-[fluid-hover-active]:text-[var(--color-text-primary)]";

  return (
    <div
      ref={scroller}
      class="commander-results-scroll pw-scroll-fade relative mt-2.5 min-h-0 flex-1 overflow-y-auto pt-1 pb-3"
      {...hover.handlers}
    >
      <FluidHoverHighlight hover={hover} class="commander-result-highlight" />
      {props.leading}
      {/* Mousedown anywhere in the list keeps focus in the search field. */}
      <div id={props.listId} role="listbox" aria-label="Commands" onMouseDown={(event) => event.preventDefault()}>
        <Index each={props.groups}>
          {(group, groupIndex) => {
            const headingId = `${props.listId}-group-${groupIndex}`;
            return (
              <div class="mb-2 last:mb-0" role="group" aria-labelledby={headingId}>
                <p
                  id={headingId}
                  class="relative m-0 px-2 py-1 text-[10px] font-medium tracking-wide text-[var(--color-text-disabled)] uppercase"
                >
                  {group()[0]}
                </p>
                <Index each={group()[1]}>
                  {(command, index) => {
                    const rowIndex = () => (groupOffsets()[groupIndex] ?? 0) + index;
                    const hint = () => {
                      if (command().shortcutId) return formatShortcut(command().shortcutId!);
                      const sequence = SHORTCUT_SEQUENCES.find((item) => item.id === command().sequenceId);
                      return sequence ? formatSequence(sequence.keys) : "";
                    };
                    return (
                      <PaletteRow
                        hover={hover}
                        index={rowIndex()}
                        id={rowElementId(rowIndex())}
                        class={`${rowClass} text-[13px]`}
                        onRun={() => runRow(rowIndex())}
                      >
                        <span class="flex min-w-0 items-center gap-2">
                          <PaletteItemTile id={command().id} tile={command().tile} />
                          <span class="min-w-0 truncate">{command().label}</span>
                        </span>
                        <Show when={hint()}>
                          <span class="shrink-0 text-[11px] text-[var(--color-text-disabled)]">{hint()}</span>
                        </Show>
                      </PaletteRow>
                    );
                  }}
                </Index>
              </div>
            );
          }}
        </Index>
        <Show when={commands().length === 0 && !props.runAgent}>
          <p role="presentation" class="px-2 py-4 text-center text-[12px] text-[var(--color-text-muted)]">
            No matches
          </p>
        </Show>
        <Show when={showSearchAll()}>
          <PaletteRow
            hover={hover}
            index={searchAllIndex()}
            id={rowElementId(searchAllIndex())}
            class={`${rowClass} mt-2 text-[12px]`}
            onRun={() => runRow(searchAllIndex())}
          >
            <span class="flex min-w-0 items-center gap-2">
              <IconTile size={20} tone="neutral" seed="commander-search" class="shrink-0">
                <Search size={12} />
              </IconTile>
              <span class="min-w-0 truncate">Show all results for “{props.query}”</span>
            </span>
          </PaletteRow>
        </Show>
      </div>
    </div>
  );
}

function PaletteRow(props: {
  hover: FluidHover;
  index: number;
  id: string;
  class: string;
  onRun: () => void;
  children: JSX.Element;
}) {
  let element: HTMLButtonElement | undefined;
  onMount(() => {
    const row = element;
    if (!row) return;
    // Re-joins at its new position when filtering moves it.
    createEffect(() => onCleanup(props.hover.registerItem(props.index, row)));
  });
  return (
    <button
      ref={element}
      type="button"
      role="option"
      tabIndex={-1}
      id={props.id}
      aria-selected={props.hover.activeIndex() === props.index}
      class={props.class}
      onClick={() => props.onRun()}
    >
      {props.children}
    </button>
  );
}
