import { describe, expect, it } from "vitest";
import { mapConstitution, mapSpecFeatures } from "./map-records";

describe("mapSpecFeatures", () => {
  it("groups five artifacts by featureKey", () => {
    const featureKey = "auth";
    const extraJson = JSON.stringify({ featureKey });
    const base = {
      title: "Auth",
      status: "draft",
      owner: "Platform",
      summary: "",
      updatedAt: "2026-09-21T12:00:00Z",
      extraJson,
    };
    const features = mapSpecFeatures([
      { ...base, id: "spec-1", kind: "spec", body: "spec", summary: "Auth feature" },
      { ...base, id: "plan-1", kind: "plan", body: "plan" },
      { ...base, id: "tasks-1", kind: "tasks", body: "tasks" },
      { ...base, id: "clarify-1", kind: "clarify", body: "clarify" },
      { ...base, id: "checklist-1", kind: "checklist", body: "checklist" },
      { ...base, id: "orphan-spec", kind: "spec", body: "only spec", extraJson: JSON.stringify({ featureKey: "orphan" }) },
    ]);

    expect(features).toHaveLength(1);
    expect(features[0]?.featureKey).toBe("auth");
    expect(features[0]?.clarify.id).toBe("clarify-1");
    expect(features[0]?.checklist.id).toBe("checklist-1");
  });
});

describe("mapConstitution", () => {
  it("returns the project constitution singleton", () => {
    const constitution = mapConstitution([
      {
        id: "constitution-1",
        kind: "constitution",
        title: "Project constitution",
        body: "# Principles",
        status: "active",
        owner: "Platform",
        updatedAt: "2026-09-21T12:00:00Z",
        extraJson: "{}",
      },
    ]);
    expect(constitution?.id).toBe("constitution-1");
  });
});
