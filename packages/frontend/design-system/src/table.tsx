import { ArrowDown, ArrowUp, X } from "./hugeicons";
import { DitherIconTile, type DitherIconTone } from "./dither-icon";
import { detailPanelStore } from "@polterware/frontend-state";
import {
  For,
  Show,
  createContext,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
  useContext,
  type Accessor,
  type JSX,
} from "solid-js";
import { Portal } from "solid-js/web";
import { createVirtualizer } from "@tanstack/solid-virtual";
import { createFluidHover, FluidHoverHighlight, type FluidHover } from "./fluid-hover";
import { spring } from "./springs";
import "./table.css";

export type DataTableSortDirection = "ascending" | "descending";

export type DataTableSortState = {
  columnId: string;
  direction: DataTableSortDirection;
} | null;

export type DataTableSortValue = string | number | boolean | Date | null | undefined;

export type DataTableSelectionState = ReadonlySet<string>;

export type DataTableSelectionMode = "none" | "single" | "multiple";

export type DataTableRootProps<T = any> = {
  rows: readonly T[];
  getRowId?: (row: T, index: number) => string;
  sort?: DataTableSortState;
  defaultSort?: DataTableSortState;
  onSortChange?: (sort: DataTableSortState) => void;
  selectionMode?: DataTableSelectionMode;
  /** @deprecated Use `selectionMode="multiple"`. */
  selectable?: boolean;
  selectedRowIds?: DataTableSelectionState;
  defaultSelectedRowIds?: Iterable<string>;
  onSelectionChange?: (selection: DataTableSelectionState) => void;
  activeRowId?: string | null;
  onActiveRowChange?: (rowId: string | null) => void;
  activateFirstRow?: boolean;
  virtualized?: boolean;
  estimateRowHeight?: number;
  gutters?: boolean;
  children: JSX.Element;
  class?: string;
};

export type DataTableSelectionToolbarState<T> = {
  selectedRows: Accessor<readonly T[]>;
  selectedRowIds: Accessor<DataTableSelectionState>;
  selectedCount: Accessor<number>;
  clearSelection: () => void;
};

export type DataTableSelectionToolbarProps<T = any> = {
  children: (selection: DataTableSelectionToolbarState<T>) => JSX.Element;
  hasActions?: boolean | ((selection: DataTableSelectionToolbarState<T>) => boolean);
  emptyActionsLabel?: string;
  class?: string;
};

export type DataTableColumnHeaderProps<T = any> = {
  id: string;
  children: JSX.Element;
  sortable?: boolean;
  sortBy?: (row: T) => DataTableSortValue;
  compare?: (left: T, right: T) => number;
  width?: string;
  align?: "left" | "center" | "right";
  class?: string;
};

type IndexedRow<T> = {
  row: T;
  sourceIndex: number;
  id: string;
};

type RegisteredColumn<T> = {
  id: string;
  sortable: boolean;
  sortBy: (row: T) => DataTableSortValue;
  compare?: (left: T, right: T) => number;
};

type DataTableContextValue<T> = {
  rows: Accessor<readonly IndexedRow<T>[]>;
  registerColumn: (column: RegisteredColumn<T>) => VoidFunction;
  sort: Accessor<DataTableSortState>;
  cycleSort: (columnId: string) => void;
  selectionMode: Accessor<DataTableSelectionMode>;
  selectable: Accessor<boolean>;
  selectedRowIds: Accessor<DataTableSelectionState>;
  selectedRows: Accessor<readonly IndexedRow<T>[]>;
  activeRowId: Accessor<string | null>;
  activeEnabled: Accessor<boolean>;
  activateRow: (rowId: string) => void;
  toggleRow: (rowId: string) => void;
  toggleAll: () => void;
  clearSelection: () => void;
  allVisibleSelected: Accessor<boolean>;
  someVisibleSelected: Accessor<boolean>;
  virtualized: Accessor<boolean>;
  estimateRowHeight: Accessor<number>;
  scrollElement: Accessor<HTMLDivElement | null>;
  setScrollElement: (element: HTMLDivElement) => void;
  columnCount: Accessor<number>;
};

