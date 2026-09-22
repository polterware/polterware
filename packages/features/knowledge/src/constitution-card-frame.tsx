import type { JSX } from "solid-js";
import { Card } from "@polterware/frontend-design-system";

/**
 * Shared Specs constitution band — nested Card with page padding.
 */
export function ConstitutionCardFrame(props: {
  children: JSX.Element;
  class?: string;
}) {
  return (
    <div class="p-2 pt-4">
      <Card class={props.class}>{props.children}</Card>
    </div>
  );
}
