import { test, expect } from "../fixtures";
import { inspectNative } from "../helpers/inspect-native";

test("real native reminder delivers with Electron and the core closed", async ({ sparky, page }, info) => {
  test.setTimeout(90_000);
  await sparky.skipOnboarding();
  const before = await page.evaluate(() => window.sparky.reminders.status({}));
  await info.attach("native-status-before", { body: JSON.stringify(before), contentType: "application/json" });
  const authorized = await page.evaluate(() => window.sparky.app.requestNotificationAuthorization());
  const after = await page.evaluate(() => window.sparky.reminders.status({}));
  await info.attach("native-status-after", { body: JSON.stringify({ authorized, after }), contentType: "application/json" });
  expect(authorized, "Grant Sparky notification permission in macOS; OS delivery cannot be replaced by a stub").toBe(true);
  const fireAt = new Date(Date.now() + 15_000).toISOString();
  const noteId = await page.evaluate(async fireAt => {
    const note = (await window.sparky.notes.create({ title: "Sparky W3 live reminder" })).note!;
    await window.sparky.schedule.set({ schedule: { noteId: note.id, active: true, fireAt, tz: "UTC", endType: "never" } });
    return note.id;
  }, fireAt);
  await expect.poll(() => page.evaluate(async () => (await window.sparky.reminders.status({})).armed)).toBeGreaterThan(0);
  const pending = await inspectNative(sparky, "pending");
  await info.attach("native-pending", { body: JSON.stringify(pending), contentType: "application/json" });
  expect(pending.ok).toBe(true);
  expect(pending.pending?.some(item => item.id.includes(noteId))).toBe(true);
  await sparky.app.context().tracing.stop({ path: info.outputPath("live-before-close.zip") });
  await sparky.app.close();
  await sparky.killCore();
  await expect.poll(async () => (await inspectNative(sparky, "delivered")).delivered?.some(item => item.id.includes(noteId)) ?? false, { timeout: 30000, intervals: [1000] }).toBe(true);
  // Notification Center Done/Snooze, sleep/wake and reboot are manual checks.
});
