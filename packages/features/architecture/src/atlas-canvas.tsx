import { For, Show, createEffect, createMemo, createSignal, on, onCleanup, untrack } from "solid-js";
import {
  animateCollapse,
  Button,
  ChevronRight,
  DetailPanel,
  Eye,
  ListChecks,
  PageContent,
  PageEmpty,
  PageLoading,
  PageToolbar,
  SchemeIcon,
  Select,
  type SelectOption,
  StatusChip,
  Tabs,
  type CollapseControls,
  type StatusTone,
} from "@polterware/frontend-design-system";
import { detailPanelStore, createDetailPanelSelector } from "@polterware/frontend-state";
import "./atlas-tree.css";
import {
  buildTree,
  entityIdOf,
  expandableKeys,
  isFolder,
  pathTo,
  visibleItems,
  type TreeItem,
} from "./atlas-tree";
import { ARCHITECTURE_TAB_ITEMS, normalizeArchitectureTab, type ArchitectureTabId } from "./tab";

const SCOPE = "atlas:hierarchy";

export type AtlasCanvasNode = {
  id: string;
  parentId: string;
  level: "system" | "domain" | "component";
  slug: string;
  path: string;
  title: string;
  intent: string;
  decisions: string[];
  invariants: { id: string; text: string }[];
  status: "proposed" | "in_progress" | "delivered" | "drift";
  childCount: number;
  featureCount: number;
  openQuestions: number;
  repo: string;
  folder: string;
  migrationFrom: string;
  migrationTo: string;
};

export type AtlasCanvasCard = {
  slug: string;
  title: string;
  intent: string;
  stage: string;
  kind: "feature" | "architecture";
  under: string;
  decisions: string[];
  tasks: { title: string; state: string }[];
  questions: { text: string; blocking: boolean; state: string }[];
  invariants: { text: string }[];
};

const STATUS_LABEL: Record<AtlasCanvasNode["status"], string> = {
  proposed: "Proposed",
  in_progress: "In progress",
  delivered: "Delivered",
  drift: "Drift",
};

const STATUS_TONE: Record<AtlasCanvasNode["status"], StatusTone> = {
  proposed: "muted",
  in_progress: "syncing",
  delivered: "healthy",
  drift: "danger",
};

const LEGEND: AtlasCanvasNode["status"][] = ["proposed", "in_progress", "delivered", "drift"];

/** A node that is moving between designs; separate from the derived status. */
function migrationLabel(node: AtlasCanvasNode): string {
  return node.migrationFrom || node.migrationTo ? `Migrating: ${node.migrationFrom} → ${node.migrationTo}` : "";
}

function cardStatus(card: AtlasCanvasCard, driftSlugs: readonly string[]): AtlasCanvasNode["status"] {
  if (driftSlugs.includes(card.slug)) return "drift";
  if (card.stage === "delivered") return "delivered";
  return "in_progress";
}

function nodeTarget(id: string) {
  return { scopeId: SCOPE, entityId: `node:${id}`, entityKind: "atlas-node" };
}

function cardTarget(slug: string) {
  return { scopeId: SCOPE, entityId: `card:${slug}`, entityKind: "atlas-card" };
}

