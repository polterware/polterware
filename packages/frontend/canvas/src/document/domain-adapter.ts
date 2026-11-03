import type { ArchNode, NodeType, Port, RelationshipType, Size } from "./types";

/** Injected by architecture domain — engine stays free of product catalogs. */
export type DomainAdapter = {
  defaultPorts: (nodeType: NodeType) => Port[];
  defaultNodeSize: (nodeType: NodeType) => Size;
  canConnect: (
    source: ArchNode,
    target: ArchNode,
    relType: RelationshipType,
  ) => { ok: boolean; reason?: string };
};