const DataTableContext = createContext<DataTableContextValue<never>>();
const DataTableSelectionToolbarHostContext = createContext<Accessor<HTMLElement | undefined>>();
/** The table's one row highlight; interactive rows join it by their position in the body. */
const DataTableHoverContext = createContext<FluidHover>();
const DataTableRowIndexContext = createContext<Accessor<number>>();

function bridgeDataTableContext<From, To>(
  context: DataTableContextValue<From>,
): DataTableContextValue<To> {
  // SAFETY: DataTable.Root and its compound children share one row type; the
  // context erases that generic only while crossing Solid's non-generic Context API.
  return context as never;
}

function useDataTable<T = never>(part: string): DataTableContextValue<T> {
  const context = useContext(DataTableContext);
  if (!context) throw new Error(`DataTable.${part} must be used inside DataTable.Root`);
  return bridgeDataTableContext<never, T>(context);
}

export function getNextDataTableSort(
  current: DataTableSortState,
  columnId: string,
): DataTableSortState {
  if (!current || current.columnId !== columnId) {
    return { columnId, direction: "ascending" };
  }
  if (current.direction === "ascending") {
    return { columnId, direction: "descending" };
  }
  return null;
}

export function getDataTableAriaSort(
  current: DataTableSortState,
  columnId: string,
  sortable = true,
): DataTableSortDirection | "none" | undefined {
  if (!sortable) return undefined;
  return current?.columnId === columnId ? current.direction : "none";
}

function isNumberSortValue(value: DataTableSortValue): value is number {
  return Object.prototype.toString.call(value) === "[object Number]";
}

function isBooleanSortValue(value: DataTableSortValue): value is boolean {
  return value === true || value === false;
}

function isStringSortValue(value: DataTableSortValue): value is string {
  return Object.prototype.toString.call(value) === "[object String]";
}

function compareSortValues(left: DataTableSortValue, right: DataTableSortValue): number {
  const leftMissing = left === null || left === undefined || left === "";
  const rightMissing = right === null || right === undefined || right === "";
  if (leftMissing && rightMissing) return 0;
  if (leftMissing) return 1;
  if (rightMissing) return -1;

  const leftValue = left instanceof Date ? left.getTime() : left;
  const rightValue = right instanceof Date ? right.getTime() : right;
  if (isNumberSortValue(leftValue) && isNumberSortValue(rightValue)) {
    return leftValue - rightValue;
  }
  if (isBooleanSortValue(leftValue) && isBooleanSortValue(rightValue)) {
    return Number(leftValue) - Number(rightValue);
  }
  return String(leftValue).localeCompare(String(rightValue), undefined, {
    numeric: true,
    sensitivity: "base",
  });
}

function isMissingSortValue(value: DataTableSortValue): boolean {
  return value === null || value === undefined || value === "";
}

export function sortDataTableRows<T>(
  rows: readonly T[],
  sort: DataTableSortState,
  column?: {
    sortBy: (row: T) => DataTableSortValue;
    compare?: (left: T, right: T) => number;
  },
): T[] {
  if (!sort || !column) return [...rows];
  const direction = sort.direction === "ascending" ? 1 : -1;
  return rows
    .map((row, sourceIndex) => ({ row, sourceIndex }))
    .sort((left, right) => {
      let comparison: number;
      if (column.compare) {
        comparison = column.compare(left.row, right.row);
      } else {
        const leftValue = column.sortBy(left.row);
        const rightValue = column.sortBy(right.row);
        const leftMissing = isMissingSortValue(leftValue);
        const rightMissing = isMissingSortValue(rightValue);
        if (leftMissing !== rightMissing) return leftMissing ? 1 : -1;
        comparison = compareSortValues(leftValue, rightValue);
      }
      return comparison === 0
        ? left.sourceIndex - right.sourceIndex
        : comparison * direction;
    })
    .map((entry) => entry.row);
}

export function getNextDataTableSelection(
  current: DataTableSelectionState,
  visibleRowIds: Iterable<string>,
  rowId?: string,
): DataTableSelectionState {
  const next = new Set(current);
  if (rowId !== undefined) {
    if (next.has(rowId)) next.delete(rowId);
    else next.add(rowId);
    return next;
  }

  const visible = [...visibleRowIds];
  const allVisibleSelected = visible.length > 0 && visible.every((id) => next.has(id));
  for (const id of visible) {
    if (allVisibleSelected) next.delete(id);
    else next.add(id);
  }
  return next;
}

