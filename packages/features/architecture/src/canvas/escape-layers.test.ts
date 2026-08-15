import { describe, expect, it } from "vitest";

/** Pure priority for Escape handling — mirrors host layering. */
function nextEscapeAction(state: {
  editing: boolean;
  connecting: boolean;
  creating: boolean;
  menuOpen: boolean;
  hasSelection: boolean;
}): "cancel-edit" | "cancel-connect" | "cancel-create" | "close-menu" | "clear-selection" | "none" {
  if (state.editing) return "cancel-edit";
  if (state.connecting) return "cancel-connect";
  if (state.creating) return "cancel-create";
  if (state.menuOpen) return "close-menu";
  if (state.hasSelection) return "clear-selection";
  return "none";
}

describe("escape layers", () => {
  it("prioritizes text edit over connect over selection", () => {
    expect(
      nextEscapeAction({
        editing: true,
        connecting: true,
        creating: false,
        menuOpen: false,
        hasSelection: true,
      }),
    ).toBe("cancel-edit");
    expect(
      nextEscapeAction({
        editing: false,
        connecting: true,
        creating: false,
        menuOpen: false,
        hasSelection: true,
      }),
    ).toBe("cancel-connect");
    expect(
      nextEscapeAction({
        editing: false,
        connecting: false,
        creating: false,
        menuOpen: false,
        hasSelection: true,
      }),
    ).toBe("clear-selection");
  });
});
