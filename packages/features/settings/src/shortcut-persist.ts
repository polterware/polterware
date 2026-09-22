import {
  getShortcutCommand,
  parseChord,
  serializeChord,
  shortcutStore,
  type ShortcutCommandId,
  type ShortcutOverrides,
} from "@polterware/frontend-state";

type ShortcutPreferencesBridge = {
  getShortcutPreferences: () => Promise<{
    version: number;
    overrides: Record<string, string>;
  }>;
  setShortcutPreferences: (
    overrides: Record<string, string>,
  ) => Promise<{
    version: number;
    overrides: Record<string, string>;
  }>;
};

function shortcutPreferencesBridge(): ShortcutPreferencesBridge | undefined {
  const root = globalThis as {
    polterware?: { app?: Partial<ShortcutPreferencesBridge> };
  };
  const app = root.polterware?.app;
  if (!app?.getShortcutPreferences || !app.setShortcutPreferences) return undefined;
  return app as ShortcutPreferencesBridge;
}

function storedOverridesFromState(overrides: ShortcutOverrides): Record<string, string> {
  const stored: Record<string, string> = {};
  for (const [id, chord] of Object.entries(overrides)) {
    if (!chord) continue;
    stored[id] = serializeChord(chord);
  }
  return stored;
}

export function decodeStoredShortcutOverrides(
  stored: Record<string, string>,
): ShortcutOverrides {
  const overrides: ShortcutOverrides = {};
  for (const [id, raw] of Object.entries(stored)) {
    if (!getShortcutCommand(id as ShortcutCommandId)) continue;
    const chord = parseChord(raw);
    if (!chord) continue;
    overrides[id as ShortcutCommandId] = chord;
  }
  return overrides;
}

export async function hydrateShortcutPreferences(): Promise<void> {
  const bridge = shortcutPreferencesBridge();
  if (!bridge) {
    shortcutStore.getState().hydrate({});
    return;
  }
  const preferences = await bridge.getShortcutPreferences();
  shortcutStore.getState().hydrate(decodeStoredShortcutOverrides(preferences.overrides));
}

export async function persistShortcutOverrides(overrides: ShortcutOverrides): Promise<void> {
  const bridge = shortcutPreferencesBridge();
  if (!bridge) return;
  await bridge.setShortcutPreferences(storedOverridesFromState(overrides));
}
