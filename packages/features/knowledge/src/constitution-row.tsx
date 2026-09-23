import { DitherIconTile, Shield } from "@polterware/frontend-design-system";
import type { WorkspaceConstitution } from "@polterware/feature-shell";
import { ConstitutionCardFrame } from "./constitution-card-frame";
import "./constitution-widget.css";

export function ConstitutionRow(props: {
  constitution: WorkspaceConstitution;
  active?: boolean;
  onActivate?: () => void;
}) {
  return (
    <ConstitutionCardFrame
      class={[
        "pw-constitution-widget pw-constitution-widget--interactive w-full !p-0",
        props.active ? "pw-constitution-widget--active" : "",
      ]
        .filter(Boolean)
        .join(" ")}
    >
      <button
        type="button"
        class="flex w-full items-center gap-3 px-3 py-3 text-left"
        aria-pressed={props.active ? "true" : "false"}
        onClick={() => props.onActivate?.()}
      >
        <DitherIconTile size={24} tone="knowledge">
          <Shield size={14} aria-hidden />
        </DitherIconTile>
        <span class="pw-constitution-widget__title min-w-0 flex-1 truncate text-[13px] font-medium text-[var(--color-text-secondary)]">
          {props.constitution.title}
        </span>
      </button>
    </ConstitutionCardFrame>
  );
}