export function AtlasCanvas(props: {
  projectName: string;
  nodes: AtlasCanvasNode[];
  driftSlugs: string[];
  cards: AtlasCanvasCard[];
  loading?: boolean;
  tab: string;
  onTabChange: (tab: ArchitectureTabId) => void;
  /** True for a replayed past graph: status chips are hidden and the page is read-only. */
  historical?: boolean;
  /** Timeline choices ("Now" first); omitted when there is no history to pick from. */
  timeline?: { value: string; options: readonly SelectOption[]; onChange: (value: string) => void };
}) {
  const [collapsed, setCollapsed] = createSignal<ReadonlySet<string>>(new Set());
  const activeTarget = createDetailPanelSelector((state) => state.activeTarget);

  const byId = createMemo(() => new Map(props.nodes.map((node) => [node.id, node])));

  const tree = createMemo(() => buildTree(props.nodes, props.cards));

  // Finder-style navigation: the list shows the children of the entered folder. A folder that vanished (for
  // example on a timeline change) falls back to the root.
  const [zoomKey, setZoomKey] = createSignal("");
  const trail = createMemo(() => (zoomKey() ? pathTo(tree(), zoomKey()) : []));
  const roots = createMemo(() => trail().at(-1)?.children ?? tree());
  const expandable = createMemo(() => expandableKeys(roots()));

  function toggle(id: string) {
    setCollapsed((current) => {
      const next = new Set(current);
      if (!next.delete(id)) next.add(id);
      return next;
    });
  }

  const panelId = () =>
    activeTarget()?.scopeId === SCOPE ? activeTarget()?.entityId ?? "" : "";

  const selectedNode = createMemo(() => {
    const id = panelId();
    return id.startsWith("node:") ? byId().get(id.slice(5)) : undefined;
  });

  const selectedCard = createMemo(() => {
    const id = panelId();
    if (!id.startsWith("card:")) return undefined;
    return props.cards.find((card) => card.slug === id.slice(5));
  });

  const [bulk, setBulk] = createSignal(false);
  function setAll(next: ReadonlySet<string>) {
    setBulk(true);
    setCollapsed(next);
    requestAnimationFrame(() => requestAnimationFrame(() => setBulk(false)));
  }

  createEffect(() => {
    detailPanelStore.getState().registerCollection({
      scopeId: SCOPE,
      status: props.loading ? "loading" : "ready",
      targets: visibleItems(roots(), collapsed()).map((item) =>
        item.kind === "node" ? nodeTarget(item.node.id) : cardTarget(item.card.slug),
      ),
    });
  });

  onCleanup(() => {
    detailPanelStore.getState().unregisterCollection(SCOPE);
  });

  function select(item: TreeItem) {
    detailPanelStore
      .getState()
      .activateTarget(item.kind === "node" ? nodeTarget(item.node.id) : cardTarget(item.card.slug));
  }

  function open(item: TreeItem) {
    if (isFolder(item)) setZoomKey(item.key);
  }

  const tab = () => normalizeArchitectureTab(props.tab);

  return (
    <>
      <div class="flex h-full min-h-0 flex-col">
        <PageToolbar.Root>
          <PageToolbar.Tabs>
            <Tabs
              items={ARCHITECTURE_TAB_ITEMS}
              value={tab()}
              onChange={(id) => props.onTabChange(normalizeArchitectureTab(id))}
              aria-label="Atlas"
            />
          </PageToolbar.Tabs>
          <PageToolbar.Actions>
            <Button
              size="sm"
              variant="ghost"
              disabled={props.nodes.length === 0}
              onClick={() => setAll(new Set())}
            >
              Expand all
            </Button>
            <Button
              size="sm"
              variant="ghost"
              disabled={props.nodes.length === 0}
              onClick={() => setAll(new Set(expandable()))}
            >
              Collapse all
            </Button>
            <Show when={props.timeline}>
              {(timeline) => (
                <Select
                  size="sm"
                  aria-label="Atlas timeline"
                  class="w-56"
                  value={timeline().value}
                  options={timeline().options}
                  onChange={timeline().onChange}
                />
              )}
            </Show>
          </PageToolbar.Actions>
        </PageToolbar.Root>
        <PageContent class="flex min-h-0 flex-1 flex-col gap-4 overflow-auto p-4">
          <Show
            when={!props.loading}
            fallback={<PageLoading layout="page" label="Loading Atlas…" />}
          >
            <Show
              when={props.nodes.length > 0}
              fallback={
                <PageEmpty
                  title="No Atlas nodes yet"
                  description="Write system, domain, and component nodes with polterware apply. Feature cards stay on the Work board."
                />
              }
            >
              <Show when={props.historical}>
                <p class="m-0 text-[12.5px] text-[var(--color-text-muted)]" role="status">
                  Read-only past view. Structure, contract, decisions, and migration state are replayed; status,
                  features, and drift are live only.
                </p>
              </Show>

              <ul class="m-0 flex list-none flex-wrap gap-2 p-0" aria-label="Status legend">
                <Show when={!props.historical}>
                  <For each={LEGEND}>
                    {(status) => (
                      <li>
                        <StatusChip tone={STATUS_TONE[status]} label={STATUS_LABEL[status]} />
                      </li>
                    )}
                  </For>
                </Show>
                <li>
                  <StatusChip tone="warn" label="Migrating: old → new" />
                </li>
              </ul>

              <nav aria-label="Atlas path">
                <ol class="m-0 flex list-none flex-wrap items-center gap-1 p-0 text-[12.5px]">
                  <Crumb label={props.projectName} current={trail().length === 0} onClick={() => setZoomKey("")} />
                  <For each={trail()}>
                    {(item, index) => (
                      <Crumb
                        label={item.kind === "node" ? item.node.title : item.card.title}
                        current={index() === trail().length - 1}
                        onClick={() => setZoomKey(item.key)}
                      />
                    )}
                  </For>
                </ol>
              </nav>

              <ul class="m-0 grid list-none p-0" role="tree" aria-label="Atlas hierarchy" data-bulk={String(bulk())}>
                <For each={roots()}>
                  {(item, index) => (
                    <TreeNode
                      item={item}
                      index={index()}
                      depth={0}
                      collapsed={collapsed}
                      bulk={bulk}
                      selectedId={panelId}
                      historical={props.historical ?? false}
                      driftSlugs={props.driftSlugs}
                      onToggle={toggle}
                      onOpen={open}
                      onDetails={select}
                    />
                  )}
                </For>
              </ul>
            </Show>
          </Show>
        </PageContent>
      </div>
      <AtlasPanel
        node={selectedNode()}
        card={selectedCard()}
        driftSlugs={props.driftSlugs}
        cards={props.cards}
        historical={props.historical ?? false}
      />
    </>
  );
}

