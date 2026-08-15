import { describe, expect, it } from "vitest";
import { clearSessionSnapshot, getSessionSnapshot, setSessionSnapshot } from "./session-viewport";

describe("session viewport cache", () => {
  it("stores and restores", () => {
    clearSessionSnapshot("sys");
    setSessionSnapshot("sys", {
      viewport: { x: 10, y: 20, zoom: 1.5 },
      selection: { nodeIds: ["a"], relationshipIds: [], boundaryIds: [] },
      environmentId: "production",
    });
    const s = getSessionSnapshot("sys");
    expect(s?.viewport.zoom).toBe(1.5);
    expect(s?.selection.nodeIds).toEqual(["a"]);
    expect(s?.environmentId).toBe("production");
  });
});
