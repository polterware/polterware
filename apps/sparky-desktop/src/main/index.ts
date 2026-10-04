import { app, BrowserWindow, dialog, ipcMain, Menu, protocol, session, shell, systemPreferences } from "electron";
import { dirname, isAbsolute, join } from "node:path";
import { readFileSync, writeFileSync } from "node:fs";
import { createCoreClients, type CoreClients } from "./core/client.js";
import { ensureCore, resolveBinary, type CoreHandle } from "./core/process-supervisor.js";
import { startChangeWatcher } from "./core/change-watcher.js";
import { createRpcHandlers, assertTrustedSender, validateAppInput, channel } from "./ipc.js";
import { createWindow, rendererDocument } from "./window.js";
import { contentSecurityPolicy, isAllowedRendererUrl, parseDeepLink } from "./navigation-policy.js";
import { attachmentResponse, resolveAttachmentUrl } from "./attachment-protocol.js";
import { APP_METHODS, EVENTS, type DeepLink } from "../shared/bridge.js";
import { updateStatus } from "./updater.js";
import { installReminders, runNotifyInspect } from "./reminders/index.js";

const e2eEnv = process.argv.find(arg => arg.startsWith("--sparky-e2e-env="))?.slice("--sparky-e2e-env=".length);
if (e2eEnv) {
  if (!isAbsolute(e2eEnv)) throw new Error("sparky e2e env path must be absolute");
  const parsed = JSON.parse(readFileSync(e2eEnv, "utf8")) as Record<string, unknown>;
  for (const [key, value] of Object.entries(parsed)) {
    if (key.startsWith("SPARKY_") && typeof value === "string") process.env[key] = value;
  }
  if (process.env.SPARKY_NOTIFY_E2E === "1") {
    const isolated = process.env.SPARKY_CORE_DB || "";
    if (!isAbsolute(isolated) || !isolated.includes("sparky-w1-native-notify")) {
      throw new Error("e2e SPARKY_CORE_DB must be an isolated temp database");
    }
  }
  writeFileSync(join(dirname(e2eEnv), "main-started.json"), JSON.stringify({ argv: process.argv, db: process.env.SPARKY_CORE_DB }));
}

const remoteDebuggingPort = process.env.REMOTE_DEBUGGING_PORT?.trim();
if (remoteDebuggingPort && /^[1-9]\d{0,4}$/.test(remoteDebuggingPort) && Number(remoteDebuggingPort) <= 65535) {
  app.commandLine.appendSwitch("remote-debugging-port", remoteDebuggingPort);
}

