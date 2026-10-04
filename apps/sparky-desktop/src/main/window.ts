import { app, BrowserWindow, shell } from "electron";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { canOpenExternalUrl, isAllowedRendererUrl } from "./navigation-policy.js";

export function rendererDocument(): string {
  return !app.isPackaged && process.env.ELECTRON_RENDERER_URL
    ? process.env.ELECTRON_RENDERER_URL : pathToFileURL(join(__dirname, "../renderer/index.html")).href;
}
export function createWindow(): BrowserWindow {
  const win = new BrowserWindow({ width: 1200, height: 800, minWidth: 800, minHeight: 560, title: "Sparky", show: false,
    titleBarStyle: "hidden", ...(process.platform === "darwin" ? { vibrancy: "under-window" as const, visualEffectState: "active" as const } : {}),
    webPreferences: { contextIsolation: true, sandbox: true, nodeIntegration: false, webSecurity: true,
      preload: join(__dirname, "../preload/index.cjs") },
  });
  configureWindowSecurity(win);
  win.once("ready-to-show", () => win.show());
  if (!app.isPackaged && process.env.ELECTRON_RENDERER_URL) void win.loadURL(rendererDocument());
  else void win.loadFile(join(__dirname, "../renderer/index.html"));
  return win;
}
export function configureWindowSecurity(win: BrowserWindow): void {
  win.webContents.setWindowOpenHandler(({ url }) => {
    if (canOpenExternalUrl(url)) void shell.openExternal(url);
    return { action: "deny" };
  });
  const denyForeign = (navigation: { preventDefault(): void }, url: string) => {
    if (!isAllowedRendererUrl(url, rendererDocument())) navigation.preventDefault();
  };
  win.webContents.on("will-navigate", denyForeign);
  win.webContents.on("will-redirect", denyForeign);
  win.webContents.on("will-attach-webview", event => event.preventDefault());
}
