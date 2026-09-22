import { Show } from "solid-js";
import { Shield } from "@polterware/frontend-design-system";
import type { WorkspaceConstitution } from "@polterware/feature-shell";

export function ConstitutionRow(props: {
  constitution: WorkspaceConstitution;
  active?: boolean;
  onActivate?: () => void;
}) {
  return (
    <button
      type="button"
      class="flex w-full items-center gap-3 border-b border-[var(--color-border)] px-[var(--space-4)] py-[var(--space-3)] text-left transition-colors hover:bg-[var(--color-surface-muted)]"
      classList={{ "bg-[var(--color-surface-muted)]": props.active }}
      onClick={() => props.onActivate?.()}
    >
      <span class="flex h-8 w-8 items-center justify-center rounded-[var(--radius-md)] bg-[var(--color-surface)] text-[var(--color-text-muted)]">
        <Shield size={16} />
      </span>
      <span class="min-w-0 flex-1">
        <span class="block truncate text-[13px] font-medium text-[var(--color-text)]">
          {props.constitution.title}
        </span>
        <span class="block text-[12px] text-[var(--color-text-muted)]">
          Project principles · Updated {props.constitution.updatedAt}
        </span>
      </span>
      <Show when={props.active}>
        <span class="text-[11px] uppercase tracking-wide text-[var(--color-text-muted)]">Open</span>
      </Show>
    </button>
  );
}
