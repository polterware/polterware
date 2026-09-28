import { app, BrowserWindow, dialog, globalShortcut, shell, Menu } from "electron";
import log from "electron-log/main";
import { getElectronAppIconController } from "./electron-app-icons.js";
import { registerGlobalCommanderShortcut } from "./global-shortcuts.js";
import {
  createMainWindow,
  getMainWindow,
  hideCommanderWindow,
  runCommanderCommand,
  showMainWindow,
  toggleCommanderWindow,
  applyMainWindowAppearance,
} from "./window.js";
import { setupAutoUpdater } from "./updater.js";
import { registerPolterwareIpcHandlers } from "./ipc.js";
import { createCoreClients, waitForHealth, type CoreClients } from "./core/client.js";
import { createCoreDomainControllers } from "./core/controllers/domain.js";
import { ensureCore, resolveUserDatabasePath } from "./core/process-supervisor.js";
import { startChangeWatcher } from "./core/change-watcher.js";
import { IPC_EVENTS } from "../shared/ipc.js";
import { createShortcutPreferencesController } from "./shortcut-preferences.js";
import { createAppearancePreferencesController } from "./appearance-preferences.js";
import { createMcpSupervisor } from "./mcp-supervisor.js";
import { installApplicationMenu } from "./application-menu.js";

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
  installApplicationMenu(
    {
      isPackaged: app.isPackaged,
      platform: process.platform,
    },
    Menu,
  );
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

  const appearancePreferences = createAppearancePreferencesController({
    userDataDirectory: app.getPath("userData"),
    onChange: (preferences) => {
      applyMainWindowAppearance(preferences.reduceTransparency);
    },
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
    appearancePreferences: {
      getPreferences: () => appearancePreferences.getPreferences(),
      setPreferences: (patch: { reduceTransparency?: boolean }) =>
        appearancePreferences.setPreferences(patch),
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
      selectDirectory: async () => {
        const parent = getMainWindow();
        const options = {
          title: "Select project directory",
          properties: ["openDirectory", "createDirectory"] as Array<"openDirectory" | "createDirectory">,
        };
        const result = parent
          ? await dialog.showOpenDialog(parent, options)
          : await dialog.showOpenDialog(options);
        const path = result.canceled ? null : (result.filePaths[0] ?? null);
        return { path };
      },
    },
    commander: {
      hideOverlay: hideCommanderWindow,
      showMainWindow,
      runCommand: runCommanderCommand,
    },
  };

  // The core is a shared daemon (ADR 0007): attach or start it, never stop it
  // on quit. Clients are swapped in place when the core restarts.
  let stopChangeWatcher = () => {};
  try {
    const dbPath = resolveUserDatabasePath();
    const coreLog = (message: string) => log.debug(message);
    let handle = await ensureCore({ dbPath, log: coreLog });
    let current = createCoreClients(handle.endpoint, handle.token);
    await waitForHealth(current);
    const clients = new Proxy({} as CoreClients, {
      get: (_target, key) => current[key as keyof CoreClients],
    });
    registerPolterwareIpcHandlers({
      ...shellControllers,
      core: createCoreDomainControllers(clients),
    });
    stopChangeWatcher = startChangeWatcher({
      getClients: () => current,
      reconnect: async () => {
        const next = await ensureCore({ dbPath, log: coreLog });
        const replaced = next.endpoint !== handle.endpoint || next.token !== handle.token;
        if (replaced) {
          handle = next;
          current = createCoreClients(next.endpoint, next.token);
        }
        return replaced;
      },
      emit: (event) => {
        for (const window of BrowserWindow.getAllWindows()) {
          if (!window.isDestroyed()) window.webContents.send(IPC_EVENTS.core.changed, event);
        }
      },
      log: coreLog,
    });
  } catch (error) {
    log.error("Failed to start Polterware core.", error);
    registerPolterwareIpcHandlers(shellControllers);
  }

  applyGlobalCommanderAccelerator(shortcutPreferences.getGlobalCommanderAccelerator());

  createMainWindow({
    reduceTransparency: appearancePreferences.getPreferences().reduceTransparency,
  });
  setupAutoUpdater();

  app.on("before-quit", () => {
    unregisterCommanderShortcut();
    void mcpSupervisor.stop();
    stopChangeWatcher();
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
