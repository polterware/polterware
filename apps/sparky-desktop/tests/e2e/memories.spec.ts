import { test, expect } from "../fixtures";

test("keyboard create, edit, pin, complete, undo and delete", async ({ sparky, page }) => {
  await sparky.skipOnboarding();
  await page.getByRole("button", { name: "Inbox", exact: true }).click();
  await page.getByRole("main").getByRole("button", { name: "New memory", exact: true }).first().focus();
  await expect(page.getByRole("main").getByRole("button", { name: "New memory", exact: true }).first()).toBeFocused();
  await page.getByRole("main").getByRole("button", { name: "New memory", exact: true }).first().press("Space");
  const dialog = page.getByRole("dialog", { name: "New memory", exact: true });
  await dialog.getByLabel("Title", { exact: true }).fill("Keyboard memory");
  await dialog.getByRole("button", { name: "Create memory" }).press("Space");
  const title = page.locator("[data-memory-title]");
  await expect(title).toHaveValue("Keyboard memory");
  await title.focus(); await page.keyboard.press("Meta+A"); await page.keyboard.type("Edited memory"); await page.keyboard.press("Meta+S");
  await expect(page.getByText("Live · Saved", { exact: true })).toBeVisible();
  await page.keyboard.press("Meta+J");
  const row = page.getByRole("row", { name: "Edited memory", exact: true });
  await row.focus(); await page.keyboard.press("p"); await expect(row.getByRole("button", { name: "Pin Edited memory" })).toHaveAttribute("aria-pressed", "true");
  await row.focus(); await page.keyboard.press("Space");
  const undo = page.getByRole("button", { name: "Undo", exact: true });
  await expect(undo).toBeVisible(); await undo.focus(); await page.keyboard.press("Enter");
  await expect(row).toBeVisible();
  await row.focus(); await page.keyboard.press("Meta+Backspace");
  await page.getByRole("dialog").getByRole("button", { name: "Delete", exact: true }).focus(); await page.keyboard.press("Enter");
  await expect(page.getByText(/Deletion queued:/)).toBeVisible();
  await page.getByRole("button", { name: "Undo", exact: true }).focus(); await page.keyboard.press("Enter");
  await expect(row).toBeVisible();
  // Verify the persisted title against the real core through the actual preload bridge.
  const persisted = await page.evaluate(async () => {
    const bridge = (window as unknown as { sparky: { memories: { list(input: object): Promise<{ memories: { title: string }[] }> } } }).sparky;
    return bridge.memories.list({ inboxOnly: true, limit: 500 });
  });
  expect(persisted.memories.some(memory => memory.title === "Edited memory")).toBe(true);
});
