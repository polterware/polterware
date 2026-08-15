import { describe, expect, it } from "vitest";
import { createEmptyDocument } from "@polterware/frontend-canvas";
import { resolveArchitectureDocument } from "./fixture-mode";

describe("fixture architecture", () => {
  it("forces the fixture document", () => {
    const empty = createEmptyDocument({ systemId: "project" });
    const fixture = {
      ...empty,
      nodes: [{
        id: "node",
        name: "API",
        type: "service" as const,
        origin: "manual" as const,
        state: "planned" as const,
        x: 0,
        y: 0,
        width: 240,
        height: 120,
        zIndex: 0,
      }],
    };
    expect(resolveArchitectureDocument({ loaded: empty, fixture, fixtureMode: "force", dataSource: "empty" })).toMatchObject({ document: fixture, dataSource: "fixture" });
  });
});
