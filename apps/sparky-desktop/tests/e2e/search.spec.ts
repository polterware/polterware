import { test, expect, _electron as electron } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

test("search: Commander opens the page, empty state, query, and FTS5 copy", async () => {
  const executablePath = process.env.SPARKY_E2E_EXECUTABLE;
  test.skip(!executablePath, "W3-E2E must provide SPARKY_E2E_EXECUTABLE (packaged Sparky executable).");
  expect(isAbsolute(executablePath!)).toBe(true);
  const directory = await mkdtemp(join(tmpdir(), "sparky-w2-search-"));
  const database = join(directory, "db", "sparky.db");
  await mkdir(join(directory, "db"));
  const environmentFile = join(directory, "environment.json");
  await writeFile(environmentFile, JSON.stringify({ SPARKY_CORE_DB: database }));
  const environment = { ...process.env };
  delete environment.SPARKY_CORE_ENDPOINT;
  delete environment.SPARKY_CORE_TOKEN;
  const app = await electron.launch({
    executablePath,
    args: [`--user-data-dir=${join(directory, "userdata")}`, `--sparky-e2e-env=${environmentFile}`],
    env: environment,
  });
  try {
    const page = await app.firstWindow();
    await page.getByRole("button", { name: "Skip onboarding", exact: true }).click();
    await page.evaluate(() => window.sparky.memories.create({ title: "Wire search fixture", note: "Plain <img src=x> snippet" }));
    await page.getByRole("button", { name: "Open Commander" }).click();
    await page.getByLabel("Search or command", { exact: true }).fill("Wire search fixture");
    const commander = page.getByRole("dialog", { name: "Commander", exact: true });
    await expect(commander.getByRole("button", { name: /Wire search fixture/ })).toBeVisible();
    await expect(commander.locator("img")).toHaveCount(0);
    await commander.getByRole("button", { name: /Wire search fixture/ }).click();
    await expect(page.locator("[data-memory-title]")).toHaveValue("Wire search fixture");
    await page.getByRole("button", { name: "Open Commander" }).click();
    await page.getByLabel("Search or command", { exact: true }).fill("Search memories");
    await page.getByRole("dialog").getByRole("button", { name: "Search memories", exact: true }).click();
    await expect(page.getByRole("main")).toHaveAttribute("aria-label", "Search");
    await expect(page.getByRole("searchbox", { name: "Search memories" })).toBeVisible();
    await expect(page.getByText("FTS5 text search only.", { exact: false })).toBeVisible();
    await page.getByRole("searchbox", { name: "Search memories" }).fill("milk");
    await expect(page.getByText("No results").or(page.getByRole("table", { name: "Search results" }))).toBeVisible();
  } finally {
    await app.close();
    try {
      const state = JSON.parse(await readFile(join(directory, "db", "run", "core.json"), "utf8"));
      if (Number.isInteger(state.pid) && state.pid > 0) {
        const command = execFileSync("ps", ["-p", String(state.pid), "-o", "args="], { encoding: "utf8" });
        if (command.includes(database) && command.includes("serve")) process.kill(state.pid, "SIGTERM");
      }
    } catch { /* A failed launch may not have started the core. */ }
  }
});