export function getDataTableSelectionMode(
  selectionMode?: DataTableSelectionMode,
  selectable = false,
): DataTableSelectionMode {
  return selectionMode ?? (selectable ? "multiple" : "none");
}

export function getNextDataTableSelectionForMode(
  current: DataTableSelectionState,
  visibleRowIds: Iterable<string>,
  mode: DataTableSelectionMode,
  rowId?: string,
): DataTableSelectionState {
  if (mode === "none") return new Set();
  if (mode === "single") {
    if (rowId === undefined) return new Set(current);
    return current.has(rowId) ? new Set() : new Set([rowId]);
  }
  return getNextDataTableSelection(current, visibleRowIds, rowId);
}

export function pruneDataTableSelection(
  current: DataTableSelectionState,
  visibleRowIds: Iterable<string>,
  mode: DataTableSelectionMode,
): DataTableSelectionState {
  if (mode === "none") return new Set();
  const next = new Set<string>();
  for (const id of visibleRowIds) {
    if (!current.has(id)) continue;
    next.add(id);
    if (mode === "single") break;
  }
  return next;
}

export function getSelectedDataTableRows<T>(
  rows: readonly T[],
  selection: DataTableSelectionState,
  getRowId: (row: T, index: number) => string = (row, index) => defaultRowId(row, index),
): T[] {
  return rows.filter((row, index) => selection.has(getRowId(row, index)));
}

function dataTableSelectionsEqual(
  left: DataTableSelectionState,
  right: DataTableSelectionState,
): boolean {
  return left.size === right.size && [...left].every((id) => right.has(id));
}

function defaultRowId<T>(row: T, index: number): string {
  if (row instanceof Object && "id" in row) {
    // SAFETY: row is an object and has an id property, we can safely read it.
    const id = (row as Record<string, DataTableSortValue>).id;
    if (isStringSortValue(id) || isNumberSortValue(id)) return String(id);
  }
  return String(index);
}

