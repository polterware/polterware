import type { StatusTone } from "@polterware/frontend-design-system";
import {
  linkStatusLabel,
  linkStatusTone,
  nodeLifecycleTone,
  nodeStatusLabel,
  type LinkStatusValue,
} from "./domain/node-lifecycle";

/** Normalize lifecycle display labels or raw state keys to a stable key. */
export function architectureLifecycleKey(
  lifecycle: string | undefined | null,
): string {
  return (lifecycle ?? "").toLowerCase().trim();
}

/** Human-readable lifecycle label for chips (accepts state key or existing label). */
export function architectureLifecycleLabel(
  lifecycle: string | undefined | null,
): string {
  const key = architectureLifecycleKey(lifecycle);
  if (!key) return "Unknown";
  // Prefer domain SSOT for known state keys; keep title-case for already-human labels.
  const fromState = nodeStatusLabel(key);
  if (fromState !== "Unknown" || key === "unknown") return fromState;
  return lifecycle!.trim();
}

/** StatusChip tone for architecture lifecycle — never invents service health from Planned alone. */
export function architectureLifecycleTone(
  lifecycle: string | undefined | null,
): StatusTone {
  const key = architectureLifecycleKey(lifecycle);
  if (!key) return "unknown";
  return nodeLifecycleTone(key);
}

export function architectureLinkStatusLabel(
  status: LinkStatusValue | string | undefined | null,
): string {
  return linkStatusLabel(status);
}

export function architectureLinkStatusTone(
  status: LinkStatusValue | string | undefined | null,
): StatusTone {
  return linkStatusTone(status);
}

/**
 * Companion text for link metric when a resource name is known.
 * Does not replace the StatusChip — shown as secondary line/description.
 */
export function architectureLinkResourceCaption(
  linkStatus: LinkStatusValue | string | undefined | null,
  resourceName: string | undefined | null,
): string | undefined {
  const name = resourceName?.trim();
  if (!name) return undefined;
  const status = (linkStatus ?? "").toLowerCase();
  if (status === "linked" || status === "partial") return name;
  return undefined;
}

export function connectionEvidenceLabel(evidence: boolean): string {
  return evidence ? "Proven" : "Unproven";
}

export function connectionEvidenceTone(evidence: boolean): StatusTone {
  return evidence ? "healthy" : "warn";
}

export function hasConnectedNodes(
  connected: readonly { nodeId: string }[] | undefined | null,
): boolean {
  return Array.isArray(connected) && connected.length > 0;
}

export function provenConnectionCount(
  connected: readonly { evidence: boolean }[] | undefined | null,
): number {
  if (!Array.isArray(connected)) return 0;
  return connected.filter((c) => c.evidence).length;
}

export function unprovenConnectionCount(
  connected: readonly { evidence: boolean }[] | undefined | null,
): number {
  if (!Array.isArray(connected)) return 0;
  return connected.filter((c) => !c.evidence).length;
}
