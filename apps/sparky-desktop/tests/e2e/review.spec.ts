import { test, expect, _electron as electron } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

test("review: range selector, summary, heatmap, and chart tables", async () => {
  const executablePath = process.env.SPARKY_E2E_EXECUTABLE;
  test.skip(!executablePath, "W3-E2E must provide SPARKY_E2E_EXECUTABLE (packaged Sparky executable).");
  expect(isAbsolute(executablePath!)).toBe(true);
  const directory = await mkdtemp(join(tmpdir(), "sparky-w2-review-"));
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
    await page.getByRole("navigation", { name: "Main navigation" }).getByRole("button", { name: "Review", exact: true }).click();
    await expect(page.getByRole("main")).toHaveAttribute("aria-label", "Review");
    await expect(page.getByRole("group", { name: "Review range" })).toBeVisible();
    await expect(page.getByText("Day streak", { exact: true })).toBeVisible();
    await expect(page.getByText("Completed", { exact: true })).toBeVisible();
    await page.getByRole("radio", { name: "Month", exact: true }).click();
    await expect(page.getByLabel("12-month contribution calendar").or(page.getByText("No completions yet"))).toBeVisible();
    if (await page.getByLabel("12-month contribution calendar").isVisible().catch(() => false)) {
      await expect(page.getByLabel("Weekly activity as a table")).toBeVisible();
      await expect(page.getByLabel("Focus minutes as a table")).toBeVisible();
      await expect(page.getByLabel("Contribution calendar as a table")).toBeVisible();
    }
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