function DataTableRoot<T = any>(props: DataTableRootProps<T>) {
  const [internalSort, setInternalSort] = createSignal<DataTableSortState>(
    props.defaultSort ?? null,
  );
  const [internalSelection, setInternalSelection] = createSignal<ReadonlySet<string>>(
    new Set(props.defaultSelectedRowIds ?? []),
  );
  const [internalActiveRowId, setInternalActiveRowId] = createSignal<string | null>(null);
  const [scrollElement, setScrollElement] = createSignal<HTMLDivElement | null>(null);
  const [columnCount, setColumnCount] = createSignal(0);
  const [columnVersion, setColumnVersion] = createSignal(0);
  const columns = new Map<string, RegisteredColumn<T>>();

  const sourceRows = createMemo<readonly IndexedRow<T>[]>(() =>
    props.rows.map((row, sourceIndex) => ({
      row,
      sourceIndex,
      id: props.getRowId?.(row, sourceIndex) ?? defaultRowId(row, sourceIndex),
    })),
  );
  const sort = (): DataTableSortState =>
    props.sort !== undefined ? props.sort : internalSort();
  const selectionMode = (): DataTableSelectionMode =>
    getDataTableSelectionMode(props.selectionMode, Boolean(props.selectable));
  const rawSelectedRowIds = (): DataTableSelectionState =>
    props.selectedRowIds !== undefined ? props.selectedRowIds : internalSelection();
  const visibleIds = createMemo(() => new Set(sourceRows().map((entry) => entry.id)));
  const selectedRowIds = createMemo<DataTableSelectionState>(() =>
    pruneDataTableSelection(rawSelectedRowIds(), visibleIds(), selectionMode()),
  );
  const selectedRows = createMemo<readonly IndexedRow<T>[]>(() =>
    sourceRows().filter((entry) => selectedRowIds().has(entry.id)),
  );
  const activeEnabled = (): boolean =>
    props.activeRowId !== undefined ||
    props.onActiveRowChange !== undefined ||
    Boolean(props.activateFirstRow);
  const activeRowId = (): string | null =>
    props.activeRowId !== undefined ? props.activeRowId : internalActiveRowId();

  const orderedRows = createMemo<readonly IndexedRow<T>[]>(() => {
    columnVersion();
    const currentSort = sort();
    if (!currentSort) return sourceRows();
    const column = columns.get(currentSort.columnId);
    if (!column?.sortable) return sourceRows();
    return sortDataTableRows(sourceRows(), currentSort, {
      sortBy: (entry) => column.sortBy(entry.row),
      compare: column.compare
        ? (left, right) => column.compare!(left.row, right.row)
        : undefined,
    });
  });

  const updateSelection = (next: ReadonlySet<string>) => {
    if (props.selectedRowIds === undefined) setInternalSelection(next);
    props.onSelectionChange?.(next);
  };
  const updateActiveRow = (next: string | null) => {
    if (props.activeRowId === undefined) setInternalActiveRowId(next);
    props.onActiveRowChange?.(next);
  };
  createEffect(() => {
    const raw = rawSelectedRowIds();
    const next = selectedRowIds();
    if (!dataTableSelectionsEqual(raw, next)) updateSelection(next);
  });
  createEffect(() => {
    if (!props.activateFirstRow) return;
    const ordered = orderedRows();
    const current = activeRowId();
    const next = ordered.some((entry) => entry.id === current)
      ? current
      : ordered[0]?.id ?? null;
    if (next !== current) updateActiveRow(next);
  });
  const allVisibleSelected = createMemo(
    () =>
      selectionMode() === "multiple" &&
      visibleIds().size > 0 &&
      [...visibleIds()].every((id) => selectedRowIds().has(id)),
  );
  const someVisibleSelected = createMemo(
    () =>
      !allVisibleSelected() &&
      [...visibleIds()].some((id) => selectedRowIds().has(id)),
  );

  const context: DataTableContextValue<T> = {
    rows: orderedRows,
    registerColumn: (column) => {
      const isNew = !columns.has(column.id);
      columns.set(column.id, column);
      setColumnVersion((version) => version + 1);
      if (isNew) setColumnCount((count) => count + 1);
      return () => {
        if (columns.delete(column.id)) {
          setColumnVersion((version) => version + 1);
          setColumnCount((count) => Math.max(0, count - 1));
        }
      };
    },
    sort,
    cycleSort: (columnId) => {
      const next = getNextDataTableSort(sort(), columnId);
      if (props.sort === undefined) setInternalSort(next);
      props.onSortChange?.(next);
    },
    selectionMode,
    selectable: () => selectionMode() !== "none",
    selectedRowIds,
    selectedRows,
    activeRowId,
    activeEnabled,
    activateRow: updateActiveRow,
    toggleRow: (rowId) => {
      updateSelection(
        getNextDataTableSelectionForMode(
          selectedRowIds(),
          visibleIds(),
          selectionMode(),
          rowId,
        ),
      );
    },
    toggleAll: () => {
      if (selectionMode() !== "multiple") return;
      updateSelection(
        getNextDataTableSelectionForMode(selectedRowIds(), visibleIds(), "multiple"),
      );
    },
    clearSelection: () => updateSelection(new Set()),
    allVisibleSelected,
    someVisibleSelected,
    virtualized: () => Boolean(props.virtualized),
    estimateRowHeight: () => props.estimateRowHeight ?? 38,
    scrollElement,
    setScrollElement,
    columnCount: () => columnCount() + (selectionMode() === "none" ? 0 : 1),
  };

  return (
    <DataTableContext.Provider value={bridgeDataTableContext<T, never>(context)}>
      <div
        class={`pw-data-table-root ${props.class ?? ""}`}
        data-selection-active={selectedRows().length > 0 ? "true" : undefined}
        data-gutters={props.gutters === false ? "false" : undefined}
      >
        {props.children}
      </div>
    </DataTableContext.Provider>
  );
}

