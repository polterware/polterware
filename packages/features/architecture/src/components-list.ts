import type { ArchNode, NodeState } from "@polterware/frontend-canvas";
import { getComponentType } from "./domain/registry";
import {
  linkStatusLabel,
  nodeStatusLabel,
  type LinkStatusValue,
} from "./domain/node-lifecycle";
import {
  DEFAULT_PROJECT_ENVIRONMENTS,
  environmentLabel,
  normalizeEnvironmentId,
} from "./domain/project-environments";
import { resolveNodeIconName } from "./domain/resolve-node-icon-name";

/** Row projection for Architecture → Components DataTable (SSOT: ArchNode). */
export type ArchitectureComponentRow = {
  id: string;
  name: string;
  type: string;
  typeLabel: string;
  subtype?: string;
  provider?: string;
  iconName: string;
  lifecycle: NodeState | string;
  lifecycleLabel: string;
  linkStatus: LinkStatusValue;
  linkLabel: string;
  environmentId: string;
  environmentLabel: string;
  owner: string;
};

export type ComponentsListFilters = {
  search?: string;
  lifecycle?: readonly string[];
  type?: readonly string[];
  environment?: readonly string[];
};

export function resolveLinkStatus(node: ArchNode): LinkStatusValue {
  if (node.linkStatus === "linked" || node.linkStatus === "partial" || node.linkStatus === "unlinked") {
    return node.linkStatus;
  }
  return node.resourceId ? "linked" : "unlinked";
}

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

/** Project architecture document nodes into list rows (same ids as canvas). */
export function projectComponentRows(nodes: readonly ArchNode[]): ArchitectureComponentRow[] {
  return nodes.map((node) => {
    const environmentId = normalizeEnvironmentId(node.environmentIds?.[0] ?? "");
    const linkStatus = resolveLinkStatus(node);
    return {
      id: node.id,
      name: node.name,
      type: node.type,
      typeLabel: typeLabelFor(node.type, node.subtype),
      subtype: node.subtype,
      provider: node.provider,
      iconName: resolveNodeIconName({
        name: node.name,
        type: node.type,
        subtype: node.subtype,
        provider: node.provider,
      }),
      lifecycle: node.state,
      lifecycleLabel: nodeStatusLabel(node.state),
      linkStatus,
      linkLabel: linkStatusLabel(linkStatus),
      environmentId,
      environmentLabel: environmentLabel(DEFAULT_PROJECT_ENVIRONMENTS, environmentId || undefined),
      owner: node.owner?.trim() ? node.owner : "—",
    };
  });
}

export function filterComponentRows(
  rows: readonly ArchitectureComponentRow[],
  filters: ComponentsListFilters = {},
): ArchitectureComponentRow[] {
  const search = (filters.search ?? "").trim().toLowerCase();
  const lifecycle = new Set((filters.lifecycle ?? []).map((v) => v.toLowerCase()));
  const types = new Set((filters.type ?? []).map((v) => v.toLowerCase()));
  const envs = new Set((filters.environment ?? []).map((v) => normalizeEnvironmentId(v)));

  return rows.filter((row) => {
    if (lifecycle.size > 0 && !lifecycle.has(String(row.lifecycle).toLowerCase())) return false;
    if (types.size > 0 && !types.has(row.type.toLowerCase())) return false;
    if (envs.size > 0) {
      const env = normalizeEnvironmentId(row.environmentId);
      if (!env || !envs.has(env)) return false;
    }
    if (!search) return true;
    return (
      row.name.toLowerCase().includes(search) ||
      row.type.toLowerCase().includes(search) ||
      row.typeLabel.toLowerCase().includes(search) ||
      row.id.toLowerCase().includes(search) ||
      row.owner.toLowerCase().includes(search) ||
      row.environmentLabel.toLowerCase().includes(search) ||
      row.lifecycleLabel.toLowerCase().includes(search)
    );
  });
}

/** Detail activation contract for Components collection rows. */
export function componentsDetailTarget(entityId: string) {
  return {
    scopeId: "architecture:components" as const,
    entityId,
    entityKind: "arch-node" as const,
  };
}
