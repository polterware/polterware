import { describe, expect, it } from "vitest";
import type { ArchNode } from "@polterware/frontend-canvas";
import { canConnect, inferRelationshipType } from "./connection-rules";

function node(partial: Partial<ArchNode> & { id: string; type: ArchNode["type"] }): ArchNode {
  return {
    name: partial.id,
    origin: "manual",
    state: "unknown",
    x: 0,
    y: 0,
    width: 100,
    height: 40,
    zIndex: 0,
    ...partial,
  };
}

describe("connection rules", () => {
  it("rejects self connect", () => {
    const a = node({ id: "a", type: "api" });
    expect(canConnect(a, a, "calls").ok).toBe(false);
  });

  it("rejects note endpoints", () => {
    const a = node({ id: "a", type: "note" });
    const b = node({ id: "b", type: "api" });
    expect(canConnect(a, b, "calls").ok).toBe(false);
  });

  it("infers reads_from for database", () => {
    const a = node({ id: "a", type: "api" });
    const b = node({ id: "b", type: "database" });
    expect(inferRelationshipType(a, b)).toBe("reads_from");
  });
});
