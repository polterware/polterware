// Canvas layout worker — horizontal / vertical auto-layout (P2)

export type LayoutNode = {
  id: string;
  x: number;
  y: number;
  width?: number;
  height?: number;
  layoutPinned?: boolean;
};

export type LayoutRequest = {
  nodes: LayoutNode[];
  mode: "horizontal" | "vertical";
  gap?: number;
};

export type LayoutResponse = { nodes: Array<{ id: string; x: number; y: number }> };

export function layoutSync(req: LayoutRequest): LayoutResponse {
  const gap = req.gap ?? 48;
  const movable = req.nodes.filter((n) => !n.layoutPinned);
  const pinned = req.nodes.filter((n) => n.layoutPinned);

  let cursor = 40;
  const placed: Array<{ id: string; x: number; y: number }> = pinned.map((n) => ({
    id: n.id,
    x: n.x,
    y: n.y,
  }));

  for (const n of movable) {
    const w = n.width ?? 140;
    const h = n.height ?? 56;
    if (req.mode === "horizontal") {
      placed.push({ id: n.id, x: cursor, y: 40 });
      cursor += w + gap;
    } else {
      placed.push({ id: n.id, x: 40, y: cursor });
      cursor += h + gap;
    }
  }
  return { nodes: placed };
}
