import { chromium } from "../../../../../node_modules/.pnpm/playwright@1.63.0/node_modules/playwright/index.mjs";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const phase = process.argv[2] === "after" ? "after" : "before";
const widths = [
  [800, 560],
  [1280, 800],
  [1920, 1080],
];

await mkdir(root, { recursive: true });
const browser = await chromium.launch();
const page = await browser.newPage();
const report = [];

async function snap(name) {
  const width = page.viewportSize().width;
  const file = path.join(root, `${phase}-${name}-${width}.png`);
  await page.screenshot({ path: file, animations: "disabled", scale: "css" });
  report.push({ name, width, file: path.relative(path.join(root, "../../.."), file) });
}

async function goto(query) {
  await page.goto(`http://127.0.0.1:5179/?${query}`, { waitUntil: "networkidle" });
  await page.waitForSelector("header");
  await page.waitForTimeout(400);
}

for (const [width, height] of widths) {
  await page.setViewportSize({ width, height });
  await goto("app=polterware");
  await snap("polterware");
  await goto("app=sparky");
  await snap("sparky");
}

await page.setViewportSize({ width: 1280, height: 800 });
await goto("app=sparky");
await page.getByRole("button", { name: "Open Commander" }).click();
await page.getByRole("dialog").waitFor();
await snap("sparky-commander");

await writeFile(path.join(root, `${phase}-capture.json`), JSON.stringify(report, null, 2));
await browser.close();
console.log(`${phase}: ${report.length} screenshots`);
