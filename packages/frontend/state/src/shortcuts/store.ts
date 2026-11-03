import { createSignal, getOwner, onCleanup, type Accessor } from "solid-js";
import { createStore } from "zustand/vanilla";
import { SHORTCUT_CATALOG, getShortcutCommand } from "./catalog";
import {
  chordFromEvent,
  findConflict,
  formatChord,
  matchesChord,
  resolveChord,
  serializeChord,
} from "./chord";
import type {
  ShortcutChord,
  ShortcutCommand,
  ShortcutCommandId,
  ShortcutOverrides,
  ShortcutPreferences,
} from "./types";

export type ShortcutStoreState = {
  overrides: ShortcutOverrides;
  hydrated: boolean;
  hydrate: (overrides: ShortcutOverrides) => void;
  setOverride: (
    id: ShortcutCommandId,
    chord: ShortcutChord,
  ) => { ok: true } | { ok: false; conflict: ShortcutCommand };
  clearOverride: (id: ShortcutCommandId) => void;
  replaceOverrides: (overrides: ShortcutOverrides) => void;
};

function isDefaultChord(id: ShortcutCommandId, chord: ShortcutChord): boolean {
  const command = getShortcutCommand(id);
  if (!command) return false;
  return serializeChord(command.defaultChord) === serializeChord(chord);
}

export const shortcutStore = createStore<ShortcutStoreState>()((set, get) => ({
  overrides: {},
  hydrated: false,

  hydrate: (overrides) => {
    set({ overrides: { ...overrides }, hydrated: true });
  },

  replaceOverrides: (overrides) => {
    set({ overrides: { ...overrides }, hydrated: true });
  },

  setOverride: (id, chord) => {
    const conflict = findConflict(id, chord, SHORTCUT_CATALOG, get().overrides);
    if (conflict) return { ok: false as const, conflict };

    const next = { ...get().overrides };
    if (isDefaultChord(id, chord)) {
      delete next[id];
    } else {
      next[id] = chord;
    }
    set({ overrides: next });
    return { ok: true as const };
  },

  clearOverride: (id) => {
    const next = { ...get().overrides };
    delete next[id];
    set({ overrides: next });
  },
}));

export function getShortcutOverrides(): ShortcutOverrides {
  return shortcutStore.getState().overrides;
}

export function resolveShortcutChord(id: ShortcutCommandId): ShortcutChord | undefined {
  const command = getShortcutCommand(id);
  if (!command) return undefined;
  return resolveChord(command, getShortcutOverrides());
}

export function formatShortcut(id: ShortcutCommandId): string {
  const chord = resolveShortcutChord(id);
  return chord ? formatChord(chord) : "";
}

export function matchesShortcut(
  id: ShortcutCommandId,
  event: Pick<KeyboardEvent, "key" | "code" | "metaKey" | "ctrlKey" | "shiftKey" | "altKey">,
): boolean {
  const command = getShortcutCommand(id);
  if (!command) return false;
  return matchesChord(command, getShortcutOverrides(), event);
}

export function eventMatchesAnyShortcut(
  event: Pick<KeyboardEvent, "key" | "code" | "metaKey" | "ctrlKey" | "shiftKey" | "altKey">,
): ShortcutCommand | null {
  const chord = chordFromEvent(event);
  if (!chord) return null;
  for (const command of SHORTCUT_CATALOG) {
    if (matchesChord(command, getShortcutOverrides(), event)) return command;
  }
  return null;
}

export function toShortcutPreferences(overrides: ShortcutOverrides = getShortcutOverrides()): ShortcutPreferences {
  return { version: 1, overrides: { ...overrides } };
}

export function createShortcutSelector<T>(
  selector: (state: ShortcutStoreState) => T,
  equals: (left: T, right: T) => boolean = Object.is,
): Accessor<T> {
  let current = selector(shortcutStore.getState());
  const [selected, setSelected] = createSignal(current, { equals: false });
  const unsubscribe = shortcutStore.subscribe((state) => {
    const next = selector(state);
    if (equals(current, next)) return;
    current = next;
    setSelected(() => next);
  });

  if (getOwner()) {
    onCleanup(unsubscribe);
  }

  return selected;
}

/** Reactive display string for a command’s current chord. */
export function createShortcutLabel(id: ShortcutCommandId): Accessor<string> {
  return createShortcutSelector((state) => {
    const command = getShortcutCommand(id);
    if (!command) return "";
    return formatChord(resolveChord(command, state.overrides));
  });
}
