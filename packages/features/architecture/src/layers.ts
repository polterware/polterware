import type { CanvasNode } from "@polterware/frontend-canvas";
import type { ArchitectureLayer } from "./types";

export function assignLayer(node: CanvasNode): ArchitectureLayer {
  const label = `${node.label} ${node.kind ?? ""} ${node.source ?? ""}`.toLowerCase();
  if (
    label.includes("github") ||
    label.includes("ci") ||
    label.includes("action") ||
    label.includes("pipeline")
  ) {
    return "cloud";
  }
  if (
    label.includes("postgres") ||
    label.includes("redis") ||
    label.includes("queue") ||
    label.includes("database") ||
    label.includes("cache") ||
    label.includes("data")
  ) {
    return "data";
  }
  if (node.kind === "system" || node.kind === "environment") return "other";
  return "applications";
}

export const LAYER_ORDER: ArchitectureLayer[] = ["cloud", "applications", "data", "other"];

export const LAYER_LABEL = {
  cloud: "Cloud",
  applications: "Applications",
  data: "Data",
  other: "Other",
} satisfies Record<ArchitectureLayer, string>;

export function layerBandY(layer: ArchitectureLayer): number {
  switch (layer) {
    case "cloud":
      return 24;
    case "applications":
      return 140;
    case "data":
      return 300;
    default:
      return 420;
  }
}
