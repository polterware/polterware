import { app, BrowserWindow, nativeImage } from "electron";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  createAppIconController,
  type AppIconController,
} from "./app-icons.js";

const __dirname = dirname(fileURLToPath(import.meta.url));

let appIconController: AppIconController | null = null;

function unique(paths: string[]): string[] {
  return [...new Set(paths)];
}

export function getAppIconResourceDirectories(): string[] {
  return unique([
    join(process.resourcesPath, "resources", "app-icons"),
    join(app.getAppPath(), "resources", "app-icons"),
    join(process.cwd(), "resources", "app-icons"),
    join(process.cwd(), "apps", "desktop", "resources", "app-icons"),
    join(__dirname, "..", "..", "resources", "app-icons"),
  ]);
}

export function applyElectronIcon(iconPath: string): void {
  const icon = nativeImage.createFromPath(iconPath);

  if (icon.isEmpty()) {
    return;
  }

  for (const window of BrowserWindow.getAllWindows()) {
    if (!window.isDestroyed()) {
      window.setIcon(icon);
    }
  }

  app.dock?.setIcon(icon);
}

export function getElectronAppIconController(): AppIconController {
  appIconController ??= createAppIconController({
    applyIcon: applyElectronIcon,
    resourceDirectories: getAppIconResourceDirectories(),
    userDataDirectory: app.getPath("userData"),
  });

  return appIconController;
}

export function getSelectedAppIconPath(): string | undefined {
  try {
    return getElectronAppIconController().getSelectedIconPath();
  } catch {
    return undefined;
  }
}
