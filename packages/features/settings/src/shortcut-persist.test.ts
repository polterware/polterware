import { describe, expect, it } from "vitest";
import { decodeStoredShortcutOverrides } from "./shortcut-persist";

describe("decodeStoredShortcutOverrides", () => {
  it("keeps known command ids and drops invalid chords", () => {
    expect(
      decodeStoredShortcutOverrides({
        "shell.toggle_commander": "mod+p",
        "not.a.command": "mod+x",
        "shell.toggle_sidebar": "not-a-chord",
      }),
    ).toEqual({
      "shell.toggle_commander": {
        key: "p",
        mod: true,
        shift: false,
        alt: false,
      },
    });
  });
});
