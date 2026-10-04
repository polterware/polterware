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
  await page.getByRole("button", { name: "Inbox", exact: true }).click();
  await page.getByRole("complementary", { name: "Sidebar", exact: true }).getByRole("treeitem", { name: /Playwright Mind/ }).click();
  await expect(page.getByRole("heading", { name: "Playwright Mind", exact: true })).toBeVisible();
  await page.getByLabel("Name").fill("Updated Mind");
  await page.getByRole("button", { name: "Save changes" }).click();
  await expect(page.getByRole("heading", { name: "Updated Mind" })).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Breadcrumb" })).toBeVisible();
  await expect(page.getByLabel("Move to", { exact: true })).toBeVisible();
  await expect(page.getByRole("region", { name: "Mind memories", exact: true }).getByRole("button", { name: "New memory", exact: true }).first()).toBeVisible();
  await page.getByRole("button", { name: "Add child Mind", exact: true }).first().click();
  const createChild = page.getByRole("dialog", { name: "Create Mind", exact: true });
  await createChild.getByLabel("Name", { exact: true }).fill("Child Mind");
  await createChild.getByRole("button", { name: "Create", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Child Minds", exact: true })).toBeVisible();
  const memoryId = await page.evaluate(async () => {
    const child = (await window.sparky.minds.list({})).minds?.find(mind => mind.name === "Child Mind");
    if (!child?.id) throw new Error("Child Mind missing");
    return (await window.sparky.memories.create({ title: "Cascade memory", mindId: child.id })).memory?.id;
  });
  await page.getByRole("button", { name: "Delete Mind" }).click();
  await expect(page.getByText("Deletes 1 child Minds; 1 memories move to Inbox.", { exact: true })).toBeVisible();
  await page.getByRole("dialog").getByRole("button", { name: "Delete Mind" }).click();
  await expect(page.getByRole("heading", { name: "Minds", level: 1, exact: true })).toBeVisible();
  await expect.poll(() => page.evaluate(async id => (await window.sparky.memories.get({ id: id! })).memory?.mindId ?? "", memoryId)).toBe("");
});
