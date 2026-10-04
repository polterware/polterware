import { expect, test } from "@playwright/test";

test("create and edit a Mind", async ({ electronApp }) => {
  const page = await electronApp.firstWindow();
  await page.getByRole("link", { name: "Minds" }).click();
  await page.getByRole("button", { name: "Create Mind" }).click();
  await page.getByLabel("Name").fill("Playwright Mind");
  await page.getByRole("button", { name: "Create", exact: true }).click();
  await expect(page.getByRole("link", { name: "Playwright Mind" })).toBeVisible();
  await page.getByRole("link", { name: "Playwright Mind" }).click();
  await page.getByLabel("Name").fill("Updated Mind");
  await page.getByRole("button", { name: "Save changes" }).click();
  await expect(page.getByRole("heading", { name: "Updated Mind" })).toBeVisible();
});
