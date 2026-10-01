/** A global size preference and one of the eight surface docks. */
export type FloatingWindowDock = "top-left" | "top" | "top-right" | "left" | "right" | "bottom-left" | "bottom" | "bottom-right";
export type FloatingWindowGeometry = { version: 2; dock: FloatingWindowDock; width: number; height: number };
export const FLOATING_WINDOW_DOCKS: readonly FloatingWindowDock[] = ["top-left", "top", "top-right", "left", "right", "bottom-left", "bottom", "bottom-right"];
export type FloatingWindowBounds = { width: number; height: number };
export type FloatingWindowRect = FloatingWindowBounds & { x: number; y: number };
export type FloatingWindowResizeEdge = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";

export const FLOATING_WINDOW_DEFAULT_WIDTH = 520;
export const FLOATING_WINDOW_DEFAULT_HEIGHT = 420;
export const FLOATING_WINDOW_MIN_WIDTH = 320;
export const FLOATING_WINDOW_MIN_HEIGHT = 280;
/** Fallback before a host has been measured; live limits use the whole surface. */
export const FLOATING_WINDOW_MAX_WIDTH = 720;
export const FLOATING_WINDOW_BUBBLE_SIZE = 48;
export const defaultFloatingWindowGeometry: FloatingWindowGeometry = {
  version: 2,
  dock: "bottom-right",
  width: FLOATING_WINDOW_DEFAULT_WIDTH,
  height: FLOATING_WINDOW_DEFAULT_HEIGHT,
};

const clamp = (value: number, min: number, max: number) => Math.min(Math.max(value, min), max);

export function windowArea(bounds: FloatingWindowBounds): FloatingWindowRect {
  const margin = Math.min(16, Math.max(0, (Math.min(bounds.width, bounds.height) - FLOATING_WINDOW_BUBBLE_SIZE) / 2));
  return {
    x: margin, y: margin,
    width: Math.max(1, bounds.width - margin * 2),
    height: Math.max(1, bounds.height - margin * 2),
  };
}

export function clampFloatingWindowWidth(width: number, containerWidth: number): number {
  const upper = containerWidth > 0 ? windowArea({ width: containerWidth, height: containerWidth }).width : FLOATING_WINDOW_MAX_WIDTH;
  return clamp(Number.isFinite(width) ? width : FLOATING_WINDOW_DEFAULT_WIDTH, Math.min(FLOATING_WINDOW_MIN_WIDTH, upper), upper);
}

/** Legacy anchors can only be migrated precisely after the surface is measured. */
export function readFloatingWindowGeometry(raw: string | null, bounds?: FloatingWindowBounds): FloatingWindowGeometry {
  try {
    const parsed: unknown = JSON.parse(raw ?? "null");
    if (typeof parsed === "number" && Number.isFinite(parsed) && parsed > 0) {
      return { ...defaultFloatingWindowGeometry, width: parsed };
    }
    if (parsed && typeof parsed === "object") {
      const value = parsed as Record<string, unknown>;
      if (typeof value.width === "number" && typeof value.height === "number"
        && [value.width, value.height].every(Number.isFinite) && value.width > 0 && value.height > 0) {
        const size = { width: value.width, height: value.height };
        if (value.version === 2 && FLOATING_WINDOW_DOCKS.includes(value.dock as FloatingWindowDock)) {
          return { version: 2, dock: value.dock as FloatingWindowDock, ...size };
        }
        if (value.version === 1 && typeof value.anchorX === "number" && typeof value.anchorY === "number"
          && [value.anchorX, value.anchorY].every(Number.isFinite)) {
          const host = bounds ?? { width: 1024, height: 768 };
          const area = windowArea(host);
          const rect = resolveFloatingWindowRect({ ...defaultFloatingWindowGeometry, ...size }, host);
          rect.x = clamp(area.x + clamp(value.anchorX, 0, 1) * area.width - rect.width, area.x, area.x + area.width - rect.width);
          rect.y = clamp(area.y + clamp(value.anchorY, 0, 1) * area.height - rect.height, area.y, area.y + area.height - rect.height);
          return { version: 2, dock: nearestFloatingWindowDock(rect, host, "bottom-right"), ...size };
        }
      }
    }
  } catch {
    // An invalid preference never prevents opening a window.
  }
  return { ...defaultFloatingWindowGeometry };
}

