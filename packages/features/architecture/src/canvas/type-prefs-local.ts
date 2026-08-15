import type { TypePrefsPort } from "../domain/type-prefs";
import { createMemoryTypePrefs } from "../domain/type-prefs";

const RECENTS_KEY = "polterware.arch.typeRecents";
const FAVORITES_KEY = "polterware.arch.typeFavorites";
const MAX_RECENTS = 12;

function readJsonArray(key: string): string[] {
  try {
    if (typeof localStorage === "undefined") return [];
    const raw = localStorage.getItem(key);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as unknown;
    return Array.isArray(parsed) ? parsed.filter((x) => typeof x === "string") : [];
  } catch {
    return [];
  }
}

function writeJsonArray(key: string, value: string[]): void {
  try {
    if (typeof localStorage === "undefined") return;
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    /* quota / private mode */
  }
}

/** Browser localStorage adapter for type recents/favorites. */
export function createLocalStorageTypePrefs(): TypePrefsPort {
  if (typeof localStorage === "undefined") {
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