type TreeContext = {
  collapsed: () => ReadonlySet<string>;
  bulk: () => boolean;
  selectedId: () => string;
  historical: boolean;
  driftSlugs: string[];
  onToggle: (key: string) => void;
  /** Click on the card: enters a folder; a leaf does nothing. Details open only from the Eye button. */
  onOpen: (item: TreeItem) => void;
  onDetails: (item: TreeItem) => void;
};

/** A node and, below it, its group of child rows. The group springs to its measured height (`animateCollapse`). */
function TreeNode(props: TreeContext & { item: TreeItem; index: number; depth: number }) {
  const open = () => props.item.hasChildren && !props.collapsed().has(props.item.key);
  let group: HTMLDivElement | undefined;
  let running: CollapseControls | undefined;

  const reducedMotion = () => window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  createEffect(
    on(
      open,
      (isOpen) => {
        if (!group) return;
        running = animateCollapse(group, isOpen, {
          previous: running,
          instant: props.bulk() || reducedMotion(),
        });
      },
      { defer: true },
    ),
  );

  return (
    <li
      role="treeitem"
      aria-level={props.depth + 1}
      aria-expanded={props.item.hasChildren ? open() : undefined}
      aria-selected={props.selectedId() === entityIdOf(props.item)}
      style={{ "--i": Math.min(props.index, 5) }}
    >
      <TreeRow
        item={props.item}
        historical={props.historical}
        driftSlugs={props.driftSlugs}
        expanded={open()}
        selected={props.selectedId() === entityIdOf(props.item)}
        onToggle={() => props.onToggle(props.item.key)}
        onOpen={() => props.onOpen(props.item)}
        onDetails={() => props.onDetails(props.item)}
      />
      <Show when={props.item.hasChildren}>
        <div
          class="atlas-tree-group"
          ref={(el) => {
            group = el;
            const initiallyOpen = untrack(open);
            el.hidden = !initiallyOpen;
            el.inert = !initiallyOpen;
            el.dataset.state = initiallyOpen ? "open" : "closed";
          }}
        >
          <ul class="m-0 grid list-none gap-1 p-0" role="group">
            <For each={props.item.children}>
              {(child, index) => <TreeNode {...props} item={child} index={index()} depth={props.depth + 1} />}
            </For>
          </ul>
        </div>
      </Show>
    </li>
  );
}

const ROW_BUTTON =
  "flex min-w-0 flex-1 items-center gap-3 rounded-[var(--radius-control)] border px-3 py-2 text-left transition-[background,border-color] duration-[var(--duration-control)] ease-[var(--ease-standard)] [@media(hover:hover)]:hover:bg-[var(--color-surface-hover)] focus-visible:outline focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-[var(--color-border-strong)]";

const ICON_BUTTON =
  "flex shrink-0 items-center justify-center rounded-[var(--radius-control)] text-[var(--color-text-muted)] transition-[background,color] duration-[var(--duration-control)] ease-[var(--ease-standard)] [@media(hover:hover)]:hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-primary)] focus-visible:outline focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-[var(--color-border-strong)]";

const CRUMB_BUTTON = `${ICON_BUTTON} px-1.5 py-0.5`;