export function windowDockRect(dock: FloatingWindowDock, size: FloatingWindowBounds, bounds: FloatingWindowBounds): FloatingWindowRect {
  const area = windowArea(bounds);
  const horizontal = dock.includes("left") ? 0 : dock.includes("right") ? 1 : 0.5;
  const vertical = dock.startsWith("top") ? 0 : dock.startsWith("bottom") ? 1 : 0.5;
  return { ...size, x: area.x + (area.width - size.width) * horizontal, y: area.y + (area.height - size.height) * vertical };
}

export function nearestFloatingWindowDock(rect: FloatingWindowRect, bounds: FloatingWindowBounds, current: FloatingWindowDock): FloatingWindowDock {
  let nearest = current;
  const distance = (dock: FloatingWindowDock) => {
    const candidate = windowDockRect(dock, rect, bounds);
    return Math.hypot(rect.x - candidate.x, rect.y - candidate.y);
  };
  let best = distance(current);
  for (const dock of FLOATING_WINDOW_DOCKS) {
    const next = distance(dock);
    if (next < best - 0.001) { nearest = dock; best = next; }
  }
  return nearest;
}

export function resolveFloatingWindowRect(geometry: FloatingWindowGeometry, bounds: FloatingWindowBounds, bubble = false): FloatingWindowRect {
  const area = windowArea(bounds);
  const diameter = Math.min(FLOATING_WINDOW_BUBBLE_SIZE, area.width, area.height);
  const width = bubble ? diameter : clamp(geometry.width, Math.min(FLOATING_WINDOW_MIN_WIDTH, area.width), area.width);
  const height = bubble ? diameter : clamp(geometry.height, Math.min(FLOATING_WINDOW_MIN_HEIGHT, area.height), area.height);
  return windowDockRect(geometry.dock, { width, height }, bounds);
}

export function windowGeometryAt(rect: FloatingWindowRect, bounds: FloatingWindowBounds, previous: FloatingWindowGeometry, resize = false): FloatingWindowGeometry {
  return {
    version: 2,
    dock: resize ? previous.dock : nearestFloatingWindowDock(rect, bounds, previous.dock),
    width: resize ? rect.width : previous.width,
    height: resize ? rect.height : previous.height,
  };
}

export function moveFloatingWindowRect(rect: FloatingWindowRect, dx: number, dy: number, bounds: FloatingWindowBounds): FloatingWindowRect {
  const area = windowArea(bounds);
  return {
    ...rect,
    x: clamp(rect.x + dx, area.x, area.x + area.width - rect.width),
    y: clamp(rect.y + dy, area.y, area.y + area.height - rect.height),
  };
}

export function resizeFloatingWindowRect(rect: FloatingWindowRect, edge: FloatingWindowResizeEdge, dx: number, dy: number, bounds: FloatingWindowBounds): FloatingWindowRect {
  const area = windowArea(bounds);
  const minWidth = Math.min(FLOATING_WINDOW_MIN_WIDTH, area.width);
  const minHeight = Math.min(FLOATING_WINDOW_MIN_HEIGHT, area.height);
  let left = rect.x;
  let top = rect.y;
  let right = rect.x + rect.width;
  let bottom = rect.y + rect.height;
  if (edge.includes("w")) left = clamp(left + dx, area.x, right - minWidth);
  if (edge.includes("e")) right = clamp(right + dx, left + minWidth, area.x + area.width);
  if (edge.includes("n")) top = clamp(top + dy, area.y, bottom - minHeight);
  if (edge.includes("s")) bottom = clamp(bottom + dy, top + minHeight, area.y + area.height);
  return { x: left, y: top, width: right - left, height: bottom - top };
}
