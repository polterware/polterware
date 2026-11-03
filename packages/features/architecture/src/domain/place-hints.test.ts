import { describe, expect, it } from "vitest";
import { getComponentType } from "./registry";
import { placeHints } from "./place-hints";

describe("placeHints", () => {
  it("labels database presets and keeps type blurb separate from purpose", () => {
    const def = getComponentType("postgres");
    expect(def).toBeTruthy();
    const hints = placeHints(def!);
    expect(hints.kindLabel).toBe("Database");
    expect(hints.typeBlurb.length).toBeGreaterThan(0);
    expect(hints.purposePlaceholder.toLowerCase()).toMatch(/store|cache|stream/);
    expect(hints.statusLine).toMatch(/planned/i);
  });
});
