import { test, expect } from "../fixtures";

// Product gap: plans/sparky-next/requests/w3-e2e-2.md
test.fixme(process.env.SPARKY_E2E_RECHECK_GAPS !== "1", "w3-e2e-2: Map requests a page larger than the core limit. Set SPARKY_E2E_RECHECK_GAPS=1 after the fix.");

test("map: offline tiles make no map request and keep the place list", async ({ sparky, page }) => {
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

});
