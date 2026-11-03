import type { Relationship } from "@polterware/frontend-canvas";

export type RelationshipAppearance = {
  dim: boolean;
};

export type RelationshipMarkers = {
  start?: string;
  end: string;
};

export function relationshipMarkers(
  direction: Relationship["direction"],
): RelationshipMarkers {
  return {
    start: direction === "bidirectional" ? "url(#arrow)" : undefined,
    end: "url(#arrow)",
  };
}

export function resolveRelationshipAppearance(
  rel: Pick<Relationship, "origin" | "evidence" | "state">,
): RelationshipAppearance {
  const dim = rel.state === "disabled" || rel.state === "unresolved";
  return { dim };
}