export function DataTableSelectionToolbarHost(props: {
  children: JSX.Element;
  class?: string;
}) {
  const [host, setHost] = createSignal<HTMLElement>();
  return (
    <DataTableSelectionToolbarHostContext.Provider value={host}>
      {props.children}
      <div
        ref={setHost}
        class={`pw-data-table-selection-toolbar-host ${props.class ?? ""}`}
      />
    </DataTableSelectionToolbarHostContext.Provider>
  );
}

function DataTableToolbar(props: { children: JSX.Element; class?: string }) {
  return <div class={`pw-data-table-toolbar ${props.class ?? ""}`}>{props.children}</div>;
}

function collapseDetailPanelFromCollectionBackground() {
  const mode = detailPanelStore.getState().mode;
  if (mode === "open" || mode === "maximized") {
    detailPanelStore.getState().collapse();
  }
}

function DataTableTable(props: {
  children: JSX.Element;
  class?: string;
  "aria-label"?: string;
}) {
  const context = useDataTable("Table");
  let scroll: HTMLDivElement | undefined;
  // A click on the empty background collapses the detail panel, so gaps stay inert.
  const hover = createFluidHover(() => scroll, { gapClick: false });
  const entryAt = (index: number | null) =>
    index === null ? undefined : context.rows()[index];
  // Over a selected row the highlight takes the selected fill, so hover never hides selection.
  const hoverClass = () => {
    const entry = entryAt(hover.activeIndex());
    return entry && context.selectedRowIds().has(entry.id)
      ? "pw-data-table-hover pw-data-table-hover--selected"
      : "pw-data-table-hover";
  };
  const activeIndex = () => {
    const rowId = context.activeEnabled() ? context.activeRowId() : null;
    if (rowId === null) return null;
    const index = context.rows().findIndex((entry) => entry.id === rowId);
    return index < 0 ? null : index;
  };
  const activeRect = () => {
    const index = activeIndex();
    return index === null ? null : (hover.rects()[index] ?? null);
  };

  return (
    <DataTableHoverContext.Provider value={hover}>
      <div
        ref={(element) => {
          scroll = element;
          context.setScrollElement(element);
        }}
        class="pw-data-table-scroll"
        data-virtualized={context.virtualized() ? "true" : undefined}
        onClick={(event) => {
          const target = event.target;
          if (!(target instanceof Element)) return;
          if (target.closest("tr")) return;
          collapseDetailPanelFromCollectionBackground();
        }}
      >
        <FluidHoverHighlight hover={hover} class={hoverClass()} />
        {/* The active row's marker travels between rows; a re-sort that moves it snaps. */}
        <FluidHoverHighlight
          class="pw-data-table-active"
          rect={activeRect()}
          targetKey={context.activeRowId()}
          transition={spring.moderate}
        />
        <table
          class={`pw-data-table ${props.class ?? ""}`}
          aria-label={props["aria-label"]}
          onPointerEnter={hover.handlers.onPointerEnter}
          onPointerMove={hover.handlers.onPointerMove}
          onPointerLeave={hover.handlers.onPointerLeave}
          onFocusIn={hover.handlers.onFocusIn}
          onFocusOut={hover.handlers.onFocusOut}
        >
          {props.children}
        </table>
      </div>
    </DataTableHoverContext.Provider>
  );
}

function DataTableHeader(props: { children: JSX.Element; class?: string }) {
  return (
    <thead class={props.class}>
      <tr>{props.children}</tr>
    </thead>
  );
}

