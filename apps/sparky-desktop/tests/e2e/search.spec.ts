import { test, expect } from "../fixtures";


test("search: Commander opens the page, empty state, query, and FTS5 copy", async ({ sparky, page }) => {
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
  await expect(page.getByText("FTS5 text search only.", { exact: true })).toBeVisible();
  await page.getByRole("searchbox", { name: "Search memories" }).fill("milk");
  await expect(page.getByText("No results").or(page.getByRole("table", { name: "Search results" }))).toBeVisible();
});
