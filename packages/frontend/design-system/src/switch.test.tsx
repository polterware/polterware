// @vitest-environment jsdom

import { describe, expect, it } from "vitest";
import { Switch, SwitchRow } from "./switch";

describe("Switch", () => {
  it("exports Switch and SwitchRow as component functions", () => {
    expect(Switch).toBeDefined();
    expect(SwitchRow).toBeDefined();
  });
});
