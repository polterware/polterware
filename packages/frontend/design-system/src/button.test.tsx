import { describe, expect, it } from "vitest";
import { Button } from "./button";

describe("Button", () => {
  it("exports a function component", () => {
    expect(Button).toBeDefined();
  });
});
