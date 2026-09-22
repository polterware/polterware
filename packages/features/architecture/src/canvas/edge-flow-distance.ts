/**
 * Advance an edge-flow dot along a path, keeping fractional progress when the
 * path length morphs (e.g. while dragging a connected node).
 */
export function nextEdgeFlowDistance(
  currentDist: number,
  prevLength: number,
  nextLength: number,
  deltaPx: number,
): number {
  if (nextLength <= 0) return 0;
  let dist = currentDist;
  if (prevLength > 0 && prevLength !== nextLength) {
    dist = (dist / prevLength) * nextLength;
  }
  return (dist + deltaPx) % nextLength;
}
