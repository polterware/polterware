#!/usr/bin/env node
import { spawn, spawnSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";

const root = dirname(fileURLToPath(import.meta.url));
const spikeRoot = join(root, "..");
const evidenceDir = join(spikeRoot, "evidence");
const screenshotPath = join(evidenceDir, "dark-map-maplibre.png");
const rendererOut = join(spikeRoot, "out/renderer");

mkdirSync(evidenceDir, { recursive: true });

console.log("Building spike…");
const build = spawnSync("pnpm", ["build"], { cwd: spikeRoot, stdio: "inherit", shell: true });
if (build.status !== 0) {
  process.exit(build.status ?? 1);
}

const mime = {
  ".html": "text/html",
  ".js": "application/javascript",
  ".css": "text/css",
  ".png": "image/png",
  ".svg": "image/svg+xml",
  ".woff2": "font/woff2",
};

function startStaticServer(port) {
  return createServer(async (req, res) => {
    const path = req.url?.split("?")[0] ?? "/";
    const filePath = join(rendererOut, path === "/" ? "index.html" : path.slice(1));
    try {
      const data = await readFile(filePath);
      const ext = filePath.slice(filePath.lastIndexOf("."));
      res.writeHead(200, { "Content-Type": mime[ext] ?? "application/octet-stream" });
      res.end(data);
    } catch {
      res.writeHead(404);
      res.end("Not found");
    }
  }).listen(port);
}

async function capture() {
  const port = 4178;
  const server = startStaticServer(port);
  const url = `http://127.0.0.1:${port}/`;

  const { chromium } = await import("playwright");
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1200, height: 800 } });
  await page.goto(url, { waitUntil: "networkidle", timeout: 60000 });
  await page.waitForTimeout(5000);
  await page.screenshot({ path: screenshotPath, fullPage: false });
  await browser.close();
  server.close();
  console.log(`Screenshot saved to ${screenshotPath}`);
}

capture().catch((error) => {
  console.error(error);
  process.exit(1);
});
