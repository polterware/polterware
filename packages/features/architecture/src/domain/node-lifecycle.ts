import type { StatusTone } from "@polterware/frontend-design-system";
import type { NodeState } from "@polterware/frontend-canvas";

/** Honest lifecycle labels — never invent health from planned/linked alone. */
export function nodeStatusLabel(state: NodeState | string | undefined | null): string {
  switch (state) {
    case "observed":
      return "Observed";
    case "linked":
      return "Linked";
    case "stale":
      return "Stale";
    case "drifted":
      return "Drifted";
    case "unreachable":
      return "Unreachable";
    case "planned":
      return "Planned";
    default:
      return "Unknown";
  }
}

/**
 * Map lifecycle state to StatusChip tone.
 * Planned/linked are neutral — not "healthy service health".
 */
export function nodeLifecycleTone(state: NodeState | string | undefined | null): StatusTone {
  switch (state) {
    case "observed":
      return "healthy";
    case "stale":
    case "drifted":
      return "degraded";
    case "unreachable":
      return "down";
    case "linked":
      return "ok";
    case "planned":
      return "muted";
    default:
      return "unknown";
  }
}

export type LinkStatusValue = "unlinked" | "linked" | "partial";

export function linkStatusLabel(status: LinkStatusValue | string | undefined | null): string {
  switch (status) {
    case "linked":
      return "Linked";
    case "partial":
      return "Partial";
    case "unlinked":
      return "Unlinked";
    default:
      return "Unlinked";
  }
}

export function linkStatusTone(status: LinkStatusValue | string | undefined | null): StatusTone {
  switch (status) {
    case "linked":
      return "ok";
    case "partial":
      return "warn";
    case "unlinked":
      return "muted";
    default:
      return "muted";
  }
}
