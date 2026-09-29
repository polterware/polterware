import { For, createSignal } from "solid-js";
import {
  SHORTCUT_GROUP_LABELS,
  shortcutsInGroup,
  type ShortcutGroup,
} from "@polterware/frontend-state";
import { SettingsSection } from "./settings-section";
import { ShortcutRow } from "./shortcut-row";

const GROUPS: ShortcutGroup[] = ["shell", "architecture"];

/** Editable keyboard shortcuts for Shell and Architecture. */
export function ShortcutsPage() {
  const [recordingId, setRecordingId] = createSignal<string | null>(null);

  return (
    <div class="flex flex-col gap-6">
      <p class="m-0 px-1 text-[11.5px] text-[var(--color-text-muted)]">
        Shortcuts apply on this computer. Architecture chords work when the canvas is focused and you
        are not typing in a field. Escape cancels recording and stays reserved for dialogs.
      </p>
      <For each={GROUPS}>
        {(group) => (
          <SettingsSection title={SHORTCUT_GROUP_LABELS[group]}>
            <For each={shortcutsInGroup(group)}>
              {(command) => (
                <ShortcutRow
                  command={command}
                  recordingId={recordingId()}
                  onRecordingChange={setRecordingId}
                />
              )}
            </For>
          </SettingsSection>
        )}
      </For>
    </div>
  );
}
