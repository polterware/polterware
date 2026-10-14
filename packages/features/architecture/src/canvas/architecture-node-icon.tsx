import {
  IconTile,
  resolveIconTileAccent,
} from "@polterware/frontend-design-system";
import { NodeIcon } from "../domain/node-icon";
import {
  resolveNodeIconName,
  type NodeIconIdentity,
} from "../domain/resolve-node-icon-name";

export type { NodeIconIdentity };
export { resolveNodeIconName };

export function nodeAccentColor(node: NodeIconIdentity): string {
  const iconName = resolveNodeIconName(node);
  return resolveIconTileAccent(
    [iconName, node.subtype ?? "", node.provider ?? ""],
    "architecture",
  );
}

export function nodeIconAccent(name: string, provider?: string): string {
  return resolveIconTileAccent([name, provider ?? ""], "neutral");
}

export function ArchitectureNodeIcon(props: {
  name: string;
  subtype?: string;
  provider?: string;
  compact: boolean;
  dimmed: boolean;
  /** Smaller than compact, for inline labels such as the domain pill. */
  tiny?: boolean;
}) {
  const iconSize = () => (props.tiny ? 10 : props.compact ? 12 : 16);

  return (
    <span class="architecture-node-icon-wrapper">
      <IconTile
        size={props.tiny ? 16 : props.compact ? 20 : 28}
        tone="architecture"
        brand={[props.name, props.subtype ?? "", props.provider ?? ""]}
        dimmed={props.dimmed}
      >
        <NodeIcon
          name={props.name}
          subtype={props.subtype}
          size={iconSize()}
          brand={false}
        />
      </IconTile>
    </span>
  );
}
