import type { Point, Rect, Viewport } from "../document/types";
import { MAX_ZOOM, MIN_ZOOM } from "../document/types";

export function clampZoom(z: number): number {
  if (!Number.isFinite(z)) return 1;
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, z));
}

export function screenToDocument(vp: Viewport, screen: Point): Point {
  return {
    x: (screen.x - vp.x) / vp.zoom,
    y: (screen.y - vp.y) / vp.zoom,
  };
}

export function documentToScreen(vp: Viewport, docPt: Point): Point {
  return {
    x: docPt.x * vp.zoom + vp.x,
    y: docPt.y * vp.zoom + vp.y,
  };
}

export function panBy(vp: Viewport, dx: number, dy: number): Viewport {
  return { ...vp, x: vp.x + dx, y: vp.y + dy };
}

/** Zoom keeping the document point under focalScreen fixed on screen. */
export function zoomAt(vp: Viewport, focalScreen: Point, nextZoom: number): Viewport {
  const z = clampZoom(nextZoom);
  const doc = screenToDocument(vp, focalScreen);
  return {
    ...vp,
    zoom: z,
    x: focalScreen.x - doc.x * z,
    y: focalScreen.y - doc.y * z,
  };
}

export function fitBounds(
  vp: Viewport,
  bounds: { minX: number; minY: number; maxX: number; maxY: number },
  viewWidth: number,
  viewHeight: number,
  padding = 48,
): Viewport {
  const w = Math.max(1, bounds.maxX - bounds.minX);
  const h = Math.max(1, bounds.maxY - bounds.minY);
  const z = clampZoom(
    Math.min((viewWidth - padding * 2) / w, (viewHeight - padding * 2) / h, 1),
  );
  const cx = (bounds.minX + bounds.maxX) / 2;
  const cy = (bounds.minY + bounds.maxY) / 2;
  return {
    ...vp,
    zoom: z,
    x: viewWidth / 2 - cx * z,
    y: viewHeight / 2 - cy * z,
  };
}

export function centerOn(
  vp: Viewport,
  point: Point,
  viewWidth: number,
  viewHeight: number,
  zoom?: number,
): Viewport {
  const z = clampZoom(zoom ?? vp.zoom);
  return {
    ...vp,
    zoom: z,
    x: viewWidth / 2 - point.x * z,
    y: viewHeight / 2 - point.y * z,
  };
}

export function visibleDocumentRect(
  vp: Viewport,
  viewWidth: number,
  viewHeight: number,
  buffer = 100,
): Rect {
  const tl = screenToDocument(vp, { x: -buffer, y: -buffer });
  const br = screenToDocument(vp, { x: viewWidth + buffer, y: viewHeight + buffer });
  return {
    x: tl.x,
    y: tl.y,
    width: br.x - tl.x,
    height: br.y - tl.y,
  };
}

export function rectsIntersect(a: Rect, b: Rect): boolean {
  return (
    a.x < b.x + b.width &&
    a.x + a.width > b.x &&
    a.y < b.y + b.height &&
    a.y + a.height > b.y
  );
}

export function rectContains(outer: Rect, inner: Rect): boolean {
  return (
    inner.x >= outer.x &&
    inner.y >= outer.y &&
    inner.x + inner.width <= outer.x + outer.width &&
    inner.y + inner.height <= outer.y + outer.height
  );
}
