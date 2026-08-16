import { app } from "electron";
import log from "electron-log/main";
import electronUpdater from "electron-updater";

const { autoUpdater } = electronUpdater;

export function setupAutoUpdater(): void {
  if (!app.isPackaged) {
    return;
  }

  autoUpdater.logger = log;
  autoUpdater.autoDownload = true;
  autoUpdater.autoInstallOnAppQuit = true;

  autoUpdater.on("error", (error: Error) => {
    log.error("Auto-update error", error);
  });

  autoUpdater.on("update-available", (info) => {
    log.info("Update available", info.version);
  });

  autoUpdater.on("update-downloaded", (info) => {
    log.info("Update downloaded", info.version);
  });

  void autoUpdater.checkForUpdatesAndNotify().catch((reason) => {
    const error = reason instanceof Error ? reason : new Error(String(reason));
    log.error("Failed to check for updates", error);
  });
}
