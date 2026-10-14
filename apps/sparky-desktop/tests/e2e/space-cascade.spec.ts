import { test, expect } from "../fixtures";

test("deleting a parent Space cascades children and moves their notes to Inbox", async ({ sparky, page }) => {
  await sparky.skipOnboarding();
  const fixture = await page.evaluate(async () => {
    const parent = (await window.sparky.spaces.create({ name: "Cascade parent" })).space!;
    const child = (await window.sparky.spaces.create({ name: "Cascade child", parentId: parent.id })).space!;
    const note = (await window.sparky.notes.create({ title: "Cascade note", spaceId: child.id })).note!;
    return { parent, child, note };
  });
  await page.getByRole("button", { name: "Inbox", exact: true }).click();
  const result = await page.evaluate(async parent => window.sparky.spaces.delete({ id: parent.id, expectedRev: parent.rev }), fixture.parent);
  expect(result.deletedSpaceIds?.sort()).toEqual([fixture.parent.id, fixture.child.id].sort());
  expect(result.movedNoteCount).toBe(1);
  await expect.poll(() => page.evaluate(async () => ((await window.sparky.spaces.list({})).spaces ?? []).length)).toBe(0);
  await expect(page.getByText("Live · 1 notes", { exact: true })).toBeVisible();
  const note = await page.evaluate(async id => (await window.sparky.notes.get({ id })).note, fixture.note.id);
  expect(note?.spaceId ?? "").toBe("");
});
