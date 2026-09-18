import type { ArchNode } from "@polterware/frontend-canvas";
import { typeLabelFor } from "./components-list";
import type { ArchViewMember } from "./extra-fixtures";

export type ViewEditorNode = {
  id: string;
  name: string;
  type?: string;
  layer?: string;
};

export function memberLayerForType(type: string): string {
  if (
    type === "database"
    || type === "cache"
    || type === "queue"
    || type === "event_stream"
    || type === "storage"
    || type === "data_warehouse"
    || type === "search_engine"
  ) {
    return "Data";
  }
  if (
    type === "api_gateway"
    || type === "load_balancer"
    || type === "cdn"
    || type === "web_application"
  ) {
    return "Edge";
  }
  if (type === "ci_pipeline" || type === "build_system" || type === "artifact_registry") {
    return "Delivery";
  }
  return "Application";
}

export function nodesFromArchitectureDocument(nodes: readonly ArchNode[]): ViewEditorNode[] {
  return nodes.map((node) => ({
    id: node.id,
    name: node.name,
    type: typeLabelFor(node.type, node.subtype),
    layer: memberLayerForType(node.type),
  }));
}

export function membersFromSelection(
  memberIds: readonly string[],
  availableNodes: readonly ViewEditorNode[],
  existingMembers?: readonly ArchViewMember[],
): Array<{ nodeId: string; name: string; type?: string; layer?: string }> {
  const nodes = new Map(availableNodes.map((node) => [node.id, node]));
  const existing = new Map((existingMembers ?? []).map((member) => [member.id, member]));
  return memberIds.flatMap((id) => {
    const node = nodes.get(id);
    const previous = existing.get(id);
    if (!node && !previous) return [];
    return [{
      nodeId: id,
      name: node?.name ?? previous?.name ?? id,
      type: previous?.type ?? node?.type,
      layer: previous?.layer ?? node?.layer,
    }];
  });
}
