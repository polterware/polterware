import { test, expect } from "../fixtures";


test("create, rename, and delete a Mind with cascade confirmation", async ({ sparky, page }) => {
  await page.getByRole("button", { name: "Skip onboarding", exact: true }).click();
  await page.getByRole("navigation", { name: "Main navigation" }).getByRole("button", { name: "Minds", exact: true }).click();
  await page.getByRole("main").getByRole("button", { name: "Create Mind", exact: true }).first().click();
  await page.getByLabel("Name").fill("Playwright Mind");
  await page.getByRole("button", { name: "Create", exact: true }).click();
  await expect(page.getByRole("main").getByRole("button", { name: /Playwright Mind/ })).toBeVisible();
  await page.getByRole("main").getByRole("button", { name: /Playwright Mind/ }).click();
  await expect(page.getByRole("heading", { name: "Playwright Mind" })).toBeVisible();
  await page.getByLabel("Name").fill("Updated Mind");
  await page.getByRole("button", { name: "Save changes" }).click();
  await expect(page.getByRole("heading", { name: "Updated Mind" })).toBeVisible();
  await page.getByRole("button", { name: "Delete Mind" }).click();
  await expect(page.getByText(/Deletes \d+ child Minds; \d+ memories move to Inbox\./)).toBeVisible();
  await page.getByRole("dialog").getByRole("button", { name: "Delete Mind" }).click();
  await expect(page.getByRole("heading", { name: "Minds" })).toBeVisible();
});
