import { hasServiceMark } from "./service-mark-keys";
import { hasTypeIcon } from "./type-icon-keys";

export type ResolvedIcon =
  | { kind: "service"; key: string }
  | { kind: "type"; key: string };

/**
 * Resolve which icon to show for a catalog entry or placed node.
 * Priority: explicit service icon → subtype mark → type Hugeicons icon → box.
 */
export function resolveNodeIcon(input: {
  icon?: string | null;
  subtype?: string | null;
  type?: string | null;
}): ResolvedIcon {
  const icon = input.icon?.trim();
  if (icon && hasServiceMark(icon)) return { kind: "service", key: icon };

  const subtype = input.subtype?.trim();
  if (subtype && hasServiceMark(subtype)) return { kind: "service", key: subtype };

  if (icon && hasTypeIcon(icon)) return { kind: "type", key: icon };
  if (icon) return { kind: "type", key: icon };
  return { kind: "type", key: "box" };
}
