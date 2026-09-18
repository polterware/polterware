import { describe, expect, it } from "vitest";
import { CATALOG_PROVIDER_NAMES, UNUSABLE_TERMINAL_TEST_TARGETS } from "./workspace/test-fixtures";

describe("local workspace test fixtures", () => {
  it("keeps the nine catalog providers", () => {
    expect([...CATALOG_PROVIDER_NAMES]).toEqual([
      "GitHub",
      "GitLab",
      "AWS",
      "Google Cloud",
      "Azure",
      "Vercel",
      "Supabase",
      "Terraform",
      "Docker",
    ]);
  });

  it("keeps unusable terminal targets for honest picker feedback", () => {
    expect(UNUSABLE_TERMINAL_TEST_TARGETS.map((target) => target.name)).toEqual([
      "offline-edge-01",
      "revoked-node-01",
    ]);
    expect(UNUSABLE_TERMINAL_TEST_TARGETS.every((target) => target.status === "offline" || target.status === "revoked")).toBe(true);
  });
});
