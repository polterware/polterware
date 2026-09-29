import type { WorkspaceDocument, WorkspaceRunbook } from "@polterware/feature-shell";
import type { StatusTone } from "@polterware/frontend-design-system";

export type { StatusTone };

export function documentStatusTone(status: WorkspaceDocument["status"]): StatusTone {
  return status === "published" ? "healthy" : "warn";
}

export function adrStatusTone(status: string): StatusTone {
  if (status === "accepted" || status === "active") return "ok";
  if (status === "proposed" || status === "superseded") return "muted";
  return "warn";
}

export function libraryStatusTone(status: string): StatusTone {
  if (status === "published" || status === "accepted" || status === "active") return "ok";
  if (status === "proposed" || status === "superseded") return "muted";
  return "warn";
}

export function runbookSeverityTone(severity: WorkspaceRunbook["severity"]): StatusTone {
  if (severity === "critical" || severity === "high") return "danger";
  if (severity === "medium") return "warn";
  return "muted";
}
