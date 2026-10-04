import { test, expect, _electron as electron } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

test("focus: start, pause, skip, reset, and Commander Start focus", async () => {
  const executablePath = process.env.SPARKY_E2E_EXECUTABLE;
  test.skip(!executablePath, "W3-E2E must provide SPARKY_E2E_EXECUTABLE (packaged Sparky executable).");
  expect(isAbsolute(executablePath!)).toBe(true);
  const directory = await mkdtemp(join(tmpdir(), "sparky-w2-focus-"));
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
    await page.getByRole("navigation", { name: "Main navigation" }).getByRole("button", { name: "Focus", exact: true }).click();
    await expect(page.getByRole("main")).toHaveAttribute("aria-label", "Focus");
    await page.getByRole("button", { name: "Start", exact: true }).click();
    await expect(page.getByText("Quick focus", { exact: true })).toBeVisible();
    await expect(page.getByRole("button", { name: "Pause", exact: true })).toBeVisible();
    await page.getByRole("button", { name: "Pause", exact: true }).click();
    await expect(page.getByRole("button", { name: "Resume", exact: true })).toBeVisible();
    await page.getByRole("button", { name: "Skip", exact: true }).click();
    await expect(page.getByText("Short break", { exact: false })).toBeVisible();
    await page.getByRole("button", { name: "Reset", exact: true }).click();
    await expect(page.getByRole("button", { name: "Start", exact: true })).toBeVisible();
    await page.keyboard.press("Meta+k");
    await page.getByLabel("Search or command", { exact: true }).fill("Start focus");
    await page.getByRole("dialog").getByRole("button", { name: "Start focus", exact: true }).click();
    await expect(page.getByRole("button", { name: "Pause", exact: true })).toBeVisible();
    const recorded = await page.evaluate(async () => {
      const from = new Date(); from.setHours(0, 0, 0, 0);
      return window.sparky.focus.listSessions({ from: from.toISOString(), to: new Date().toISOString() });
    });
    expect(recorded.sessions.length).toBeGreaterThan(0);
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
