const { app, BrowserWindow, powerMonitor } = require('electron');
const fs = require('node:fs');
const path = require('node:path');
const { spawn } = require('node:child_process');
const { loadNotify } = require('./adapter.cjs');
const config = require('./spike-config.json');
const run = path.join(config.data, 'run');
fs.mkdirSync(run, { recursive:true });
fs.mkdirSync(path.join(config.data, 'chromium'), { recursive:true });
fs.mkdirSync(path.join(run, 'electron-logs'), { recursive:true });
app.setPath('userData', path.join(config.data, 'chromium'));
app.setPath('logs', path.join(run, 'electron-logs'));
function log(event, fields = {}) {
  fs.appendFileSync(path.join(run, 'notify.jsonl'), JSON.stringify({ at:new Date().toISOString(), pid:process.pid, ...fields, event }) + '\n');
}
// Installed before app readiness: cold notification responses must not be dropped.
const notify = loadNotify(config.form, path.join(__dirname, 'native'));
log('delegate-installed', { form:config.form, electron:process.versions.electron, abi:process.versions.modules, napi:process.versions.napi, argv:process.argv.slice(1) });
let windowRef, idleTimer, refreshTimer, active = 0, ready = false;
const queued = [];
const background = !process.argv.includes('--show-window');
const command = process.argv.find(arg => arg.startsWith('--command='))?.slice(10);
const commandFile = path.join(run, 'command.json');
function idleExit() {
  clearTimeout(idleTimer);
  if (background) idleTimer = setTimeout(() => {
    if (active) { idleExit(); return; }
    log('idle-exit', { windows:BrowserWindow.getAllWindows().length });
    app.quit();
  }, 10000);
}
async function request(method, route, body) {
  const state = JSON.parse(fs.readFileSync(path.join(run, 'core.json'), 'utf8'));
  const url = new URL(state.endpoint);
  if (url.hostname !== '127.0.0.1') throw Error('non-loopback core');
  const response = await fetch(new URL(route, url), { method, headers:{Authorization:`Bearer ${state.token}`, 'Content-Type':'application/json'},
    body:body == null ? undefined : JSON.stringify(body), signal:AbortSignal.timeout(2000) });
  if (!response.ok) throw Error(`core ${route} HTTP ${response.status}`);
  return response.json();
}
let ensuring;
async function ensureCore() {
  if (ensuring) return ensuring;
  ensuring = (async () => {
    try { await request('GET', '/health'); return; } catch {}
    // This stub only spawns its bundled core; production uses W1-ELECTRON's supervisor.
    const child = spawn(path.join(process.resourcesPath, 'stubcore'), ['-data', config.data], { detached:true, stdio:'ignore' });
    child.on('error', error => log('core-spawn-error', { error:error.message }));
    child.unref(); log('core-spawn', { corePid:child.pid });
    for (let i=0; i<40; i++) {
      try { await request('GET', '/health'); return; } catch {}
      await new Promise(resolve => setTimeout(resolve, 25));
    }
    throw Error('stub core failed to start');
  })();
  try { await ensuring; } finally { ensuring = null; }
}
let refresh = Promise.resolve();
function replan(reason) {
  active++; idleExit();
  const operation = refresh.then(async () => {
    await ensureCore();
    const plan = await request('GET', '/plan');
    const diff = await notify.apply(plan.items);
    log('replan', { reason, diff, windows:BrowserWindow.getAllWindows().length });
  });
  refresh = operation.catch(error => log('replan-error', { reason, error:error.message }));
  return operation.finally(() => { active--; idleExit(); });
}
function scheduleReplan(reason) {
  clearTimeout(refreshTimer);
  refreshTimer = setTimeout(() => { replan(reason).catch(() => {}); }, 500);
}
async function action(event) {
  active++; idleExit();
  log('os-action', event);
  try {
    if (!/^[0-9A-HJKMNP-TV-Z]{26}$/.test(event.memoryId) || !/^(rem:|one:)/.test(event.id)) throw Error('invalid action payload');
    if (['DONE', 'SNOOZE'].includes(event.action)) {
      await ensureCore();
      await request('POST', event.action === 'DONE' ? '/complete' : '/snooze', {
        memoryId:event.memoryId, occurrenceId:event.id, occurrenceAt:event.occurrenceAt,
        source:'os', minutes:10, receivedAtMs:event.receivedAtMs
      });
      log('action-applied', { action:event.action, delegateToAppliedMs:Date.now()-event.receivedAtMs });
      await replan('action');
    } else if (["OPEN", "START_FOCUS", "com.apple.UNNotificationDefaultActionIdentifier"].includes(event.action)) {
      showMemory(event.memoryId, event.action === 'START_FOCUS');
    }
  } catch (error) { log('action-error', { error:error.message }); }
  finally { await notify.finishAction(event.token); active--; idleExit(); }
}
notify.onAction(event => { ready ? action(event).catch(error => log('fatal-action', { error:error.message })) : queued.push(event); });
notify.onEvent(event => { log('native-event', event); if (ready) scheduleReplan(event.reason); });
function showMemory(memory, focus) {
  log('open-memory', { url:`sparky://memory/${memory}${focus ? '?action=focus' : ''}` });
  clearTimeout(idleTimer);
  if (!windowRef) {
    windowRef = new BrowserWindow({ width:420, height:180, title:'Sparky native spike',
      webPreferences:{ nodeIntegration:false, contextIsolation:true, sandbox:true, webSecurity:true } });
    windowRef.webContents.setWindowOpenHandler(() => ({ action:'deny' }));
    windowRef.webContents.on('will-navigate', event => event.preventDefault());
    windowRef.loadFile(path.join(__dirname, 'index.html'));
  }
  windowRef.show();
}
app.on('open-url', (event, text) => {
  event.preventDefault();
  try {
    const url = new URL(text);
    if (url.protocol !== 'sparky:' || url.hostname !== 'memory' || !/^\/[0-9A-HJKMNP-TV-Z]{26}$/.test(url.pathname)) return;
    app.whenReady().then(() => showMemory(url.pathname.slice(1), url.searchParams.get('action') === 'focus'));
  } catch {}
});
app.whenReady().then(async () => {
  ready = true;
  fs.writeFileSync(path.join(run, 'electron.pid'), String(process.pid));
  log('ready', { background, windows:BrowserWindow.getAllWindows().length, dockVisible:app.dock?.isVisible() });
  powerMonitor.on('resume', () => scheduleReplan('electron-resume'));
  const timer = setInterval(() => scheduleReplan('six-hour'), 6*60*60*1000);
  timer.unref();
  for (const event of queued.splice(0)) action(event).catch(error => log('fatal-action', { error:error.message }));
  try {
    if (command) {
      const payload = fs.existsSync(commandFile) ? JSON.parse(fs.readFileSync(commandFile, 'utf8')) : null;
      const allowed = ['requestAuthorization','status','pending','delivered','apply','removeAll','testHooks'];
      if (!allowed.includes(command)) throw Error('unknown spike command');
      if (command === 'requestAuthorization' && !background) {
        showMemory('01ARZ3NDEKTSV4RRFFQ69G5FAV', false);
        app.focus({ steal:true });
      }
      const value = await notify[command](payload);
      fs.writeFileSync(path.join(run, 'result.json'), JSON.stringify({ command, value }));
      log('command', { command, value });
      if (command !== 'testHooks' && !process.argv.includes('--hold')) { app.quit(); return; }
      if (process.argv.includes('--hold')) await ensureCore();
    } else await replan('start');
  } catch (error) {
    fs.writeFileSync(path.join(run, 'result.json'), JSON.stringify({ command, error:error.message }));
    log('startup-error', { error:error.message });
    if (command) { app.quit(); return; }
  }
  if (!background) showMemory('01ARZ3NDEKTSV4RRFFQ69G5FAV', false);
  idleExit();
}).catch(error => { log('startup-fatal', { error:error.stack }); app.quit(); });
app.on('before-quit', () => notify.close());
app.on('window-all-closed', () => {});