function DataTableColumnHeader<T = any>(props: DataTableColumnHeaderProps<T>) {
  const context = useDataTable<T>("ColumnHeader");
  const sortable = () => props.sortable !== false;
  const sortBy = (row: T): DataTableSortValue => {
    if (props.sortBy) return props.sortBy(row);
    if (row instanceof Object && props.id in row) {
      // SAFETY: We verified row is an object containing the id key.
      return (row as Record<string, DataTableSortValue>)[props.id];
    }
    return undefined;
  };
  const dispose = context.registerColumn({
    id: props.id,
    sortable: sortable(),
    sortBy,
    compare: props.compare,
  });
  onCleanup(dispose);

  const state = () =>
    context.sort()?.columnId === props.id ? context.sort()?.direction : undefined;
  const ariaSort = () => getDataTableAriaSort(context.sort(), props.id, sortable());
  const alignClass = () => `pw-data-table-align-${props.align ?? "left"}`;

  return (
    <th
      class={`pw-data-table-column-header ${alignClass()} ${props.class ?? ""}`}
      style={{ width: props.width }}
      aria-sort={sortable() ? ariaSort() : undefined}
      scope="col"
    >
      <Show
        when={sortable()}
        fallback={<span class="pw-data-table-column-label">{props.children}</span>}
      >
        <button
          type="button"
          class="pw-data-table-sort-trigger"
          classList={{ "pw-data-table-sort-trigger--active": Boolean(state()) }}
          onClick={() => context.cycleSort(props.id)}
          aria-label={`Sort by ${String(props.children)}`}
        >
          <span>{props.children}</span>
          <Show when={state()}>
            {(direction) =>
              direction() === "ascending" ? (
                <ArrowUp size={13} aria-hidden />
              ) : (
                <ArrowDown size={13} aria-hidden />
              )
            }
          </Show>
        </button>
      </Show>
    </th>
  );
}

function DataTableBody<T = any>(props: {
  children: (row: T, index: Accessor<number>) => JSX.Element;
  class?: string;
}) {
  const context = useDataTable<T>("Body");
  const virtualizer = createVirtualizer<HTMLDivElement, HTMLTableRowElement>({
    get count() {
      return context.rows().length;
    },
    getScrollElement: context.scrollElement,
    estimateSize: context.estimateRowHeight,
    overscan: 8,
  });
  const virtualItems = () => virtualizer.getVirtualItems();
  const topSpacer = () => virtualItems()[0]?.start ?? 0;
  const bottomSpacer = () => {
    const last = virtualItems().at(-1);
    return last ? Math.max(0, virtualizer.getTotalSize() - last.end) : 0;
  };
  const renderEntry = (entry: IndexedRow<T>, index: Accessor<number>) => (
    <DataTableRowIndexContext.Provider value={index}>
      {props.children(entry.row, index)}
    </DataTableRowIndexContext.Provider>
  );

  return (
    <tbody class={props.class}>
      <Show when={context.rows().length > 0}>
        <Show
          when={context.virtualized()}
          fallback={
            <For each={context.rows()}>
              {(entry, index) => renderEntry(entry, index)}
            </For>
          }
        >
          <Show when={topSpacer() > 0}>
            <tr aria-hidden="true" class="pw-data-table-spacer">
              <td colspan={context.columnCount()} style={{ height: `${topSpacer()}px` }} />
            </tr>
          </Show>
          <For each={virtualItems()}>
            {(virtualRow) => {
              const entry = () => context.rows()[virtualRow.index];
              return (
                <Show when={entry()}>
                  {(current) => renderEntry(current(), () => virtualRow.index)}
                </Show>
              );
            }}
          </For>
          <Show when={bottomSpacer() > 0}>
            <tr aria-hidden="true" class="pw-data-table-spacer">
              <td colspan={context.columnCount()} style={{ height: `${bottomSpacer()}px` }} />
            </tr>
          </Show>
        </Show>
      </Show>
    </tbody>
  );
}

export function isDataTableInteractiveTarget(
  target: EventTarget | null,
  row: HTMLTableRowElement,
): boolean {
  if (!(target instanceof Element)) return false;
  const interactive = target.closest(
    "button, a, input, select, textarea, summary, [role='button'], [role='link'], [data-row-action]",
  );
  return Boolean(interactive && interactive !== row);
}

export function isDataTableActivationKey(key: string): boolean {
  return key === "Enter" || key === " ";
}

export function hasDataTablePrimaryLeading(leading: JSX.Element): boolean {
  return Boolean(leading);
}

export function shouldRenderDataTableEmpty(rowCount: number): boolean {
  return rowCount === 0;
}

