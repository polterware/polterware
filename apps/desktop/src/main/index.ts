import { app, globalShortcut } from "electron";
import log from "electron-log/main";
import { getElectronAppIconController } from "./electron-app-icons.js";
import { registerGlobalCommanderShortcut } from "./global-shortcuts.js";
import {
  createMainWindow,
  hideCommanderWindow,
  showMainWindow,
  toggleCommanderWindow,
} from "./window.js";
import { setupAutoUpdater } from "./updater.js";
import { registerPolterwareIpcHandlers } from "./ipc.js";

function configureLogging() {
  log.initialize();
  log.transports.file.level = "info";
  log.transports.console.level = app.isPackaged ? "warn" : "debug";
  process.on("uncaughtException", (error) => {
    log.error("Uncaught exception in main process.", error);
  });
  process.on("unhandledRejection", (reason) => {
    log.error("Unhandled rejection in main process.", reason);
  });
}

async function bootstrap() {
  configureLogging();
  const appIcons = getElectronAppIconController();
  appIcons.applySelectedIcon();
  registerPolterwareIpcHandlers({
    appInfo: {
      getInfo: () => ({ name: app.getName(), version: app.getVersion() }),
    },
    appIcons: {
      getIcon: () => appIcons.getIcon(),
      listIcons: () => appIcons.listIcons(),
      setIcon: (id) => {
        const selection = appIcons.setIcon(id);
        appIcons.applySelectedIcon();
        return selection;
      },
    },
    commander: {
      hideOverlay: hideCommanderWindow,
      showMainWindow,
    },
  });

  const unregisterCommanderShortcut = registerGlobalCommanderShortcut({
    log,
    onToggleCommander: () => {
      toggleCommanderWindow();
    },
    shortcuts: globalShortcut,
  });

  createMainWindow();
  setupAutoUpdater();

  app.on("before-quit", () => {
    unregisterCommanderShortcut();
  });
}

app.whenReady().then(() => {
  void bootstrap();
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});

app.on("activate", () => {
  showMainWindow();
});
