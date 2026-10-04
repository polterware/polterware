import { test, expect } from "../fixtures";

// Product gap: plans/sparky-next/requests/w3-e2e-4.md
test.fixme(process.env.SPARKY_E2E_RECHECK_GAPS !== "1", "w3-e2e-4: Calendar slot does not open the created memory details. Set SPARKY_E2E_RECHECK_GAPS=1 after the fix.");

test("create a memory by clicking an empty calendar slot", async ({ sparky, page }) => {
  await page.getByRole("button", { name: "Skip onboarding", exact: true }).click().catch(() => {});
  await page.evaluate(() => window.sparky.settings.set({ keys: ["calendar.weekStart"], settings: { calendarWeekStart: "sun" } }));
  await page.getByRole("button", { name: "Calendar", exact: true }).click();
  await expect(page.locator("[data-calendar]")).toBeVisible({ timeout: 30000 });
  await expect.poll(() => page.evaluate(async () => (await window.sparky.settings.get({})).settings?.calendarWeekStart)).toBe("sun");
  await page.getByRole("button", { name: /Add at 09:00/ }).first().click();
  await expect(page.locator("[data-memory-title]")).toHaveValue("Untitled memory");
  const persisted = await page.evaluate(async () => {
    const bridge = (window as unknown as { sparky: { schedule: { listOccurrences(input: object): Promise<{ occurrences: { title: string; tz: string }[] }> } } }).sparky;
    const from = new Date();
    from.setHours(0, 0, 0, 0);
    const to = new Date(from);
    to.setDate(to.getDate() + 1);
    return bridge.schedule.listOccurrences({ from: from.toISOString(), to: to.toISOString(), includeCompleted: true });
  });
  expect(persisted.occurrences.some(item => item.title === "Untitled memory")).toBe(true);
  expect(persisted.occurrences.find(item => item.title === "Untitled memory")?.tz).toBeTruthy();
});
