import type { ArchNode, ArchitectureDocument, Point } from "../document/types";
import { edgeHitPadding, hitTestPolyline, resolveRelationshipRoute } from "../render/edge-path";

export type HitKind = "port" | "handle" | "node" | "relationship" | "boundary" | "empty";

export type Hit =
  | { kind: "port"; id: string; nodeId: string; portId: string }
  | { kind: "handle"; id: string }
  | { kind: "node"; id: string }
  | { kind: "relationship"; id: string }
  | { kind: "boundary"; id: string }
  | { kind: "empty" };

const PORT_R = 10;

export function isNodeConnectable(n: Pick<ArchNode, "type" | "ports">): boolean {
  return n.type !== "note" && n.ports?.length !== 0;
}

function portPositions(
  n: Pick<ArchNode, "type" | "x" | "y" | "width" | "height" | "ports">,
): { id: string; x: number; y: number }[] {
  if (!isNodeConnectable(n)) return [];
  return [
    { id: "left", x: n.x, y: n.y + n.height / 2 },
    { id: "right", x: n.x + n.width, y: n.y + n.height / 2 },
    { id: "top", x: n.x + n.width / 2, y: n.y },
    { id: "bottom", x: n.x + n.width / 2, y: n.y + n.height },
  ];
}

/**
 * Priority: port → handle → node → relationship → boundary → empty
 */
export function hitTest(
  doc: ArchitectureDocument,
  docPoint: Point,
  opts?: { showPorts?: boolean },
): Hit {
  const showPorts = opts?.showPorts ?? true;

  // nodes top-most by zIndex desc
  const nodes = [...doc.nodes].sort((a, b) => b.zIndex - a.zIndex);

  if (showPorts) {
    for (const n of nodes) {
      for (const p of portPositions(n)) {
        const dx = docPoint.x - p.x;
        const dy = docPoint.y - p.y;
        if (dx * dx + dy * dy <= PORT_R * PORT_R) {
          return { kind: "port", id: `${n.id}:${p.id}`, nodeId: n.id, portId: p.id };
        }
      }
    }
  }

  for (const n of nodes) {
    if (
      docPoint.x >= n.x &&
      docPoint.x <= n.x + n.width &&
      docPoint.y >= n.y &&
      docPoint.y <= n.y + n.height
    ) {
      return { kind: "node", id: n.id };
    }
  }

  const pad = edgeHitPadding();
  for (const r of [...doc.relationships].reverse()) {
    const a = doc.nodes.find((n) => n.id === r.sourceNodeId);
    const b = doc.nodes.find((n) => n.id === r.targetNodeId);
    if (!a || !b) continue;
    const pts = resolveRelationshipRoute(a, b, r.manualPoints);
    if (hitTestPolyline(docPoint, pts, pad)) {
      return { kind: "relationship", id: r.id };
    }
  }

  const bounds = [...doc.boundaries].sort((a, b) => b.zIndex - a.zIndex);
  for (const b of bounds) {
    if (
      docPoint.x >= b.x &&
      docPoint.x <= b.x + b.width &&
      docPoint.y >= b.y &&
      docPoint.y <= b.y + b.height
    ) {
      return { kind: "boundary", id: b.id };
    }
  }

  return { kind: "empty" };
}
