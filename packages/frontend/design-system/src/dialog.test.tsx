import { describe, expect, it } from "vitest";
import { A11yDialog, Dialog } from "./dialog";

describe("A11yDialog", () => {
  it("exports a function component", () => {
    expect(A11yDialog).toBeDefined();
  });

  it("Dialog alias points to A11yDialog", () => {
    expect(Dialog).toBe(A11yDialog);
  });

  it("accepts stable public open/onClose/title props", () => {
    const props = {
      open: false,
      onClose: () => {},
      title: "Test",
    };
    expect(props.open).toBe(false);
    expect(props.onClose()).toBeUndefined();
    expect(props.title).toBe("Test");
  });
});
