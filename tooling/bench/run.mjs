#!/usr/bin/env node
// Measures the Electron Desktop against the GPUI Desktop under the same conditions: release
// builds, one shared core on the same seeded database (tooling/bench/seed.mjs), outside either
// app's process tree, and the same window. Writes dist/bench/<run>/results.json and prints a table.
//
//   node tooling/bench/run.mjs [--db dist/bench/typical/local.db] [--runs 5] [--no-build]
//                              [--only electron|gpui] [--no-tour]
//
// Per app: bundle size on disk and zipped (with and without the 30 MB Go core both ship), the
// release build time, then for each run (after one warm-up) the time from launch to the frame
// that first shows project rows (`projects-ready`), the memory of the app's whole process tree
// (phys_footprint, as Activity Monitor's Memory, and RSS) once it settles, and its idle CPU over
// 10 seconds with the window in front. A last run walks the parity tour and keeps the peak
// memory and the CPU time it took. The core is measured on its own at the end. Both apps read
// Dispatch's history from the parity tour's stand-in Dispatch core, not the installed app.
import { execFileSync, spawn } from "node:child_process";
import { copyFileSync, cpSync, existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { startDispatchFixture } from "../parity/dispatch-fixture.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const argv = process.argv.slice(2);
const option = (name, fallback) => {
  const index = argv.indexOf(`--${name}`);
  return index >= 0 ? argv[index + 1] : fallback;
};
const runs = Number(option("runs", "5"));
const only = option("only", "");
const build = !argv.includes("--no-build");
const tour = !argv.includes("--no-tour");
const db = path.resolve(option("db", path.join(root, "dist/bench/typical/local.db")));
const stamp = new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
const out = path.join(root, "dist/bench", `run-${stamp}`);
const buildDir = path.join(root, "dist/bench/build");
const desktop = path.join(root, "apps/polterware-desktop");
const gpuiDir = path.join(root, "desktop-gpui");
const core = path.join(root, "dist/polterware");
mkdirSync(out, { recursive: true });

const fail = (message) => {
  console.error(`bench: ${message}`);
  process.exit(1);
};
const sh = (command, args, options = {}) => execFileSync(command, args, { stdio: "inherit", ...options });
const quiet = (command, args) => execFileSync(command, args, { encoding: "utf8" });
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const median = (values) => {
  const sorted = [...values].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
};
const timed = (fn) => {
  const started = Date.now();
  fn();
  return (Date.now() - started) / 1000;
};

const APPS = {
  electron: {
    label: "Electron",
    bundle: path.join(buildDir, "electron/mac-arm64/Polterware.app"),
    exe: (bundle) => path.join(bundle, "Contents/MacOS/Polterware"),
    build() {
      // The shipped bundle carries Sparkle; the release builds it the same way (dist-mac.sh).
      if (!existsSync(path.join(root, "tooling/updater/build/updater.node"))) {
        sh("bash", [path.join(root, "tooling/updater/build.sh"), "polterware"]);
      }
      // The release's environment (dist-mac.sh): no signing identity, a fixed build stamp.
      const env = {
        ...process.env,
        NODE_ENV: "production",
        CSC_IDENTITY_AUTO_DISCOVERY: "false",
        SOURCE_DATE_EPOCH: process.env.SOURCE_DATE_EPOCH || quiet("git", ["-C", root, "log", "-1", "--format=%ct"]).trim(),
      };
      return timed(() => {
        sh("pnpm", ["--dir", desktop, "exec", "electron-vite", "build", "--mode", "production"], { env });
        sh("pnpm", [
          "--dir", desktop, "exec", "electron-builder", "--mac", "dir", "--arm64", "--publish", "never",
          `--config.directories.output=${path.join(buildDir, "electron")}`,
        ], { env });
      });
    },
    env: (bench) => ({ POLTERWARE_BENCH: bench, POLTERWARE_USER_DATA: path.join(out, "electron-profile") }),
    tourEnv: (dir) => ({ POLTERWARE_TOUR: dir, POLTERWARE_USER_DATA: path.join(out, "electron-profile") }),
  },
  gpui: {
    label: "GPUI",
    bundle: path.join(buildDir, "gpui/Polterware GPUI.app"),
    exe: (bundle) => path.join(bundle, "Contents/MacOS/polterware-gpui"),
    build() {
      const seconds = timed(() => sh("cargo", ["build", "--release", "--manifest-path", path.join(gpuiDir, "Cargo.toml")]));
      // There is no GPUI packaging yet: the bundle a release would ship, the binary and the core.
      const bundle = this.bundle;
      rmSync(bundle, { recursive: true, force: true });
      mkdirSync(path.join(bundle, "Contents/MacOS"), { recursive: true });
      mkdirSync(path.join(bundle, "Contents/Resources"), { recursive: true });
      copyFileSync(path.join(gpuiDir, "target/release/polterware-gpui"), this.exe(bundle));
      copyFileSync(core, path.join(bundle, "Contents/Resources/polterware"));
      writeFileSync(
        path.join(bundle, "Contents/Info.plist"),
        `<?xml version="1.0" encoding="UTF-8"?>\n<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">\n<plist version="1.0"><dict>\n<key>CFBundleExecutable</key><string>polterware-gpui</string>\n<key>CFBundleIdentifier</key><string>com.polterware.polterware.gpui-bench</string>\n<key>CFBundleName</key><string>Polterware GPUI</string>\n<key>CFBundlePackageType</key><string>APPL</string>\n<key>NSHighResolutionCapable</key><true/>\n</dict></plist>\n`,
      );
      return seconds;
    },
    env: (bench) => ({ POLTERWARE_BENCH: bench }),
    tourEnv: (dir) => ({ POLTERWARE_GPUI_SHOTS: dir }),
  },
};
const chosen = Object.keys(APPS).filter((key) => !only || key === only);

// --- Build and size -------------------------------------------------------------------------
const results = { stamp, db, runs, machine: `${os.cpus()[0]?.model} · ${Math.round(os.totalmem() / 2 ** 30)} GB · macOS ${os.release()}`, apps: {} };
if (build) sh("pnpm", ["build:cli"], { cwd: root });
const coreBytes = statSync(core).size;
const duBytes = (target) => Number(quiet("du", ["-sk", target]).split(/\s+/)[0]) * 1024;
for (const key of chosen) {
  const app = APPS[key];
  const buildSeconds = build ? app.build() : null;
  if (!existsSync(app.exe(app.bundle))) fail(`${app.bundle} is missing; run without --no-build`);
  const zip = path.join(buildDir, `${key}.zip`);
  rmSync(zip, { force: true });
  sh("ditto", ["-c", "-k", "--sequesterRsrc", "--keepParent", app.bundle, zip]);
  const bundleBytes = duBytes(app.bundle);
  results.apps[key] = {
    label: app.label,
    buildSeconds,
    bundleBytes,
    bundleWithoutCoreBytes: bundleBytes - coreBytes,
    zipBytes: statSync(zip).size,
    executableBytes: statSync(app.exe(app.bundle)).size,
  };
}

// --- The shared core ------------------------------------------------------------------------
if (!existsSync(db)) sh("node", [path.join(root, "tooling/bench/seed.mjs"), "--db", db]);
const coreProcess = spawn(core, ["-db", db, "serve", "--listen", "127.0.0.1:0"], { stdio: "ignore" });
let coreEnv;
for (let i = 0; i < 150 && !coreEnv; i++) {
  try {
    const state = JSON.parse(readFileSync(path.join(path.dirname(db), "run/core.json"), "utf8"));
    if (state.pid === coreProcess.pid) coreEnv = { POLTERWARE_CORE_ENDPOINT: state.endpoint, POLTERWARE_CORE_TOKEN: state.token };
  } catch {}
  if (!coreEnv) await sleep(100);
}
if (!coreEnv) fail(`no core for ${db} (is another one running on it?)`);
const dispatch = await startDispatchFixture(path.join(path.dirname(db), "dispatch"));
Object.assign(coreEnv, dispatch.env);

// --- Process tree probes --------------------------------------------------------------------
function tree(pid) {
  const rows = quiet("ps", ["-A", "-o", "pid=,ppid="]).trim().split("\n").map((line) => line.trim().split(/\s+/).map(Number));
  const pids = [pid];
  for (let i = 0; i < pids.length; i++) for (const [child, parent] of rows) if (parent === pids[i]) pids.push(child);
  return pids;
}
function cpuSeconds(pids) {
  const text = quiet("ps", ["-o", "time=", "-p", pids.join(",")]);
  return text.trim().split("\n").reduce((sum, value) => {
    const parts = value.trim().split(":").map(Number);
    return sum + parts.reduce((total, part) => total * 60 + part, 0);
  }, 0);
}
function memory(pids) {
  const file = path.join(out, "footprint.json");
  try {
    execFileSync("footprint", ["-j", file, ...pids.map(String)], { stdio: "ignore" });
    const data = JSON.parse(readFileSync(file, "utf8"));
    const footprint = data["total footprint"] ?? data.processes.reduce((sum, item) => sum + item.footprint, 0);
    const rss = quiet("ps", ["-o", "rss=", "-p", pids.join(",")]).trim().split("\n").reduce((sum, kb) => sum + Number(kb) * 1024, 0);
    return { footprintBytes: footprint, rssBytes: rss, processes: pids.length };
  } catch (error) {
    return { error: String(error.message || error) };
  }
}
function stop(child) {
  for (const pid of tree(child.pid).reverse()) {
    try {
      process.kill(pid, "SIGKILL");
    } catch {}
  }
}

async function launch(key, env) {
  const app = APPS[key];
  const started = Date.now();
  const child = spawn(app.exe(app.bundle), [], { env: { ...process.env, ...coreEnv, POLTERWARE_DB: db, ...env }, stdio: "ignore" });
  return { child, started };
}

async function measureRun(key, index) {
  const bench = path.join(out, `${key}-${index}.jsonl`);
  const { child, started } = await launch(key, APPS[key].env(bench));
  let ready;
  for (let i = 0; i < 3000 && !ready; i++) {
    if (existsSync(bench)) {
      const line = readFileSync(bench, "utf8").split("\n").find((entry) => entry.includes("projects-ready"));
      if (line) ready = JSON.parse(line).at;
    }
    if (!ready) await sleep(20);
  }
  if (!ready) {
    stop(child);
    fail(`${key} never showed its projects`);
  }
  await sleep(5000);
  const pids = tree(child.pid);
  const settled = memory(pids);
  const cpuBefore = cpuSeconds(tree(child.pid));
  await sleep(10_000);
  const cpuAfter = cpuSeconds(tree(child.pid));
  stop(child);
  await sleep(1000);
  return { startupMs: ready - started, ...settled, idleCpuPercent: ((cpuAfter - cpuBefore) / 10) * 100 };
}

async function measureTour(key) {
  const dir = path.join(out, `${key}-tour`);
  const { child, started } = await launch(key, APPS[key].tourEnv(dir));
  let peak = 0;
  let cpu = 0;
  let exited = false;
  child.on("exit", () => (exited = true));
  while (!exited && Date.now() - started < 120_000) {
    const pids = tree(child.pid);
    try {
      cpu = Math.max(cpu, cpuSeconds(pids));
    } catch {}
    const sample = memory(pids);
    if (sample.footprintBytes) peak = Math.max(peak, sample.footprintBytes);
    await sleep(1000);
  }
  if (!exited) stop(child);
  return { peakFootprintBytes: peak, cpuSeconds: cpu, seconds: (Date.now() - started) / 1000 };
}

for (const key of chosen) {
  console.log(`\n${APPS[key].label}: warm-up and ${runs} runs`);
  await measureRun(key, 0);
  const samples = [];
  for (let i = 1; i <= runs; i++) {
    samples.push(await measureRun(key, i));
    console.log(`  run ${i}: ${JSON.stringify(samples.at(-1))}`);
  }
  Object.assign(results.apps[key], {
    samples,
    startupMs: median(samples.map((s) => s.startupMs)),
    footprintBytes: median(samples.map((s) => s.footprintBytes)),
    rssBytes: median(samples.map((s) => s.rssBytes)),
    processes: median(samples.map((s) => s.processes)),
    idleCpuPercent: median(samples.map((s) => s.idleCpuPercent)),
  });
  if (tour) results.apps[key].tour = await measureTour(key);
}

const coreTree = tree(coreProcess.pid);
results.core = { ...memory(coreTree), cpuSeconds: cpuSeconds(coreTree) };
coreProcess.kill();
dispatch.close();
writeFileSync(path.join(out, "results.json"), `${JSON.stringify(results, null, 2)}\n`);

// --- Table ----------------------------------------------------------------------------------
const mb = (bytes) => (bytes == null ? "—" : `${(bytes / 2 ** 20).toFixed(1)} MB`);
const rows = [
  ["Bundle on disk", (a) => mb(a.bundleBytes)],
  ["Bundle without the Go core", (a) => mb(a.bundleWithoutCoreBytes)],
  ["Zipped bundle", (a) => mb(a.zipBytes)],
  ["Release build", (a) => (a.buildSeconds == null ? "—" : `${a.buildSeconds.toFixed(0)} s`)],
  ["Launch to project rows (median)", (a) => `${Math.round(a.startupMs)} ms`],
  ["Memory, settled (footprint)", (a) => mb(a.footprintBytes)],
  ["Memory, settled (RSS)", (a) => mb(a.rssBytes)],
  ["Processes", (a) => String(a.processes)],
  ["Idle CPU, 10 s", (a) => `${a.idleCpuPercent.toFixed(2)} %`],
  ["Tour: peak memory", (a) => (a.tour ? mb(a.tour.peakFootprintBytes) : "—")],
  ["Tour: CPU time", (a) => (a.tour ? `${a.tour.cpuSeconds.toFixed(1)} s` : "—")],
];
const keys = Object.keys(results.apps);
console.log(`\n| | ${keys.map((key) => results.apps[key].label).join(" | ")} |\n|---|${keys.map(() => "---").join("|")}|`);
for (const [label, value] of rows) console.log(`| ${label} | ${keys.map((key) => value(results.apps[key])).join(" | ")} |`);
console.log(`\nCore (shared): ${mb(results.core.footprintBytes)} footprint, ${results.core.cpuSeconds?.toFixed(1)} s CPU over the whole run`);
console.log(path.join(out, "results.json"));
