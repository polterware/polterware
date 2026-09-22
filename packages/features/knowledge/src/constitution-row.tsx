import { Shield } from "@polterware/frontend-design-system";
import type { WorkspaceConstitution } from "@polterware/feature-shell";
import { ConstitutionCardFrame } from "./constitution-card-frame";

export function ConstitutionRow(props: {
  constitution: WorkspaceConstitution;
  active?: boolean;
  onActivate?: () => void;
}) {
  return (
    <ConstitutionCardFrame class="!p-0">
      <button
        type="button"
        class="flex w-full items-center gap-3 rounded-[var(--radius-card)] p-3 text-left transition-colors hover:bg-[var(--color-surface-muted)]"
        classList={{ "bg-[var(--color-surface-muted)]": props.active }}
        onClick={() => props.onActivate?.()}
      >
        <span class="flex h-8 w-8 items-center justify-center rounded-[var(--radius-md)] bg-[var(--color-surface)] text-[var(--color-text-muted)]">
          <Shield size={16} />
        </span>
        <span class="min-w-0 flex-1 truncate text-[13px] font-medium text-[var(--color-text)]">
          {props.constitution.title}
        </span>
      </button>
    </ConstitutionCardFrame>
  );
}
