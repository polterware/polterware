import { describe, expect, it } from "vitest";
import { resolveNodeIcon } from "./resolve-node-icon";
import { hasServiceMark } from "./service-mark-keys";

describe("resolveNodeIcon", () => {
  it("prefers service mark icon key", () => {
    expect(resolveNodeIcon({ icon: "postgres" })).toEqual({
      kind: "service",
      key: "postgres",
    });
  });

  it("uses subtype service mark over type Hugeicons key", () => {
    expect(
      resolveNodeIcon({ icon: "database", subtype: "mongodb" }),
    ).toEqual({ kind: "service", key: "mongodb" });
  });

  it("falls back to type Hugeicons icon key", () => {
    expect(resolveNodeIcon({ icon: "database" })).toEqual({
      kind: "type",
      key: "database",
    });
  });

  it("falls back to box", () => {
    expect(resolveNodeIcon({})).toEqual({ kind: "type", key: "box" });
  });

  it("has common service marks", () => {
    for (const k of [
      "postgres",
      "mongodb",
      "redis",
      "rabbitmq",
      "vercel",
      "aws",
      "docker",
      "github",
    ]) {
      expect(hasServiceMark(k), k).toBe(true);
    }
  });
});
