import {
  isNodeConnectable,
  type ArchNode,
} from "@polterware/frontend-canvas";

export function shouldShowConnectionHandles(
  node: Pick<ArchNode, "type" | "ports">,
  connectActive: boolean,
): boolean {
  return connectActive && isNodeConnectable(node);
}

/** Canvas port sides — matches hit-test `portId` values. */
export type HandleSide = "left" | "right" | "top" | "bottom";

/**
 * Visual intensity for a connection handle.
 * - `hot` — ports visible (node hover / connect tool), not specially targeted
 * - `target` — whole card is a valid drop target during an in-progress connection
 * - `focus` — cursor is on this exact port (start or complete a connection)
 */
export type HandleState = "hot" | "target" | "focus";

export function resolveHandleState(args: {
  nodeId: string;
  side: HandleSide;
  hoveredPort: { nodeId: string; portId: string } | null;
  isConnectTarget: boolean;
}): HandleState {
  const port = args.hoveredPort;
  if (port && port.nodeId === args.nodeId && port.portId === args.side) {
    return "focus";
  }
  if (args.isConnectTarget) return "target";
  return "hot";
}

export const HANDLE_SIDES: readonly HandleSide[] = [
  "left",
  "right",
  "top",
  "bottom",
];
