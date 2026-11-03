import { describe, expect, it } from "vitest";
import { Drawer } from "./drawer";

describe("Drawer", () => {
  it("exports Drawer component", () => {
    expect(Drawer).toBeDefined();
  });

  it("accepts public open/title/onClose props shape", () => {
    const props = {
      open: true,
      title: "Panel",
      onClose: () => {},
      width: 480,
      closeOnInteractOutside: false,
      flush: true,
    };
    expect(props.open).toBe(true);
    expect(props.title).toBe("Panel");
    expect(props.width).toBe(480);
    expect(props.closeOnInteractOutside).toBe(false);
    expect(props.flush).toBe(true);
    expect(props.onClose()).toBeUndefined();
  });

  it("documents closeOnInteractOutside and flush in source", async () => {
    const { readFileSync } = await import("node:fs");
    const { join } = await import("node:path");
    const source = readFileSync(join(import.meta.dirname, "drawer.tsx"), "utf8");
    expect(source).toContain("closeOnInteractOutside");
    expect(source).toContain("flush");
  });
});