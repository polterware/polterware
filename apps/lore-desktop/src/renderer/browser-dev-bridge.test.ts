// @vitest-environment jsdom

import { afterEach, describe, expect, it } from "vitest";
import { installBrowserDevBridge } from "./browser-dev-bridge";

describe("installBrowserDevBridge", () => {
  afterEach(() => {
    Reflect.deleteProperty(window, "lore");
  });

  it("leaves an existing bridge in place", () => {
    const existing = { marker: "electron" };
    window.lore = existing as unknown as Window["lore"];

    expect(installBrowserDevBridge()).toBe(false);
    expect(window.lore).toBe(existing);
  });

  it("serves the full project first and an empty one second when the preload is missing", async () => {
    expect(installBrowserDevBridge()).toBe(true);

    const { core } = window.lore;
    const listed = await core.projects.list();
    expect(listed.projects).toEqual([
      expect.objectContaining({ id: "browser-preview", name: "Browser preview" }),
      expect.objectContaining({ id: "browser-empty", name: "Empty project" }),
    ]);

    const full = await core.work.listCards({ projectId: "browser-preview" });
    const empty = await core.work.listCards({ projectId: "browser-empty" });
    expect(full.cards.length).toBeGreaterThanOrEqual(12);
    expect(empty.cards).toEqual([]);
  });
});
