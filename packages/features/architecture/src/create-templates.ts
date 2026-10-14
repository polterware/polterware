import type { IconTileTone } from "@polterware/frontend-design-system";
import type { ArchNodeDraft } from "./create-node-dialog";
import { getComponentType } from "./domain/registry";

export type ArchitectureComponentTemplate = {
  id: string;
  title: string;
  description: string;
  tone: IconTileTone;
  brand?: string;
  draft: ArchNodeDraft;
};

export type ArchitectureViewTemplate = {
  id: string;
  title: string;
  description: string;
  tone: IconTileTone;
  name: string;
};

function draftFromCatalogId(catalogId: string): ArchNodeDraft | null {
  const def = getComponentType(catalogId);
  if (!def || !def.placeable || def.placementKind !== "node") return null;
  return {
    name: def.label,
    type: def.label,
    typeId: String(def.nodeType ?? def.id),
    subtype: def.subtype,
    provider: def.provider,
    environment: "production",
    owner: "",
    description: "",
    linkedResourceId: "",
  };
}

function componentTemplate(
  catalogId: string,
  description: string,
  tone: IconTileTone,
  brand?: string,
): ArchitectureComponentTemplate | null {
  const draft = draftFromCatalogId(catalogId);
  if (!draft) return null;
  return {
    id: `arch-${catalogId}`,
    title: draft.name,
    description,
    tone,
    brand,
    draft,
  };
}

export const ARCHITECTURE_COMPONENT_TEMPLATES: ArchitectureComponentTemplate[] = [
  componentTemplate("api", "HTTP or gRPC API surface for the system.", "architecture"),
  componentTemplate("service", "Backend service or microservice.", "architecture"),
  componentTemplate("postgres", "Primary relational database.", "data", "postgres"),
  componentTemplate("redis", "Shared cache and short-lived state.", "data", "redis"),
  componentTemplate("worker", "Async worker or job consumer.", "execution"),
].filter((template): template is ArchitectureComponentTemplate => template !== null);

export const ARCHITECTURE_VIEW_TEMPLATES: ArchitectureViewTemplate[] = [
  {
    id: "view-overview",
    title: "Overview",
    description: "All components currently on the canvas.",
    tone: "architecture",
    name: "Overview",
  },
  {
    id: "view-applications",
    title: "Applications",
    description: "Focus on APIs, services, and workers.",
    tone: "architecture",
    name: "Applications",
  },
  {
    id: "view-data-plane",
    title: "Data plane",
    description: "Focus on databases, caches, and stores.",
    tone: "data",
    name: "Data plane",
  },
];
