import type { ArchNode, RelationshipType } from "@polterware/frontend-canvas";
import { inferRelationshipType as suggestFirst } from "./suggestions";

export function canConnect(
  source: ArchNode,
  target: ArchNode,
  relType: RelationshipType,
): { ok: boolean; reason?: string } {
  if (source.id === target.id && relType !== "custom") {
    return { ok: false, reason: "Self-connections are not allowed" };
  }
  if (source.type === "note" || target.type === "note") {
    return { ok: false, reason: "Notes cannot participate in relationships" };
  }
  return { ok: true };
}

/** @deprecated prefer suggestRelationshipTypes from suggestions.ts */
export function inferRelationshipType(
  source: ArchNode,
  target: ArchNode,
): RelationshipType {
  return suggestFirst(source, target);
}
