import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const electronMocks = vi.hoisted(() => ({
  createFromPath: vi.fn(),
  dockSetIcon: vi.fn(),
  getAllWindows: vi.fn(),
}));

vi.mock("electron", () => ({
  app: {
    dock: {
      setIcon: electronMocks.dockSetIcon,
    },
    getAppPath: () => "/app-path",
  },
  BrowserWindow: {
    getAllWindows: electronMocks.getAllWindows,
  },
  nativeImage: {
    createFromPath: electronMocks.createFromPath,
  },
}));

import {
  applyElectronIcon,
  getAppIconResourceDirectories,
} from "./electron-app-icons.js";

const originalResourcesPath = Object.getOwnPropertyDescriptor(
  process,
  "resourcesPath",
);

beforeEach(() => {
  electronMocks.createFromPath.mockReset();
  electronMocks.dockSetIcon.mockReset();
  electronMocks.getAllWindows.mockReset();
});

afterEach(() => {
  if (originalResourcesPath) {
    Object.defineProperty(process, "resourcesPath", originalResourcesPath);
  } else {
    Reflect.deleteProperty(process, "resourcesPath");
  }
});

describe("getAppIconResourceDirectories", () => {
  it("resolves the app-icons subdirectory in every supported runtime context", () => {
    Object.defineProperty(process, "resourcesPath", {
      configurable: true,
      value: "/runtime-resources",
    });

    expect(getAppIconResourceDirectories()).toEqual([
      "/runtime-resources/resources/app-icons",
      "/app-path/resources/app-icons",
      join(process.cwd(), "resources/app-icons"),
      join(process.cwd(), "apps/desktop/resources/app-icons"),
    ]);
  });
});

describe("applyElectronIcon", () => {
  it("applies a valid selected icon to open windows and the macOS Dock", () => {
    const icon = { isEmpty: () => false };
    const activeWindow = { isDestroyed: () => false, setIcon: vi.fn() };
    const destroyedWindow = { isDestroyed: () => true, setIcon: vi.fn() };
    electronMocks.createFromPath.mockReturnValue(icon);
    electronMocks.getAllWindows.mockReturnValue([
      activeWindow,
      destroyedWindow,
    ]);

    applyElectronIcon("/runtime-resources/resources/app-icons/white.png");

    expect(electronMocks.createFromPath).toHaveBeenCalledWith(
      "/runtime-resources/resources/app-icons/white.png",
    );
    expect(activeWindow.setIcon).toHaveBeenCalledWith(icon);
    expect(destroyedWindow.setIcon).not.toHaveBeenCalled();
    expect(electronMocks.dockSetIcon).toHaveBeenCalledWith(icon);
  });

  it("does not apply an unreadable icon", () => {
    electronMocks.createFromPath.mockReturnValue({ isEmpty: () => true });
    electronMocks.getAllWindows.mockReturnValue([]);

    applyElectronIcon("/missing.png");

    expect(electronMocks.dockSetIcon).not.toHaveBeenCalled();
  });
});
