import { test, expect } from "@playwright/test";
import { _electron as electron, type ElectronApplication, type Page } from "playwright";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";

// W3-E2E supplies the packaged app and Playwright runner. Never use a personal database.
const executablePath = process.env.SPARKY_E2E_EXECUTABLE;
test.skip(!executablePath, "W3-E2E must provide SPARKY_E2E_EXECUTABLE for a packaged Sparky app");
let app: ElectronApplication, page: Page, directory: string;

test.beforeEach(async () => {
  directory = await mkdtemp(join(tmpdir(), "sparky-attachments-e2e-"));
  app = await electron.launch({
    executablePath: resolve(executablePath!),
    args: [`--user-data-dir=${join(directory, "electron")}`],
    env: { ...process.env, SPARKY_CORE_DB: join(directory, "sparky.db") },
  });
  page = await app.firstWindow();
  await expect(page.getByRole("main")).toBeVisible({ timeout: 30000 });
  await page.getByRole("button", { name: "Skip onboarding", exact: true }).click({ timeout: 1000 }).catch(() => {});
});
test.afterEach(async () => {
  await app?.close();
  if (!directory) return;
  try {
    const state = JSON.parse(await readFile(join(directory, "run", "core.json"), "utf8"));
    if (state.dbPath === join(directory, "sparky.db") && Number.isSafeInteger(state.pid) && state.pid > 1) process.kill(state.pid, "SIGTERM");
  } catch { /* A core that never started has no state. */ }
  await rm(directory, { recursive: true, force: true });
});

async function openInboxMemory(title: string) {
  await page.getByRole("button", { name: "Inbox", exact: true }).click();
  const created = await page.evaluate(async (name) => {
    const memory = await window.sparky.memories.create({ title: name });
    return memory.memory?.id ?? "";
  }, title);
  await page.evaluate(id => window.sparky.app.openDeepLink(`sparky://memory/${id}`), created);
  await expect(page.locator("[data-attachments-section]")).toBeVisible();
  return created;
}

test("imports 20 mixed files through pickFiles and never uses file:// previews", async () => {
  const files = await Promise.all(Array.from({ length: 20 }, async (_, index) => {
    const ext = index === 0 ? "png" : index === 1 ? "webm" : "txt";
    const path = join(directory, `mix-${String(index).padStart(2, "0")}.${ext}`);
    await writeFile(path, ext === "png" ? Buffer.from("89504e470d0a1a0a", "hex") : `file ${index}`);
    return path;
  }));
  await app.evaluate(async ({ dialog }, paths) => {
    dialog.showOpenDialog = async () => ({ canceled: false, filePaths: paths });
  }, files);
  const memoryId = await openInboxMemory("Attachment import");
  await page.getByRole("button", { name: "Add attachment", exact: true }).click();
  await page.getByRole("menuitem", { name: "Add file", exact: true }).click();
  await expect(page.getByText("Live", { exact: true })).toBeVisible();
  const listed = await page.evaluate(async (id) => window.sparky.attachments.list({ memoryId: id }), memoryId);
  expect(listed.attachments?.length).toBe(20);
  const previewUrls = await page.locator("[data-preview-url]").evaluateAll(nodes => nodes.map(node => node.getAttribute("data-preview-url") ?? ""));
  expect(previewUrls.length).toBeGreaterThan(0);
  expect(previewUrls.every(url => url.startsWith("sparky-file://attachment/"))).toBe(true);
  expect(await page.locator("[src^='file:']").count()).toBe(0);
});

test("drag-drop import uses a real drop and granted paths", async () => {
  const memoryId = await openInboxMemory("Attachment drop");
  const zone = page.locator("[data-drop-zone]");
  await expect(zone).toBeVisible();
  const path = join(directory, "drop.txt");
  await writeFile(path, "dropped");
  const box = await zone.boundingBox();
  expect(box).toBeTruthy();
  const session = await page.context().newCDPSession(page);
  const point = { x: box!.x + box!.width / 2, y: box!.y + box!.height / 2 };
  const data = { items: [{ mimeType: "Files", data: path }], files: [path], dragOperationsMask: 1 };
  await session.send("Input.dispatchDragEvent", { type: "dragEnter", ...point, data });
  await session.send("Input.dispatchDragEvent", { type: "dragOver", ...point, data });
  await session.send("Input.dispatchDragEvent", { type: "drop", ...point, data });
  await expect(page.getByText("drop.txt")).toBeVisible();
  await expect(page.getByText("Live", { exact: true })).toBeVisible();
  const listed = await page.evaluate(async (id) => window.sparky.attachments.list({ memoryId: id }), memoryId);
  expect(listed.attachments?.some(item => item.name === "drop.txt")).toBe(true);
  expect(await page.locator("[src^='file:']").count()).toBe(0);
  await expect(page.getByText(/no filesystem path/)).toHaveCount(0);
});

test("denied microphone permission is shown honestly", async () => {
  await openInboxMemory("Attachment mic");
  await page.evaluate(() => {
    const original = window.sparky.app.startMicrophone;
    window.sparky.app.startMicrophone = async () => false;
    Reflect.set(window.sparky.app, "startMicrophone", async () => false);
    return original;
  });
  await page.getByRole("button", { name: "Add attachment", exact: true }).click();
  await page.getByRole("menuitem", { name: "Record audio", exact: true }).click();
  await expect(page.getByText("Microphone permission denied. Grant access in System Settings to record audio.")).toBeVisible();
});
