import { describe, expect, it } from "vitest";
import { clampZoom, zoomAt, fitBounds, screenToDocument, documentToScreen } from "./viewport";

describe("viewport", () => {
  it("clamps zoom", () => {
    expect(clampZoom(0.01)).toBe(0.25);
    expect(clampZoom(10)).toBe(2.5);
  });

  it("zooms about focal point keeping document under cursor", () => {
    const vp = { x: 0, y: 0, zoom: 1 };
    const focal = { x: 100, y: 50 };
    const next = zoomAt(vp, focal, 2);
    const before = screenToDocument(vp, focal);
    const after = screenToDocument(next, focal);
    expect(after.x).toBeCloseTo(before.x, 5);
    expect(after.y).toBeCloseTo(before.y, 5);
  });

  it("fits bounds into view", () => {
    const vp = fitBounds({ x: 0, y: 0, zoom: 1 }, { minX: 0, minY: 0, maxX: 400, maxY: 200 }, 800, 600, 0);
    expect(vp.zoom).toBeLessThanOrEqual(1);
    const c = documentToScreen(vp, { x: 200, y: 100 });
    expect(c.x).toBeCloseTo(400, 0);
    expect(c.y).toBeCloseTo(300, 0);
  });
});
