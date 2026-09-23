import type { JSX } from "solid-js";
import { Card } from "@polterware/frontend-design-system";

/**
 * Shared Specs constitution band — nested Card with page padding.
 * Full-width band aligned to the Features table gutters.
 */
export function ConstitutionCardFrame(props: {
  children: JSX.Element;
  class?: string;
  frameClass?: string;
}) {
  return (
    <div class={props.frameClass ?? "px-3 pt-4 pb-2"}>
      <Card class={props.class}>{props.children}</Card>
    </div>
  );
}
