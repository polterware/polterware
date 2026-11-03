import { describe, expect, it, vi } from "vitest";
import { readFileSync } from "node:fs";
import { repoPath } from "../test/repo-path";
import { Tabs, TabPanel } from "./tabs";

describe("Tabs", () => {
  it("exports Tabs and TabPanel components", () => {
    expect(Tabs).toBeDefined();
    expect(TabPanel).toBeDefined();
  });

  it("names tab triggers through Root ids instead of overriding the DOM id", () => {
    const source = readFileSync(repoPath("packages/frontend/design-system/src/tabs.tsx"), "utf8");
    expect(source).toContain("ids={{ trigger: (value) => `tab-${value}` }}");
    expect(source).toContain("triggerId={`tab-${props.item.id}`}");
    expect(source).not.toContain("id={`tab-${props.item.id}`}");
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

  it("accepts optional rich tooltip metadata on items", () => {
    const onChange = vi.fn();
    const props = {
      items: [
        {
          id: "canvas",
          label: "Canvas",
          tooltip: {
            title: "System map",
            description: "Draw and arrange components on a freeform canvas.",
            icon: (() => null) as never,
          },
        },
        { id: "components", label: "Components" },
      ],
      value: "canvas",
      onChange,
    };
    expect(props.items[0]?.tooltip?.title).toBe("System map");
    expect(props.items[1]?.tooltip).toBeUndefined();
  });
});
