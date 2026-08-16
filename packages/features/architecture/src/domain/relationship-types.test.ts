import { describe, expect, it } from "vitest";
import type { ArchNode, NodeType } from "@polterware/frontend-canvas";
import {
  CREATABLE_RELATIONSHIP_TYPES,
  labelForRelType,
  listCreatableRelationshipTypes,
  RELATIONSHIP_TYPES,
} from "./registry";
import {
  hasRelationshipTypeIcon,
  relationshipTypeIconKey,
} from "./relationship-type-icon-keys";
import { suggestRelationshipTypes } from "./suggestions";

const EXPECTED_CREATABLE = [
  "depends_on",
  "calls",
  "reads_from",
  "writes_to",
  "publishes_to",
  "consumes_from",
  "routes_to",
  "deployed_on",
  "connects_to",
] as const;

function node(id: string, type: NodeType): ArchNode {
  return {
    id,
    name: id,
    type,
    origin: "manual",
    state: "unknown",
    x: 0,
    y: 0,
    width: 188,
    height: 68,
    zIndex: 0,
  };
}

describe("creatable relationship types", () => {
  it("exposes exactly the focused nine types", () => {
    expect(CREATABLE_RELATIONSHIP_TYPES).toEqual(EXPECTED_CREATABLE);
    expect(listCreatableRelationshipTypes().map((type) => type.id)).toEqual(
      EXPECTED_CREATABLE,
    );
  });

  it("keeps legacy labels readable", () => {
    expect(labelForRelType("deploys_to")).toBe("Deploys to");
    expect(labelForRelType("hosted_on")).toBe("Hosted on");
    expect(labelForRelType("related")).toBe("Related");
  });

  it("limits automatic suggestions to the focused catalog", () => {
    const allowed = new Set<string>(EXPECTED_CREATABLE);
    const suggestions = suggestRelationshipTypes(
      node("service", "service"),
      node("runtime", "runtime_host"),
    );

    expect(suggestions[0]).toBe("deployed_on");
    expect(suggestions.every((type) => allowed.has(type))).toBe(true);
  });

  it("maps every catalog relationship type to a semantic icon key", () => {
    for (const type of EXPECTED_CREATABLE) {
      expect(hasRelationshipTypeIcon(type)).toBe(true);
      expect(relationshipTypeIconKey(type)).not.toBe("link");
    }
    for (const def of RELATIONSHIP_TYPES) {
      // Known catalog ids resolve; unknown falls back to generic "link".
      expect(relationshipTypeIconKey(def.id)).toEqual(expect.any(String));
      expect(relationshipTypeIconKey(def.id).length).toBeGreaterThan(0);
    }
    expect(relationshipTypeIconKey("not_a_real_type")).toBe("link");
  });
});