/** The card-style row: a chevron when there is something to expand, then the status, title, and chips. */
function TreeRow(props: {
  item: TreeItem;
  historical: boolean;
  driftSlugs: string[];
  expanded: boolean;
  selected: boolean;
  onToggle: () => void;
  onOpen: () => void;
  onDetails: () => void;
}) {
  const status = () =>
    props.item.kind === "node" ? props.item.node.status : cardStatus(props.item.card, props.driftSlugs);
  const title = () => (props.item.kind === "node" ? props.item.node.title : props.item.card.title);
  const subtitle = () =>
    props.item.kind === "node"
      ? `${props.item.node.level} · ${props.item.node.path}`
      : `${props.item.card.kind === "architecture" ? "architecture change" : "feature"} · ${props.item.card.slug}`;
  const migration = () => (props.item.kind === "node" ? migrationLabel(props.item.node) : "");
  return (
    <div class="flex items-stretch gap-1">
      <Show when={props.item.hasChildren} fallback={<span class="w-6 shrink-0" aria-hidden="true" />}>
        <button
          type="button"
          class="flex w-6 shrink-0 items-center justify-center rounded-[var(--radius-control)] text-[var(--color-text-muted)] transition-[background,color] duration-[var(--duration-control)] ease-[var(--ease-standard)] [@media(hover:hover)]:hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-primary)] focus-visible:outline focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-[var(--color-border-strong)]"
          aria-label={`${props.expanded ? "Collapse" : "Expand"} ${title()}`}
          onClick={props.onToggle}
        >
          <ChevronRight
            size={14}
            aria-hidden
            class="transition-transform duration-[var(--duration-control)] ease-[var(--ease-enter)] motion-reduce:transition-none"
            classList={{ "rotate-90": props.expanded }}
          />
        </button>
      </Show>
      <button
        type="button"
        class={ROW_BUTTON}
        classList={{
          "border-[var(--color-border-strong)] bg-[var(--color-surface-hover)]": props.selected,
          "border-[var(--color-border-subtle)] bg-[var(--color-surface)]": !props.selected,
        }}
        aria-current={props.selected ? "true" : undefined}
        onClick={props.onOpen}
      >
        <Show when={!(props.historical && props.item.kind === "node")}>
          <StatusChip tone={STATUS_TONE[status()]} label={STATUS_LABEL[status()]} />
        </Show>
        <span class="min-w-0 flex-1">
          <span class="block truncate text-[13px] font-medium text-[var(--color-text-primary)]">{title()}</span>
          <span class="block truncate text-[11px] text-[var(--color-text-muted)]">{subtitle()}</span>
        </span>
        <Show when={migration()}>
          <StatusChip tone="warn" label={migration()} />
        </Show>
        <Show when={props.item.kind === "card" && props.item.card.kind === "architecture"}>
          <StatusChip tone="muted" label="Architecture change" />
        </Show>
      </button>
      <button
        type="button"
        class={`${ICON_BUTTON} w-8`}
        aria-label={`Open details for ${title()}`}
        onClick={props.onDetails}
      >
        <Eye size={14} aria-hidden />
      </button>
    </div>
  );
}

function Crumb(props: { label: string; current: boolean; onClick: () => void }) {
  return (
    <li class="flex items-center gap-1">
      <Show when={props.current} fallback={<button type="button" class={CRUMB_BUTTON} onClick={props.onClick}>{props.label}</button>}>
        <span class="px-1.5 py-0.5 font-medium text-[var(--color-text-primary)]" aria-current="page">
          {props.label}
        </span>
      </Show>
      <Show when={!props.current}>
        <ChevronRight size={12} aria-hidden class="text-[var(--color-text-muted)]" />
      </Show>
    </li>
  );
}

function AtlasPanel(props: {
  node: AtlasCanvasNode | undefined;
  card: AtlasCanvasCard | undefined;
  driftSlugs: string[];
  cards: AtlasCanvasCard[];
  historical: boolean;
}) {
  return (
    <DetailPanel.Content>
      <Show
        when={props.card}
        fallback={
          <Show when={props.node}>
            {(node) => <NodePanel node={node()} cards={props.cards} historical={props.historical} />}
          </Show>
        }
      >
        {(card) => <CardPanel card={card()} driftSlugs={props.driftSlugs} />}
      </Show>
    </DetailPanel.Content>
  );
}

