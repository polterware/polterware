import { test, expect, _electron as electron } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

// W3-E2E supplies a packaged executable and @playwright/test. This spec never builds an app.
test("shell: keyboard navigation, Commander quick add, real core, and global details", async () => {
  const executablePath = process.env.SPARKY_E2E_EXECUTABLE;
  test.skip(!executablePath, "W3-E2E must provide SPARKY_E2E_EXECUTABLE (packaged Sparky executable).");
  expect(isAbsolute(executablePath!)).toBe(true);
  const directory = await mkdtemp(join(tmpdir(), "sparky-w2-shell-"));
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
    await expect(page.getByRole("dialog", { name: "Welcome to Sparky" })).toBeVisible();
    await page.getByRole("button", { name: "Continue", exact: true }).click();
    await expect(page.getByRole("button", { name: "Enable notifications", exact: true })).toBeVisible();
    await expect(page.getByRole("dialog")).toContainText("does not arm a reminder");
    await expect(page.getByRole("dialog")).not.toContainText(/Reminders: \d+ armed/);
    await page.getByRole("button", { name: "Skip onboarding", exact: true }).click();
    await expect(page.getByRole("main")).toHaveAttribute("aria-label", "Today");
    await expect(page.getByText("Connecting…", { exact: true })).toHaveCount(0);
    await expect(page.getByText("Core offline", { exact: true })).toHaveCount(0);
    await page.keyboard.press("Meta+1");
    await expect(page.getByRole("navigation", { name: "Main navigation" }).getByRole("button", { name: "Inbox", exact: true })).toHaveAttribute("aria-current", "page");
    await page.keyboard.press("Meta+[");
    await expect(page.getByRole("main")).toHaveAttribute("aria-label", "Today");
    await page.keyboard.press("Meta+k");
    await page.getByLabel("Search or command", { exact: true }).fill("new memory");
    await page.getByRole("dialog").getByRole("button", { name: "New memory", exact: true }).click();
    const title = `Shell smoke ${Date.now()}`;
    await page.getByLabel("Memory", { exact: true }).fill(`${title} tomorrow 9am #shell-smoke !!`);
    await page.getByRole("button", { name: "Create memory", exact: true }).click();
    await expect(page.getByRole("dialog")).toHaveCount(0);
    await expect(page.getByRole("main")).toContainText(title);
    const memories = await page.evaluate(async (query) => {
      const result = await window.sparky.memories.list({ query });
      return result;
    }, title);
    expect(memories).toMatchObject({ total: 1, memories: [expect.objectContaining({ title, priority: 2 })] });
    await page.keyboard.press("Meta+j");
    await page.keyboard.press("Meta+,");
    await expect(page.getByRole("dialog", { name: "Settings", exact: true })).toBeVisible();
  } finally {
    await app.close();
    // Stop only the daemon registered for this isolated database. Keep artifacts for W3-E2E diagnostics.
    try {
      const state = JSON.parse(await readFile(join(directory, "db", "run", "core.json"), "utf8"));
      if (Number.isInteger(state.pid) && state.pid > 0) {
        const command = execFileSync("ps", ["-p", String(state.pid), "-o", "args="], { encoding: "utf8" });
        if (command.includes(database) && command.includes("serve")) process.kill(state.pid, "SIGTERM");
      }
    } catch { /* A failed launch may not have started the core. */ }
  }
});
