import { test, expect, _electron as electron } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

// W3-E2E supplies a packaged executable and @playwright/test. This spec never builds an app.
test("settings: round-trip, MCP off test, and token rotation against the real core", async () => {
  const executablePath = process.env.SPARKY_E2E_EXECUTABLE;
  test.skip(!executablePath, "W3-E2E must provide SPARKY_E2E_EXECUTABLE (packaged Sparky executable).");
  expect(isAbsolute(executablePath!)).toBe(true);
  const directory = await mkdtemp(join(tmpdir(), "sparky-w2-settings-"));
  const database = join(directory, "db", "sparky.db");
  await mkdir(join(directory, "db"));
  const environmentFile = join(directory, "environment.json");
  await writeFile(environmentFile, JSON.stringify({ SPARKY_CORE_DB: database }));
  const environment = { ...process.env };
  delete environment.SPARKY_CORE_ENDPOINT;
  delete environment.SPARKY_CORE_TOKEN;
  delete environment.SPARKY_NOTIFY_E2E;
  const app = await electron.launch({ executablePath, args: [`--user-data-dir=${join(directory, "userdata")}`, `--sparky-e2e-env=${environmentFile}`], env: environment });
  try {
    const page = await app.firstWindow();
    if (await page.getByRole("button", { name: "Skip onboarding", exact: true }).count()) {
      await page.getByRole("button", { name: "Skip onboarding", exact: true }).click();
    }
    await page.keyboard.press("Meta+,");
    const settings = page.getByRole("dialog", { name: "Settings", exact: true });
    await expect(settings).toBeVisible();
    await settings.getByRole("button", { name: "General", exact: true }).click();
    const keepRunning = settings.getByRole("switch").first();
    const before = await keepRunning.getAttribute("data-state") ?? await keepRunning.getAttribute("aria-checked");
    await keepRunning.click();
    const after = await page.evaluate(async () => {
      const result = await window.sparky.settings.get({});
      return result.settings?.appKeepRunningInBackground;
    });
    expect(after).toBe(before === "checked" || before === "true" ? false : true);

    await settings.getByRole("button", { name: "Access", exact: true }).click();
    const enabled = settings.getByRole("switch").first();
    if ((await enabled.getAttribute("data-state")) === "checked" || (await enabled.getAttribute("aria-checked")) === "true") {
      await enabled.click();
    }
    await settings.getByRole("button", { name: "Test connection", exact: true }).click();
    await expect(settings).toContainText("MCP is off");

    if ((await enabled.getAttribute("data-state")) !== "checked" && (await enabled.getAttribute("aria-checked")) !== "true") {
      await enabled.click();
    }
    const previous = await page.evaluate(async () => (await window.sparky.agents.getInfo({})).token);
    await settings.getByRole("button", { name: "Rotate", exact: true }).click();
    await expect(settings).toContainText("Previous token is no longer valid");
    const next = await page.evaluate(async () => (await window.sparky.agents.getInfo({})).token);
    expect(next).not.toBe(previous);

    await settings.getByRole("button", { name: "Search", exact: true }).click();
    await expect(settings).toContainText("FTS5");
    await expect(settings).not.toContainText("semantic");
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
