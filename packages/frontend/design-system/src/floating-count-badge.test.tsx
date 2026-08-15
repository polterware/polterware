import { describe, expect, it } from "vitest";
import {
  FloatingCountBadge,
  getFloatingCountAriaLabel,
  hasFloatingCount,
} from "./floating-count-badge";

describe("FloatingCountBadge", () => {
  it("keeps zero visible and provides an accessible count label", () => {
    expect(hasFloatingCount(0)).toBe(true);
    expect(getFloatingCountAriaLabel(0, "0 results")).toBe("0 results");
  });

  it("keeps multi-digit values and omits absent counts", () => {
    expect(getFloatingCountAriaLabel(12)).toBe("12 items");
    expect(hasFloatingCount(undefined)).toBe(false);
    expect(typeof FloatingCountBadge).toBe("function");
  });
});
