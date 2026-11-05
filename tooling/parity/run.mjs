#!/usr/bin/env node
// Captures the same screens in the Electron Desktop and the GPUI Desktop, over the same seeded
// database and one shared core, and writes a side-by-side page: dist/parity/<run>/index.html.
// Each app walks its own tour with the same stop names (apps/polterware-desktop/src/renderer/
// instrument-tour.ts, desktop-gpui/src/shots.rs), on a profile of its own, attached to the core
// through POLTERWARE_CORE_ENDPOINT and POLTERWARE_CORE_TOKEN. Both read Dispatch's history from
// one stand-in Dispatch core (./dispatch-fixture.mjs), never the Dispatch app installed here.
//
//   node tooling/parity/run.mjs [--db dist/bench/typical/local.db] [--no-build] [--only electron|gpui]
//
// The database comes from tooling/bench/seed.mjs (seeded here when missing). The windows come
// to the front while they are captured.
import { execFileSync, spawn } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { startDispatchFixture } from "./dispatch-fixture.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const argv = process.argv.slice(2);
const option = (name, fallback) => {
  const index = argv.indexOf(`--${name}`);
  return index >= 0 ? argv[index + 1] : fallback;
};
const build = !argv.includes("--no-build");
const only = option("only", "");
const db = path.resolve(option("db", path.join(root, "dist/bench/typical/local.db")));
const bin = path.join(root, "dist/polterware");
const desktop = path.join(root, "apps/polterware-desktop");
const gpuiDir = path.join(root, "desktop-gpui");
const run = new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
const out = path.join(root, "dist/parity", run);

const fail = (message) => {
  console.error(`parity: ${message}`);
  process.exit(1);
};
const sh = (command, args, options = {}) => execFileSync(command, args, { stdio: "inherit", ...options });

if (build) {
  sh("pnpm", ["build:cli"], { cwd: root });
  if (only !== "gpui") sh("pnpm", ["--dir", desktop, "exec", "electron-vite", "build"]);
  if (only !== "electron") sh("cargo", ["build", "--manifest-path", path.join(gpuiDir, "Cargo.toml")]);
}
if (!existsSync(db)) sh("node", [path.join(root, "tooling/bench/seed.mjs"), "--db", db]);

// One core for both apps, outside either app's process tree.
const core = spawn(bin, ["-db", db, "serve", "--listen", "127.0.0.1:0"], { stdio: ["ignore", "ignore", "ignore"] });
async function coreEnv() {
  const file = path.join(path.dirname(db), "run/core.json");
  for (let i = 0; i < 150; i++) {
    try {
      const { endpoint, token, pid } = JSON.parse(readFileSync(file, "utf8"));
      if (endpoint && token && pid === core.pid) return { POLTERWARE_CORE_ENDPOINT: endpoint, POLTERWARE_CORE_TOKEN: token };
    } catch {}
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  fail(`no core for ${db} (is another one running on it?)`);
}
const dispatch = await startDispatchFixture(path.join(path.dirname(db), "dispatch"));
const env = { ...process.env, ...(await coreEnv()), ...dispatch.env, POLTERWARE_DB: db };

function runApp(command, args, extraEnv, cwd) {
  return new Promise((resolve) => {
    const child = spawn(command, args, { cwd, env: { ...env, ...extraEnv }, stdio: ["ignore", "ignore", "inherit"] });
    const timer = setTimeout(() => child.kill("SIGKILL"), 120_000);
    child.on("exit", (code) => {
      clearTimeout(timer);
      resolve(code);
    });
  });
}

try {
  if (only !== "gpui") {
    const dir = path.join(out, "electron");
    await runApp(path.join(desktop, "node_modules/.bin/electron"), ["."], {
      POLTERWARE_TOUR: dir,
      POLTERWARE_USER_DATA: path.join(path.dirname(db), "electron-profile"),
    }, desktop);
  }
  if (only !== "electron") {
    await runApp(path.join(gpuiDir, "target/debug/polterware-gpui"), [], { POLTERWARE_GPUI_SHOTS: path.join(out, "gpui") }, gpuiDir);
  }
} finally {
  core.kill();
  dispatch.close();
}

const shots = (app) => (existsSync(path.join(out, app)) ? readdirSync(path.join(out, app)).filter((name) => name.endsWith(".png")) : []);
const names = [...new Set([...shots("electron"), ...shots("gpui")])].sort();
if (!names.length) fail("no captures were written");
const size = (file) => {
  if (!existsSync(file)) return "";
  const info = execFileSync("sips", ["-g", "pixelWidth", "-g", "pixelHeight", file], { encoding: "utf8" });
  const [width, height] = [/pixelWidth: (\d+)/, /pixelHeight: (\d+)/].map((pattern) => info.match(pattern)?.[1]);
  return `${width}×${height}`;
};
const cell = (app, name) =>
  existsSync(path.join(out, app, name))
    ? `<figure><img src="${app}/${name}" alt="${app} ${name}"><figcaption>${app === "gpui" ? "GPUI" : "Electron"} · ${size(path.join(out, app, name))}</figcaption></figure>`
    : `<figure class="missing"><div>No ${app === "gpui" ? "GPUI" : "Electron"} capture</div></figure>`;
const rows = names
  .map((name) => {
    const stop = name.replace(/\.png$/, "");
    return `<section id="${stop}"><h2>${stop}</h2>
<div class="pair">${cell("electron", name)}${cell("gpui", name)}</div>
<div class="overlay" data-stop="${stop}">
  <img src="electron/${name}" alt=""><img class="top" src="gpui/${name}" alt="">
</div></section>`;
  })
  .join("\n");
writeFileSync(
  path.join(out, "index.html"),
  `<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Desktop parity ${run}</title>
<style>
:root { color-scheme: dark; --bg: #131313; --fg: #ededed; --muted: #8a8a8a; --edge: #2a2a2a; }
body { margin: 0; padding: 16px; background: var(--bg); color: var(--fg); font: 13px/1.4 -apple-system, system-ui, sans-serif; }
header { display: flex; gap: 16px; align-items: baseline; flex-wrap: wrap; margin-bottom: 16px; }
h1 { font-size: 16px; margin: 0; } h2 { font-size: 13px; margin: 24px 0 8px; color: var(--muted); }
nav a { color: var(--fg); margin-right: 8px; }
.pair { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
figure { margin: 0; } img { width: 100%; display: block; border: 1px solid var(--edge); border-radius: 8px; }
figcaption { color: var(--muted); padding: 4px 2px; }
.missing div { aspect-ratio: 1440 / 920; display: grid; place-items: center; border: 1px dashed var(--edge); border-radius: 8px; color: var(--muted); }
.overlay { position: relative; margin-top: 12px; display: none; }
.overlay img.top { position: absolute; inset: 0; mix-blend-mode: difference; }
body.diff .overlay { display: block; } body.diff .pair { display: none; }
</style>
<header><h1>Electron and GPUI, same screens, same data</h1>
<label><input type="checkbox" onchange="document.body.classList.toggle('diff', this.checked)"> Difference overlay (black is identical)</label>
<nav>${names.map((name) => `<a href="#${name.replace(/\.png$/, "")}">${name.replace(/\.png$/, "")}</a>`).join("")}</nav></header>
${rows}
`,
);
console.log(path.join(out, "index.html"));
