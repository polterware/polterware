import { describe, expect, it } from "vitest";
import { buildOperationalMap, selectedNode, selectNodes } from "../../../frontend/canvas/src/index";

describe("operational architecture map", () => {
  it("builds nodes from systems and resources (not hard-coded only)", () => {
    const model = buildOperationalMap(
      [{ id: "sys-local", name: "Default System" }],
      [
        {
          id: "res-local-core",
          name: "polterware-core",
          systemId: "sys-local",
          source: "seed",
          externalKey: "local:polterware-core",
        },
      ],
    );
    expect(model.nodes.map((n) => n.id)).toEqual(["sys-local", "res-local-core"]);
    expect(model.edges).toHaveLength(1);
    expect(model.edges[0]!.evidence).toBe(true);
  });

  it("selecting a resource surfaces that id for inspector", () => {
    const model = buildOperationalMap(
      [{ id: "sys-local", name: "S" }],
      [{ id: "res-a", name: "A", systemId: "sys-local", source: "discovery" }],
    );
    const selected = selectNodes(model, ["res-a"]);
    const node = selectedNode(selected);
    expect(node?.id).toBe("res-a");
    expect(node?.label).toBe("A");
  });
});
