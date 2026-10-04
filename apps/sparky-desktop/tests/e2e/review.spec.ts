import { test, expect } from "../fixtures";

test("review: range selector, summary, heatmap, and chart tables", async ({ sparky, page }) => {
  await page.getByRole("button", { name: "Skip onboarding", exact: true }).click();
  await page.evaluate(() => window.sparky.settings.set({ keys: ["calendar.weekStart"], settings: { calendarWeekStart: "sun" } }));
  await page.getByRole("navigation", { name: "Main navigation" }).getByRole("button", { name: "Review", exact: true }).click();
  await expect(page.getByRole("main")).toHaveAttribute("aria-label", "Review");
  await expect(page.getByRole("radiogroup", { name: "Review range" })).toBeVisible();
  await expect(page.getByText("Day streak", { exact: true })).toBeVisible();
  await expect(page.getByText("Completed", { exact: true })).toBeVisible();
  const art = page.locator('.sparky-review-summary img');
  await expect(art).toHaveCount(2);
  for (const image of await art.all()) {
    await expect(image).toHaveAttribute("src", /^\/characters\//);
    await expect.poll(() => image.evaluate((element: HTMLImageElement) => element.complete && element.naturalWidth > 0)).toBe(true);
  }
  await expect(page.getByText("Character art unavailable", { exact: true })).toHaveCount(0);
  await page.getByText("Month", { exact: true }).click();
  await expect(page.getByLabel("12-month contribution calendar").or(page.getByText("No completions yet"))).toBeVisible();
  if (await page.getByLabel("12-month contribution calendar").isVisible().catch(() => false)) {
    await expect(page.getByLabel("Weekly activity as a table")).toBeVisible();
    await expect(page.getByLabel("Focus minutes as a table")).toBeVisible();
    await expect(page.getByLabel("Contribution calendar as a table")).toBeVisible();
  }
});
