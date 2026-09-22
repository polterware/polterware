import { getComponentType } from "./domain/registry";

/** Human label for an architecture node type / subtype. */
export function typeLabelFor(type: string, subtype?: string): string {
  if (subtype) {
    const sub = getComponentType(subtype);
    if (sub?.label) return sub.label;
  }
  const def = getComponentType(type);
  if (def?.label) return def.label;
  return type
    .split("_")
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}
