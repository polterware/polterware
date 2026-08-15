import { describe, expect, it } from "vitest";
import { createMemoryTypePrefs } from "./type-prefs";

describe("type prefs (memory)", () => {
  it("tracks recents and favorites without mutating document", () => {
    const p = createMemoryTypePrefs();
    p.pushRecent("api");
    p.pushRecent("database");
    p.pushRecent("api");
    expect(p.getRecents()[0]).toBe("api");
    expect(p.getRecents()).toContain("database");
    p.toggleFavorite("database");
    expect(p.getFavorites()).toEqual(["database"]);
    p.toggleFavorite("database");
    expect(p.getFavorites()).toEqual([]);
  });
});
