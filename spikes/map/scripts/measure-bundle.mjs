#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const spikeRoot = join(root, "..");
const evidenceDir = join(spikeRoot, "evidence");

mkdirSync(evidenceDir, { recursive: true });

console.log("Building spike renderer for bundle measurement…");
const build = spawnSync("pnpm", ["build"], { cwd: spikeRoot, stdio: "inherit", shell: true });
if (build.status !== 0) {
  process.exit(build.status ?? 1);
}

function dirSizeKb(dir) {
  const result = spawnSync("du", ["-sk", dir], { encoding: "utf8" });
  return Number.parseInt(result.stdout.split("\t")[0] ?? "0", 10);
}

function measurePackage(name) {
  try {
    const pkgPath = join(spikeRoot, "node_modules", name, "package.json");
    const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
    const mainFile = join(spikeRoot, "node_modules", name, pkg.module ?? pkg.main ?? "");
    return statSync(mainFile).size;
  } catch {
    return null;
  }
}

const rendererOut = join(spikeRoot, "out/renderer");
const assetsDir = join(rendererOut, "assets");
const assetFiles = [];

try {
  for (const name of readdirSync(assetsDir)) {
    const path = join(assetsDir, name);
    if (statSync(path).isFile()) {
      assetFiles.push({ name, bytes: statSync(path).size });
    }
  }
} catch {
  // flat output
}

const report = {
  measuredAt: new Date().toISOString(),
  rendererTotalKb: dirSizeKb(rendererOut),
  largestAssets: assetFiles.sort((a, b) => b.bytes - a.bytes).slice(0, 10),
  nodeModulesMainBytes: {
    "maplibre-gl": measurePackage("maplibre-gl"),
    leaflet: measurePackage("leaflet"),
    "leaflet.markercluster": measurePackage("leaflet.markercluster"),
    pmtiles: measurePackage("pmtiles"),
    supercluster: measurePackage("supercluster"),
  },
  notes:
    "Production renderer bundle includes all three providers; W2-MAP should lazy-load per provider.",
};

const outPath = join(evidenceDir, "bundle-sizes.json");
writeFileSync(outPath, JSON.stringify(report, null, 2));
console.log(`Wrote ${outPath}`);
console.log(JSON.stringify(report, null, 2));
