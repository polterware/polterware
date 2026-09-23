import { describe, expect, it } from "vitest";
import {
  driftFindingSeverityTone,
  parseDriftStepPayload,
} from "./drift-step-payload";

describe("parseDriftStepPayload", () => {
  it("parses findings from drift step payloadJson", () => {
    const payload = JSON.stringify({
      findings: [
        {
          type: "dependency_added",
          severity: "warning",
          summary: "New import fact",
          subject_ids: ["fact-1", "ent-1"],
        },
        {
          type: "declared_observed_conflict",
          severity: "conflict",
          summary: "Statement mismatch",
        },
      ],
    });
    expect(parseDriftStepPayload(payload)).toEqual({
      findings: [
        {
          type: "dependency_added",
          severity: "warning",
          summary: "New import fact",
          subjectIds: ["fact-1", "ent-1"],
        },
        {
          type: "declared_observed_conflict",
          severity: "conflict",
          summary: "Statement mismatch",
          subjectIds: [],
        },
      ],
    });
  });

  it("returns empty findings for invalid or empty payload", () => {
    expect(parseDriftStepPayload("")).toEqual({ findings: [] });
    expect(parseDriftStepPayload("{")).toEqual({ findings: [] });
    expect(parseDriftStepPayload("{}")).toEqual({ findings: [] });
  });
});

describe("driftFindingSeverityTone", () => {
  it("maps narrow checker severities", () => {
    expect(driftFindingSeverityTone("info")).toBe("ok");
    expect(driftFindingSeverityTone("warning")).toBe("warn");
    expect(driftFindingSeverityTone("conflict")).toBe("danger");
    expect(driftFindingSeverityTone("other")).toBe("muted");
  });
});
