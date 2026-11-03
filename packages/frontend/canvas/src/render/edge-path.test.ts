import { describe, expect, it } from "vitest";
import {
  hitTestPolyline,
  orthogonalPath,
  pointNearSegment,
  pointsToSvgPath,
  polylineMidpoint,
  pointsToRoundedSvgPath,
  resolveAutomaticConnection,
} from "./edge-path";

describe("edge path", () => {
  it("builds orthogonal path", () => {
    const p = orthogonalPath(0, 0, 100, 50);
    expect(p.startsWith("M ")).toBe(true);
    expect(p.includes("L ")).toBe(true);
  });

  it("detects near segment", () => {
    expect(pointNearSegment({ x: 50, y: 2 }, { x: 0, y: 0 }, { x: 100, y: 0 }, 4)).toBe(true);
    expect(pointNearSegment({ x: 50, y: 20 }, { x: 0, y: 0 }, { x: 100, y: 0 }, 4)).toBe(false);
  });

  it("routes right-to-left based on dominant horizontal delta", () => {
    const route = resolveAutomaticConnection(
      { x: 0, y: 0, width: 100, height: 100 },
      { x: 200, y: 0, width: 100, height: 100 },
    );
    const pts = route.points;
    expect(pts.length).toBe(4);
    expect(route.source.side).toBe("right");
    expect(route.target.side).toBe("left");
    expect(pts[0]!.x).toBe(100);
    expect(pts[3]!.x).toBe(200);
  });

  it("routes bottom-to-top based on dominant vertical delta", () => {
    const route = resolveAutomaticConnection(
      { x: 0, y: 0, width: 100, height: 100 },
      { x: 0, y: 200, width: 100, height: 100 },
    );
    const pts = route.points;
    expect(pts.length).toBe(4);
    expect(route.source.side).toBe("bottom");
    expect(route.target.side).toBe("top");
    expect(pts[0]!.y).toBe(100);
    expect(pts[3]!.y).toBe(200);
  });

  it("routes left-to-right when the target is left of the source", () => {
    const route = resolveAutomaticConnection(
      { x: 200, y: 0, width: 100, height: 100 },
      { x: 0, y: 0, width: 100, height: 100 },
    );
    expect(route.source.side).toBe("left");
    expect(route.target.side).toBe("right");
  });

  it("routes top-to-bottom when the target is above the source", () => {
    const route = resolveAutomaticConnection(
      { x: 0, y: 200, width: 100, height: 100 },
      { x: 0, y: 0, width: 100, height: 100 },
    );
    expect(route.source.side).toBe("top");
    expect(route.target.side).toBe("bottom");
  });

  it("routes with horizontal tie-break", () => {
    // dx = 100, dy = 100
    const route = resolveAutomaticConnection(
      { x: 0, y: 0, width: 100, height: 100 },
      { x: 100, y: 100, width: 100, height: 100 },
    );
    const pts = route.points;
    expect(pts.length).toBe(4);
    expect(route.source.side).toBe("right");
    expect(route.target.side).toBe("left");
    expect(pts[0]!.x).toBe(100);
    expect(pts[3]!.x).toBe(100);
  });

  it("converts points to SVG path", () => {
    const path = pointsToSvgPath([{ x: 0, y: 0 }, { x: 100, y: 0 }]);
    expect(path).toBe("M 0 0 L 100 0");
  });

  it("hit tests polyline", () => {
    const pts = [{ x: 0, y: 0 }, { x: 100, y: 0 }, { x: 100, y: 100 }];
    expect(hitTestPolyline({ x: 50, y: 0 }, pts, 4)).toBe(true);
    expect(hitTestPolyline({ x: 100, y: 50 }, pts, 4)).toBe(true);
    expect(hitTestPolyline({ x: 50, y: 50 }, pts, 4)).toBe(false);
  });

  it("calculates polyline midpoint", () => {
    expect(polylineMidpoint([])).toBeNull();

    expect(polylineMidpoint([{ x: 10, y: 10 }])).toEqual({ x: 10, y: 10 });

    const two = [{ x: 0, y: 0 }, { x: 100, y: 0 }];
    expect(polylineMidpoint(two)).toEqual({ x: 50, y: 0 });

    const three = [{ x: 0, y: 0 }, { x: 100, y: 0 }, { x: 100, y: 100 }];
    expect(polylineMidpoint(three)).toEqual({ x: 100, y: 0 });

    const asymmetric = [{ x: 0, y: 0 }, { x: 10, y: 0 }, { x: 10, y: 100 }];
    expect(polylineMidpoint(asymmetric)).toEqual({ x: 10, y: 45 });

    const zeroLength = [{ x: 0, y: 0 }, { x: 0, y: 0 }, { x: 100, y: 0 }];
    expect(polylineMidpoint(zeroLength)).toEqual({ x: 50, y: 0 });

    const zeroLengthCenter = [
      { x: 0, y: 0 },
      { x: 50, y: 0 },
      { x: 50, y: 0 },
      { x: 100, y: 0 },
    ];
    expect(polylineMidpoint(zeroLengthCenter)).toEqual({ x: 50, y: 0 });
  });

  describe("pointsToRoundedSvgPath", () => {    it("handles empty and degenerate routes", () => {
      expect(pointsToRoundedSvgPath([])).toBe("");
      expect(pointsToRoundedSvgPath([{ x: 10, y: 10 }])).toBe("M 10 10");
      expect(pointsToRoundedSvgPath([{ x: 10, y: 10 }, { x: 10, y: 10 }])).toBe("M 10 10");
    });

    it("handles straight routes", () => {
      expect(pointsToRoundedSvgPath([{ x: 0, y: 0 }, { x: 100, y: 0 }])).toBe("M 0 0 L 100 0");
    });

    it("handles four-point rounded route", () => {
      const pts = [
        { x: 0, y: 0 },
        { x: 100, y: 0 },
        { x: 100, y: 100 },
        { x: 200, y: 100 },
      ];
      const p = pointsToRoundedSvgPath(pts, 10);
      expect(p).toBe("M 0 0 L 90 0 Q 100 0 100 10 L 100 90 Q 100 100 110 100 L 200 100");
    });

    it("clamps radius to half of segment lengths", () => {
      const pts = [
        { x: 0, y: 0 },
        { x: 10, y: 0 },
        { x: 10, y: 100 },
      ];
      const p = pointsToRoundedSvgPath(pts, 10);
      // d01 = 10, d12 = 100. r = min(10, 5, 50) = 5
      expect(p).toBe("M 0 0 L 5 0 Q 10 0 10 5 L 10 100");
    });

    it("clamps invalid radii without reversing the route", () => {
      const pts = [
        { x: 0, y: 0 },
        { x: 10, y: 0 },
        { x: 10, y: 10 },
      ];

      expect(pointsToRoundedSvgPath(pts, -4)).toBe(
        "M 0 0 L 10 0 L 10 10",
      );
      expect(pointsToRoundedSvgPath(pts, Number.NaN)).toContain("Q");
    });

    it("removes consecutive duplicate points", () => {
      const pts = [
        { x: 0, y: 0 },
        { x: 100, y: 0 },
        { x: 100, y: 0 },
        { x: 100, y: 100 },
      ];
      const p = pointsToRoundedSvgPath(pts, 10);
      expect(p).toBe("M 0 0 L 90 0 Q 100 0 100 10 L 100 100");
    });
  });
});
