import { describe, expect, it } from "vitest";
import {
  relationshipMarkers,
  resolveRelationshipAppearance,
} from "./relationship-appearance";

describe("resolveRelationshipAppearance", () => {
  it("legacy without origin falls back to non-dim", () => {
    expect(resolveRelationshipAppearance({ evidence: true })).toEqual({ dim: false });
    expect(resolveRelationshipAppearance({ evidence: false })).toEqual({ dim: false });
    expect(resolveRelationshipAppearance({})).toEqual({ dim: false });
  });

  it("manual/discovered is non-dim", () => {
    expect(resolveRelationshipAppearance({ origin: "manual", evidence: false })).toEqual({ dim: false });
    expect(resolveRelationshipAppearance({ origin: "discovered" })).toEqual({ dim: false });
  });

  it("imported/inferred is non-dim", () => {
    expect(resolveRelationshipAppearance({ origin: "imported", evidence: true })).toEqual({ dim: false });
    expect(resolveRelationshipAppearance({ origin: "inferred" })).toEqual({ dim: false });
  });

  it("disabled/unresolved forces dim", () => {
    expect(resolveRelationshipAppearance({ origin: "manual", state: "disabled" })).toEqual({ dim: true });
    expect(resolveRelationshipAppearance({ origin: "manual", state: "unresolved" })).toEqual({ dim: true });
    // even with evidence
    expect(resolveRelationshipAppearance({ evidence: true, state: "disabled" })).toEqual({ dim: true });
  });

  it("adds a source arrow only to bidirectional relationships", () => {
    expect(relationshipMarkers("forward")).toEqual({
      start: undefined,
      end: "url(#arrow)",
    });
    expect(relationshipMarkers("bidirectional")).toEqual({
      start: "url(#arrow)",
      end: "url(#arrow)",
    });
  });
});
