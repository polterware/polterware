import { describe, expect, it } from "vitest";
import { buildOperationalMap, deserialize, serialize, selectedNode, selectNodes } from "./index";

describe("canvas operational map", () => {
  it("serializes and deserializes edges", () => {
    const m = buildOperationalMap(
      [{ id: "s1", name: "Sys" }],
      [{ id: "r1", name: "Res", systemId: "s1", source: "seed" }],
    );
    const round = deserialize(serialize(m));
    expect(round.nodes).toHaveLength(2);
    expect(round.edges).toHaveLength(1);
  });

  it("selection returns resource for inspector wiring", () => {
    const m = buildOperationalMap([{ id: "s1", name: "Sys" }], [{ id: "r1", name: "Res", systemId: "s1" }]);
    const sel = selectNodes(m, ["r1"]);
    expect(selectedNode(sel)?.label).toBe("Res");
  });
});
