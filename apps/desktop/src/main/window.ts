import { app, BrowserWindow, screen, shell, type BrowserWindowConstructorOptions } from "electron";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { getSelectedAppIconPath } from "./electron-app-icons.js";
import { IPC_EVENTS } from "../shared/ipc.js";

const __dirname = dirname(fileURLToPath(import.meta.url));
const COMMANDER_WINDOW_WIDTH = 704;
const COMMANDER_WINDOW_HEIGHT = 490;

let mainWindow: BrowserWindow | null = null;
let commanderWindow: BrowserWindow | null = null;

function getAppIconWindowOptions(): Pick<BrowserWindowConstructorOptions, "icon"> {
  const icon = getSelectedAppIconPath();
  return icon ? { icon } : {};
}

function canOpenExternalUrl(rawUrl: string): boolean {
  try {
    const url = new URL(rawUrl);
    return url.protocol === "https:" || url.protocol === "http:";
  } catch {
    return false;
  }
}

function getPackagedRendererUrl(): URL {
  return pathToFileURL(join(__dirname, "../renderer/index.html"));
}

function isAllowedRendererUrl(rawUrl: string): boolean {
  try {
    const currentUrl = new URL(rawUrl);
    const rendererUrl = process.env["ELECTRON_RENDERER_URL"];

    if (!app.isPackaged && rendererUrl) {
      const expectedUrl = new URL(rendererUrl);
      return currentUrl.origin === expectedUrl.origin && currentUrl.pathname === expectedUrl.pathname;
    }

    const packagedRendererUrl = getPackagedRendererUrl();
    return (
      currentUrl.protocol === packagedRendererUrl.protocol &&
      currentUrl.pathname === packagedRendererUrl.pathname
    );
  } catch {
    return false;
  }
}

function configureWindowSecurity(window: BrowserWindow): void {
  window.webContents.session.setPermissionRequestHandler((_webContents, _permission, callback) => {
    callback(false);
  });

  window.webContents.setWindowOpenHandler(({ url }) => {
    if (canOpenExternalUrl(url)) {
      void shell.openExternal(url);
    }

    return { action: "deny" };
  });

  window.webContents.on("will-navigate", (event, url) => {
    if (isAllowedRendererUrl(url)) {
      return;
    }

    event.preventDefault();

    if (canOpenExternalUrl(url)) {
      void shell.openExternal(url);
    }
  });
}

function loadRenderer(window: BrowserWindow, surface?: "commander"): void {
  const rendererUrl = process.env["ELECTRON_RENDERER_URL"];

  if (!app.isPackaged && rendererUrl) {
    const url = new URL(rendererUrl);
    if (surface) {
      url.searchParams.set("surface", surface);
    }
    void window.loadURL(url.toString());
    return;
  }

  void window.loadFile(join(__dirname, "../renderer/index.html"), {
    search: surface ? `surface=${surface}` : undefined,
  });
}

export function createMainWindow(): BrowserWindow {
  const options: BrowserWindowConstructorOptions = {
    width: 1440,
    height: 920,
    minWidth: 1180,
    minHeight: 760,
    title: "Polterware Desktop",
    titleBarStyle: "hidden",
    backgroundColor: "#070707",
    show: false,
    ...getAppIconWindowOptions(),
    webPreferences: {
      preload: join(__dirname, "../preload/index.cjs"),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
      webSecurity: true,
    },
  };

  if (process.platform !== "darwin") {
    options.titleBarOverlay = true;
  }

  const window = new BrowserWindow(options);

  mainWindow = window;
  configureWindowSecurity(window);

  window.once("ready-to-show", () => {
    window.show();
  });

  window.on("closed", () => {
    if (mainWindow === window) {
      mainWindow = null;
    }

    if (process.platform !== "darwin") {
      app.quit();
    }
  });

  loadRenderer(window);
  return window;
}

export function getMainWindow(): BrowserWindow | null {
  return mainWindow && !mainWindow.isDestroyed() ? mainWindow : null;
}

function centerCommanderWindow(window: BrowserWindow): void {
  const cursorPoint = screen.getCursorScreenPoint();
  const display = screen.getDisplayNearestPoint(cursorPoint);
  const { height, width, x, y } = display.workArea;
  const nextX = Math.round(x + (width - COMMANDER_WINDOW_WIDTH) / 2);
  const nextY = Math.round(y + Math.max(32, (height - COMMANDER_WINDOW_HEIGHT) * 0.18));

  window.setBounds({
    x: nextX,
    y: nextY,
    width: COMMANDER_WINDOW_WIDTH,
    height: COMMANDER_WINDOW_HEIGHT,
  });
}

function focusCommanderSearch(window: BrowserWindow): void {
  window.webContents.send(IPC_EVENTS.commander.focusSearch);
}

function showAndFocusCommanderWindow(window: BrowserWindow): void {
  centerCommanderWindow(window);
  window.show();
  window.focus();
  focusCommanderSearch(window);
}

export function createCommanderWindow(): BrowserWindow {
  const window = new BrowserWindow({
    width: COMMANDER_WINDOW_WIDTH,
    height: COMMANDER_WINDOW_HEIGHT,
    show: false,
    frame: false,
    resizable: false,
    minimizable: false,
    maximizable: false,
    fullscreenable: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    transparent: false,
    hasShadow: true,
    backgroundColor: "#0f0f0f",
    ...getAppIconWindowOptions(),
    webPreferences: {
      preload: join(__dirname, "../preload/index.cjs"),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
      webSecurity: true,
    },
  });

  commanderWindow = window;
  configureWindowSecurity(window);
  window.setAlwaysOnTop(true, "floating");

  window.on("blur", () => {
    window.hide();
  });

  window.on("closed", () => {
    if (commanderWindow === window) {
      commanderWindow = null;
    }
  });

  loadRenderer(window, "commander");
  return window;
}

export function getCommanderWindow(): BrowserWindow | null {
  return commanderWindow && !commanderWindow.isDestroyed() ? commanderWindow : null;
}

export function hideCommanderWindow(): void {
  getCommanderWindow()?.hide();
}

export function showCommanderWindow(): void {
  const window = getCommanderWindow() ?? createCommanderWindow();

  if (window.webContents.isLoading()) {
    window.webContents.once("did-finish-load", () => {
      if (!window.isDestroyed()) {
        showAndFocusCommanderWindow(window);
      }
    });
    return;
  }

  showAndFocusCommanderWindow(window);
}

export function toggleCommanderWindow(): void {
  const window = getCommanderWindow();

  if (window?.isVisible()) {
    window.hide();
    return;
  }

  showCommanderWindow();
}

export function showMainWindow(): BrowserWindow {
  const window = getMainWindow() ?? createMainWindow();

  if (window.isMinimized()) {
    window.restore();
  }

  window.show();
  window.focus();
  return window;
}
