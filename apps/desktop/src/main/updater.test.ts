import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  app: { isPackaged: false },
  autoUpdater: {
    logger: undefined as unknown,
    autoDownload: false,
    autoInstallOnAppQuit: false,
    on: vi.fn(),
    checkForUpdatesAndNotify: vi.fn().mockResolvedValue(undefined),
  },
}));

vi.mock("electron", () => ({ app: mocks.app }));
vi.mock("electron-log/main", () => ({ default: { info: vi.fn(), error: vi.fn(), warn: vi.fn() } }));
vi.mock("electron-updater", () => ({ default: { autoUpdater: mocks.autoUpdater } }));

const { setupAutoUpdater } = await import("./updater.js");

describe("updater FR-039a", () => {
  beforeEach(() => {
    mocks.autoUpdater.on.mockClear();
    mocks.autoUpdater.checkForUpdatesAndNotify.mockClear();
    mocks.autoUpdater.autoDownload = false;
  });

  it("is a no-op when not packaged", () => {
    mocks.app.isPackaged = false;
    setupAutoUpdater();
    expect(mocks.autoUpdater.on).not.toHaveBeenCalled();
    expect(mocks.autoUpdater.checkForUpdatesAndNotify).not.toHaveBeenCalled();
    expect(mocks.autoUpdater.autoDownload).toBe(false);
  });

  it("configures the updater when packaged", () => {
    mocks.app.isPackaged = true;
    setupAutoUpdater();
    expect(mocks.autoUpdater.autoDownload).toBe(true);
    expect(mocks.autoUpdater.on).toHaveBeenCalled();
  });

  it("does not export renderer update APIs", async () => {
    const src = await import("node:fs").then((fs) =>
      fs.readFileSync(new URL("./updater.ts", import.meta.url), "utf8"),
    );
    expect(src.includes("ipcMain.handle")).toBe(false);
    expect(src.toLowerCase().includes("renderer")).toBe(false);
  });
});
