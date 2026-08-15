export type DataSource = "fixture" | "empty" | "unavailable";

export type EnvironmentKind = "production" | "staging" | "development" | "local";

export type EnvironmentItem = {
  id: string;
  name: string;
  kind: EnvironmentKind;
  projectId: string;
};

export type TerminalTargetStatus =
  | "online"
  | "offline"
  | "degraded"
  | "registering"
  | "draining"
  | "upgrading"
  | "revoked"
  | "unknown";

export type TerminalTargetItem = {
  id: string;
  name: string;
  shell: string;
  status: TerminalTargetStatus;
  environmentId?: string;
};

/** Degraded still accepts interactive sessions; lifecycle/offline states do not. */
export function isTerminalTargetUsable(status: string): boolean {
  const s = status.toLowerCase();
  return s === "online" || s === "degraded";
}

export function terminalTargetStatusTone(
  status: string,
): "healthy" | "warn" | "down" | "unknown" {
  switch (status.toLowerCase()) {
    case "online":
      return "healthy";
    case "degraded":
    case "registering":
    case "draining":
    case "upgrading":
      return "warn";
    case "offline":
    case "revoked":
      return "down";
    case "unknown":
    default:
      return "unknown";
  }
}

export function terminalTargetStatusLabel(status: string): string {
  switch (status.toLowerCase()) {
    case "online":
      return "Online";
    case "offline":
      return "Offline";
    case "degraded":
      return "Degraded";
    case "registering":
      return "Registering";
    case "draining":
      return "Draining";
    case "upgrading":
      return "Upgrading";
    case "revoked":
      return "Revoked";
    case "unknown":
      return "Unknown";
    default:
      return status;
  }
}

export type ProjectSwitcherItem = {
  projectId: string;
  name: string;
  directory?: string;
  environmentSummary: string;
  isCurrent: boolean;
  /** Optional table/card display fields (fixture or live). */
  status?: "active" | "paused" | "archived";
  compute?: string;
  region?: string;
  createdAt?: string;
  /** Secondary id under project name in table view. */
  refId?: string;
};
