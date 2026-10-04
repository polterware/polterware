import { test, expect } from "@playwright/test";
import { _electron as electron, type ElectronApplication, type Page } from "playwright";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";

// W3-E2E supplies the packaged app and Playwright runner. Never use a personal database.
const executablePath = process.env.SPARKY_E2E_EXECUTABLE;
test.skip(!executablePath, "W3-E2E must provide SPARKY_E2E_EXECUTABLE for a packaged Sparky app");
let app: ElectronApplication, page: Page, directory: string;
test.beforeEach(async () => {
  directory = await mkdtemp(join(tmpdir(), "sparky-memories-e2e-"));
  app = await electron.launch({ executablePath: resolve(executablePath!), args: [`--user-data-dir=${join(directory, "electron")}`], env: { ...process.env, SPARKY_CORE_DB: join(directory, "sparky.db") } });
  page = await app.firstWindow();
  // Onboarding is skippable and independent of this flow.
  await expect(page.getByRole("main")).toBeVisible({ timeout: 30000 });
  const skip = page.getByRole("button", { name: "Skip onboarding", exact: true });
  await skip.click({ timeout: 1000 }).catch(() => {});
});
test.afterEach(async () => {
  await app?.close();
  // The detached core belongs to this test only, as confirmed by its canonical database path.
  if (!directory) return;
  try {
    const state = JSON.parse(await readFile(join(directory, "run", "core.json"), "utf8"));
    if (state.dbPath === join(directory, "sparky.db") && Number.isSafeInteger(state.pid) && state.pid > 1) process.kill(state.pid, "SIGTERM");
  } catch { /* A core that never started has no state; cleanup still removes the fixture directory. */ }
  await rm(directory, { recursive: true, force: true });
});

test("keyboard create, edit, pin, complete, undo and delete", async () => {
  await page.getByRole("button", { name: "Inbox", exact: true }).click();
  await page.getByRole("button", { name: "New memory", exact: true }).last().focus();
  await page.keyboard.press("Enter");
  const dialog = page.getByRole("dialog", { name: "New memory", exact: true });
  await dialog.getByLabel("Title", { exact: true }).fill("Keyboard memory");
  await dialog.getByRole("button", { name: "Create memory" }).focus(); await page.keyboard.press("Enter");
  const title = page.locator("[data-memory-title]");
  await expect(title).toHaveValue("Keyboard memory");
  await title.focus(); await page.keyboard.press("Meta+A"); await page.keyboard.type("Edited memory"); await page.keyboard.press("Meta+S");
  await expect(page.getByText("Live · Saved", { exact: true })).toBeVisible();
  await page.keyboard.press("Meta+J");
  const row = page.getByRole("row", { name: "Edited memory", exact: true });
  await row.focus(); await page.keyboard.press("p"); await expect(row.getByRole("button", { name: "Pin Edited memory" })).toHaveAttribute("aria-pressed", "true");
  await row.focus(); await page.keyboard.press("Space");
  const undo = page.getByRole("button", { name: "Undo", exact: true });
  await expect(undo).toBeVisible(); await undo.focus(); await page.keyboard.press("Enter");
  await expect(row).toBeVisible();
  await row.focus(); await page.keyboard.press("Meta+Backspace");
  await page.getByRole("dialog").getByRole("button", { name: "Delete", exact: true }).focus(); await page.keyboard.press("Enter");
  await expect(page.getByText(/Deletion queued:/)).toBeVisible();
  await page.getByRole("button", { name: "Undo", exact: true }).focus(); await page.keyboard.press("Enter");
  await expect(row).toBeVisible();
  // Verify the persisted title against the real core through the actual preload bridge.
  const persisted = await page.evaluate(async () => {
    const bridge = (window as unknown as { sparky: { memories: { list(input: object): Promise<{ memories: { title: string }[] }> } } }).sparky;
    return bridge.memories.list({ inboxOnly: true, limit: 500 });
  });
  expect(persisted.memories.some(memory => memory.title === "Edited memory")).toBe(true);
});

test("10k-row rendering, filter budget and scrolling trace", async ({}, testInfo) => {
  test.skip(process.env.SPARKY_E2E_PERFORMANCE !== "1", "Opt in to the isolated 10k real-core fixture");
  test.setTimeout(600000);
  await page.evaluate(async () => {
    const memories = (window as unknown as { sparky: { memories: { create(input: object): Promise<unknown> } } }).sparky.memories;
    for (let index = 0; index < 10000; index++) await memories.create({ title: `Performance memory ${String(index).padStart(5, "0")}` });
  });
  await page.getByRole("button", { name: "Inbox", exact: true }).click();
  await expect(page.getByText("Live · 10000 memories", { exact: true })).toBeVisible({ timeout: 30000 });
  const scroll = page.locator('.pw-data-table-scroll[data-virtualized="true"]');
  await expect(scroll).toBeVisible();
  expect(await page.locator("[data-row-id]").count()).toBeLessThan(100);
  const tracePath = testInfo.outputPath("memories-10k-scroll.zip");
  await page.context().tracing.start({ screenshots: true, snapshots: true });
  const samples = await scroll.evaluate(async element => {
    const durations: number[] = [];
    let previous = performance.now();
    for (let frame = 0; frame < 120; frame++) {
      element.scrollTop = (element.scrollHeight - element.clientHeight) * frame / 119;
      const current = await new Promise<number>(requestAnimationFrame);
      if (frame > 0) durations.push(current - previous);
      previous = current;
    }
    durations.sort((a, b) => a - b);
    return { averageMs: durations.reduce((total, value) => total + value, 0) / durations.length, p95Ms: durations[Math.floor(durations.length * 0.95)] };
  });
  // Trace records the same real scroll frames; allow timer precision around the 60 Hz frame budget.
  expect(samples.averageMs).toBeLessThanOrEqual(17.2);
  expect(samples.p95Ms).toBeLessThanOrEqual(20);
  const filterMs = await page.getByRole("searchbox", { name: "Filter memories" }).evaluate(async (input: HTMLInputElement) => {
    const start = performance.now();
    const changed = new Promise<void>(done => {
      const observer = new MutationObserver(() => { observer.disconnect(); done(); });
      observer.observe(document.querySelector(".pw-data-table")!, { childList: true, subtree: true });
    });
    input.value = "Performance memory 09999"; input.dispatchEvent(new InputEvent("input", { bubbles: true }));
    await changed;
    return performance.now() - start;
  });
  expect(filterMs).toBeLessThan(50);
  await page.context().tracing.stop({ path: tracePath });
  await testInfo.attach("10k-scroll", { path: tracePath, contentType: "application/zip" });
  await testInfo.attach("10k-budgets", { body: JSON.stringify({ ...samples, filterMs }), contentType: "application/json" });
});
