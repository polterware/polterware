const { app, BrowserWindow } = require("electron");
const fs = require("fs");
const os = require("os");
const path = require("path");

const logPaths = [];
if (process.env.SPARKY_DATA_DIR) {
  logPaths.push(path.join(process.env.SPARKY_DATA_DIR, "run", "electron.log"));
}
logPaths.push(path.join(os.homedir(), "Library", "Logs", "sparky-notifier-spike.log"));

function log(line) {
  const row = `${new Date().toISOString()} ${line}\n`;
  for (const logPath of logPaths) {
    try {
      fs.mkdirSync(path.dirname(logPath), { recursive: true });
      fs.appendFileSync(logPath, row);
    } catch (error) {
      // Keep going so a missing data dir does not drop the cold-start URL.
    }
  }
}

let windowRef = null;
let lastURL = "";

function render(url) {
  const safe = String(url || "waiting for sparky://").replace(/[<>&]/g, "");
  const html = `<!doctype html><html lang="en" translate="no"><head><meta charset="utf-8"><meta name="google" content="notranslate"><title>Sparky</title></head><body style="margin:24px;background:#161616;color:#f4f4f4;font-family:ui-sans-serif,system-ui"><h1 style="font-size:20px;font-weight:600">Sparky</h1><p>${safe}</p></body></html>`;
  if (windowRef && !windowRef.isDestroyed()) {
    windowRef.loadURL("data:text/html;charset=utf-8," + encodeURIComponent(html));
  }
}

app.on("open-url", (event, url) => {
  event.preventDefault();
  lastURL = url;
  log(`open-url ${url}`);
  render(url);
});

const gotLock = app.requestSingleInstanceLock();
if (!gotLock) {
  app.quit();
} else {
  app.on("second-instance", (_event, argv) => {
    const url = argv.find((arg) => typeof arg === "string" && arg.startsWith("sparky://"));
    if (url) {
      lastURL = url;
      log(`second-instance ${url}`);
      render(url);
    } else {
      log(`second-instance ${argv.join(" ")}`);
    }
  });
}

function register(type, serviceName) {
  try {
    app.setLoginItemSettings({ openAtLogin: true, type, serviceName });
    const got = app.getLoginItemSettings({ type, serviceName });
    log(`register ${type} ${serviceName} => ${JSON.stringify(got)}`);
  } catch (error) {
    log(`register ${type} ${serviceName} ERROR ${error && error.stack ? error.stack : error}`);
  }
}

function unregister(type, serviceName) {
  try {
    app.setLoginItemSettings({ openAtLogin: false, type, serviceName });
    const got = app.getLoginItemSettings({ type, serviceName });
    log(`unregister ${type} ${serviceName} => ${JSON.stringify(got)}`);
  } catch (error) {
    log(`unregister ${type} ${serviceName} ERROR ${error && error.message ? error.message : error}`);
  }
}

app.whenReady().then(() => {
  log(`ready pid=${process.pid} exec=${process.execPath} identity=${app.name}`);
  const registered = app.setAsDefaultProtocolClient("sparky");
  log(`protocol-registered sparky ${registered}`);
  const pidDir = process.env.SPARKY_DATA_DIR
    ? path.join(process.env.SPARKY_DATA_DIR, "run")
    : path.join(os.homedir(), "Library", "Logs");
  fs.mkdirSync(pidDir, { recursive: true });
  fs.writeFileSync(path.join(pidDir, "electron.pid"), String(process.pid));

  windowRef = new BrowserWindow({
    width: 420,
    height: 180,
    show: true,
    title: "Sparky",
    backgroundColor: "#161616",
    webPreferences: { nodeIntegration: false, contextIsolation: true, sandbox: true },
  });
  render(lastURL);

  if (process.env.SPARKY_REGISTER === "1") {
    register("agentService", "com.polterware.sparky.notifier.plist");
    register("loginItemService", "com.polterware.sparky.notifier");
  }
  if (process.env.SPARKY_UNREGISTER === "1") {
    unregister("agentService", "com.polterware.sparky.notifier.plist");
    unregister("loginItemService", "com.polterware.sparky.notifier");
  }
  if (process.env.SPARKY_QUIT_AFTER === "1") {
    setTimeout(() => app.quit(), 500);
  }
});
