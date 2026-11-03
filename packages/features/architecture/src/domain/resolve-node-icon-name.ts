import type { NodeType } from "@polterware/frontend-canvas";
import { getComponentType } from "./registry";
import { hasServiceMark } from "./service-mark-keys";

export type NodeIconIdentity = {
  name: string;
  type: NodeType;
  subtype?: string;
  provider?: string;
};

/** Pure icon key resolution for canvas cards and related architecture chrome. */
export function resolveNodeIconName(node: NodeIconIdentity): string {
  const normalize = (value: string) =>
    value.toLowerCase().trim().replace(/[^a-z0-9]+/g, "_");
  const normalizedName = normalize(node.name);
  if (hasServiceMark(normalizedName)) return normalizedName;

  const provider = node.provider ? normalize(node.provider) : undefined;
  if (
    provider &&
    hasServiceMark(provider) &&
    normalizedName.split("_").includes(provider)
  ) {
    return provider;
  }

  return (
    (node.subtype ? getComponentType(node.subtype)?.icon : undefined) ??
    getComponentType(node.type)?.icon ??
    "box"
  );
}
