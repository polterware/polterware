/**
 * Resources inventory templates.
 */
import type { DitherIconTone } from "@polterware/frontend-design-system";

export type ResourceInventoryDraft = {
  name: string;
  type: string;
};

export type ResourceInventoryTemplate = {
  id: string;
  title: string;
  description: string;
  tone: DitherIconTone;
  brand?: string;
  draft: ResourceInventoryDraft;
};

export const RESOURCE_INVENTORY_TEMPLATES: ResourceInventoryTemplate[] = [
  {
    id: "inventory-api-gateway",
    title: "API gateway",
    description: "Edge service that routes and authenticates traffic.",
    tone: "architecture",
    draft: { name: "API gateway", type: "Service" },
  },
  {
    id: "inventory-postgres",
    title: "Postgres",
    description: "Primary relational database for the project.",
    tone: "data",
    brand: "postgres",
    draft: { name: "Postgres", type: "Database" },
  },
  {
    id: "inventory-redis",
    title: "Redis cache",
    description: "Shared cache and short-lived session store.",
    tone: "data",
    brand: "redis",
    draft: { name: "Redis cache", type: "Cache" },
  },
];
