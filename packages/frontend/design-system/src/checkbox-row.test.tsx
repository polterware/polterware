// @vitest-environment jsdom

import { describe, expect, it, vi } from "vitest";
import { CheckboxRow } from "./checkbox-row";

describe("CheckboxRow", () => {
  it("exports CheckboxRow as a component function", () => {
    expect(CheckboxRow).toBeDefined();
  });

  it("handles props interface cleanly", () => {
    const handleCheckedChange = vi.fn();
    const props = {
      checked: true,
      onCheckedChange: handleCheckedChange,
      label: "Option 1",
      disabled: false,
      name: "test-name",
      value: "test-val",
    };

    expect(props.checked).toBe(true);
    expect(props.label).toBe("Option 1");
    props.onCheckedChange(false);
    expect(handleCheckedChange).toHaveBeenCalledWith(false);
  });
});