function DataTableRow<T = any>(props: {
  row: T;
  index?: number;
  rowId?: string;
  children: JSX.Element;
  onActivate?: (row: T) => void;
  "aria-label"?: string;
  class?: string;
}) {
  const context = useDataTable("Row");
  const rowId = () => props.rowId ?? defaultRowId(props.row, props.index ?? 0);
  const selected = () => context.selectedRowIds().has(rowId());
  const active = () => context.activeEnabled() && context.activeRowId() === rowId();
  const interactive = () => Boolean(props.onActivate) || context.activeEnabled();
  const activate = () => {
    if (context.activeEnabled()) context.activateRow(rowId());
    props.onActivate?.(props.row);
  };
  // Only rows a click can land on join the highlight.
  const hover = useContext(DataTableHoverContext);
  const bodyIndex = useContext(DataTableRowIndexContext);
  let rowElement: HTMLTableRowElement | undefined;
  onMount(() => {
    const element = rowElement;
    if (!hover || !bodyIndex || !element) return;
    createEffect(() => {
      if (!interactive()) return;
      onCleanup(hover.registerItem(bodyIndex(), element));
    });
  });

  return (
    <tr
      ref={rowElement}
      class={`pw-data-table-row ${props.class ?? ""}`}
      classList={{
        "pw-data-table-row--interactive": interactive(),
        "pw-data-table-row--selected": selected(),
        "pw-data-table-row--active": active(),
      }}
      data-row-id={rowId()}
      aria-selected={context.selectable() ? selected() : undefined}
      aria-current={active() ? "true" : undefined}
      aria-label={props["aria-label"]}
      tabIndex={interactive() ? 0 : undefined}
      onClick={(event) => {
        if (!interactive() || isDataTableInteractiveTarget(event.target, event.currentTarget)) return;
        activate();
      }}
      onDblClick={(event) => {
        if (!interactive() || isDataTableInteractiveTarget(event.target, event.currentTarget)) return;
        activate();
        detailPanelStore.getState().maximize();
      }}
      onKeyDown={(event) => {
        if (!interactive() || event.target !== event.currentTarget) return;
        if (isDataTableActivationKey(event.key)) {
          event.preventDefault();
          activate();
        }
      }}
    >
      {props.children}
    </tr>
  );
}

function DataTableCell(props: {
  children?: JSX.Element;
  class?: string;
  align?: "left" | "center" | "right";
  colspan?: number;
  onClick?: JSX.EventHandlerUnion<HTMLTableCellElement, MouseEvent>;
}) {
  return (
    <td
      class={`pw-data-table-cell pw-data-table-align-${props.align ?? "left"} ${props.class ?? ""}`}
      colspan={props.colspan}
      onClick={props.onClick}
    >
      {props.children}
    </td>
  );
}

export type DataTablePrimaryCellProps = {
  title: JSX.Element;
  trailing?: JSX.Element;
  class?: string;
} & (
  | {
      icon: JSX.Element;
      iconTone: DitherIconTone;
      iconBrand?: string | readonly string[];
      leading?: never;
    }
  | {
      icon?: never;
      iconTone?: never;
      iconBrand?: never;
      leading?: JSX.Element;
    }
);

function DataTablePrimaryCell(props: DataTablePrimaryCellProps) {
  const leading = () =>
    props.icon ? (
      <DitherIconTile size={24} tone={props.iconTone} brand={props.iconBrand}>
        {props.icon}
      </DitherIconTile>
    ) : (
      props.leading
    );
  return (
    <td class={`pw-data-table-cell pw-data-table-primary-cell ${props.class ?? ""}`}>
      <div class="pw-data-table-primary">
        <Show when={hasDataTablePrimaryLeading(leading())}>
          <span class="pw-data-table-primary-leading" aria-hidden>
            {leading()}
          </span>
        </Show>
        <span class="pw-data-table-primary-title">{props.title}</span>
        <Show when={props.trailing}>
          <span class="pw-data-table-primary-trailing">{props.trailing}</span>
        </Show>
      </div>
    </td>
  );
}

