import { For, createSignal } from "solid-js";
import { PageContent } from "@polterware/frontend-design-system";
import {
  SHORTCUT_GROUP_LABELS,
  shortcutsInGroup,
  type ShortcutGroup,
} from "@polterware/frontend-state";
import { ShortcutRow } from "./shortcut-row";

const GROUPS: ShortcutGroup[] = ["shell", "architecture"];

/** Editable keyboard shortcuts for Shell and Architecture. */
export function ShortcutsPage() {
  const [recordingId, setRecordingId] = createSignal<string | null>(null);

  return (
    <PageContent class="max-w-2xl space-y-6 overflow-auto p-4 text-[12.5px]">
      <p class="m-0 text-[11px] text-[var(--color-text-muted)]">
        Shortcuts apply on this computer. Architecture chords work when the canvas is focused and you
        are not typing in a field. Escape cancels recording and stays reserved for dialogs.
      </p>
      <For each={GROUPS}>
        {(group) => (
          <section>
            <h2 class="m-0 mb-1 text-[12px] font-medium text-[var(--color-text-primary)]">
              {SHORTCUT_GROUP_LABELS[group]}
            </h2>
            <div class="rounded-[var(--radius-card)] border border-[var(--color-border-subtle)] bg-[var(--color-surface)] px-3">
              <For each={shortcutsInGroup(group)}>
                {(command) => (
                  <ShortcutRow
                    command={command}
                    recordingId={recordingId()}
                    onRecordingChange={setRecordingId}
                  />
                )}
              </For>
            </div>
          </section>
        )}
      </For>
    </PageContent>
  );
}
