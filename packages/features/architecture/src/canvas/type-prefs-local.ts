import type { TypePrefsPort } from "../domain/type-prefs";
import { createMemoryTypePrefs } from "../domain/type-prefs";

const RECENTS_KEY = "polterware.arch.typeRecents";
const FAVORITES_KEY = "polterware.arch.typeFavorites";
const MAX_RECENTS = 12;

type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };
type TypePreferencesWire = string[];

function decodeTypePreferences(value: JsonValue): TypePreferencesWire | undefined {
  if (!Array.isArray(value)) return undefined;
  const items = value.map((item) => String(item));
  return items.every((item, index) => item === value[index]) ? items : undefined;
}

function readJsonArray(key: string): string[] {
  try {
    const raw = globalThis.localStorage?.getItem(key);
    if (!raw) return [];
    const parsed: JsonValue = JSON.parse(raw);
    return decodeTypePreferences(parsed) ?? [];
  } catch {
    return [];
  }
}

function writeJsonArray(key: string, value: string[]): void {
  try {
    globalThis.localStorage?.setItem(key, JSON.stringify(value));
  } catch {
    /* quota / private mode */
  }
}

/** Browser localStorage adapter for type recents/favorites. */
export function createLocalStorageTypePrefs(): TypePrefsPort {
  if (!globalThis.localStorage) {
    return createMemoryTypePrefs();
  }

  return {
    getRecents: () => readJsonArray(RECENTS_KEY),
    pushRecent: (id: string) => {
      const next = [id, ...readJsonArray(RECENTS_KEY).filter((x) => x !== id)].slice(
        0,
        MAX_RECENTS,
      );
      writeJsonArray(RECENTS_KEY, next);
    },
    getFavorites: () => readJsonArray(FAVORITES_KEY),
    toggleFavorite: (id: string) => {
      const cur = readJsonArray(FAVORITES_KEY);
      const next = cur.includes(id) ? cur.filter((x) => x !== id) : [...cur, id];
      writeJsonArray(FAVORITES_KEY, next);
    },
  };
}
