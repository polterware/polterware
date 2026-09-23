export type DriftFindingView = {
  type: string;
  severity: string;
  summary: string;
  subjectIds: string[];
};

export type DriftStepPayloadView = {
  findings: DriftFindingView[];
};

/** Parse AgentRun drift step payloadJson (`findings` from core DriftPayload). */
export function parseDriftStepPayload(payloadJson: string): DriftStepPayloadView {
  try {
    const raw = JSON.parse(payloadJson || "{}") as Record<string, unknown>;
    const list = Array.isArray(raw.findings) ? raw.findings : [];
    const findings: DriftFindingView[] = [];
    for (const item of list) {
      if (!item || typeof item !== "object") continue;
      const row = item as Record<string, unknown>;
      const subjectIds = Array.isArray(row.subject_ids)
        ? row.subject_ids.flatMap((value) =>
            typeof value === "string" && value ? [value] : [],
          )
        : [];
      findings.push({
        type: typeof row.type === "string" ? row.type : "",
        severity: typeof row.severity === "string" ? row.severity : "",
        summary: typeof row.summary === "string" ? row.summary : "",
        subjectIds,
      });
    }
    return { findings };
  } catch {
    return { findings: [] };
  }
}

export function driftFindingSeverityTone(
  severity: string,
): "ok" | "warn" | "danger" | "muted" {
  switch (severity.toLowerCase()) {
    case "conflict":
      return "danger";
    case "warning":
      return "warn";
    case "info":
      return "ok";
    default:
      return "muted";
  }
}
