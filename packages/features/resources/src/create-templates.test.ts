import { describe, expect, it } from "vitest";
import { RESOURCE_INVENTORY_TEMPLATES } from "./create-templates";

describe("resource create templates", () => {
  it("exports unique inventory template ids", () => {
    const ids = RESOURCE_INVENTORY_TEMPLATES.map((template) => template.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("covers inventory presets", () => {
    expect(RESOURCE_INVENTORY_TEMPLATES).toHaveLength(3);
    expect(RESOURCE_INVENTORY_TEMPLATES.map((item) => item.draft.type)).toEqual([
      "Service",
      "Database",
      "Cache",
    ]);
  });
});
