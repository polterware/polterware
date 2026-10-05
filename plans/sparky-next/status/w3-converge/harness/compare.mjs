import { chromium } from "../../../../../node_modules/.pnpm/playwright@1.63.0/node_modules/playwright/index.mjs";
import { readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const files = (await readdir(root)).filter((name) => name.startsWith("before-") && name.endsWith(".png")).sort();
const browser = await chromium.launch();
const page = await browser.newPage();
const rows = [];

for (const beforeName of files) {
  const afterName = beforeName.replace(/^before-/, "after-");
  const a64 = (await readFile(path.join(root, beforeName))).toString("base64");
  let b64;
  try {
    b64 = (await readFile(path.join(root, afterName))).toString("base64");
  } catch {
    rows.push({ before: beforeName, after: afterName, equal: false, reason: "missing after" });
    continue;
  }
  const result = await page.evaluate(async ({ a64, b64 }) => {
    const load = (data) =>
      new Promise((resolve, reject) => {
        const img = new Image();
        img.onload = () => resolve(img);
        img.onerror = reject;
        img.src = "data:image/png;base64," + data;
      });
    const a = await load(a64);
    const b = await load(b64);
    if (a.width !== b.width || a.height !== b.height) {
      return { equal: false, reason: "size", a: [a.width, a.height], b: [b.width, b.height] };
    }
    const canvas = document.createElement("canvas");
    canvas.width = a.width;
    canvas.height = a.height;
    const ctx = canvas.getContext("2d");
    ctx.drawImage(a, 0, 0);
    const da = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    ctx.drawImage(b, 0, 0);
    const db = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    let diff = 0;
    for (let i = 0; i < da.length; i += 4) {
      if (da[i] !== db[i] || da[i + 1] !== db[i + 1] || da[i + 2] !== db[i + 2] || da[i + 3] !== db[i + 3]) diff++;
    }
    const total = canvas.width * canvas.height;
    return { equal: diff === 0, diff, total, ratio: diff / total };
  }, { a64, b64 });
  rows.push({ before: beforeName, after: afterName, ...result });
}

await browser.close();
await writeFile(path.join(root, "pixel-diff.json"), JSON.stringify(rows, null, 2));
console.log(JSON.stringify(rows, null, 2));
const failed = rows.filter((row) => !row.equal);
if (failed.length) {
  console.error(`pixel mismatch: ${failed.length}/${rows.length}`);
  process.exitCode = 1;
} else {
  console.log(`pixel match: ${rows.length}/${rows.length}`);
}
