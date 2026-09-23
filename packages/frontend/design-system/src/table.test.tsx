// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import { createSignal } from "solid-js";
import { render } from "solid-js/web";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { detailPanelDefaultState, detailPanelStore } from "@polterware/frontend-state";

import {
  DataTable,
  DataTableSelectionToolbarHost,
  getDataTableAriaSort,
  getDataTableSelectionMode,
  getNextDataTableSelectionForMode,
  getSelectedDataTableRows,
  hasDataTablePrimaryLeading,
  getNextDataTableSelection,
  getNextDataTableSort,
  isDataTableActivationKey,
  isDataTableInteractiveTarget,
  pruneDataTableSelection,
  shouldRenderDataTableEmpty,
  sortDataTableRows,
  type DataTableSortState,
} from "./table";

const tableSource = readFileSync(fileURLToPath(new URL("./table.tsx", import.meta.url)), "utf8");

describe("DataTable", () => {
  it("exports the complete compound component contract", () => {
    expect(Object.keys(DataTable)).toEqual([
      "Root",
      "Toolbar",
      "Table",
      "Header",
      "ColumnHeader",
      "Body",
      "Row",
      "Cell",
      "PrimaryCell",
      "SelectionCell",
      "SelectionToolbar",
      "Empty",
      "Footer",
    ]);
    for (const component of Object.values(DataTable)) expect(component).toBeDefined();
    expect(DataTableSelectionToolbarHost).toBeDefined();
  });

  it("cycles source, ascending, descending, and source order", () => {
    let state: DataTableSortState = null;
    state = getNextDataTableSort(state, "name");
    expect(state).toEqual({ columnId: "name", direction: "ascending" });
    state = getNextDataTableSort(state, "name");
    expect(state).toEqual({ columnId: "name", direction: "descending" });
    state = getNextDataTableSort(state, "name");
    expect(state).toBeNull();
    expect(getNextDataTableSort({ columnId: "name", direction: "ascending" }, "status"))
      .toEqual({ columnId: "status", direction: "ascending" });
  });

  it("sorts values naturally, keeps missing values last, and preserves equal source order", () => {
    const rows = [
      { id: "a", name: "Node 10", score: 2 },
      { id: "b", name: "Node 2", score: 2 },
      { id: "c", name: "Unknown", score: undefined },
    ];
    expect(
      sortDataTableRows(rows, { columnId: "name", direction: "ascending" }, {
        sortBy: (row) => row.name,
      }).map((row) => row.id),
    ).toEqual(["b", "a", "c"]);
    expect(
      sortDataTableRows(rows, { columnId: "score", direction: "ascending" }, {
        sortBy: (row) => row.score,
      }).map((row) => row.id),
    ).toEqual(["a", "b", "c"]);
    expect(
      sortDataTableRows(rows, { columnId: "score", direction: "descending" }, {
        sortBy: (row) => row.score,
      }).map((row) => row.id),
    ).toEqual(["a", "b", "c"]);
  });

  it("sorts a 1,000-row collection without changing the source", () => {
    const rows = Array.from({ length: 1_000 }, (_, index) => ({
      id: String(index),
      value: 1_000 - index,
    }));
    const sorted = sortDataTableRows(
      rows,
      { columnId: "value", direction: "ascending" },
      { sortBy: (row) => row.value },
    );
    expect(sorted[0]?.value).toBe(1);
    expect(sorted.at(-1)?.value).toBe(1_000);
    expect(rows[0]?.value).toBe(1_000);
  });

  it("selects one row or all visible rows while preserving hidden IDs", () => {
    const current = new Set(["hidden"]);
    expect([...getNextDataTableSelection(current, ["a", "b"], "a")]).toEqual([
      "hidden",
      "a",
    ]);
    const selected = getNextDataTableSelection(current, ["a", "b"]);
    expect([...selected]).toEqual(["hidden", "a", "b"]);
    expect([...getNextDataTableSelection(selected, ["a", "b"])]).toEqual(["hidden"]);
  });

  it("supports none, single, and multiple selection modes", () => {
    expect(getDataTableSelectionMode(undefined, false)).toBe("none");
    expect(getDataTableSelectionMode(undefined, true)).toBe("multiple");
    expect(getDataTableSelectionMode("single", true)).toBe("single");

    expect([
      ...getNextDataTableSelectionForMode(new Set(["a"]), ["a", "b"], "single", "b"),
    ]).toEqual(["b"]);
    expect([
      ...getNextDataTableSelectionForMode(new Set(["b"]), ["a", "b"], "single", "b"),
    ]).toEqual([]);
    expect([
      ...getNextDataTableSelectionForMode(new Set(), ["a", "b"], "multiple"),
    ]).toEqual(["a", "b"]);
  });

  it("prunes hidden IDs and resolves selected rows in source order", () => {
    const rows = [
      { id: "a", name: "Alpha" },
      { id: "b", name: "Beta" },
      { id: "c", name: "Gamma" },
    ];
    expect([...pruneDataTableSelection(new Set(["hidden", "b"]), ["a", "b", "c"], "multiple")])
      .toEqual(["b"]);
    expect([...pruneDataTableSelection(new Set(["c", "a"]), ["a", "b", "c"], "single")])
      .toEqual(["a"]);
    expect(getSelectedDataTableRows(rows, new Set(["c", "a"]), (row) => row.id))
      .toEqual([rows[0], rows[2]]);
  });

  it("maps controlled sorting to aria-sort", () => {
    expect(getDataTableAriaSort(null, "name")).toBe("none");
    expect(getDataTableAriaSort({ columnId: "name", direction: "ascending" }, "name"))
      .toBe("ascending");
    expect(getDataTableAriaSort({ columnId: "name", direction: "descending" }, "name"))
      .toBe("descending");
    expect(getDataTableAriaSort({ columnId: "name", direction: "ascending" }, "status"))
      .toBe("none");
    expect(getDataTableAriaSort(null, "actions", false)).toBeUndefined();
  });

  it("recognizes keyboard activation and isolates nested interactive actions", () => {
    expect(isDataTableActivationKey("Enter")).toBe(true);
    expect(isDataTableActivationKey(" ")).toBe(true);
    expect(isDataTableActivationKey("Escape")).toBe(false);

    const row = document.createElement("tr");
    const cell = document.createElement("td");
    const button = document.createElement("button");
    cell.append(button);
    row.append(cell);
    expect(isDataTableInteractiveTarget(button, row)).toBe(true);
    expect(isDataTableInteractiveTarget(cell, row)).toBe(false);
  });

  it("renders optional leading content and empty state only when applicable", () => {
    expect(hasDataTablePrimaryLeading(undefined)).toBe(false);
    expect(hasDataTablePrimaryLeading(null)).toBe(false);
    expect(hasDataTablePrimaryLeading("icon")).toBe(true);
    expect(shouldRenderDataTableEmpty(0)).toBe(true);
    expect(shouldRenderDataTableEmpty(1_000)).toBe(false);
  });

  it("renders single-line primary cells and shared row/header selection inputs", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const activate = vi.fn();
    const dispose = render(
      () => (
        <DataTable.Root
          rows={[
            { id: "a", name: "Alpha" },
            { id: "b", name: "Beta" },
          ]}
          selectionMode="multiple"
        >
          <DataTable.Table aria-label="Selection fixture">
            <DataTable.Header>
              <DataTable.SelectionCell />
              <DataTable.ColumnHeader id="name">Name</DataTable.ColumnHeader>
            </DataTable.Header>
            <DataTable.Body>
              {(row) => (
                <DataTable.Row row={row} rowId={row.id} onActivate={activate}>
                  <DataTable.SelectionCell rowId={row.id} label={`Select ${row.name}`} />
                  <DataTable.PrimaryCell icon={<span>Icon</span>} iconTone="identity" title={row.name} />
                </DataTable.Row>
              )}
            </DataTable.Body>
          </DataTable.Table>
        </DataTable.Root>
      ),
      host,
    );

    const inputs = host.querySelectorAll<HTMLInputElement>(".pw-data-table-selection-input");
    const headerInput = inputs[0]!;
    const firstRowInput = inputs[1]!;
    const firstRow = host.querySelector<HTMLTableRowElement>("[data-row-id='a']")!;
    expect(inputs).toHaveLength(3);
    expect(host.querySelector(".pw-data-table-primary-title")?.textContent).toBe("Alpha");
    expect(host.querySelector(".pw-data-table-primary-description")).toBeNull();
    expect(host.querySelector(".pw-dither-icon")?.getAttribute("data-size")).toBe("24");
    expect(host.querySelector(".pw-dither-icon-mark")?.textContent).toBe("Icon");

    firstRowInput.click();
    expect(firstRowInput.checked).toBe(true);
    await vi.waitFor(() => expect(headerInput.indeterminate).toBe(true));
    expect(activate).not.toHaveBeenCalled();

    firstRowInput.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    expect(activate).not.toHaveBeenCalled();
    firstRow.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    expect(activate).toHaveBeenCalledWith({ id: "a", name: "Alpha" });
    dispose();
    host.remove();
  });

  it("keeps the legacy leading slot unwrapped", () => {
    const host = document.createElement("div");
    document.body.append(host);
    const dispose = render(
      () => (
        <DataTable.Root rows={[{ id: "a" }]}>
          <DataTable.Table aria-label="Legacy leading fixture">
            <DataTable.Body>
              {(row) => (
                <DataTable.Row row={row} rowId={row.id}>
                  <DataTable.PrimaryCell
                    leading={<span data-legacy-leading>Legacy</span>}
                    title={row.id}
                  />
                </DataTable.Row>
              )}
            </DataTable.Body>
          </DataTable.Table>
        </DataTable.Root>
      ),
      host,
    );

    expect(host.querySelector("[data-legacy-leading]")?.textContent).toBe("Legacy");
    expect(host.querySelector(".pw-dither-icon")).toBeNull();
    dispose();
    host.remove();
  });

  it("keeps the active row independent from checkbox selection", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const [activeRowId, setActiveRowId] = createSignal<string | null>("a");
    const activate = vi.fn();
    const activeChange = vi.fn((rowId: string | null) => setActiveRowId(rowId));
    const dispose = render(
      () => (
        <DataTable.Root
          rows={[{ id: "a" }, { id: "b" }]}
          selectionMode="multiple"
          activeRowId={activeRowId()}
          onActiveRowChange={activeChange}
        >
          <DataTable.Table aria-label="Active row fixture">
            <DataTable.Header>
              <DataTable.SelectionCell />
              <DataTable.ColumnHeader id="id">ID</DataTable.ColumnHeader>
            </DataTable.Header>
            <DataTable.Body>
              {(row) => (
                <DataTable.Row row={row} rowId={row.id} onActivate={activate}>
                  <DataTable.SelectionCell rowId={row.id} />
                  <DataTable.Cell>{row.id}</DataTable.Cell>
                </DataTable.Row>
              )}
            </DataTable.Body>
          </DataTable.Table>
        </DataTable.Root>
      ),
      host,
    );

    const firstRow = host.querySelector<HTMLTableRowElement>("[data-row-id='a']")!;
    const secondRow = host.querySelector<HTMLTableRowElement>("[data-row-id='b']")!;
    const secondCheckbox = secondRow.querySelector<HTMLInputElement>("input")!;
    expect(firstRow.getAttribute("aria-current")).toBe("true");
    expect(firstRow.classList.contains("pw-data-table-row--active")).toBe(true);

    secondCheckbox.click();
    await vi.waitFor(() => expect(secondCheckbox.checked).toBe(true));
    expect(activeChange).not.toHaveBeenCalled();
    expect(secondRow.getAttribute("aria-current")).toBeNull();
    expect(secondRow.classList.contains("pw-data-table-row--selected")).toBe(true);

    secondRow.click();
    await vi.waitFor(() => expect(secondRow.getAttribute("aria-current")).toBe("true"));
    expect(activeChange).toHaveBeenCalledWith("b");
    expect(activate).toHaveBeenCalledWith({ id: "b" });
    expect(secondCheckbox.checked).toBe(true);

    dispose();
    host.remove();
  });

  it("activates the first ordered visible row and repairs it after filtering", async () => {
    const host = document.createElement("div");
    document.body.append(host);
    const [rows, setRows] = createSignal([
      { id: "a", name: "Alpha" },
      { id: "b", name: "Beta" },
      { id: "c", name: "Charlie" },
    ]);
    const [activeRowId, setActiveRowId] = createSignal<string | null>(null);
    const activeChange = vi.fn((rowId: string | null) => setActiveRowId(rowId));
    const dispose = render(
      () => (
        <DataTable.Root
          rows={rows()}
          defaultSort={{ columnId: "name", direction: "descending" }}
          activeRowId={activeRowId()}
          onActiveRowChange={activeChange}
          activateFirstRow
        >
          <DataTable.Table aria-label="Automatic activation fixture">
            <DataTable.Header>
              <DataTable.ColumnHeader id="name" sortBy={(row) => row.name}>
                Name
              </DataTable.ColumnHeader>
            </DataTable.Header>
            <DataTable.Body>
              {(row) => (
                <DataTable.Row row={row} rowId={row.id}>
                  <DataTable.Cell>{row.name}</DataTable.Cell>
                </DataTable.Row>
              )}
            </DataTable.Body>
          </DataTable.Table>
        </DataTable.Root>
      ),
      host,
    );

    await vi.waitFor(() => expect(activeRowId()).toBe("c"));
    expect(host.querySelector("[data-row-id='c']")?.getAttribute("aria-current")).toBe("true");

    setRows([
      { id: "a", name: "Alpha" },
      { id: "b", name: "Beta" },
    ]);
    await vi.waitFor(() => expect(activeRowId()).toBe("b"));
    expect(host.querySelector("[data-row-id='b']")?.getAttribute("aria-current")).toBe("true");
    expect(activeChange).toHaveBeenLastCalledWith("b");

    dispose();
    host.remove();
  });

  it("omits the select-all input in single-selection mode", () => {
    const host = document.createElement("div");
    document.body.append(host);
    const dispose = render(
      () => (
        <DataTable.Root rows={[{ id: "a" }, { id: "b" }]} selectionMode="single">
          <DataTable.Table aria-label="Single selection fixture">
            <DataTable.Header>
              <DataTable.SelectionCell />
              <DataTable.ColumnHeader id="id">ID</DataTable.ColumnHeader>
            </DataTable.Header>
            <DataTable.Body>
              {(row) => (
                <DataTable.Row row={row} rowId={row.id}>
                  <DataTable.SelectionCell rowId={row.id} />
                  <DataTable.PrimaryCell icon={<span>Icon</span>} iconTone="neutral" title={row.id} />
                </DataTable.Row>
              )}
            </DataTable.Body>
          </DataTable.Table>
        </DataTable.Root>
      ),
      host,
    );

    expect(host.querySelectorAll(".pw-data-table-selection-header input")).toHaveLength(0);
    expect(host.querySelectorAll(".pw-data-table-selection-cell input")).toHaveLength(2);
    dispose();
    host.remove();
  });

  it("wires collection background collapse and row double-click maximize", () => {
    expect(tableSource).toContain("collapseDetailPanelFromCollectionBackground");
    expect(tableSource).toContain("target.closest(\"tr\")");
    expect(tableSource).toContain("onDblClick");
    expect(tableSource).toContain("detailPanelStore.getState().maximize()");
  });

  it("collapses the detail panel when clicking empty table scroll background", () => {
    detailPanelStore.setState({
      ...detailPanelDefaultState,
      activeScope: "resources:inventory",
      activeTarget: {
        scopeId: "resources:inventory",
        entityId: "a",
        entityKind: "resource",
      },
      mode: "open",
      presence: "visible",
    });

    const host = document.createElement("div");
    document.body.append(host);
    const dispose = render(
      () => (
        <DataTable.Root rows={[{ id: "a" }]} activeRowId="a">
          <DataTable.Table aria-label="Background collapse fixture">
            <DataTable.Body>
              {(row) => (
                <DataTable.Row row={row} rowId={row.id}>
                  <DataTable.Cell>{row.id}</DataTable.Cell>
                </DataTable.Row>
              )}
            </DataTable.Body>
          </DataTable.Table>
        </DataTable.Root>
      ),
      host,
    );

    const scroll = host.querySelector<HTMLElement>(".pw-data-table-scroll")!;
    scroll.click();
    expect(detailPanelStore.getState().mode).toBe("collapsed");
    expect(detailPanelStore.getState().activeTarget?.entityId).toBe("a");

    scroll.click();
    expect(detailPanelStore.getState().mode).toBe("collapsed");

    dispose();
    host.remove();
  });

  it("maximizes the detail panel when double-clicking an interactive row", () => {
    detailPanelStore.setState({
      ...detailPanelDefaultState,
      activeScope: "resources:inventory",
      activeTarget: {
        scopeId: "resources:inventory",
        entityId: "a",
        entityKind: "resource",
      },
      mode: "open",
      presence: "visible",
    });

    const host = document.createElement("div");
    document.body.append(host);
    const dispose = render(
      () => (
        <DataTable.Root
          rows={[{ id: "a" }]}
          activeRowId="a"
          onActiveRowChange={() => undefined}
        >
          <DataTable.Table aria-label="Double-click maximize fixture">
            <DataTable.Body>
              {(row) => (
                <DataTable.Row row={row} rowId={row.id}>
                  <DataTable.Cell>{row.id}</DataTable.Cell>
                </DataTable.Row>
              )}
            </DataTable.Body>
          </DataTable.Table>
        </DataTable.Root>
      ),
      host,
    );

    const row = host.querySelector<HTMLElement>("[data-row-id='a']")!;
    row.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
    expect(detailPanelStore.getState().mode).toBe("maximized");

    dispose();
    host.remove();
  });
});
