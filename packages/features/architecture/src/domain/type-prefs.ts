/** Pure preferences port — no localStorage / DOM (Constitution II). */

export type TypePrefsPort = {
  getRecents(): string[];
  pushRecent(id: string): void;
  getFavorites(): string[];
  toggleFavorite(id: string): void;
};

const MAX_RECENTS = 12;

export function createMemoryTypePrefs(): TypePrefsPort {
  let recents: string[] = [];
  let favorites: string[] = [];

  return {
    getRecents: () => [...recents],
    pushRecent: (id: string) => {
      recents = [id, ...recents.filter((x) => x !== id)].slice(0, MAX_RECENTS);
    },
    getFavorites: () => [...favorites],
    toggleFavorite: (id: string) => {
      if (favorites.includes(id)) {
        favorites = favorites.filter((x) => x !== id);
      } else {
        favorites = [...favorites, id];
      }
    },
  };
}