function NodePanel(props: { node: AtlasCanvasNode; cards: AtlasCanvasCard[]; historical: boolean }) {
  const work = createMemo(() =>
    props.cards.filter((card) => card.under === props.node.path),
  );
  return (
    <>
      <DetailPanel.Header icon={<SchemeIcon size={16} />} title={props.node.title}>
        <Show when={!props.historical}>
          <StatusChip tone={STATUS_TONE[props.node.status]} label={STATUS_LABEL[props.node.status]} />
        </Show>
        <Show when={migrationLabel(props.node)}>
          <StatusChip tone="warn" label={migrationLabel(props.node)} />
        </Show>
      </DetailPanel.Header>
      <DetailPanel.Body>
        <DetailPanel.Main>
          <DetailPanel.Section title="Intent">
            <Show
              when={props.node.intent}
              fallback={<p class="pw-detail-panel__text" data-empty>No intent yet.</p>}
            >
              <p class="pw-detail-panel__text whitespace-pre-wrap">{props.node.intent}</p>
            </Show>
          </DetailPanel.Section>
          <DetailPanel.Section title="Contract">
            <TextList items={props.node.invariants.map((item) => item.text)} empty="No invariants yet." />
            <Show when={migrationLabel(props.node)}>
              <p class="pw-detail-panel__text">
                In transition from {props.node.migrationFrom} to {props.node.migrationTo}. The invariants above say what
                applies where.
              </p>
            </Show>
          </DetailPanel.Section>
          <DetailPanel.Section title="Decisions">
            <TextList items={props.node.decisions} empty="No decisions yet." />
          </DetailPanel.Section>
          <DetailPanel.Section title="Reality">
            <p class="pw-detail-panel__text">
              {props.node.featureCount} features · {props.node.openQuestions} open questions
            </p>
            <Show when={props.node.repo || props.node.folder}>
              <p class="pw-detail-panel__text">
                Checkout {props.node.repo || "project root"}
                <Show when={props.node.folder}> · {props.node.folder}</Show>
              </p>
            </Show>
          </DetailPanel.Section>
          <DetailPanel.Section title="Work">
            <TextList
              items={work().map((card) => `${card.title} (${card.stage})`)}
              empty="No feature cards under this node."
            />
          </DetailPanel.Section>
        </DetailPanel.Main>
      </DetailPanel.Body>
    </>
  );
}

function CardPanel(props: { card: AtlasCanvasCard; driftSlugs: string[] }) {
  const status = () => cardStatus(props.card, props.driftSlugs);
  const openTasks = createMemo(() => props.card.tasks.filter((task) => task.state === "open"));
  return (
    <>
      <DetailPanel.Header icon={<ListChecks size={16} />} title={props.card.title}>
        <StatusChip tone={STATUS_TONE[status()]} label={STATUS_LABEL[status()]} />
        <Show when={props.card.kind === "architecture"}>
          <StatusChip tone="muted" label="Architecture change" />
        </Show>
      </DetailPanel.Header>
      <DetailPanel.Body>
        <DetailPanel.Main>
          <DetailPanel.Section title="Intent">
            <Show
              when={props.card.intent}
              fallback={<p class="pw-detail-panel__text" data-empty>No intent yet.</p>}
            >
              <p class="pw-detail-panel__text whitespace-pre-wrap">{props.card.intent}</p>
            </Show>
          </DetailPanel.Section>
          <DetailPanel.Section title="Contract">
            <TextList items={props.card.invariants.map((item) => item.text)} empty="No invariants yet." />
          </DetailPanel.Section>
          <DetailPanel.Section title="Decisions">
            <TextList items={props.card.decisions} empty="No decisions yet." />
          </DetailPanel.Section>
          <DetailPanel.Section title="Reality">
            <p class="pw-detail-panel__text">Stage {props.card.stage}</p>
          </DetailPanel.Section>
          <DetailPanel.Section title="Work">
            <TextList
              items={openTasks().map((task) => task.title)}
              empty="No open tasks."
            />
          </DetailPanel.Section>
        </DetailPanel.Main>
      </DetailPanel.Body>
    </>
  );
}

function TextList(props: { items: string[]; empty: string }) {
  return (
    <Show
      when={props.items.length > 0}
      fallback={<p class="pw-detail-panel__text" data-empty>{props.empty}</p>}
    >
      <ul class="m-0 grid list-none gap-1 p-0">
        <For each={props.items}>
          {(item) => <li class="pw-detail-panel__text">{item}</li>}
        </For>
      </ul>
    </Show>
  );
}
