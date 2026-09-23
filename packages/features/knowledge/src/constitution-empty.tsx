import { Button, DitherIconTile, Plus, Shield } from "@polterware/frontend-design-system";
import { Show } from "solid-js";
import { ConstitutionCardFrame } from "./constitution-card-frame";
import "./constitution-widget.css";

export function KnowledgeConstitutionEmpty(props: { onCreate?: () => void }) {
  return (
    <ConstitutionCardFrame class="pw-constitution-widget flex w-full items-center justify-between gap-3">
      <div class="flex min-w-0 items-center gap-3">
        <DitherIconTile size={24} tone="security" class="!text-[var(--color-warning)]">
          <Shield size={14} aria-hidden />
        </DitherIconTile>
        <p class="m-0 truncate text-[13px] font-medium text-[var(--color-text)]">
          Create constitution
        </p>
      </div>
      <Show when={props.onCreate}>
        <Button
          size="icon"
          variant="secondary"
          aria-label="Create constitution"
          onClick={() => props.onCreate?.()}
        >
          <Plus size={14} aria-hidden />
        </Button>
      </Show>
    </ConstitutionCardFrame>
  );
}
