// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import {
  FilterPopover,
  getFilterCategoryTriggerAriaLabel,
  getFilterPopoverMode,
} from "./filter-popover";

vi.mock("./hugeicons", () => ({
  Check: () => null,
  ChevronRight: () => null,
  ListFilter: () => null,
}));

describe("FilterPopover", () => {
  it("exports a shared filter component", () => {
    expect(typeof FilterPopover).toBe("function");
  });

  it("selects the empty, direct, and grouped panel structures", () => {
    expect(getFilterPopoverMode(0)).toBe("empty");
    expect(getFilterPopoverMode(1)).toBe("direct");
    expect(getFilterPopoverMode(2)).toBe("grouped");
  });

  it("includes the selected count in a category trigger's accessible name", () => {
    expect(getFilterCategoryTriggerAriaLabel("Actor", 0)).toBe("Actor filter");
    expect(getFilterCategoryTriggerAriaLabel("Actor", 2)).toBe("Actor, 2 selected");
    expect(getFilterCategoryTriggerAriaLabel("Actor", 2, "Filter by actor")).toBe(
      "Filter by actor, 2 selected",
    );
  });

  it("preserves multiple selection, clear, final action, and controlled search contracts", () => {
    const change = vi.fn();
    const clear = vi.fn();
    const finalAction = vi.fn();
    const searchChange = vi.fn();
    const props = {
      active: true,
      onClear: clear,
      search: {
        placeholder: "Search resources…",
        value: "",
        onInput: searchChange,
      },
      groups: [
        {
          id: "status",
          label: "Status",
          selectionMode: "multiple" as const,
          options: [
            {
              id: "running",
              label: "Running",
              checked: false,
              onCheckedChange: change,
            },
            { id: "active", label: "Active", checked: true, onCheckedChange: change },
          ],
          action: { label: "Manage resources…", onSelect: finalAction },
        },
      ],
    };

    props.groups[0]?.options[0]?.onCheckedChange(true);
    props.groups[0]?.action?.onSelect();
    props.onClear();
    props.search.onInput("node");

    expect(change).toHaveBeenCalledWith(true);
    expect(finalAction).toHaveBeenCalledOnce();
    expect(clear).toHaveBeenCalledOnce();
    expect(searchChange).toHaveBeenCalledWith("node");
  });

  it("preserves single selection including clearing the current value", () => {
    const select = vi.fn();
    const option = {
      id: "system",
      label: "System view",
      checked: true,
      onCheckedChange: select,
    };

    option.onCheckedChange(false);

    expect(select).toHaveBeenCalledWith(false);
  });
});
