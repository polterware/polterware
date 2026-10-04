import { createShellBridge, type ShellBridge, type ShellWireBridge } from "@polterware/feature-sparky-shell";
import type { Mind } from "@polterware/sparky-contracts";
import { cascadeDeleteIds, type DeletePreview, type MindNode } from "./tree";

export function readSparkyWire(): ShellWireBridge {
  const sparky = typeof window === "undefined" ? undefined : (window as unknown as { sparky?: ShellWireBridge }).sparky;
  if (!sparky) throw new Error("Sparky bridge unavailable");
  return sparky;
}

export function mindsBridge(wire?: ShellWireBridge): ShellBridge {
  return createShellBridge(wire ?? readSparkyWire());
}

/** int64 travels on the wire as a decimal string. */
export function wireRev(rev: bigint | string | number | undefined): string {
  return rev === undefined ? "0" : String(rev);
}

export function toMindNode(mind: Mind): MindNode {
  return {
    id: mind.id,
    parentId: mind.parentId,
    name: mind.name,
    color: mind.color,
    icon: mind.icon,
    sortOrder: mind.sortOrder,
  };
}

export async function loadMinds(bridge: ShellBridge): Promise<Mind[]> {
  const response = await bridge.minds.list({});
  return response.minds;
}

export async function loadMemoryCounts(bridge: ShellBridge, ids: readonly string[]): Promise<Record<string, number>> {
  const pairs = await Promise.all(
    ids.map(async (mindId) => {
      const listed = await bridge.memories.list({ mindId, includeDescendants: false, limit: 1 });
      return [mindId, listed.total] as const;
    }),
  );
  return Object.fromEntries(pairs);
}

export async function loadDeletePreview(bridge: ShellBridge, minds: readonly MindNode[], mindId: string): Promise<DeletePreview> {
  const deletedMindIds = cascadeDeleteIds(minds, mindId);
  const listed = await bridge.memories.list({ mindId, includeDescendants: true, limit: 1 });
  return { deletedMindIds, movedMemoryCount: listed.total };
}

export function parentPatch(parentId: string): { parentId: string } | { clearParent: true } {
  return parentId ? { parentId } : { clearParent: true };
}

export function hexFromPicker(details: unknown): string {
  const candidate =
    details && typeof details === "object" && "value" in details
      ? (details as { value: { toString: (format?: string) => string } }).value
      : details;
  if (typeof candidate === "string") return isLikelyHex(candidate) ? candidate.slice(0, 7).toUpperCase() : "#7EA8F0";
  if (candidate && typeof candidate === "object" && "toString" in candidate) {
    const raw = (candidate as { toString: (format?: string) => string }).toString("hex");
    const hex = raw.startsWith("#") ? raw : `#${raw}`;
    return isLikelyHex(hex) ? hex.slice(0, 7).toUpperCase() : "#7EA8F0";
  }
  return "#7EA8F0";
}

function isLikelyHex(value: string): boolean {
  return /^#[0-9A-Fa-f]{6}/.test(value);
}
