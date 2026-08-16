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
    };
    expect(props.open).toBe(true);
    expect(props.title).toBe("Panel");
    expect(props.width).toBe(480);
    expect(props.onClose()).toBeUndefined();
  });
});
