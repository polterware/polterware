import { Show, createSignal } from "solid-js";
import { Input } from "@polterware/frontend-design-system";

export function EditablePath(props: {
  value: string;
  onChange?: (path: string) => void;
}) {
  const [editing, setEditing] = createSignal(false);

  return (
    <Show
      when={editing() && props.onChange}
      fallback={
        <button
          type="button"
          class="pw-knowledge-path font-mono pw-selectable"
          disabled={!props.onChange}
          title={props.onChange ? "Edit path relative to the project root" : undefined}
          onClick={() => setEditing(true)}
        >
          {props.value || "Set path"}
        </button>
      }
    >
      <Input
        size="sm"
        class="w-[320px] max-w-full font-mono"
        value={props.value}
        aria-label="Path relative to the project root"
        autofocus
        onInput={(event) => props.onChange?.(event.currentTarget.value)}
        onBlur={() => setEditing(false)}
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === "Escape") {
            event.preventDefault();
            setEditing(false);
          }
        }}
      />
    </Show>
  );
}
