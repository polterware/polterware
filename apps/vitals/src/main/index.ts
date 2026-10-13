import { join } from "node:path";
import { app, BrowserWindow, ipcMain, nativeImage } from "electron";
import { electronApp, optimizer } from "@electron-toolkit/utils";
import { createWindow } from "./window.js";
import { SessionManager } from "./session-manager.js";
import { registerVitalsIpcHandlers } from "./ipc.js";
import { getSettings } from "./settings.js";

const sessionManager = new SessionManager();

// Packaged builds take the icon from the bundle; the dev Electron binary needs it set explicitly.
function applyDevDockIcon(): void {
  if (app.isPackaged) return;
  const icon = nativeImage.createFromPath(join(app.getAppPath(), "resources/app-icons/dock.png"));
  if (!icon.isEmpty()) app.dock?.setIcon(icon);
}

app.whenReady().then(() => {
  electronApp.setAppUserModelId("com.polterware.vitals");
  applyDevDockIcon();
  app.on("browser-window-created", (_, window) => {
    optimizer.watchWindowShortcuts(window);
  });

  registerVitalsIpcHandlers({ sessions: sessionManager }, ipcMain);

  const win = createWindow({ reduceTransparency: getSettings().reduceTransparency });
  sessionManager.setWindow(win);

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      const next = createWindow();
      sessionManager.setWindow(next);
    }
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});
