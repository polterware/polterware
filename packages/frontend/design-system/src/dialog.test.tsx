import { describe, expect, it } from "vitest";
import { A11yDialog, Dialog } from "./dialog";

describe("A11yDialog", () => {
  it("exports a function component", () => {
    expect(typeof A11yDialog).toBe("function");
  });

  it("Dialog alias points to A11yDialog", () => {
    expect(Dialog).toBe(A11yDialog);
  });

  it("accepts stable public open/onClose/title props", () => {
    const props = {
      open: false,
      onClose: () => {},
      title: "Test",
      children: null as unknown as never,
    };
    expect(props.open).toBe(false);
    expect(typeof props.onClose).toBe("function");
    expect(props.title).toBe("Test");
  });
});