// W1-NATIVE-NOTIFY registers its notification delegate here, before app readiness.
protocol.registerSchemesAsPrivileged([{ scheme: "sparky-file", privileges: { standard: true, secure: true, supportFetchAPI: true } }]);
app.setName("Sparky");
const hiddenLaunch = process.argv.includes("--background-replan");
if (hiddenLaunch) app.dock?.hide();
const dbPath = process.env.SPARKY_CORE_DB || join(app.getPath("appData"), "Sparky", "sparky.db");
if (!isAbsolute(dbPath)) throw new Error("SPARKY_CORE_DB must be absolute");
const data = dirname(dbPath);
let win: BrowserWindow | null = null;
let clients: CoreClients | undefined;
let core: CoreHandle | undefined;
let stopWatcher: (() => void) | undefined;
let pendingLink: DeepLink | null = null;
let onlineMaps = false;
let suppressWindow = hiddenLaunch;
const grants = { import: new Set<string>(), export: new Set<string>(), backup: new Set<string>() };
function getClients(): CoreClients {
  if (!clients) throw new Error("Core offline");
  return clients;
}
function emit(event: string, value: unknown): void {
  if (win && !win.isDestroyed()) win.webContents.send(event, value);
}
function showWindow(): void {
  app.dock?.show();
  if (!win || win.isDestroyed()) {
    win = createWindow();
    win.on("closed", () => { win = null; });
  }
  if (win.isMinimized()) win.restore();
  win.show(); win.focus();
}
function routeDeepLink(raw: string): void {
  const link = parseDeepLink(raw);
  if (!link) return;
  pendingLink = link;
  if (!app.isReady()) return;
  showWindow();
  if (win!.webContents.isLoading()) win!.webContents.once("did-finish-load", () => emit(EVENTS.deepLink, link));
  else emit(EVENTS.deepLink, link);
}
async function reconnect(): Promise<boolean> {
  const next = await ensureCore({ dbPath, binary: resolveBinary(app.getAppPath(), process.resourcesPath, app.isPackaged) });
  const changed = next.pid !== core?.pid || next.token !== core?.token || next.endpoint !== core?.endpoint;
  core = next;
  clients = createCoreClients(next.endpoint, next.token);
  await refreshMapPolicy();
  return changed;
}
async function refreshMapPolicy(): Promise<void> {
  const previous = onlineMaps;
  try { onlineMaps = Boolean((await getClients().settings.get({}, { timeoutMs: 2000 })).settings?.mapOnline); }
  catch { onlineMaps = false; }
  // A document CSP cannot be widened in place. Reload only when this setting actually changes.
  if (previous !== onlineMaps && win && !win.isDestroyed()) win.webContents.reload();
}
function registerIpc(): void {
  const handlers = createRpcHandlers({ getClients, authorizePath: (purpose, path) => grants[purpose].has(path),
    settingsChanged: () => { void refreshMapPolicy(); } });
  const appHandlers: Record<typeof APP_METHODS[number], (input: unknown) => unknown> = {
    openDeepLink: input => { if (typeof input !== "string" || !parseDeepLink(input)) throw new Error("Invalid deep link"); routeDeepLink(input); },
    getDeepLink: () => { const link = pendingLink; pendingLink = null; return link; },
    pickFiles: async () => {
      const result = await dialog.showOpenDialog(win!, { properties: ["openFile", "multiSelections"] });
      const paths = result.canceled ? [] : result.filePaths;
      paths.forEach(path => grants.import.add(path)); return paths;
    },
    saveZip: () => chooseDestination("export"), saveBackup: () => chooseDestination("backup"),
    revealInFinder: async input => {
      if (typeof input !== "string") throw new Error("Invalid attachment URL");
      shell.showItemInFolder(await resolveAttachmentUrl(input, join(data, "attachments")));
    },
    startMicrophone: async () => process.platform === "darwin" && systemPreferences.askForMediaAccess("microphone"),
    getInfo: () => ({ name: "Sparky", version: app.getVersion(), updates: updateStatus }),
  };
  for (const [name, handler] of Object.entries(appHandlers)) handlers[channel("app", name)] = async input => {
    validateAppInput(name, input);
    return handler(input) as never;
  };
  for (const [name, handler] of Object.entries(handlers)) {
    ipcMain.handle(name, async (event, ...args) => {
      if (!win || win.isDestroyed()) throw new Error("No renderer window");
      assertTrustedSender(event, win.webContents);
      if (args.length > 1) throw new Error("Invalid IPC arguments");
      return handler(args[0]);
    });
  }
}
async function chooseDestination(purpose: "export" | "backup"): Promise<string | null> {
  const result = await dialog.showSaveDialog(win!, { defaultPath: purpose === "export" ? "Sparky.zip" : "Sparky.db",
    filters: [{ name: purpose === "export" ? "ZIP archive" : "SQLite database", extensions: [purpose === "export" ? "zip" : "db"] }] });
  const path = !result.canceled && result.filePath ? result.filePath : null;
  if (path) grants[purpose].add(path);
  return path;
}
function installSessionSecurity(): void {
  const trusted = (url: string) => isAllowedRendererUrl(url, rendererDocument());
  session.defaultSession.setPermissionCheckHandler((contents, permission, _origin, details) =>
    contents === win?.webContents && permission === "media" && trusted(details.requestingUrl ?? "") && details.mediaType === "audio" &&
    process.platform === "darwin" && systemPreferences.getMediaAccessStatus("microphone") === "granted");
  session.defaultSession.setPermissionRequestHandler((contents, permission, callback, details) => {
    callback(contents === win?.webContents && permission === "media" && trusted(details.requestingUrl) &&
      "mediaTypes" in details && details.mediaTypes?.length === 1 && details.mediaTypes[0] === "audio" && process.platform === "darwin" &&
      systemPreferences.getMediaAccessStatus("microphone") === "granted");
  });
  session.defaultSession.webRequest.onHeadersReceived((details, callback) => callback({ responseHeaders: {
    ...details.responseHeaders, "Content-Security-Policy": [contentSecurityPolicy(onlineMaps, !app.isPackaged)],
  } }));
  void protocol.handle("sparky-file", request => attachmentResponse(request.url, join(data, "attachments")));
}
function installMenu(): void {
  Menu.setApplicationMenu(Menu.buildFromTemplate([
    { label: "Sparky", submenu: [{ role: "about" }, { type: "separator" }, { role: "hide" }, { role: "hideOthers" }, { role: "unhide" }, { type: "separator" }, { role: "quit" }] },
    { label: "File", submenu: [{ label: "Open Sparky", click: showWindow }, { role: "close" }] },
    { role: "editMenu" }, { role: "viewMenu" }, { role: "windowMenu" },
    { role: "help", submenu: [
      { label: "Open Data Folder", click: () => { void shell.openPath(data); } },
      { label: "Reveal Core Log", click: () => shell.showItemInFolder(join(data, "run/core.log")) },
      { label: "Updates are not available yet", enabled: false },
    ] },
  ]));
}
if (!app.requestSingleInstanceLock()) app.quit();
else {
  const inspectCommand = process.env.SPARKY_NOTIFY_INSPECT;
  if (inspectCommand) {
    const inspectResult = process.env.SPARKY_NOTIFY_INSPECT_RESULT || join(app.getPath("temp"), "sparky-notify-inspect.json");
    app.on("ready", () => { void runNotifyInspect(inspectCommand, inspectResult).finally(() => app.quit()); });
  } else {
  const reminders = installReminders({
    dataDirectory: data,
    ensureCore: async () => { await reconnect(); return getClients(); },
    openMemory: url => {
      suppressWindow = false;
      app.setActivationPolicy("regular");
      reminders.setBackgroundLaunch(false);
      routeDeepLink(url);
    },
    hasWindow: () => Boolean(win && !win.isDestroyed()),
    quit: () => app.quit(),
  });
  app.on("open-url", (event, url) => { event.preventDefault(); routeDeepLink(url); });
  app.on("second-instance", (_event, argv) => {
    const link = argv.find(arg => arg.startsWith("sparky://"));
    if (link) routeDeepLink(link);
    else if (argv.includes("--background-replan")) reminders.requestRefresh("core-write");
    else {
      suppressWindow = false;
      reminders.setBackgroundLaunch(false);
      app.setActivationPolicy("regular");
      if (app.isReady()) showWindow(); else app.once("ready", showWindow);
    }
  });
  const startupLink = process.argv.find(arg => arg.startsWith("sparky://"));
  if (startupLink) routeDeepLink(startupLink);
  app.on("window-all-closed", () => app.quit());
  app.on("before-quit", () => { reminders.close(); stopWatcher?.(); });
  app.on("activate", () => { if (!suppressWindow || win) showWindow(); });
  app.on("ready", (_event, launchInfo) => {
    void (async () => {
      suppressWindow = reminders.prepareLaunch(launchInfo) || hiddenLaunch;
      if (app.isPackaged) app.setAsDefaultProtocolClient("sparky");
      else app.setAsDefaultProtocolClient("sparky", process.execPath, [app.getAppPath()]);
      registerIpc(); installSessionSecurity(); installMenu();
      try {
        await reconnect();
        stopWatcher = startChangeWatcher({ getClients, reconnect, emit: event => {
          emit(EVENTS.change, event);
          reminders.onChanges({ snapshot: event.resync, changes: event.changes });
          if (event.resync || event.changes.some(change => change.entity === "settings" || change.entity === "setting")) void refreshMapPolicy();
        } });
      } catch { console.error("Sparky core offline; inspect the core log"); }
      reminders.start();
      if (suppressWindow && !pendingLink) return;
      showWindow();
      if (process.env.SPARKY_NOTIFY_E2E === "1") {
        setTimeout(() => { void reminders.requestAuthorization().catch(() => {}); }, 1500);
      }
      if (pendingLink) routeDeepLink(pendingLink.url);
    })().catch(() => { console.error("Sparky startup failed"); app.quit(); });
  });
  }
}
