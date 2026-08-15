import {
  buildOperationalMap,
  createEmptyDocument,
  deserializeAndMigrate,
  serializeDocument,
  type ArchitectureDocument,
  type ParseResult,
} from "@polterware/frontend-canvas";

export function loadDocumentFromLayoutBody(
  bodyJson: string | undefined,
  systemId: string,
  systems: Array<{ id: string; name: string }> = [],
  resources: Array<{
    id: string;
    name: string;
    type?: string;
    source?: string;
    externalKey?: string;
    systemId?: string;
  }> = [],
): ParseResult {
  if (bodyJson && bodyJson !== "{}" && bodyJson.trim() !== "") {
    return deserializeAndMigrate(bodyJson, systemId);
  }

  // Build from systems/resources as legacy operational map then migrate
  const legacy = buildOperationalMap(systems, resources);
  if (legacy.nodes.length === 0) {
    return { doc: createEmptyDocument({ systemId }), warnings: [], migrated: false };
  }
  return deserializeAndMigrate(
    JSON.stringify({ nodes: legacy.nodes, edges: legacy.edges }),
    systemId,
  );
}

export function documentToLayoutBody(doc: ArchitectureDocument): string {
  return serializeDocument(doc);
}
