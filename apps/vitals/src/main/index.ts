import { app, BrowserWindow, ipcMain } from "electron";
import { electronApp, optimizer } from "@electron-toolkit/utils";
import { createWindow } from "./window.js";
import { SessionManager } from "./session-manager.js";
import { registerVitalsIpcHandlers } from "./ipc.js";

const sessionManager = new SessionManager();

app.whenReady().then(() => {
  electronApp.setAppUserModelId("com.polterware.vitals");
  app.on("browser-window-created", (_, window) => {
    optimizer.watchWindowShortcuts(window);
  });

  registerVitalsIpcHandlers({ sessions: sessionManager }, ipcMain);

  const win = createWindow();
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
