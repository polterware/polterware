import type { WorkspaceDocument, WorkspaceRunbook, WorkspaceSpecFeature } from "@polterware/feature-shell";

export type StatusTone = "healthy" | "warn" | "muted" | "unknown" | "syncing" | "danger";

export function documentStatusTone(status: WorkspaceDocument["status"]): StatusTone {
  return status === "published" ? "healthy" : "warn";
}

export function runbookSeverityTone(severity: WorkspaceRunbook["severity"]): StatusTone {
  if (severity === "critical" || severity === "high") return "danger";
  if (severity === "medium") return "warn";
  return "muted";
}

export function specStatusTone(status: WorkspaceSpecFeature["status"]): StatusTone {
  return status === "ready" ? "healthy" : "warn";
}
