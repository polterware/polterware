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

export function defaultProtectionForKind(kind: EnvironmentKind | string): ProtectionLevel {
  const value = kind.toLowerCase();
  if (value === "production") return "strict";
  if (value === "staging") return "standard";
  return "relaxed";
}

/** Honest local-store toast copy — never claims remote environment mutation. */
export function addEnvironmentToast(name: string): string {
  return `Environment "${name}" saved locally.`;
}

export function setDefaultEnvironmentToast(name: string): string {
  return `${name} is now the default environment.`;
}

export function saveGeneralSettingsToast(): string {
  return "Project settings saved.";
}

export function archiveProjectToast(name?: string): string {
  return name ? `Project "${name}" archived.` : "Project archived.";
}

export function deleteEnvironmentToast(name: string): string {
  return `Environment "${name}" deleted.`;
}
