import type { StatusTone } from "@polterware/frontend-design-system";

export type SyncEndpointHealth = "connected" | "degraded" | "offline" | "stale" | "syncing";
export type SyncOutcome = "Success" | "Failed" | "Partial" | "Running" | "success" | "failed" | "partial" | "running";
export type SyncQueueStatus = "pending" | "uploading" | "failed" | "synced";

export function syncHealthTone(health: string): StatusTone {
  const value = health.toLowerCase();
  if (value === "connected" || value === "healthy" || value === "ok") return "healthy";
  if (value === "degraded" || value === "stale" || value === "partial") return "warn";
  if (value === "syncing" || value === "running") return "syncing";
  if (value === "offline" || value === "failed" || value === "error" || value === "down") {
    return "danger";
  }
  return "muted";
}

export function syncHealthLabel(health: string): string {
  const value = health.toLowerCase();
  if (value === "connected") return "Connected";
  if (value === "degraded") return "Degraded";
  if (value === "offline") return "Offline";
  if (value === "stale") return "Stale";
  if (value === "syncing") return "Syncing";
  return health;
}

export function syncOutcomeTone(outcome: string): StatusTone {
  const value = outcome.toLowerCase();
  if (value === "success" || value === "succeeded" || value === "ok") return "healthy";
  if (value === "partial") return "warn";
  if (value === "running" || value === "syncing" || value === "pending") return "syncing";
  if (value === "failed" || value === "error") return "danger";
  return "muted";
}

export function syncOutcomeLabel(outcome: string): string {
  const value = outcome.toLowerCase();
  if (value === "success" || value === "succeeded") return "Success";
  if (value === "failed" || value === "error") return "Failed";
  if (value === "partial") return "Partial";
  if (value === "running" || value === "syncing") return "Running";
  if (value === "pending") return "Pending";
  return outcome;
}

export function syncQueueStatusTone(status: string): StatusTone {
  const value = status.toLowerCase();
  if (value === "synced" || value === "success") return "healthy";
  if (value === "pending" || value === "uploading") return "warn";
  if (value === "failed") return "danger";
  return "muted";
}

export function syncQueueStatusLabel(status: string): string {
  const value = status.toLowerCase();
  if (value === "pending") return "Pending";
  if (value === "uploading") return "Uploading";
  if (value === "failed") return "Failed";
  if (value === "synced") return "Synced";
  return status;
}

/** True when any endpoint is degraded/stale/offline or last history outcome failed. */
export function isSyncCenterStale(input: {
  endpoints: readonly { health: string }[];
  history: readonly { outcome: string }[];
  pendingChanges?: number;
  openConflicts?: number;
}): boolean {
  if (input.endpoints.some((endpoint) => {
    const health = endpoint.health.toLowerCase();
    return health === "degraded" || health === "stale" || health === "offline";
  })) {
    return true;
  }
  const latest = input.history[0];
  if (latest && latest.outcome.toLowerCase() === "failed") return true;
  if ((input.openConflicts ?? 0) > 0) return true;
  return false;
}

export function lastHistoryByOutcome(
  history: readonly { outcome: string; at: string; summary: string }[],
  outcome: "success" | "failed",
): { at: string; summary: string } | null {
  const needle = outcome.toLowerCase();
  const match = history.find((entry) => {
    const value = entry.outcome.toLowerCase();
    if (needle === "success") return value === "success" || value === "succeeded";
    return value === "failed" || value === "error";
  });
  return match ? { at: match.at, summary: match.summary } : null;
}
