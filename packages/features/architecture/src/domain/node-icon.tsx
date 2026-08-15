import type { JSX } from "solid-js";
import { resolveNodeIcon } from "./resolve-node-icon";
import { ServiceMark } from "./service-marks";
import { TypeIcon } from "./type-icons";

/** Card/catalog icon: service mark (soft brand) or Hugeicons type icon. */
export function NodeIcon(props: {
  /** Registry icon key (Hugeicons type key or service mark). */
  name?: string;
  subtype?: string;
  type?: string;
  size?: number;
  class?: string;
  brand?: boolean;
}): JSX.Element {
  const resolved = () =>
    resolveNodeIcon({
      icon: props.name,
      subtype: props.subtype,
      type: props.type,
    });
  const r = resolved();
  if (r.kind === "service") {
    return (
      <ServiceMark
        name={r.key}
        size={props.size ?? 16}
        class={props.class}
        brand={props.brand}
      />
    );
  }
  return <TypeIcon name={r.key} size={props.size ?? 16} class={props.class} />;
}
