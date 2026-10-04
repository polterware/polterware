import { test, expect, _electron as electron } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

test("configure a daily schedule, preview occurrences, and snooze", async () => {
  const executablePath = process.env.SPARKY_E2E_EXECUTABLE;
  test.skip(!executablePath, "W3-E2E must provide SPARKY_E2E_EXECUTABLE (packaged Sparky executable).");
  expect(isAbsolute(executablePath!)).toBe(true);
  const directory = await mkdtemp(join(tmpdir(), "sparky-w2-schedule-"));
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
    await page.getByRole("button", { name: "Inbox", exact: true }).click();
    await page.getByRole("button", { name: "New memory", exact: true }).last().click();
    const dialog = page.getByRole("dialog", { name: "New memory", exact: true });
    await dialog.getByLabel("Title", { exact: true }).fill("Scheduled memory");
    await dialog.getByRole("button", { name: "Create memory" }).click();
    await expect(page.locator("[data-memory-title]")).toHaveValue("Scheduled memory");
    const card = page.locator("[data-schedule-card]");
    await expect(card).toBeVisible();
    await card.getByLabel("Start date").fill("2026-03-01");
    await card.getByRole("combobox", { name: "Time zone" }).click();
    await page.getByRole("menuitem", { name: "America/Sao_Paulo", exact: true }).click();
    await card.getByRole("combobox", { name: "Repeat" }).click();
    await page.getByRole("menuitem", { name: "Daily", exact: true }).click();
    await expect(card.locator("[data-schedule-preview] [data-occurrence]")).toHaveCount(5, { timeout: 5000 });
    await card.getByRole("button", { name: "Save schedule" }).focus();
    await page.keyboard.press("Enter");
    await expect(card.getByText("Live", { exact: true })).toBeVisible();
    await expect.poll(async () => card.locator("[data-reminder-state]").getAttribute("data-reminder-state"), { timeout: 2000 }).not.toBe("notifier off");
    await card.getByRole("button", { name: "Snooze" }).click();
    await expect(card.locator("[data-reminder-state]")).toBeVisible();
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
