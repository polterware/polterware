import { app, globalShortcut, shell } from "electron";
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
import { createCoreClients, waitForHealth } from "./core/client.js";
import { createCoreDomainControllers } from "./core/controllers/domain.js";
import {
  mintSessionToken,
  resolveUserDatabasePath,
  startCoreProcess,
  stopCoreProcess,
  type CoreProcessHandle,
} from "./core/process-supervisor.js";
import { createShortcutPreferencesController } from "./shortcut-preferences.js";
import { createMcpSupervisor } from "./mcp-supervisor.js";

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

  let unregisterCommanderShortcut = () => {};

  const applyGlobalCommanderAccelerator = (accelerator: string) => {
    unregisterCommanderShortcut();
    unregisterCommanderShortcut = registerGlobalCommanderShortcut({
      accelerator,
      log,
      onToggleCommander: () => {
        toggleCommanderWindow();
      },
      shortcuts: globalShortcut,
    });
  };

  const shortcutPreferences = createShortcutPreferencesController({
    userDataDirectory: app.getPath("userData"),
    onGlobalAcceleratorChange: applyGlobalCommanderAccelerator,
  });

  const mcpSupervisor = createMcpSupervisor({
    log: (message) => log.debug(message),
  });

  const shellControllers = {
    appInfo: {
      getInfo: () => ({ name: app.getName(), version: app.getVersion() }),
    },
    appIcons: {
      getIcon: () => appIcons.getIcon(),
      listIcons: () => appIcons.listIcons(),
      setIcon: (id: string) => {
        const selection = appIcons.setIcon(id);
        appIcons.applySelectedIcon();
        return selection;
      },
    },
    shortcutPreferences: {
      getPreferences: () => shortcutPreferences.getPreferences(),
      setPreferences: (overrides: Record<string, string>) =>
        shortcutPreferences.setPreferences(overrides),
    },
    mcp: {
      getStatus: () => mcpSupervisor.getStatus(),
      start: () => mcpSupervisor.start(),
      stop: () => mcpSupervisor.stop(),
    },
    appShell: {
      openExternal: async (url: string) => {
        if (!/^https?:\/\//i.test(url)) {
          throw new Error("Only http(s) URLs can be opened.");
        }
        await shell.openExternal(url);
        return { ok: true };
      },
    },
    commander: {
      hideOverlay: hideCommanderWindow,
      showMainWindow,
    },
  };

  let coreHandle: CoreProcessHandle | undefined;
  try {
    const token = mintSessionToken();
    coreHandle = await startCoreProcess({
      dbPath: resolveUserDatabasePath(),
      token,
      log: (message) => log.debug(message),
    });
    const clients = createCoreClients(coreHandle.endpoint, coreHandle.token);
    await waitForHealth(clients);
    registerPolterwareIpcHandlers({
      ...shellControllers,
      core: createCoreDomainControllers(clients),
    });
  } catch (error) {
    log.error("Failed to start Polterware core.", error);
    registerPolterwareIpcHandlers(shellControllers);
  }

  applyGlobalCommanderAccelerator(shortcutPreferences.getGlobalCommanderAccelerator());

  createMainWindow();
  setupAutoUpdater();

  app.on("before-quit", () => {
    unregisterCommanderShortcut();
    void mcpSupervisor.stop();
    void stopCoreProcess(coreHandle);
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
