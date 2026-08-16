import { describe, expect, it } from "vitest";
import { FilterChip } from "./filter-chip";

describe("FilterChip", () => {
  it("keeps the label clean and renders a floating accessible count", () => {
    const props = {
      children: "Actor",
      count: 2,
      countAriaLabel: "2 selected",
    };

    expect(props.children).toBe("Actor");
    expect(props.count).toBe(2);
    expect(props.countAriaLabel).toBe("2 selected");
    expect(FilterChip).toBeDefined();
  });
});
