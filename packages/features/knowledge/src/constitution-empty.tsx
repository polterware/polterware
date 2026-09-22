import { Button } from "@polterware/frontend-design-system";
import { Show } from "solid-js";
import { ConstitutionCardFrame } from "./constitution-card-frame";

export function KnowledgeConstitutionEmpty(props: { onCreate?: () => void }) {
  return (
    <ConstitutionCardFrame class="flex items-center justify-between gap-3">
      <p class="m-0 text-[13px] font-medium text-[var(--color-text)]">Constitution</p>
      <Show when={props.onCreate}>
        <Button size="sm" variant="secondary" onClick={() => props.onCreate?.()}>
          Create constitution
        </Button>
      </Show>
    </ConstitutionCardFrame>
  );
}