function DataTableSelectionCell(props: {
  rowId?: string;
  label?: string;
  class?: string;
}) {
  const context = useDataTable("SelectionCell");
  let input: HTMLInputElement | undefined;
  const header = () => props.rowId === undefined;
  const checked = () =>
    header() ? context.allVisibleSelected() : context.selectedRowIds().has(props.rowId!);
  createEffect(() => {
    const indeterminate = header() && context.someVisibleSelected();
    if (input) input.indeterminate = indeterminate;
  });

  return (
    <Show when={context.selectable()}>
      <Show
        when={header()}
        fallback={
          <td class={`pw-data-table-selection-cell ${props.class ?? ""}`}>
            <input
              ref={(el) => { input = el; }}
              class="pw-data-table-selection-input"
              type="checkbox"
              checked={checked()}
              aria-label={props.label ?? `Select row ${props.rowId}`}
              onClick={(event) => event.stopPropagation()}
              onChange={() => context.toggleRow(props.rowId!)}
            />
          </td>
        }
      >
        <th
          class={`pw-data-table-selection-header ${props.class ?? ""}`}
          scope="col"
          aria-label={context.selectionMode() === "single" ? "Selection" : undefined}
        >
          <Show when={context.selectionMode() === "multiple"}>
            <input
              ref={(el) => { input = el; }}
              class="pw-data-table-selection-input"
              type="checkbox"
              checked={checked()}
              aria-label={props.label ?? "Select all visible rows"}
              onChange={() => context.toggleAll()}
            />
          </Show>
        </th>
      </Show>
    </Show>
  );
}

function DataTableSelectionToolbar<T = any>(
  props: DataTableSelectionToolbarProps<T>,
) {
  const context = useDataTable<T>("SelectionToolbar");
  const host = useContext(DataTableSelectionToolbarHostContext);
  const state: DataTableSelectionToolbarState<T> = {
    selectedRows: () => context.selectedRows().map((entry) => entry.row),
    selectedRowIds: context.selectedRowIds,
    selectedCount: () => context.selectedRows().length,
    clearSelection: context.clearSelection,
  };

  const hasActions = () =>
    props.hasActions instanceof Function
      ? props.hasActions(state)
      : props.hasActions !== false;

  const toolbar = () => (
    <Show when={state.selectedCount() > 0}>
      <div
        class={`pw-data-table-selection-toolbar surface-panel-float ${props.class ?? ""}`}
        role="toolbar"
        aria-label="Selected row actions"
      >
        <span class="pw-data-table-selection-summary" aria-live="polite">
          {state.selectedCount()} selected
        </span>
        <span class="pw-data-table-selection-actions">
          <Show
            when={hasActions()}
            fallback={
              <span class="pw-data-table-selection-empty-actions">
                {props.emptyActionsLabel ?? "No shared actions"}
              </span>
            }
          >
            {props.children(state)}
          </Show>
        </span>
        <button
          type="button"
          class="pw-data-table-selection-clear"
          aria-label="Clear selection"
          title="Clear selection"
          onClick={state.clearSelection}
        >
          <X size={14} aria-hidden />
        </button>
      </div>
    </Show>
  );

  return (
    <Show
      when={host?.()}
      fallback={<div class="pw-data-table-selection-toolbar-inline">{toolbar()}</div>}
    >
      {(mount) => <Portal mount={mount()}>{toolbar()}</Portal>}
    </Show>
  );
}

function DataTableEmpty(props: { children: JSX.Element; class?: string; colspan?: number }) {
  const context = useDataTable("Empty");
  return (
    <Show when={shouldRenderDataTableEmpty(context.rows().length)}>
      <tbody>
        <tr>
          <td
            colspan={props.colspan ?? Math.max(1, context.columnCount())}
            class={`pw-data-table-empty ${props.class ?? ""}`}
          >
            {props.children}
          </td>
        </tr>
      </tbody>
    </Show>
  );
}

function DataTableFooter(props: { children: JSX.Element; class?: string }) {
  return <div class={`pw-data-table-footer ${props.class ?? ""}`}>{props.children}</div>;
}

export const DataTable = {
  Root: DataTableRoot,
  Toolbar: DataTableToolbar,
  Table: DataTableTable,
  Header: DataTableHeader,
  ColumnHeader: DataTableColumnHeader,
  Body: DataTableBody,
  Row: DataTableRow,
  Cell: DataTableCell,
  PrimaryCell: DataTablePrimaryCell,
  SelectionCell: DataTableSelectionCell,
  SelectionToolbar: DataTableSelectionToolbar,
  Empty: DataTableEmpty,
  Footer: DataTableFooter,
};
