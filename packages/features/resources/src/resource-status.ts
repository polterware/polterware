import type { StatusTone } from "@polterware/frontend-design-system";
import type { ResourceStatus } from "./types";

export function healthTone(status: ResourceStatus): StatusTone {
  if (status === "healthy") return "healthy";
  if (status === "degraded") return "warn";
  if (status === "offline") return "danger";
  return "unknown";
}
