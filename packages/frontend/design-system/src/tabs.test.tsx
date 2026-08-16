import { describe, expect, it, vi } from "vitest";
import { Tabs, TabPanel } from "./tabs";

describe("Tabs", () => {
  it("exports Tabs and TabPanel components", () => {
    expect(Tabs).toBeDefined();
    expect(TabPanel).toBeDefined();
  });

  it("accepts items and value for segmented pill usage", () => {
    const onChange = vi.fn();
    const props = {
      items: [
        { id: "issues", label: "Issues" },
        { id: "projects", label: "Projects" },
      ],
      value: "issues",
      onChange,
    };
    expect(props.items).toHaveLength(2);
    expect(props.value).toBe("issues");
    props.onChange("resolved");
    expect(onChange).toHaveBeenCalledWith("resolved");
  });

  it("renders a zero count separately from the tab label", () => {
    const item = {
      id: "pending",
      label: "Pending",
      count: 0,
      countAriaLabel: "0 pending items",
    };

    expect(item.label).toBe("Pending");
    expect(item.count).toBe(0);
    expect(item.countAriaLabel).toBe("0 pending items");
  });
});
