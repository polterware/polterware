import { test, expect, _electron as electron } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

test("create, rename, and delete a Mind with cascade confirmation", async () => {
  const executablePath = process.env.SPARKY_E2E_EXECUTABLE;
  test.skip(!executablePath, "W3-E2E must provide SPARKY_E2E_EXECUTABLE (packaged Sparky executable).");
  expect(isAbsolute(executablePath!)).toBe(true);
  const directory = await mkdtemp(join(tmpdir(), "sparky-w2-minds-"));
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
    await page.getByRole("navigation", { name: "Main navigation" }).getByRole("button", { name: "Minds", exact: true }).click();
    await page.getByRole("button", { name: "Create Mind" }).click();
    await page.getByLabel("Name").fill("Playwright Mind");
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await expect(page.getByRole("button", { name: /Playwright Mind/ })).toBeVisible();
    await page.getByRole("button", { name: /Playwright Mind/ }).click();
    await expect(page.getByRole("heading", { name: "Playwright Mind" })).toBeVisible();
    await page.getByLabel("Name").fill("Updated Mind");
    await page.getByRole("button", { name: "Save changes" }).click();
    await expect(page.getByRole("heading", { name: "Updated Mind" })).toBeVisible();
    await page.getByRole("button", { name: "Delete Mind" }).click();
    await expect(page.getByText(/Deletes \d+ child Minds; \d+ memories move to Inbox\./)).toBeVisible();
    await page.getByRole("dialog").getByRole("button", { name: "Delete Mind" }).click();
    await expect(page.getByRole("heading", { name: "Minds" })).toBeVisible();
  } finally {
    await app.close();
    try {
      const state = JSON.parse(await readFile(join(directory, "db", "run", "core.json"), "utf8"));
      if (Number.isInteger(state.pid) && state.pid > 0) {
        const command = execFileSync("ps", ["-p", String(state.pid), "-o", "args="], { encoding: "utf8" });
        if (command.includes(database) && command.includes("serve")) process.kill(state.pid, "SIGTERM");
      }
    } catch {
      /* A failed launch may not have started the core. */
    }
  }
});
