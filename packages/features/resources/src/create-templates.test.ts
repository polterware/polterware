import { describe, expect, it } from "vitest";
import {
  RESOURCE_INVENTORY_TEMPLATES,
  RESOURCE_NODE_TEMPLATES,
  RESOURCE_PROCESS_TEMPLATES,
} from "./create-templates";

describe("resource create templates", () => {
  it("exports unique ids across catalogs", () => {
    const ids = [
      ...RESOURCE_INVENTORY_TEMPLATES,
      ...RESOURCE_NODE_TEMPLATES,
      ...RESOURCE_PROCESS_TEMPLATES,
    ].map((template) => template.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("covers inventory, node, and process presets", () => {
    expect(RESOURCE_INVENTORY_TEMPLATES).toHaveLength(3);
    expect(RESOURCE_NODE_TEMPLATES).toHaveLength(3);
    expect(RESOURCE_PROCESS_TEMPLATES).toHaveLength(3);
    expect(RESOURCE_INVENTORY_TEMPLATES.map((item) => item.draft.type)).toEqual([
      "Service",
      "Database",
      "Cache",
    ]);
    expect(RESOURCE_NODE_TEMPLATES.every((item) => item.draft.connectionKind === "ssh")).toBe(true);
  });
});
