import { describe, expect, it } from "vitest";
import {
  defaultNodeSize,
  defaultPorts,
  getComponentType,
  labelForNode,
  labelForNodeType,
  listCatalogTypes,
  listComponentTypes,
  NODE_TYPES,
  searchCatalogTypes,
  searchComponentTypes,
} from "./registry";
import { hasServiceMark } from "./service-mark-keys";
import { hasTypeIcon } from "./type-icon-keys";

describe("architecture registry", () => {
  it("has required node types", () => {
    expect(NODE_TYPES.some((t) => t.id === "api")).toBe(true);
    expect(NODE_TYPES.some((t) => t.id === "database")).toBe(true);
    expect(NODE_TYPES.every((t) => !getComponentType(t.id)?.subtype)).toBe(true);
    expect(listComponentTypes().length).toBeGreaterThan(20);
  });

  it("default ports for services", () => {
    expect(defaultPorts("api")).toHaveLength(2);
    expect(defaultPorts("note")).toHaveLength(0);
    expect(defaultPorts("database")).toHaveLength(2);
  });

  it("labels", () => {
    expect(labelForNodeType("api")).toBe("API");
    expect(labelForNodeType("database")).toBe("Database");
    expect(labelForNode({ type: "database", subtype: "postgres" })).toBe("PostgreSQL");
    expect(defaultNodeSize("note").height).toBeGreaterThan(50);
  });

  it("Add catalog is product-specific for brand services", () => {
    const catalogIds = listCatalogTypes().map((t) => t.id);
    expect(catalogIds).toContain("postgres");
    expect(catalogIds).toContain("mongodb");
    expect(catalogIds).toContain("rabbitmq");
    expect(catalogIds).toContain("vercel");
    expect(catalogIds).toContain("aws");
    // Generic semantic shells stay out of Add
    expect(catalogIds).not.toContain("database");
    expect(catalogIds).not.toContain("cache");
    expect(catalogIds).not.toContain("queue");
    expect(catalogIds).not.toContain("saas_provider");
    expect(catalogIds).not.toContain("boundary_system");
    expect(getComponentType("database")?.placeable).toBe(false);
  });

  it("search catalog finds products, not generic shells", () => {
    const pg = searchCatalogTypes("postgres");
    expect(pg.some((t) => t.id === "postgres")).toBe(true);
    expect(pg.some((t) => t.id === "database")).toBe(false);
    expect(searchCatalogTypes("supabase").some((t) => t.id === "supabase")).toBe(true);
    expect(searchCatalogTypes("rabbitmq").some((t) => t.id === "rabbitmq")).toBe(true);
    expect(searchCatalogTypes("next").some((t) => t.id === "nextjs")).toBe(true);
    expect(searchCatalogTypes("cron").some((t) => t.id === "scheduled_job")).toBe(true);
    expect(searchCatalogTypes("vercel").some((t) => t.id === "vercel")).toBe(true);
    // Full registry still holds semantic type for document/ports
    expect(searchComponentTypes("database").some((t) => t.id === "database")).toBe(true);
  });

  it("product presets map to semantic types with subtype", () => {
    const pg = getComponentType("postgres");
    expect(pg?.nodeType).toBe("database");
    expect(pg?.subtype).toBe("postgres");
    expect(pg?.icon).toBe("postgres");
    expect(pg?.placeable).toBe(true);
  });

  it("every catalog type has a mapped Hugeicons or service-mark icon key", () => {
    for (const t of listComponentTypes()) {
      const ok = hasTypeIcon(t.icon) || hasServiceMark(t.icon);
      expect(ok, `missing icon map for ${t.id} (${t.icon})`).toBe(true);
    }
  });
});
