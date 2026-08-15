import type { StatusTone } from "@polterware/frontend-design-system";
import type { EnvironmentKind, ProtectionLevel } from "./types";

export function environmentKindLabel(kind: EnvironmentKind | string): string {
  switch (kind.toLowerCase()) {
    case "production":
      return "Production";
    case "staging":
      return "Staging";
    case "development":
      return "Development";
    case "local":
      return "Local";
    default:
      return kind;
  }
}

export function environmentKindTone(kind: EnvironmentKind | string): StatusTone {
  const value = kind.toLowerCase();
  if (value === "production") return "danger";
  if (value === "staging") return "warn";
  if (value === "development") return "healthy";
  if (value === "local") return "muted";
  return "unknown";
}

export function protectionLabel(protection: ProtectionLevel | string): string {
  switch (protection.toLowerCase()) {
    case "strict":
      return "Strict";
    case "standard":
      return "Standard";
    case "relaxed":
      return "Relaxed";
    default:
      return protection;
  }
}

export function protectionTone(protection: ProtectionLevel | string): StatusTone {
  const value = protection.toLowerCase();
  if (value === "strict") return "danger";
  if (value === "standard") return "warn";
  if (value === "relaxed") return "muted";
  return "unknown";
}

export function defaultFlagLabel(isDefault: boolean): string {
  return isDefault ? "Default" : "Not default";
}

export function defaultFlagTone(isDefault: boolean): StatusTone {
  return isDefault ? "healthy" : "muted";
}

/** Honest mock toast copy — never claims remote environment mutation. */
export function promoteEnvironmentToast(name: string): string {
  return `Promote requested for ${name} (fixture — not a live environment promote)`;
}

export function demoteEnvironmentToast(name: string): string {
  return `Demote requested for ${name} (fixture — not a live environment demote)`;
}

export function addEnvironmentToast(): string {
  return "Environment added to this preview session only";
}

export function saveGeneralSettingsToast(): string {
  return "Project settings saved for this preview session only";
}

export function archiveProjectToast(): string {
  return "Project archive previewed — no files or data were deleted";
}
