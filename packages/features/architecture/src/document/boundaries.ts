import type { ArchitectureDocument } from "@polterware/frontend-canvas";
import {
  addBoundary,
  moveBoundaryWithMembers,
  touchUpdated,
  updateNode,
} from "@polterware/frontend-canvas";

export function createBoundaryFromSelection(
  doc: ArchitectureDocument,
  memberIds: string[],
  name = "Group",
): ArchitectureDocument {
  const nodes = doc.nodes.filter((n) => memberIds.includes(n.id));
  if (!nodes.length) return doc;
  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity;
  for (const n of nodes) {
    minX = Math.min(minX, n.x);
    minY = Math.min(minY, n.y);
    maxX = Math.max(maxX, n.x + n.width);
    maxY = Math.max(maxY, n.y + n.height);
  }
  return addBoundary(
    doc,
    {
      name,
      x: minX - 16,
      y: minY - 28,
      width: maxX - minX + 32,
      height: maxY - minY + 44,
    },
    memberIds,
  );
}

export function setBoundaryCollapsed(
  doc: ArchitectureDocument,
  boundaryId: string,
  collapsed: boolean,
): ArchitectureDocument {
  return touchUpdated({
    ...doc,
    boundaries: doc.boundaries.map((b) =>
      b.id === boundaryId ? { ...b, collapsed } : b,
    ),
  });
}

export function removeFromBoundary(
  doc: ArchitectureDocument,
  nodeId: string,
): ArchitectureDocument {
  return updateNode(doc, nodeId, { boundaryId: null });
}

export { moveBoundaryWithMembers };
