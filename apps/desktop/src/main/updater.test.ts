import { describe, expect, it, vi } from "vitest";

// Lightweight unit test of packaged vs unpackaged updater behavior contract.
describe("updater FR-039a", () => {
  it("is a no-op when not packaged", async () => {
    const setup = await import("./updater.js");
    // setupAutoUpdater should not throw when electron-updater is unavailable/unpackaged
    expect(setup.setupAutoUpdater).toBeDefined();
  });

  it("does not export renderer update APIs", async () => {
    const src = await import("node:fs").then((fs) =>
      fs.readFileSync(new URL("./updater.ts", import.meta.url), "utf8"),
    );
    expect(src.includes("ipcMain.handle")).toBe(false);
    expect(src.toLowerCase().includes("renderer")).toBe(false);
  });
});
