import { test, expect, _electron as electron } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

test("create a memory by clicking an empty calendar slot", async () => {
  const executablePath = process.env.SPARKY_E2E_EXECUTABLE;
  test.skip(!executablePath, "W3-E2E must provide SPARKY_E2E_EXECUTABLE (packaged Sparky executable).");
  expect(isAbsolute(executablePath!)).toBe(true);
  const directory = await mkdtemp(join(tmpdir(), "sparky-w2-calendar-"));
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
    await page.getByRole("button", { name: "Skip onboarding", exact: true }).click().catch(() => {});
    await page.getByRole("button", { name: "Calendar", exact: true }).click();
    await expect(page.locator("[data-calendar]")).toBeVisible({ timeout: 30000 });
    await page.getByRole("button", { name: /Add at 09:00/ }).first().click();
    await expect(page.locator("[data-memory-title]")).toHaveValue("Untitled memory");
    const persisted = await page.evaluate(async () => {
      const bridge = (window as unknown as { sparky: { schedule: { listOccurrences(input: object): Promise<{ occurrences: { title: string }[] }> } } }).sparky;
      const from = new Date();
      from.setHours(0, 0, 0, 0);
      const to = new Date(from);
      to.setDate(to.getDate() + 1);
      return bridge.schedule.listOccurrences({ from: from.toISOString(), to: to.toISOString(), includeCompleted: true });
    });
    expect(persisted.occurrences.some(item => item.title === "Untitled memory")).toBe(true);
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
