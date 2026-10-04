import { test, expect, _electron as electron } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { isAbsolute, join } from "node:path";

test("map: offline tiles make no map request and keep the place list", async () => {
  const executablePath = process.env.SPARKY_E2E_EXECUTABLE;
  test.skip(!executablePath, "W3-E2E must provide SPARKY_E2E_EXECUTABLE (packaged Sparky executable).");
  expect(isAbsolute(executablePath!)).toBe(true);
  const directory = await mkdtemp(join(tmpdir(), "sparky-w2-map-"));
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
    const requests: string[] = [];
    page.on("request", request => { requests.push(request.url()); });
    await page.getByRole("button", { name: "Skip onboarding", exact: true }).click();
    await page.evaluate(async () => {
      await window.sparky.settings.set({ keys: ["map.online"], settings: { mapOnline: false } });
    });
    await page.evaluate(() => window.sparky.app.openDeepLink("sparky://map"));
    await expect(page.getByRole("main")).toHaveAttribute("aria-label", "Map");
    await page.getByRole("navigation", { name: "Main navigation" }).getByRole("button", { name: "Map", exact: true }).click();
    await expect(page.getByRole("main")).toHaveAttribute("aria-label", "Map");
    await expect(page.getByText("Places do not trigger notifications in v1.")).toBeVisible();
    await page.getByRole("button", { name: "Keep tiles off", exact: true }).click().catch(() => undefined);
    await expect(page.getByText("Map tiles are off")).toBeVisible();
    await expect(page.getByRole("complementary", { name: "Places" })).toBeVisible();
    const created = await page.evaluate(async () => (await window.sparky.memories.create({ title: "Wire place fixture" })).memory?.id);
    await page.evaluate(id => window.sparky.app.openDeepLink(`sparky://memory/${id}`), created);
    await expect(page.locator("#place-name")).toBeVisible();
    await expect(page.locator("#place-address")).toHaveCount(0);
    const mapRequests = requests.filter(url => /openfreemap|nominatim\.openstreetmap|protomaps|tile\.openstreetmap/.test(url));
    expect(mapRequests, `unexpected map network: ${mapRequests.join(", ")}`).toEqual([]);
    // Turning geocoding on changes Main's CSP and reloads the renderer.
    const reloaded = page.waitForEvent("load");
    await page.evaluate(() => window.sparky.settings.set({ keys: ["map.geocoding"], settings: { mapGeocoding: true } })).catch(error => {
      if (!/context was destroyed/i.test(String(error))) throw error;
    });
    await reloaded;
    await page.evaluate(id => window.sparky.app.openDeepLink(`sparky://memory/${id}`), created);
    await expect(page.locator("#place-address")).toBeVisible();
    await expect(page.getByText(/typed address text to Nominatim/)).toBeVisible();
    expect(requests.filter(url => /nominatim\.openstreetmap/.test(url))).toEqual([]);

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
