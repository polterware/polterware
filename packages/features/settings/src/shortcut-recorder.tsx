import { onCleanup, onMount } from "solid-js";
import { chordFromEvent, type ShortcutChord } from "@polterware/frontend-state";

/** Capture the next non-modifier keydown as a chord. Escape cancels. */
export function ShortcutRecorder(props: {
  active: boolean;
  onCapture: (chord: ShortcutChord) => void;
  onCancel: () => void;
}) {
  onMount(() => {
    if (!props.active) return;

    const onKeyDown = (event: KeyboardEvent) => {
      if (!props.active) return;
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        props.onCancel();
        return;
      }

      const chord = chordFromEvent(event);
      if (!chord) return;

      event.preventDefault();
      event.stopPropagation();
      props.onCapture(chord);
    };

    window.addEventListener("keydown", onKeyDown, true);
    onCleanup(() => window.removeEventListener("keydown", onKeyDown, true));
  });

  return null;
}
